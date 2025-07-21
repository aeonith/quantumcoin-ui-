mod cli;

use std::net::SocketAddr;
use warp::Filter;

#[tokio::main]
async fn main() {
    // ✅ Run CLI interface
    cli::start_cli();

    // ✅ Set up simple HTTP server so Render detects the port
    let route = warp::path::end().map(|| "🚀 QuantumCoin Web Server Running");

    let addr: SocketAddr = ([0, 0, 0, 0], 8080).into();
    println!("🌐 Web server listening at http://{}", addr);

    // 🔁 Start web server
    warp::serve(route).run(addr).await;
}