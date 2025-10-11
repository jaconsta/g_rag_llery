use futures_util::StreamExt;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::types::S3Api;
use minio::s3::{Client, ClientBuilder};

use crate::errors::BucketOperationsError;

pub fn b3_client() -> Client {
    // let region = std::env::var("MINIO_REGION").unwrap_or("true".to_string());
    let bucket_check_ssl = std::env::var("MINIO_CHECK_SSL").unwrap_or("true".to_string());
    let ignore_ssl = Some(bucket_check_ssl == "false");

    let bucket_url =
        std::env::var("MINIO_BUCKET_URL").expect("Minio {MINIO_BUCKET_URL} is missing.");
    let access_key =
        std::env::var("MINIO_ACCESS_KEY").expect("Minio {MINIO_ACCESS_KEY} is missing.");
    let secret_key =
        std::env::var("MINIO_SECRET_KEY").expect("Minio {MINIO_SECRET_KEY} is missing.");
    let url: BaseUrl = bucket_url.parse().expect("Minio bucket_url is missing.");
    // url.region = region;
    let credentials = StaticProvider::new(&access_key, &secret_key, None);
    let client = ClientBuilder::new(url)
        .provider(Some(Box::new(credentials)))
        .ignore_cert_check(ignore_ssl)
        .build()
        .expect("Failed to create client");
    client
}

pub async fn upload(
    filename: &str,
    bytes: Vec<u8>,
    bucket: Option<&str>,
) -> Result<(), BucketOperationsError> {
    let client = b3_client();
    let bucket_name = std::env::var("MINIO_BUCKET_NAME").unwrap_or("rag-upload".to_string());
    let bucket = bucket.unwrap_or(&bucket_name);
    client
        .put_object_content(bucket, filename, bytes)
        .send()
        .await
        .map_err(|e| {
            log::error!("{e:#?}");
            BucketOperationsError::BlobUpload
        })?;
    Ok(())
}

pub async fn download(filename: &str) -> Result<Vec<u8>, BucketOperationsError> {
    let bucket_name = std::env::var("MINIO_BUCKET_NAME").unwrap_or("rag-upload".to_string());
    let client = b3_client();

    let file_obj = client
        .get_object(&bucket_name, filename)
        .send()
        .await
        .map_err(|_| BucketOperationsError::BlobDownload)?;

    let mut file_stream = file_obj
        .content
        .to_stream()
        .await
        .map_err(|_| BucketOperationsError::BlobRead)?;

    // Collect the stream into bytes.
    let mut body = Vec::new();
    while let Some(stream) = file_stream.0.next().await {
        let stream = stream.map_err(|_| BucketOperationsError::BlobRead)?;
        body.push(stream);
    }

    let bu = body.concat();
    Ok(bu)
}

#[cfg(test)]
mod tests {
    use minio::s3::types::S3Api;
    use reqwest::header::USER_AGENT;

    use crate::bucket::{b3_client, download};

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_bucket_connection() {
        let bucket_name = std::env::var("MINIO_BUCKET_NAME").unwrap_or("rag-upload".to_string());
        let client = b3_client();
        println!(
            "Regioness \n{:?}\n---",
            client.get_region(&bucket_name).send().await
        );

        let exists = match client.bucket_exists(&bucket_name).send().await {
            Err(err) => {
                println!("Existess err \n{:?}\n---", err);
                assert!(false);
                return;
            }
            Ok(good) => {
                println!("Existess good \n{:?}\n---", good);
                println!("Existess exists \n{:?}\n---", good.exists);
                good.exists
            }
        };

        if !exists {
            match client.create_bucket(&bucket_name).send().await {
                Err(err) => {
                    println!("Createss err \n{:?}\n---", err);
                    assert!(false);
                    return;
                }
                Ok(good) => {
                    println!("Createss good \n{:?}\n---", good);
                }
            }
        }

        let exists = client.bucket_exists(bucket_name).send().await.unwrap();

        assert!(exists.exists, "Bucket does not exist");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_bucket_upload() {
        let client = b3_client();
        let url = "https://upload.wikimedia.org/wikipedia/commons/thumb/8/89/Portrait_Placeholder.png/120px-Portrait_Placeholder.png";
        let req_client = reqwest::Client::new();
        let image_response = req_client
            .get(url)
            .header(USER_AGENT, "jaconsta_gallery 1.0")
            .send()
            .await
            .unwrap();

        let bytes = image_response.bytes().await.unwrap();

        let bucket_name = std::env::var("MINIO_BUCKET_NAME").unwrap_or("rag-upload".to_string());
        let filename = "placeholder.png";
        client
            .put_object_content(&bucket_name, filename, bytes.to_vec())
            .send()
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_bucket_download() {
        let filename = "placeholder.png";
        assert!(download(filename).await.is_ok());
    }
}
