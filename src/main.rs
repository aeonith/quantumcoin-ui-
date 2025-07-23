#[macro_use] extern crate rocket;

mod wallet;
mod blockchain;
mod transaction;
mod revstop;
mod routes;
mod p2p;
mod btc;
mod coingecko;

use rocket::tokio::sync::Mutex;
use std::sync::Arc;
use wallet::Wallet;
use blockchain::Blockchain;

#[launch]
fn rocket() -> _ {
    let wallet = Arc::new(Mutex::new(Wallet::load_or_create()));
    let blockchain = Arc::new(Mutex::new(Blockchain::load_or_create()));
    let revstop = Arc::new(Mutex::new(revstop::RevStop::load_status()));
    let peers = Arc::new(Mutex::new(Vec::new()));

    std::thread::spawn({
        let blockchain = Arc::clone(&blockchain);
        let peers = Arc::clone(&peers);
        move || p2p::start_node(3030, blockchain, peers)
    });

    rocket::build()
        .manage(wallet)
        .manage(blockchain)
        .manage(revstop)
        .manage(peers)
        .mount("/", routes![
            routes::health,
            routes::address,
            routes::balance,
            routes::send,
            routes::mine,
            routes::btc_payment,
            routes::revstop_status,
            routes::enable_revstop,
            routes::disable_revstop
        ])
}