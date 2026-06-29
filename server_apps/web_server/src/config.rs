use crate::error::Result;
use derive_getters::Getters;
use rand::distr::{Alphanumeric, SampleString};

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
pub struct Auth {
    /// Token and session expiry. In minutes.
    ttl_mins: u64,
    /// Signature secret for the jwt.
    jwt_secret: String,
    /// Seed for hash map key and user_id key.
    hash_seed: u64,
}

impl Auth {
    fn from_env() -> Self {
        let jwt_secret_default = Alphanumeric.sample_string(&mut rand::rng(), 32);
        let ttl_mins_default = 1200;
        let hash_seed_default = 0xdead_cafe;

        let ttl_mins = match std::env::var("AUTH_TTL_MINS").unwrap_or(format!("{}", ttl_mins_default)).parse(){
            Ok(ttl)=> ttl,
            Err(_) => {
                println!("AUTH_TTL_MINS accepts only numbers");
                ttl_mins_default
            }
        };
        let jwt_secret = std::env::var("AUTH_JWT_SECRET").unwrap_or(jwt_secret_default);
        let hash_seed = match u64::from_str_radix(std::env::var("AUTH_HASH_SEED").unwrap_or(format!("{}", hash_seed_default)).as_str(), 16) {
            Ok(seed)=> seed,
            Err(_) => {
                println!("AUTH_HASH_SEED accepts only hexadecimal numbers. ie: 0x1234_cdef");
                hash_seed_default
            }
        };

        Self { 
            ttl_mins,
            jwt_secret,
            hash_seed,
        }
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
        })
    }
}

#[derive(Getters)]
pub struct Config {
    server: Server,
    bucket: Bucket,
    db: Database,
    auth: Auth,
}

impl Default for Config {
    fn default() -> Self {
        // Eventually solve the unwraps
        Self {
            server: Server::from_env(),
            bucket: Bucket::from_env().unwrap(),
            db: Database::from_env(),
            auth: Auth::from_env(),
        }
    }
}
