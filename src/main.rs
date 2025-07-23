#![allow(unused)]
#[macro_use] extern crate rocket;

use std::sync::{Arc, Mutex};
use rocket::{Rocket, Build};
use crate::wallet::Wallet;
use crate::revstop::RevStop;
use crate::blockchain::Blockchain;
use crate::p2p::start_node;

mod wallet;
mod revstop;
mod blockchain;
mod transaction;
mod block;
mod routes;
mod p2p;
mod btc;
mod coingecko;

#[launch]
fn rocket() -> Rocket<Build> {
    let wallet = Arc::new(Mutex::new(Wallet::load_or_generate()));
    let revstop = Arc::new(Mutex::new(RevStop::status()));
    let blockchain = Arc::new(Mutex::new(Blockchain::new(wallet.clone())));

    let peers = Arc::new(Mutex::new(vec![]));

    // Start the P2P node in a separate thread
    {
        let blockchain_clone = blockchain.clone();
        let peers_clone = peers.clone();
        std::thread::spawn(move || {
            start_node(6000, blockchain_clone, peers_clone);
        });
    }

    rocket::build()
        .manage(wallet)
        .manage(revstop)
        .manage(blockchain)
        .mount("/", routes![
            routes::get_balance,
            routes::send_coins,
            routes::mine,
            routes::revstop_status,
            routes::revstop_enable,
            routes::revstop_disable,
            routes::last_transactions,
            routes::export_wallet,
            routes::get_address,
            routes::buy_qtc
        ])
}