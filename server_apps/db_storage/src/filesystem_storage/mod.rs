use std::path::Path;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::errors::BucketError;

/***
 * The purpose of this file is to create a simple alternative for a local deployment.
 *
 * After some research on self-hostable alternatives that support the base features here used.
 *   - Block storage,
 *   - (Possible) creation log,
 *   - Signed urls,
 *   - Queue event notifications (preferably kafka, since it's the one implemented form the
 *   beginning ),
 *
 * The few findings didn't have all features or are no longer in the open-source effort.
 *
 * This does not attempt to be a full-featured replacement of bucket storage service.
 * But rather a feature-less version of it. Designed for local deployment with little to none
 * internet client access, offering the minimum required.
 *
 * This also asumes that all server-users will be hosted in the same machine, sharing the same drive.
 ***/

/// Bunch of wrapper functions around file system operations
/// All file destinations are relative to be the base_path provided.
#[derive(Debug, Clone)]
pub struct LocalBlock {
    base_path: String,
}

impl LocalBlock {
    /// Initialize a new LocalBlock
    pub fn new(base_path: String) -> LocalBlock {
        LocalBlock { base_path }
    }

    /// Add a new file.
    pub async fn insert_file(&self, file_name: &str, data: &[u8]) -> Result<(), BucketError> {
        let path = Path::new(&self.base_path).join(file_name);
        let mut file = File::create_new(path).await?;

        let _ = file.write_all(data).await;

        Ok(())
    }

    /// Get the contents of the file.
    pub async fn read_file(&self, file_name: &str) -> Result<Vec<u8>, BucketError> {
        let path = Path::new(&self.base_path).join(file_name);
        let data = fs::read(path).await?;

        Ok(data)
    }

    /// Move (rename) a file.
    pub async fn move_file(
        &self,
        filename: &str,
        into_bucket: &str,
    ) -> Result<(), BucketError> {
        let from_path = Path::new(&self.base_path)
            .join(filename);
        let into_path = Path::new(into_bucket) 
            .join(filename);

        Ok(fs::rename(from_path, into_path).await?)
    }

    pub async fn delete_file(&self, filename: &str) -> Result<(), BucketError> {
        let path = Path::new(&self.base_path).join(filename);
        fs::remove_file(path).await?;

        Ok(())
    }
}

pub mod buckets {
    // use core::hash;
    use std::path::Path;

    use super::LocalBlock;
    use crate::errors::BucketError;

    pub struct UploadOpts<'a> {
        filename: &'a str,
        bytes: Vec<u8>,
        bucket: &'a str,
    }
    impl<'a> UploadOpts<'a> {
        pub fn new(filename: &'a str, bytes: Vec<u8>, bucket: &'a str) -> UploadOpts<'a> {
            Self {
                filename,
                bytes,
                bucket,
            }
        }
    }

    pub struct DownloadOpts<'a> {
        filename: &'a str,
        bucket: &'a str,
    }
    impl<'a> DownloadOpts<'a> {
        pub fn new(filename: &'a str, bucket: &'a str) -> DownloadOpts<'a> {
            Self { filename, bucket }
        }
    }

    pub struct UploadSignedUrlOpts<'a> {
        filename: &'a str,
        bucket: &'a str,
    }
    impl<'a> UploadSignedUrlOpts<'a> {
        pub fn new(filename: &'a str, bucket: &'a str) -> UploadSignedUrlOpts<'a> {
            Self {
                filename,
                bucket,
            }
        }
    }

    trait BucketOperations {
        fn upload(opts: UploadOpts<'_>) -> Result<(), BucketError>;
        fn download(opts: DownloadOpts<'_>) -> Result<Vec<u8>, BucketError>;
        fn move_to_ragged(filename: &str) -> Result<(),BucketError>;
    }

    #[derive(Debug, Clone)]
    pub struct FilesystemBucket {
        bucket: LocalBlock,
        // Used to determine the destination of files when moved.
        ragged_path: Option<String>,
    }

    impl FilesystemBucket {
        pub fn new(base_path: Option<String>, ragged_path: Option<String>) -> Self {
            let base_path = base_path.unwrap_or("/tmp/g_rag_lerry".into());
            FilesystemBucket {
                bucket: LocalBlock::new(base_path),
                ragged_path,
            }
        }

        fn build_path(&self, bucket: &str, filename: &str) -> Result<String, BucketError> {
            let path = match Path::new(bucket).join(filename).to_str() {
                Some(p) => p.to_owned(),
                None => return Err(BucketError::MalformedPath),
            };
            Ok(path)
        }
        pub async fn upload(&self, opts: UploadOpts<'_>) -> Result<(), BucketError> {
            let path = self.build_path(opts.bucket, opts.filename)?;
            let _ = self.bucket.insert_file(&path, &opts.bytes).await;

            Ok(())
        }

        pub async fn download(&self, opts: DownloadOpts<'_>) -> Result<Vec<u8>, BucketError> {
            let path = self.build_path(opts.bucket, opts.filename)?;
            self.bucket.read_file(&path).await
        }

        /// Move the file into a the ragged bucket, returns the destination path.
        pub async fn move_to_ragged(
            &self,
            filename: &str,
        ) -> Result<String, BucketError> {
            if let Some(into_bucket) = &self.ragged_path {
                let into_path = self.build_path(into_bucket, filename)?;

                let _ = self.bucket.move_file(filename, &into_path).await;
                Ok(into_path)
            } else {
                Err(BucketError::Str)
            }
        }

        // Reference for a local nginx that serves as the a party storage service.
        // https://stackoverflow.com/questions/43241067/upload-files-to-nginx#70432807
        pub async fn get_upload_signed_url(&self, opts: UploadSignedUrlOpts<'_>) -> Result<String, BucketError>{
                let into_path = self.build_path(opts.bucket, opts.filename)?;
                Ok(into_path)
        }

        pub async fn get_download_signed_url(&self, opts: UploadSignedUrlOpts<'_>) -> Result<String, BucketError>{
            self.get_upload_signed_url(opts).await
                //let into_path = self.build_path(opts.bucket, opts.filename)?;
                //Ok(into_path)
        }
    }
}
