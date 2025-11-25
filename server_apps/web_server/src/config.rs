use crate::error::Result;
use derive_getters::Getters;

#[derive(Getters)]
pub struct Database {
    url: String,
}

impl Database {
    fn from_env() -> Self {
        let pg_url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");

        Self { url: pg_url }
    }
}

#[derive(Getters)]
pub struct Bucket {
    // The following are parameters to connect
    ignore_ssl: bool,
    bucket_url: String,
    access_key: String,
    secret_key: String,
    // The following are bucket names for operations
    /// Feeded stores unprocessed data
    feeder_bucket: String,
    /// Ragged stores curated and data after the rag processing
    ragged_bucket: String,
}

impl Bucket {
    fn from_env() -> Result<Self> {
        let bucket_check_ssl = std::env::var("MINIO_CHECK_SSL").unwrap_or("true".to_string());
        let ignore_ssl = bucket_check_ssl == "false";

        Ok(Self {
            ignore_ssl,
            bucket_url: std::env::var("MINIO_BUCKET_URL")?,
            access_key: std::env::var("MINIO_ACCESS_KEY")?,
            secret_key: std::env::var("MINIO_SECRET_KEY")?,
            feeder_bucket: std::env::var("BUCKET_FEEDER_NAME")?,
            ragged_bucket: std::env::var("BUCKET_RAGGED_NAME")?,
        })
    }
}

#[derive(Getters)]
pub struct Config {
    bucket: Bucket,
    db: Database,
}

impl Default for Config {
    fn default() -> Self {
        // Eventually solve the unwraps
        Self {
            bucket: Bucket::from_env().unwrap(),
            db: Database::from_env(),
        }
    }
}
