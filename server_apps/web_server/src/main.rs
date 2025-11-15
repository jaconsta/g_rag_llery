#[macro_use]
extern crate rocket;

use serde_json::Value;
use serde_json::json;
use tokio::{signal, sync::broadcast};
use tonic::transport::Server;

mod error;
mod user_auth;

#[get("/")]
fn get_heartbeat() -> Value {
    json!({"status": "ok"})
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Shutdown handler
    let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
    // Spawn signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen to ctrl+c");
        println!("Received ctrl+c Signal. Shutdown!");
        let _ = shutdown_tx_clone.send(());
    });

    let _ = tokio::join!(
        rocket_task(shutdown_tx.subscribe()),
        tonic_task(shutdown_rx)
    );

    Ok(())
}

/// Http server
async fn rocket_task(mut shutdown_rx: broadcast::Receiver<()>) {
    // Http server
    let rocket_server = rocket::build()
        .mount("/hearbeat", routes![get_heartbeat])
        .launch();

    tokio::select! {
        _ = rocket_server => {
            println!("Stop signal comming from Rocket");
        },
        _ = shutdown_rx.recv() => {
            println!("Rocket task. Stop signal comming from system");
        }
    }
}

/// gRPC server
async fn tonic_task(mut shutdown_rx: broadcast::Receiver<()>) {
    println!("gRPC server running on port 50051.");
    let addr = "[::1]:50051".parse().expect("Failed to parse address");
    let greeter = user_auth::UserAuthGreeter::default();

    let grpc_server = Server::builder()
        .add_service(user_auth::AuthGreeterServer::new(greeter))
        .serve(addr);

    tokio::select! {
        _ = grpc_server => {
            println!("Stop signal comming from Tonic");
        },
        _ = shutdown_rx.recv() => {
            println!("Tonic task. Stop signal comming from system");
        }
    }
}
