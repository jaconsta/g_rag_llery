use std::{io::Cursor, time::Duration};

use db_storage::{
    DbConn, db_connect,
    filesystem_storage::buckets::{DownloadOpts, FilesystemBucket, UploadOpts},
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

use crate::queue_messages::ImageFeed;

mod bucket;
mod embeddings;
mod errors;
mod image_operations;
// mod llm_llava;
mod llm_messages;
mod llm_retrieval;
mod queue;
mod queue_messages;

async fn process_new_file(
    msg: ImageFeed,
    bucket_to_upload: &str,
    db_pool: &DbConn,
    genai_tx: mpsc::UnboundedSender<(DynamicImage, GalleryEmbeddings)>,
) -> Result<(), Box<dyn std::error::Error>> {
    let feeder_path = std::env::var("BUCKET_FEEDER_NAME").expect("Missing BUCKET_FEEDER_NAME");
    let ragged_path = std::env::var("BUCKET_RAGGED_NAME").expect("Missing BUCKET_RAGGED_NAME");
    let filesystem_bucket_path =
        std::env::var("FILESYSTEM_BUCKET").unwrap_or("../www-data/incoming".into());
    let fs_bucket = FilesystemBucket::new(Some(filesystem_bucket_path), Some(ragged_path));

    let down_opts = DownloadOpts::new(&msg.filename, &feeder_path);
    let file_bytes = fs_bucket.download(down_opts).await?;

    // Find the user owner of this image
    // If the upload record is not found in db. Block the process.
    let mut source_image_info = match UserUpload::get_by_filename(db_pool, &msg.filename).await {
        Ok(i) => i,
        Err(_) => {
            log::warn!("process_new_file {} notFound.", &msg.filename);
            return Ok(());
        }
    };

    let i = image_from_bytes(&file_bytes)?;
    let thumbnail_512p = create_thumbnail(&i);

    // Generate embeddings from thumbnail image.
    let embeddings = get_img_embeddings(thumbnail_512p.image().clone())?;

    // BlobStore thumbnail image.
    let mut webp_bytes: Vec<u8> = Vec::new();
    let _ = thumbnail_512p
        .image()
        .write_to(&mut Cursor::new(&mut webp_bytes), image::ImageFormat::WebP);
    let thumbnail_name = format!("thumbnail/{}.webp", uuid::Uuid::new_v4());
    let up_opts = UploadOpts::new(&thumbnail_name, webp_bytes, bucket_to_upload);
    let _ = fs_bucket.upload(up_opts).await;

    // Create db records
    let mut img_gallery = Gallery::new(&msg.filename).create(db_pool).await?;
    source_image_info
        .set_gallery_id(db_pool, img_gallery.id())
        .await?;

    let mut img_embeddings = GalleryEmbeddings::new(thumbnail_name.clone(), embeddings);
    img_embeddings.create(db_pool).await?;

    let moved_feeded_img_filepath = fs_bucket.move_to_ragged(&msg.filename).await?;

    img_gallery
        .update_with_processed(
            db_pool,
            &moved_feeded_img_filepath,
            NewThumbnail {
                path: &thumbnail_name,
                height: *thumbnail_512p.height() as i32,
                width: *thumbnail_512p.width() as i32,
                ratio: &thumbnail_512p.ratio_as_str(),
            },
            NewEmbeddings {
                embeddings_id: img_embeddings.id(),
            },
        )
        .await?;

    if let Err(e) = genai_tx.send((thumbnail_512p.image().clone(), img_embeddings)) {
        log::error!("Failed to send thumbnail to genai thread\n{e:?}");
    }

    Ok(())
}

async fn generate_image_embeddings(
    msg: (DynamicImage, GalleryEmbeddings),
    llm_to_use: &str,
    db_pool: &DbConn,
) -> Result<(), Box<dyn std::error::Error>> {
    let (img_thumbnail, img_embeddings) = msg;

    let structured = match llm_to_use {
        "openai" => {
            let img_str = to_base64(&img_thumbnail);
            fetch_description(&img_str, ImagePrompt::SemiStructured).await?
        }
        "off" => return Ok(()),
        _ => {
            // Ollama
            let ollama_str = to_llava_base64(&img_thumbnail);
            fetch_llava_description(&ollama_str, ImagePrompt::SemiStructured).await?
        }
    };

    let structures = match serde_json::from_str::<SemiStructuredMessage>(&structured) {
        Ok(s) => s,
        Err(e) => {
            log::error!("received from LLM {}. \n and error {e:?}", structured);
            return Ok(());
        }
    };

    if let Err(e) = img_embeddings
        .link_genai_descriptors(
            db_pool,
            &structures.tags,
            &structures.description,
            &structures.theme,
            &structures.alt,
            &structures.caption,
        )
        .await
    {
        println!("img_embeddings error: {e:?}")
    };

    Ok(())
}

pub async fn db_feed_protocol(
    db_pool: &DbConn,
    feed_producer: mpsc::UnboundedSender<ImageFeed>,
) -> Result<(), Box<dyn std::error::Error>> {
    let up = UserUpload::get_unprocessed(db_pool, None).await?;
    up.iter().for_each(|img| {
        let im = ImageFeed {
            filename: img.filename().clone(),
            content_type: "".to_string(),
            bucket: "".to_string(),
        };

        if let Err(err) = feed_producer.send(im) {
            log::error!("{err:?}");
        }
    });

    Ok(())
}

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

    let bucket_to_upload = std::env::var("BUCKET_RAGGED_NAME").expect("Missing BUCKET_RAGGED_NAME");

    let db_pool = db_connect(&pg_url).await?;

    let kafka_feeder_tx = feeder_tx.clone();
    tokio::spawn(async move {
        // Note, willl probably deprecate kafka consuming, while finding a way to manually trigger the
        // upload.
        let feeder_consumer = match create_consumer(&kafka_url) {
            Ok(f) => f,
            Err(_) => todo!(),
        };

        if let Err(_err) =
            feeder_protocol(feeder_consumer, vec![&kafka_topic], kafka_feeder_tx).await
        {
            log::error!("Error on the feeder");
            panic!();
        };
    });

    let feed_db_pool = db_pool.clone();
    tokio::spawn(async move {
        loop {
            if let Err(_err) = db_feed_protocol(&feed_db_pool, feeder_tx.clone()).await {
                log::error!("Error on the feeder");
                panic!();
            };
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

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
                // log::info!("msg {:?}", msg);

                match process_new_file(msg, &bucket_to_upload, &db_pool, genai_tx.clone()).await{
                    Ok(_) => (),
                    Err(e) => log::error!("{e:?}"),
                };
            },
            Some(msg) = genai_rx.recv() => {
                generate_image_embeddings(msg, &llm_to_use, &db_pool).await?;
            },
        }
    }
}
