use std::io::Cursor;

use bucket::{download, upload};
use db_storage::{
    db_connect,
    models::{Gallery, GalleryEmbeddings, NewEmbeddings, NewThumbnail, UserUpload},
};
use embeddings::get_img_embeddings;
use image::DynamicImage;
use image_operations::{create_thumbnail, image_from_bytes, to_base64, to_llava_base64};
use llm_messages::SemiStructuredMessage;
use llm_retrieval::{ImagePrompt, fetch_description, fetch_llava_description};
use queue::{create_consumer, feeder_protocol};
use simple_logger::SimpleLogger;
use tokio::sync::mpsc;

use crate::bucket::move_to_ragged;

mod bucket;
mod embeddings;
mod errors;
mod image_operations;
// mod llm_llava;
mod llm_messages;
mod llm_retrieval;
mod queue;
mod queue_messages;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        // .with_threads(true)
        .init()
        .unwrap();

    let (feeder_tx, mut feeder_rx) = mpsc::unbounded_channel();
    let (genai_tx, mut genai_rx) = mpsc::unbounded_channel::<(DynamicImage, GalleryEmbeddings)>();
    let pg_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");
    let kafka_url = std::env::var("KAFKA_SERVER_LISTENER").expect("Missing KAFKA_SERVER_LISTENER");
    let kafka_topic = std::env::var("KAFKA_MINIO_TOPIC").expect("Missing KAFKA_MINIO_TOPIC");
    let llm_to_use = std::env::var("USE_LLM_SERVICE").unwrap_or_else(|_| "ollama".into());
    let bucket_to_upload =
        std::env::var("BUCKET_RAGGED_BUCKET").expect("Missing BUCKET_RAGGED_BUCKET");

    tokio::spawn(async move {
        let feeder_consumer = match create_consumer(&kafka_url) {
            Ok(f) => f,
            Err(_) => todo!(),
        };

        if let Err(_err) = feeder_protocol(feeder_consumer, vec![&kafka_topic], feeder_tx).await {
            log::error!("Error on the feeder");
            panic!();
        };
    });

    let db_pool = db_connect(&pg_url).await?;
    loop {
        tokio::select! {
            msg = feeder_rx.recv() => {
                let msg = match msg {
                    Some(m) => m,
                    None => {
                        log::debug!("empty message received");

                        panic!();
                    }
                };
                log::info!("msg {:?}", msg);

                let file_bytes = download(&msg.filename).await?;

                // Find the user owner of this image
                // If the upload record is not found in db. Block the process.
                let mut user_info = UserUpload::get_by_filename(&db_pool, &msg.filename).await?;

                let i = image_from_bytes(&file_bytes)?;
                let thumbnail_512p = create_thumbnail(&i);
                // Generate embeddings from thumbnail image.
                let embeddings = get_img_embeddings(thumbnail_512p.image().clone())?;

                // BlobStore thumbnail image.
                let mut webp_bytes: Vec<u8> = Vec::new();
                let _ =
                    thumbnail_512p.image().write_to(&mut Cursor::new(&mut webp_bytes), image::ImageFormat::WebP);
                let thumbnail_name = format!("thumbnail/{}.webp", uuid::Uuid::new_v4().to_string());

                let _ = upload(&thumbnail_name, webp_bytes, Some(&bucket_to_upload)).await?;

                // Create db records
                let mut img_gallery = Gallery::new(&msg.filename).create(&db_pool).await?;
                user_info.set_gallery_id(&db_pool, &img_gallery.id()).await?;

                let mut img_embeddings = GalleryEmbeddings::new(thumbnail_name.clone(), embeddings);
                img_embeddings.create(&db_pool).await?;

                let moved_feeded_img_filepath = move_to_ragged(&msg.filename).await?;

                img_gallery.update_with_processed(&db_pool, &moved_feeded_img_filepath,
                    NewThumbnail{
                        path: &thumbnail_name, height: *thumbnail_512p.height() as i32, width: *thumbnail_512p.width() as i32, ratio: &thumbnail_512p.ratio_as_str() }, NewEmbeddings{embeddings_id: img_embeddings.id()})
                    .await?;


                if let Err(e) = genai_tx.send((thumbnail_512p.image().clone(), img_embeddings)){
                    log::error!("Failed to send thumbnail to genai thread\n{e:?}");
                }
            },
            Some(msg) = genai_rx.recv() => {
                let ( img_thumbnail, img_embeddings) = msg;

                let structured = match llm_to_use.as_str() {
                    "openai" => {
                         let img_str = to_base64(&img_thumbnail);
                         let structured_output = fetch_description(&img_str, ImagePrompt::SemiStructured).await?;
                         structured_output
                    },
                    _ => {
                        // Ollama
                        let ollama_str = to_llava_base64(&img_thumbnail);
                        let ollama_structured = fetch_llava_description(&ollama_str, ImagePrompt::SemiStructured).await?;
                         ollama_structured
                    }
                };

                let structures = match serde_json::from_str::<SemiStructuredMessage>(&structured) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("received from LLM {}. \n and error {e:?}", structured);
                        continue;
                    }
                };

                img_embeddings
                    .link_genai_descriptors(&db_pool, &structures.tags, &structures.description, &structures.theme, &structures.alt, &structures.caption)
                    .await?;
            },
        }
    }
}
