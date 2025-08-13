#[cfg(feature = "explorer")]
pub mod explorer {
    use axum::{routing::get, Router};
    use std::net::SocketAddr;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Height { height: u64 }

    async fn get_height() -> axum::Json<Height> {
        // TODO: read real height from global state/handle
        axum::Json(Height{ height: 0 })
    }

    pub async fn run_http(addr: &str) {
        let app = Router::new()
            .route("/api/height", get(get_height));
        let socket: SocketAddr = addr.parse().expect("addr");
        axum::Server::bind(&socket).serve(app.into_make_service())
            .await.expect("server");
    }
}
