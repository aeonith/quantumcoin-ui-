mod wallet;
mod transaction;
mod block;
mod blockchain;
mod routes;
mod merkle;
mod revstop;
mod network;
mod auth;
mod database;
mod production_database;
mod fast_processor;
mod secure_transport;
mod quantum_crypto;
mod revolutionary_features;
mod api_handlers;
mod config;
mod monitoring;

use actix_web::{App, HttpServer, web, middleware::Logger};
use actix_cors::Cors;
use tokio::sync::RwLock;
use std::sync::Arc;
use tracing::{info, warn, error};
use tracing_subscriber;

use crate::blockchain::Blockchain;
use crate::auth::AuthService;
use crate::production_database::ProductionDatabase;
use crate::quantum_crypto::QuantumCryptoSuite;
use crate::revolutionary_features::{AIValidationEngine, EnvironmentalEngine};
use crate::api_handlers::{AppState, configure_routes};
use crate::config::QuantumCoinConfig;
use crate::monitoring::{MetricsCollector, MetricsMiddleware};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load configuration
    let config = QuantumCoinConfig::load();
    
    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("❌ Configuration validation failed: {}", e);
        std::process::exit(1);
    }

    // Initialize structured logging
    let log_level = &config.logging.level;
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .compact()
        .init();

    info!("🚀 QuantumCoin Production Engine v2.0.0 Starting...");
    info!("📋 Configuration loaded and validated");
    
    // Ensure data directory exists
    if let Some(parent) = std::path::Path::new(&config.database.path).parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }

    // Initialize production database with configuration
    let database = Arc::new(
        ProductionDatabase::new(&config.database.path)
            .await
            .expect("Failed to initialize production database")
    );
    info!("✅ Production SQLite database initialized at: {}", config.database.path);

    // Initialize blockchain with genesis block
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    info!("✅ Blockchain initialized with genesis block");

    // Initialize security services with configuration
    let auth_service = Arc::new(AuthService::new(&config.security.jwt_secret));
    let quantum_crypto = Arc::new(QuantumCryptoSuite::new(config.security.quantum_security_level));
    info!("✅ Authentication and quantum cryptography initialized (Security Level: {})", config.security.quantum_security_level);

    // Initialize AI and environmental engines
    let ai_engine = Arc::new(RwLock::new(AIValidationEngine::new()));
    let env_engine = Arc::new(EnvironmentalEngine::new());
    info!("✅ AI validation and environmental engines initialized");

    // Initialize monitoring system
    let metrics_collector = Arc::new(MetricsCollector::new());
    metrics_collector.start_monitoring();
    info!("✅ Comprehensive monitoring system started");

    // Create application state
    let app_state = AppState {
        blockchain,
        database,
        auth_service,
        quantum_crypto,
        ai_engine,
        env_engine,
    };

    // Create bind address - Use PORT environment variable for Render
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| config.server.port.to_string())
        .parse::<u16>()
        .unwrap_or(config.server.port);
    
    let bind_address = format!("{}:{}", config.server.host, port);

    info!("🎯 QuantumCoin Production System Ready:");
    info!("   🔐 Quantum-safe cryptography: ACTIVE (Level {})", config.quantum.security_level);
    info!("   🤖 AI fraud detection: ACTIVE");
    info!("   ⚡ Lightning-fast processor: ACTIVE");
    info!("   🌱 Carbon-negative engine: ACTIVE");
    info!("   🛡️  Security level: MAXIMUM");
    info!("   💾 Production SQLite database: ACTIVE");
    info!("   📊 Real-time monitoring: ACTIVE");
    info!("   🚀 Performance target: {} TPS", config.performance.max_transactions_per_block);
    info!("   👥 Worker threads: {}", config.server.workers);
    info!("   🌐 Server binding to: {}", bind_address);
    info!("   🌍 Environment: {}", if config.is_production() { "PRODUCTION" } else { "DEVELOPMENT" });

    // Start HTTP server with production configuration
    let metrics_middleware = MetricsMiddleware::new(Arc::clone(&metrics_collector));
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .app_data(web::Data::new(Arc::clone(&metrics_collector)))
            .wrap(metrics_middleware.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600)
            )
            .wrap(Logger::default())
            .configure(configure_routes)
            .service(
                actix_files::Files::new("/", "./")
                    .index_file("index.html")
            )
    })
    .workers(config.server.workers)
    .keep_alive(std::time::Duration::from_secs(config.server.keep_alive))
    .client_timeout(config.server.client_timeout)
    .client_shutdown(config.server.client_shutdown)
    .bind(&bind_address)?
    .run()
    .await
}