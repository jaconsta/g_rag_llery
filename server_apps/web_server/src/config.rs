use crate::error::Result;
use derive_getters::Getters;

#[derive(Getters)]
pub struct Server {
    grpc_port: String,
}

impl Server{
    fn from_env() -> Self {
        let grpc_port= std::env::var("SERVER_GRPC_PORT").unwrap_or("4200".to_string());

        Self { grpc_port }
    }
}

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
    #[allow(dead_code)]
    // The following are parameters to connect
    ignore_ssl: bool,
    bucket_url: String,
    #[allow(dead_code)]
    access_key: String,
    #[allow(dead_code)]
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

        // Note, do I need it? db_storage knows about the bucket
        Ok(Self {
            ignore_ssl,
            bucket_url: String::from("delme"),
            access_key: String::from("delme"),
            secret_key: String::from("delme"),
            feeder_bucket:std::env::var("BUCKET_FEEDER_NAME")?,
            ragged_bucket: std::env::var("BUCKET_RAGGED_NAME")?,

    //         ignore_ssl,
    //         bucket_url: std::env::var("MINIO_BUCKET_URL")?,
    //         access_key: std::env::var("MINIO_ACCESS_KEY")?,
    //         secret_key: std::env::var("MINIO_SECRET_KEY")?,
    //         feeder_bucket: std::env::var("BUCKET_FEEDER_NAME")?,
    //         ragged_bucket: std::env::var("BUCKET_RAGGED_NAME")?,
        })
    }
}

#[derive(Getters)]
pub struct Config {
    server: Server,
    bucket: Bucket,
    db: Database,
}

impl Default for Config {
    fn default() -> Self {
        // Eventually solve the unwraps
        Self {
            server: Server::from_env(),
            bucket: Bucket::from_env().unwrap(),
            db: Database::from_env(),
        }
    }
}
