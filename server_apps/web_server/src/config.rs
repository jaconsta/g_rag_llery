use derive_getters::Getters;
use rand::distr::{Alphanumeric, SampleString};

#[derive(Getters)]
pub struct Server {
    /// Port where the GRPC server listens to.
    grpc_port: String,
}

impl Server {
    fn from_env() -> Self {
        let grpc_port = std::env::var("SERVER_GRPC_PORT").unwrap_or("4200".to_string());

        Self { grpc_port }
    }
}

#[derive(Getters)]
pub struct Database {
    /// Database connection string
    url: String,
}

impl Database {
    fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL").expect("Missing DATABASE_URL");

        Self { url }
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

        let ttl_mins = match std::env::var("AUTH_TTL_MINS")
            .unwrap_or(format!("{}", ttl_mins_default))
            .parse()
        {
            Ok(ttl) => ttl,
            Err(_) => {
                println!("AUTH_TTL_MINS accepts only numbers");
                ttl_mins_default
            }
        };
        let jwt_secret = std::env::var("AUTH_JWT_SECRET").unwrap_or(jwt_secret_default);
        let hash_seed = match u64::from_str_radix(
            std::env::var("AUTH_HASH_SEED")
                .unwrap_or(format!("{}", hash_seed_default))
                .as_str(),
            16,
        ) {
            Ok(seed) => seed,
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
    filesystem_path: String,
    /// Feeded stores unprocessed data
    feeder_bucket: String,
    /// Ragged stores curated and data after the rag processing
    ragged_bucket: String,
}

impl Bucket {
    fn from_env() -> Self {
        Self {
            filesystem_path: std::env::var("FILESYSTEM_BUCKET").unwrap_or("../www-data/incoming".into()),
            feeder_bucket: std::env::var("BUCKET_FEEDER_NAME").unwrap_or("rag_upload".into()),
            ragged_bucket: std::env::var("BUCKET_RAGGED_NAME").unwrap_or("rag_processed".into()),
        }
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
        Self {
            server: Server::from_env(),
            bucket: Bucket::from_env(),
            db: Database::from_env(),
            auth: Auth::from_env(),
        }
    }
}
