use std::io::Cursor;

use bucket::{download, upload};
use db_storage::{
    db_connect,
    models::{Gallery, GalleryEmbeddings},
};
use embeddings::get_img_embeddings;
use image_operations::{create_thumbnail, image_from_bytes, image_load_and_decode, to_base64};
use llm_retrieval::{ImagePrompt, fetch_description};
use queue::{create_consumer, feeder_protocol};
use simple_logger::SimpleLogger;
use tokio::sync::mpsc;

mod bucket;
mod embeddings;
mod errors;
mod image_operations;
mod llm_messages;
mod llm_retrieval;
mod queue;
mod queue_messages;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Trace)
        // .with_threads(true)
        .init()
        .unwrap();

    let (feeder_tx, mut feeder_rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        let feeder_consumer = match create_consumer("localhost:9092") {
            Ok(f) => f,
            Err(_) => todo!(),
        };

        if let Err(_err) = feeder_protocol(feeder_consumer, vec!["minio-topic"], feeder_tx).await {
            log::error!("Error on the feeder");
            panic!();
        };
    });

    let pg_url = env!("DATABASE_URL");
    let db_pool = db_connect(pg_url).await?;
    loop {
        let msg = match feeder_rx.recv().await {
            Some(m) => m,
            None => {
                log::debug!("empty message received");
                continue;
            }
        };
        log::info!("msg {:?}", msg);

        let file_bytes = download(&msg.filename).await?;
        let i = image_from_bytes(&file_bytes)?;
        let thumbnail_512p = create_thumbnail(&i);
        // Generate embeddings from thumbnail image.
        let embeddings = get_img_embeddings(thumbnail_512p.clone())?;

        // BlobStore thumbnail image.
        let mut webp_bytes: Vec<u8> = Vec::new();
        let _ =
            thumbnail_512p.write_to(&mut Cursor::new(&mut webp_bytes), image::ImageFormat::WebP);
        let thumbnail_name = format!("rag-thumbnail/{}.webp", uuid::Uuid::new_v4().to_string());

        let _ = upload(&thumbnail_name, webp_bytes, Some("rag-upload")).await?;

        // Create db records
        // Fix the path when the proceesed_imaged bucket is done
        let img_gallery = Gallery::new(msg.filename).create(&db_pool).await?;
        // .set_thumbnail(thumbnail_name.clone());

        let mut img_embeddings = GalleryEmbeddings::new(thumbnail_name.clone(), embeddings);
        img_embeddings.create(&db_pool).await?;
        img_gallery
            .link_thumbnail(&db_pool, &thumbnail_name)
            .await?;
        img_gallery
            .link_embeddings(&db_pool, img_embeddings.id())
            .await?;

        let img_str = to_base64(&thumbnail_512p);
        let img_description = fetch_description(&img_str, ImagePrompt::Description).await?;
        let img_tags = fetch_description(&img_str, ImagePrompt::Tags).await?;
        let tags_split: Vec<String> = img_tags.split(",").map(|f| f.to_string()).collect();
        img_embeddings
            .link_genai_descriptors(&db_pool, &tags_split, &img_description)
            .await?;
    }
}
