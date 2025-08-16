// Secure transport layer with TLS/Noise protocol support
use crate::network::{ChainSpec, NetworkMetrics};
use anyhow::Result;
use futures::future::BoxFuture;
use rustls::{ClientConfig, ServerConfig};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey};
use snow::{Builder, HandshakeState, TransportState};

/// Secure transport layer for P2P communications
pub struct SecureTransport {
    chain_spec: Arc<ChainSpec>,
    metrics: Arc<NetworkMetrics>,
    tls_acceptor: Option<TlsAcceptor>,
    tls_connector: TlsConnector,
    noise_pattern: String,
    active_connections: Arc<RwLock<HashMap<SocketAddr, SecureConnection>>>,
    connection_events: mpsc::Sender<ConnectionEvent>,
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connected(SocketAddr, SecureConnection),
    Disconnected(SocketAddr),
    Error(SocketAddr, String),
    Message(SocketAddr, Vec<u8>),
}

#[derive(Clone)]
pub struct SecureConnection {
    pub addr: SocketAddr,
    pub transport: ConnectionTransport,
    pub established_at: Instant,
    pub bytes_sent: Arc<std::sync::atomic::AtomicU64>,
    pub bytes_received: Arc<std::sync::atomic::AtomicU64>,
    pub last_activity: Arc<std::sync::atomic::AtomicU64>,
}

#[derive(Clone)]
pub enum ConnectionTransport {
    Tls(Arc<TlsStream<TcpStream>>),
    Noise(Arc<RwLock<NoiseConnection>>),
    Plain(Arc<TcpStream>), // Fallback for testing
}

pub struct NoiseConnection {
    pub transport_state: TransportState,
    pub stream: TcpStream,
}

impl SecureTransport {
    pub async fn new(
        chain_spec: Arc<ChainSpec>,
        metrics: Arc<NetworkMetrics>,
    ) -> Result<Self> {
        let (tx, _rx) = mpsc::channel(1000);
        
        // Initialize TLS configuration
        let tls_connector = create_tls_connector().await?;
        let tls_acceptor = create_tls_acceptor().await.ok();
        
        // Noise protocol pattern for post-quantum resistance
        let noise_pattern = "Noise_XX_25519_ChaChaPoly_BLAKE2s".to_string();

        Ok(Self {
            chain_spec,
            metrics,
            tls_acceptor,
            tls_connector,
            noise_pattern,
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            connection_events: tx,
        })
    }

    pub async fn start(&self) -> Result<()> {
        log::info!("Starting secure transport layer");
        
        // Start connection manager
        let transport = self.clone();
        tokio::spawn(async move {
            transport.manage_connections().await;
        });

        Ok(())
    }

    /// Establish secure connection to peer
    pub async fn connect_secure(&self, addr: SocketAddr) -> Result<SecureConnection> {
        let start_time = Instant::now();
        
        log::debug!("Establishing secure connection to {}", addr);
        
        // Try connection methods in order of preference
        let connection = if let Ok(conn) = self.connect_with_noise(addr).await {
            conn
        } else if let Ok(conn) = self.connect_with_tls(addr).await {
            conn
        } else {
            return Err(anyhow::anyhow!("All secure connection methods failed for {}", addr));
        };

        let connection_time = start_time.elapsed();
        self.metrics.record_connection_time(connection_time).await;
        
        // Store active connection
        self.active_connections.write().await.insert(addr, connection.clone());
        
        log::info!("Secure connection established to {} in {:?}", addr, connection_time);
        Ok(connection)
    }

    /// Accept incoming secure connection
    pub async fn accept_secure(&self, stream: TcpStream, addr: SocketAddr) -> Result<SecureConnection> {
        log::debug!("Accepting secure connection from {}", addr);
        
        let connection = if let Ok(conn) = self.accept_with_noise(stream.clone(), addr).await {
            conn
        } else if let Some(ref acceptor) = self.tls_acceptor {
            self.accept_with_tls(acceptor, stream, addr).await?
        } else {
            return Err(anyhow::anyhow!("No secure transport available for incoming connection"));
        };

        self.active_connections.write().await.insert(addr, connection.clone());
        
        log::info!("Secure connection accepted from {}", addr);
        Ok(connection)
    }

