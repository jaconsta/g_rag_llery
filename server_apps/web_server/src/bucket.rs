use http::Method;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use minio::s3::{Client, ClientBuilder};

use crate::config::Bucket as BucketConfig;
use crate::error::Result;

pub enum Bucket {
    Feeder,
    Ragged,
}

#[derive(Debug, Clone, Copy)]
struct Buckets<'a> {
    feeder: &'a str,
    ragged: &'a str,
}

#[derive(Debug, Clone)]
pub struct BucketClient<'a> {
    client: Client,
    buckets: Buckets<'a>,
    expiry_url_secs: u32,
}

impl<'a> BucketClient<'a> {
    pub fn new(config: &'a BucketConfig) -> Result<BucketClient<'a>> {
        let url: BaseUrl = config.bucket_url().parse()?;
        // .map_err(|_| "Minio bucket_url is missing.")?;
        // url.region = region;
        let credentials = StaticProvider::new(config.access_key(), config.secret_key(), None);
        let client = ClientBuilder::new(url)
            .provider(Some(Box::new(credentials)))
            .ignore_cert_check(Some(*config.ignore_ssl()))
            .build()?;
        // .map_err(|_| "Failed to create client")?;

        Ok(Self {
            client,
            buckets: Buckets {
                feeder: config.feeder_bucket(),
                ragged: config.ragged_bucket(),
            },
            expiry_url_secs: 300,
        })
    }

    pub fn bucket(&self, b: Bucket) -> &str {
        match b {
            Bucket::Feeder => self.buckets.feeder,
            Bucket::Ragged => self.buckets.ragged,
        }
    }

    pub async fn get_upload_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String> {
        let signed = self
            .client
            .get_presigned_object_url(self.bucket(bucket), filename, Method::PUT)
            .expiry_seconds(self.expiry_url_secs)
            .send()
            .await?;
        Ok(signed.url)
    }

    pub async fn get_download_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String> {
        let signed = self
            .client
            .get_presigned_object_url(self.bucket(bucket), filename, Method::GET)
            .expiry_seconds(self.expiry_url_secs)
            .send()
            .await?;
        Ok(signed.url)
    }
}
