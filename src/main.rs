mod block;
mod blockchain;
mod transaction;
mod wallet;
mod revstop;
mod routes;

use blockchain::Blockchain;
use rocket::tokio::sync::Mutex;
use std::sync::Arc;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let wallet = Arc::new(Mutex::new(wallet::Wallet::new()));
    let revstop = Arc::new(Mutex::new(revstop::RevStop::new()));

    rocket::build()
        .manage(blockchain)
        .manage(wallet)
        .manage(revstop)
        .mount("/", routes::routes())
}