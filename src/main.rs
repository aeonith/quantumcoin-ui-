mod wallet;
mod blockchain;
mod routes;
mod revstop;
mod p2p;

use crate::wallet::Wallet;
use crate::blockchain::Blockchain;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let wallet = Wallet::load_from_files();

    let blockchain = Arc::new(Mutex::new(
        Blockchain::load_from_file()
            .unwrap_or_else(|| Blockchain::new(&wallet.public_key)),
    ));

    let peers = Arc::new(Mutex::new(vec![]));

    {
        let blockchain_clone = blockchain.clone();
        let peers_clone = peers.clone();
        thread::spawn(move || {
            p2p::start_node(6000, blockchain_clone, peers_clone);
        });
    }

    {
        let pubkey = wallet.public_key.clone();
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
                    revstop::get_revstop_status(&pubkey)
                );
            }
            thread::sleep(Duration::from_secs(60));
        });
    }

    rocket::build()
        .manage(blockchain)
        .manage(peers)
        .mount("/", routes::get_routes())
        .launch()
        .await
}