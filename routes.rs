use actix_web::web;

use crate::handlers::*;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg
        .service(health_check)
        .service(create_wallet)
        .service(get_balance)
        .service(send_transaction)
        .service(mine_block)
        .service(get_price)
        .service(get_last_transactions)
        .service(get_revstop_status)
        .service(lock_revstop)
        .service(unlock_revstop);
}