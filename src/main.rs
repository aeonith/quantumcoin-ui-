#[macro_use] extern crate rocket;

mod wallet;
mod blockchain;
mod routes;
mod revstop;
mod p2p; // ✅ Make sure your p2p.rs file is in src/

use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Rocket + P2P Server Initialization
#[launch]
fn rocket() -> _ {
    // ✅ Load wallet or generate new one
    let wallet = Wallet::load_from_files();

    // ✅ Load blockchain or initialize with genesis block
    let blockchain = Arc::new(Mutex::new(
        Blockchain::load_from_file()
            .unwrap_or_else(|| Blockchain::new(&wallet.public_key)),
    ));

    // ✅ Shared peer list for P2P node
    let peers = Arc::new(Mutex::new(vec![]));

    // ✅ Spawn background P2P listener on port 6000
    {
        let p2p_blockchain = blockchain.clone();
        let p2p_peers = peers.clone();
        thread::spawn(move || {
            p2p::start_node(6000, p2p_blockchain, p2p_peers);
        });
    }

    // ✅ Optional status thread (diagnostics)
    {
        let wallet_pub = wallet.public_key.clone();
        let chain = blockchain.clone();
        thread::spawn(move || loop {
            {
                let chain = chain.lock().unwrap();
                let height = chain.blocks.len();
                let mined = chain.total_mined;
                println!(
                    "🧱 Height: {} | 💰 Mined: {} QTC | 🧠 RevStop: {}",
                    height,
                    mined,
                    revstop::get_revstop_status(&wallet_pub)
                );
            }
            thread::sleep(Duration::from_secs(60));
        });
    }

    // ✅ Launch Rocket HTTP API with all state injected
    rocket::build()
        .manage(blockchain)
        .manage(peers)
        .mount("/", routes![
            routes::balance,
            routes::address,
            routes::send,
            routes::mine,
            routes::revstop_status,
            routes::export
        ])
}