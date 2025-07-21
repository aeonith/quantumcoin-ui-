use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("🚀 QuantumCoin Web Server is Live!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🟢 Starting QuantumCoin Web Server...");

    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}