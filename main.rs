use actix_web::{web, App, HttpServer};
mod handlers;
mod routes;
mod wallet;
mod blockchain;
mod transaction;
mod revstop;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 QuantumCoin API running at http://localhost:8080");

    HttpServer::new(|| {
        App::new().configure(routes::init_routes)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}