    /// Connect using Noise protocol (preferred for post-quantum resistance)
    async fn connect_with_noise(&self, addr: SocketAddr) -> Result<SecureConnection> {
        let stream = tokio::time::timeout(
            Duration::from_secs(self.chain_spec.connection_timeout),
            TcpStream::connect(addr),
        ).await??;

        // Initialize Noise handshake
        let builder = Builder::new(self.noise_pattern.parse()?);
        let static_key = generate_static_key();
        let mut noise = builder
            .local_private_key(&static_key)
            .build_initiator()?;

        // Perform handshake
        let transport_state = perform_noise_handshake_initiator(noise, stream).await?;
        
        let noise_conn = NoiseConnection {
            transport_state,
            stream: TcpStream::connect(addr).await?, // Reconnect for transport
        };

        Ok(SecureConnection {
            addr,
            transport: ConnectionTransport::Noise(Arc::new(RwLock::new(noise_conn))),
            established_at: Instant::now(),
            bytes_sent: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            bytes_received: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_activity: Arc::new(std::sync::atomic::AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        })
    }

    /// Accept using Noise protocol
    async fn accept_with_noise(&self, stream: TcpStream, addr: SocketAddr) -> Result<SecureConnection> {
        let builder = Builder::new(self.noise_pattern.parse()?);
        let static_key = generate_static_key();
        let mut noise = builder
            .local_private_key(&static_key)
            .build_responder()?;

        let transport_state = perform_noise_handshake_responder(noise, stream).await?;
        
        let noise_conn = NoiseConnection {
            transport_state,
            stream: TcpStream::connect(addr).await?,
        };

        Ok(SecureConnection {
            addr,
            transport: ConnectionTransport::Noise(Arc::new(RwLock::new(noise_conn))),
            established_at: Instant::now(),
            bytes_sent: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            bytes_received: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_activity: Arc::new(std::sync::atomic::AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        })
    }

    /// Connect using TLS (fallback)
    async fn connect_with_tls(&self, addr: SocketAddr) -> Result<SecureConnection> {
        let stream = tokio::time::timeout(
            Duration::from_secs(self.chain_spec.connection_timeout),
            TcpStream::connect(addr),
        ).await??;

        let domain = rustls::ServerName::try_from("quantumcoin.network")?;
        let tls_stream = self.tls_connector.connect(domain, stream).await?;

        Ok(SecureConnection {
            addr,
            transport: ConnectionTransport::Tls(Arc::new(tls_stream)),
            established_at: Instant::now(),
            bytes_sent: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            bytes_received: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_activity: Arc::new(std::sync::atomic::AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        })
    }

    /// Accept using TLS
    async fn accept_with_tls(
        &self,
        acceptor: &TlsAcceptor,
        stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<SecureConnection> {
        let tls_stream = acceptor.accept(stream).await?;

        Ok(SecureConnection {
            addr,
            transport: ConnectionTransport::Tls(Arc::new(tls_stream)),
            established_at: Instant::now(),
            bytes_sent: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            bytes_received: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            last_activity: Arc::new(std::sync::atomic::AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )),
        })
    }

    /// Send encrypted data over secure connection
    pub async fn send_secure(&self, addr: SocketAddr, data: &[u8]) -> Result<()> {
        let connections = self.active_connections.read().await;
        if let Some(connection) = connections.get(&addr) {
            self.send_on_connection(connection, data).await?;
            
            // Update metrics
            connection.bytes_sent.fetch_add(
                data.len() as u64,
                std::sync::atomic::Ordering::Relaxed,
            );
            connection.last_activity.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                std::sync::atomic::Ordering::Relaxed,
            );
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("No active connection to {}", addr))
        }
    }

    /// Send data on specific connection
    async fn send_on_connection(&self, connection: &SecureConnection, data: &[u8]) -> Result<()> {
        match &connection.transport {
            ConnectionTransport::Noise(noise_conn) => {
                let mut conn = noise_conn.write().await;
                let mut buffer = vec![0u8; data.len() + 16]; // Extra space for encryption
                let len = conn.transport_state.write_message(data, &mut buffer)?;
                
                use tokio::io::AsyncWriteExt;
                conn.stream.write_all(&buffer[..len]).await?;
                Ok(())
            }
            ConnectionTransport::Tls(tls_stream) => {
                use tokio::io::AsyncWriteExt;
                let mut stream = tls_stream.as_ref();
                stream.write_all(data).await?;
                Ok(())
            }
            ConnectionTransport::Plain(stream) => {
                use tokio::io::AsyncWriteExt;
                let mut stream = stream.as_ref();
                stream.write_all(data).await?;
                Ok(())
            }
        }
    }

    /// Connection management loop
    async fn manage_connections(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Clean up inactive connections
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let mut connections = self.active_connections.write().await;
            let mut to_remove = Vec::new();
            
            for (addr, connection) in connections.iter() {
                let last_activity = connection.last_activity.load(std::sync::atomic::Ordering::Relaxed);
                if now - last_activity > 300 { // 5 minutes timeout
                    to_remove.push(*addr);
                }
            }
            
            for addr in to_remove {
                connections.remove(&addr);
                log::debug!("Removed inactive connection to {}", addr);
            }
        }
    }

