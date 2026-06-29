#[macro_use]
extern crate rocket;

use std::sync::Arc;

use db_storage::db_connect;
use serde_json::Value;
use serde_json::json;
use simple_logger::SimpleLogger;
use tokio::sync::RwLock;
use tokio::{signal, sync::broadcast};
use tonic::transport::Server;

use crate::error::Result;
// use crate::user_auth::UserSessions;
use crate::user_auth::JwtService;

mod bucket;
mod config;
mod error;
mod gallery_view;
mod user_auth;

#[get("/")]
fn get_heartbeat() -> Value {
    json!({"status": "ok"})
}

#[rocket::main]
async fn main() -> Result<()> {
    // Init logging
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        // .with_threads(true)
        .init()
        .unwrap();

    // Load configuration
    let configs: config::Config = config::Config::default();
    let boxed = Box::new(configs);
    let config_s: &'static config::Config = Box::leak(boxed);

    // Libsodium init 
    libsodium_rs::ensure_init().expect("Failed to initialize libsodium.");

    // Shutdown handler
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    // Spawn signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c()
            .await
            .map_err(|_| "Failed to listen to ctrl+c")
            .unwrap();
        log::info!("Received ctrl+c Signal. Shutdown!");
        let _ = shutdown_tx_clone.send(());
    });

    let _ = tokio::join!(
        rocket_task(shutdown_tx.subscribe()),
        tonic_task(shutdown_rx, config_s)
    );

    Ok(())
}

/// Http server
/// Tasks: healthcheck enpoints only
async fn rocket_task(mut shutdown_rx: broadcast::Receiver<()>) {
    // Http server
    let rocket_server = rocket::build()
        .mount("/hearbeat", routes![get_heartbeat])
        .launch();

    tokio::select! {
        _ = rocket_server => {
            log::warn!("Stop signal comming from Rocket");
        },
        _ = shutdown_rx.recv() => {
            log::warn!("Rocket task. Stop signal comming from system");
        }
    }
}

/// gRPC server
/// Taks: Business tasks and app facing endpoints.
async fn tonic_task(mut shutdown_rx: broadcast::Receiver<()>, config: &'static config::Config) {
    log::info!("gRPC server running on port {}.", config.server().grpc_port());
    let addr = format!("0.0.0.0:{}", config.server().grpc_port()).parse().expect("Failed to parse address");

    let jwt_service = Arc::new(JwtService::new(config.auth()));
    let auth_rpc_service = user_auth::UserAuthGreeter::new(jwt_service.clone());
    let session_middleware = user_auth::SessionValidator::new(jwt_service);

    let db_pool = db_connect(config.db().url()).await.unwrap();
    let bucket_client = bucket::local_bucket_storage::BucketLocal::new(config.bucket()).unwrap();
    let img_gallery_rpc_service =
        gallery_view::GalleryService::new(db_pool.clone(), bucket_client, session_middleware);

    let grpc_server = Server::builder()
        .add_service(user_auth::AuthGreeterServer::new(auth_rpc_service))
        .add_service(gallery_view::GalleryViewServer::new(img_gallery_rpc_service))
        .serve(addr);

    tokio::select! {
        _ = grpc_server => {
            log::warn!("Stop signal comming from Tonic");
        },
        _ = shutdown_rx.recv() => {
            log::warn!("Tonic task. Stop signal comming from system");
        }
    };
}
