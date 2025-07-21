use std::net::TcpListener;
use std::io::Write;
use quantumcoin::wallet::Wallet;

fn main() {
    // Attempt to load wallet or generate new one
    let wallet = Wallet::load_from_files("wallet").unwrap_or_else(|| {
        let w = Wallet::generate();
        w.save_to_files("wallet");
        w
    });

    let address = wallet.get_address();

    // Launch minimal web server
    let listener = TcpListener::bind("0.0.0.0:8080").expect("Failed to bind to port 8080");
    println!("🚀 QuantumCoin Web Server Running");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                <html><body style='background-color: black; color: white; font-family: monospace;'>\
                <h3>QuantumCoin Wallet Address:</h3>\
                <p>{}</p>\
                </body></html>", address
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}