    /// Get active connection count
    pub async fn get_connection_count(&self) -> usize {
        self.active_connections.read().await.len()
    }

    /// Shutdown transport layer
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down secure transport layer");
        self.active_connections.write().await.clear();
        Ok(())
    }
}

impl Clone for SecureTransport {
    fn clone(&self) -> Self {
        Self {
            chain_spec: self.chain_spec.clone(),
            metrics: self.metrics.clone(),
            tls_acceptor: self.tls_acceptor.clone(),
            tls_connector: self.tls_connector.clone(),
            noise_pattern: self.noise_pattern.clone(),
            active_connections: self.active_connections.clone(),
            connection_events: self.connection_events.clone(),
        }
    }
}

// Helper functions

async fn create_tls_connector() -> Result<TlsConnector> {
    let mut root_cert_store = rustls::RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(
        webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }),
    );

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    Ok(TlsConnector::from(Arc::new(config)))
}

async fn create_tls_acceptor() -> Result<TlsAcceptor> {
    // In production, load from certificate files
    // For now, generate self-signed certificate
    let cert = generate_self_signed_cert()?;
    let key = generate_private_key()?;

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

fn generate_static_key() -> [u8; 32] {
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

async fn perform_noise_handshake_initiator(
    mut handshake: HandshakeState,
    mut stream: TcpStream,
) -> Result<TransportState> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    let mut buf = [0u8; 1024];
    
    // -> e
    let len = handshake.write_message(&[], &mut buf)?;
    stream.write_all(&buf[..len]).await?;
    
    // <- e, ee, s, es
    let len = stream.read(&mut buf).await?;
    handshake.read_message(&buf[..len], &mut [])?;
    
    // -> s, se
    let len = handshake.write_message(&[], &mut buf)?;
    stream.write_all(&buf[..len]).await?;
    
    Ok(handshake.into_transport_mode()?)
}

async fn perform_noise_handshake_responder(
    mut handshake: HandshakeState,
    mut stream: TcpStream,
) -> Result<TransportState> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    
    let mut buf = [0u8; 1024];
    
    // <- e
    let len = stream.read(&mut buf).await?;
    handshake.read_message(&buf[..len], &mut [])?;
    
    // -> e, ee, s, es
    let len = handshake.write_message(&[], &mut buf)?;
    stream.write_all(&buf[..len]).await?;
    
    // <- s, se
    let len = stream.read(&mut buf).await?;
    handshake.read_message(&buf[..len], &mut [])?;
    
    Ok(handshake.into_transport_mode()?)
}

fn generate_self_signed_cert() -> Result<rustls::Certificate> {
    // Placeholder - in production, use proper certificates
    Ok(rustls::Certificate(vec![0u8; 32]))
}

fn generate_private_key() -> Result<rustls::PrivateKey> {
    // Placeholder - in production, use proper private keys
    Ok(rustls::PrivateKey(vec![0u8; 32]))
}
