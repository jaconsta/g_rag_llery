// use http::Method;
// use minio::s3::creds::StaticProvider;
// use minio::s3::http::BaseUrl;
// use minio::s3::{Client, ClientBuilder};

// use crate::config::Bucket as BucketConfig;
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

// #[async::trait]
trait BucketClientOperations {
    fn get_upload_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String>;
    fn get_download_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String>;
}

// // Deprecated:: DeleteMe
// #[derive(Debug, Clone)]
// pub struct BucketClient<'a> {
//     client: Client,
//     buckets: Buckets<'a>,
//     expiry_url_secs: u32,
// }
//
// impl<'a> BucketClient<'a> {
//     pub fn new(config: &'a BucketConfig) -> Result<BucketClient<'a>> {
//         let url: BaseUrl = config.bucket_url().parse()?;
//         // url.region = region;
//         let credentials = StaticProvider::new(config.access_key(), config.secret_key(), None);
//         let client = ClientBuilder::new(url)
//             .provider(Some(Box::new(credentials)))
//             .ignore_cert_check(Some(*config.ignore_ssl()))
//             .build()?;
//
//         Ok(Self {
//             client,
//             buckets: Buckets {
//                 feeder: config.feeder_bucket(),
//                 ragged: config.ragged_bucket(),
//             },
//             expiry_url_secs: 300,
//         })
//     }
//
//     pub fn bucket(&self, b: Bucket) -> &str {
//         match b {
//             Bucket::Feeder => self.buckets.feeder,
//             Bucket::Ragged => self.buckets.ragged,
//         }
//     }
//
//     pub async fn get_upload_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String> {
//         let signed = self
//             .client
//             .get_presigned_object_url(self.bucket(bucket), filename, Method::PUT)
//             .expiry_seconds(self.expiry_url_secs)
//             .send()
//             .await?;
//         Ok(signed.url)
//     }
//
//     pub async fn get_download_signed_url(&self, filename: &str, bucket: Bucket) -> Result<String> {
//         let signed = self
//             .client
//             .get_presigned_object_url(self.bucket(bucket), filename, Method::GET)
//             .expiry_seconds(self.expiry_url_secs)
//             .send()
//             .await?;
//         Ok(signed.url)
//     }
// }

pub mod local_bucket_storage {

    use db_storage::filesystem_storage::buckets;
    // use http::Method;

    use crate::config::Bucket as BucketConfig;
    use crate::error::Result;

    #[derive(Debug, Clone)]
    pub struct BucketLocal<'a> {
        client: buckets::FilesystemBucket,
        buckets: super::Buckets<'a>,
        expiry_url_secs: u32,
    }

    impl<'a> BucketLocal<'a> {
        pub fn new(config: &'a BucketConfig) -> Result<BucketLocal<'a>> {
            // Maybe this could be deleted
            let url: String = config.bucket_url().parse()?;
            let client =
                buckets::FilesystemBucket::new(Some(url), Some(config.ragged_bucket().clone()));

            Ok(Self {
                client,
                buckets: super::Buckets {
                    feeder: config.feeder_bucket(),
                    ragged: config.ragged_bucket(),
                },
                expiry_url_secs: 300,
            })
        }

        pub fn bucket(&self, b: super::Bucket) -> &str {
            match b {
                super::Bucket::Feeder => self.buckets.feeder,
                super::Bucket::Ragged => self.buckets.ragged,
            }
        }

        pub async fn get_upload_signed_url(
            &self,
            filename: &str,
            bucket: super::Bucket,
        ) -> Result<String> {
            let opts = buckets::UploadSignedUrlOpts::new(filename, self.bucket(bucket));
            Ok(self.client.get_upload_signed_url(opts).await?)
        }

        pub async fn get_download_signed_url(
            &self,
            filename: &str,
            bucket: super::Bucket,
        ) -> Result<String> {
            let opts = buckets::UploadSignedUrlOpts::new(filename, self.bucket(bucket));
            Ok(self.client.get_upload_signed_url(opts).await?)
        }
    }
}
