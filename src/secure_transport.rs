use tokio_rustls::{TlsAcceptor, TlsConnector, server::TlsStream as ServerTlsStream, client::TlsStream as ClientTlsStream};
use rustls::{Certificate, PrivateKey, ServerConfig, ClientConfig, RootCertStore};
use webpki_roots;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead, generic_array::GenericArray}};
use rand::Rng;
use tokio::net::TcpStream;
use std::sync::Arc;
use std::io;
use thiserror::Error;
use base64::{encode, decode};
use pqcrypto_dilithium::dilithium2::{keypair, sign_detached, verify_detached_signature, PublicKey, SecretKey, DetachedSignature};

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("TLS handshake failed: {0}")]
    TlsHandshakeFailed(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub struct SecureTransport {
    tls_acceptor: Option<TlsAcceptor>,
    tls_connector: Option<TlsConnector>,
    encryption_key: Option<Key<Aes256Gcm>>,
    signing_key: SecretKey,
    verify_key: PublicKey,
}

impl SecureTransport {
    pub fn new() -> Result<Self, TransportError> {
        let (public_key, secret_key) = keypair();
        
        Ok(Self {
            tls_acceptor: None,
            tls_connector: None,
            encryption_key: None,
            signing_key: secret_key,
            verify_key: public_key,
        })
    }

    pub fn with_tls_server(mut self, cert_chain: Vec<Certificate>, private_key: PrivateKey) -> Result<Self, TransportError> {
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, private_key)
            .map_err(|e| TransportError::InvalidCertificate(e.to_string()))?;
        
        self.tls_acceptor = Some(TlsAcceptor::from(Arc::new(config)));
        Ok(self)
    }

    pub fn with_tls_client(mut self) -> Result<Self, TransportError> {
        let mut root_store = RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        
        self.tls_connector = Some(TlsConnector::from(Arc::new(config)));
        Ok(self)
    }

    pub fn with_encryption_key(mut self, key: &[u8; 32]) -> Self {
        self.encryption_key = Some(*Key::from_slice(key));
        self
    }

    pub async fn accept_secure_connection(&self, stream: TcpStream) -> Result<SecureConnection, TransportError> {
        if let Some(acceptor) = &self.tls_acceptor {
            let tls_stream = acceptor.accept(stream).await
                .map_err(|e| TransportError::TlsHandshakeFailed(e.to_string()))?;
            
            Ok(SecureConnection::Tls(Box::new(tls_stream)))
        } else {
            Ok(SecureConnection::Plain(stream))
        }
    }

    pub async fn connect_secure(&self, stream: TcpStream, domain: &str) -> Result<SecureConnection, TransportError> {
        if let Some(connector) = &self.tls_connector {
            let tls_stream = connector.connect(domain.try_into().unwrap(), stream).await
                .map_err(|e| TransportError::TlsHandshakeFailed(e.to_string()))?;
            
            Ok(SecureConnection::TlsClient(Box::new(tls_stream)))
        } else {
            Ok(SecureConnection::Plain(stream))
        }
    }

    pub fn encrypt_message(&self, plaintext: &[u8]) -> Result<Vec<u8>, TransportError> {
        if let Some(key) = &self.encryption_key {
            let cipher = Aes256Gcm::new(key);
            
            // Generate random nonce
            let mut nonce_bytes = [0u8; 12];
            rand::thread_rng().fill(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)
                .map_err(|e| TransportError::EncryptionFailed(e.to_string()))?;
            
            // Prepend nonce to ciphertext
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&ciphertext);
            
            Ok(result)
        } else {
            // No encryption configured, return plaintext
            Ok(plaintext.to_vec())
        }
    }

    pub fn decrypt_message(&self, ciphertext: &[u8]) -> Result<Vec<u8>, TransportError> {
        if let Some(key) = &self.encryption_key {
            if ciphertext.len() < 12 {
                return Err(TransportError::DecryptionFailed("Invalid ciphertext length".to_string()));
            }
            
            let cipher = Aes256Gcm::new(key);
            
            // Extract nonce and ciphertext
            let nonce = Nonce::from_slice(&ciphertext[..12]);
            let encrypted_data = &ciphertext[12..];
            
            let plaintext = cipher.decrypt(nonce, encrypted_data)
                .map_err(|e| TransportError::DecryptionFailed(e.to_string()))?;
            
            Ok(plaintext)
        } else {
            // No encryption configured, return ciphertext as-is
            Ok(ciphertext.to_vec())
        }
    }

    pub fn sign_message(&self, message: &[u8]) -> String {
        let signature = sign_detached(message, &self.signing_key);
        encode(signature.as_bytes())
    }

    pub fn verify_message(&self, message: &[u8], signature: &str, public_key: &PublicKey) -> bool {
        if let Ok(sig_bytes) = decode(signature) {
            if let Ok(signature) = DetachedSignature::from_bytes(&sig_bytes) {
                return signature.verify_detached(message, public_key).is_ok();
            }
        }
        false
    }

    pub fn get_public_key(&self) -> String {
        encode(self.verify_key.as_bytes())
    }

    pub fn get_public_key_bytes(&self) -> &PublicKey {
        &self.verify_key
    }
}

pub enum SecureConnection {
    Plain(TcpStream),
    Tls(Box<ServerTlsStream<TcpStream>>),
    TlsClient(Box<ClientTlsStream<TcpStream>>),
}

impl SecureConnection {
    pub async fn send_encrypted(&mut self, transport: &SecureTransport, data: &[u8]) -> Result<(), TransportError> {
        use tokio::io::AsyncWriteExt;
        
        let encrypted_data = transport.encrypt_message(data)?;
        let signature = transport.sign_message(&encrypted_data);
        
        // Create message with signature
        let message = EncryptedMessage {
            data: encode(&encrypted_data),
            signature,
            sender_public_key: transport.get_public_key(),
        };
        
        let serialized = serde_json::to_vec(&message)
            .map_err(|e| TransportError::EncryptionFailed(e.to_string()))?;
        
        // Send length prefix + data
        let length = serialized.len() as u32;
        let length_bytes = length.to_be_bytes();
        
        match self {
            SecureConnection::Plain(stream) => {
                stream.write_all(&length_bytes).await?;
                stream.write_all(&serialized).await?;
            }
            SecureConnection::Tls(stream) => {
                stream.write_all(&length_bytes).await?;
                stream.write_all(&serialized).await?;
            }
            SecureConnection::TlsClient(stream) => {
                stream.write_all(&length_bytes).await?;
                stream.write_all(&serialized).await?;
            }
        }
        
        Ok(())
    }

    pub async fn receive_encrypted(&mut self, transport: &SecureTransport) -> Result<(Vec<u8>, PublicKey), TransportError> {
        use tokio::io::AsyncReadExt;
        
        // Read length prefix
        let mut length_bytes = [0u8; 4];
        match self {
            SecureConnection::Plain(stream) => {
                stream.read_exact(&mut length_bytes).await?;
            }
            SecureConnection::Tls(stream) => {
                stream.read_exact(&mut length_bytes).await?;
            }
            SecureConnection::TlsClient(stream) => {
                stream.read_exact(&mut length_bytes).await?;
            }
        }
        
        let length = u32::from_be_bytes(length_bytes) as usize;
        if length > 1024 * 1024 { // 1MB limit
            return Err(TransportError::DecryptionFailed("Message too large".to_string()));
        }
        
        // Read message data
        let mut buffer = vec![0u8; length];
        match self {
            SecureConnection::Plain(stream) => {
                stream.read_exact(&mut buffer).await?;
            }
            SecureConnection::Tls(stream) => {
                stream.read_exact(&mut buffer).await?;
            }
            SecureConnection::TlsClient(stream) => {
                stream.read_exact(&mut buffer).await?;
            }
        }
        
        // Deserialize message
        let message: EncryptedMessage = serde_json::from_slice(&buffer)
            .map_err(|e| TransportError::DecryptionFailed(e.to_string()))?;
        
        // Decode sender's public key
        let sender_key_bytes = decode(&message.sender_public_key)
            .map_err(|e| TransportError::AuthenticationFailed(e.to_string()))?;
        let sender_public_key = PublicKey::from_bytes(&sender_key_bytes)
            .map_err(|e| TransportError::AuthenticationFailed(e.to_string()))?;
        
        // Decode encrypted data
        let encrypted_data = decode(&message.data)
            .map_err(|e| TransportError::DecryptionFailed(e.to_string()))?;
        
        // Verify signature
        if !transport.verify_message(&encrypted_data, &message.signature, &sender_public_key) {
            return Err(TransportError::AuthenticationFailed("Invalid signature".to_string()));
        }
        
        // Decrypt data
        let plaintext = transport.decrypt_message(&encrypted_data)?;
        
        Ok((plaintext, sender_public_key))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct EncryptedMessage {
    data: String,
    signature: String,
    sender_public_key: String,
}

pub struct SecureTransportBuilder {
    use_tls: bool,
    cert_path: Option<String>,
    key_path: Option<String>,
    encryption_key: Option<[u8; 32]>,
}

impl SecureTransportBuilder {
    pub fn new() -> Self {
        Self {
            use_tls: true,
            cert_path: None,
            key_path: None,
            encryption_key: None,
        }
    }

    pub fn with_tls(mut self, cert_path: String, key_path: String) -> Self {
        self.use_tls = true;
        self.cert_path = Some(cert_path);
        self.key_path = Some(key_path);
        self
    }

    pub fn with_encryption(mut self, key: [u8; 32]) -> Self {
        self.encryption_key = Some(key);
        self
    }

    pub fn build_server(self) -> Result<SecureTransport, TransportError> {
        let mut transport = SecureTransport::new()?;

        if self.use_tls {
            if let (Some(cert_path), Some(key_path)) = (self.cert_path, self.key_path) {
                // Load certificate and private key from files
                let cert_chain = load_certs(&cert_path)?;
                let private_key = load_private_key(&key_path)?;
                transport = transport.with_tls_server(cert_chain, private_key)?;
            }
        }

        if let Some(key) = self.encryption_key {
            transport = transport.with_encryption_key(&key);
        }

        Ok(transport)
    }

    pub fn build_client(self) -> Result<SecureTransport, TransportError> {
        let mut transport = SecureTransport::new()?;

        if self.use_tls {
            transport = transport.with_tls_client()?;
        }

        if let Some(key) = self.encryption_key {
            transport = transport.with_encryption_key(&key);
        }

        Ok(transport)
    }
}

fn load_certs(filename: &str) -> Result<Vec<Certificate>, TransportError> {
    let certfile = std::fs::File::open(filename)
        .map_err(|e| TransportError::InvalidCertificate(format!("Failed to open cert file: {}", e)))?;
    let mut reader = std::io::BufReader::new(certfile);
    
    let certs = rustls_pemfile::certs(&mut reader)
        .map_err(|e| TransportError::InvalidCertificate(format!("Failed to parse certs: {}", e)))?;
    
    Ok(certs.into_iter().map(Certificate).collect())
}

fn load_private_key(filename: &str) -> Result<PrivateKey, TransportError> {
    let keyfile = std::fs::File::open(filename)
        .map_err(|e| TransportError::InvalidCertificate(format!("Failed to open key file: {}", e)))?;
    let mut reader = std::io::BufReader::new(keyfile);
    
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|e| TransportError::InvalidCertificate(format!("Failed to parse private key: {}", e)))?;
    
    if keys.len() != 1 {
        return Err(TransportError::InvalidCertificate("Expected exactly one private key".to_string()));
    }
    
    Ok(PrivateKey(keys[0].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_encryption_decryption() {
        let key = [0u8; 32]; // Test key
        let transport = SecureTransport::new().unwrap().with_encryption_key(&key);
        
        let message = b"Hello, secure world!";
        let encrypted = transport.encrypt_message(message).unwrap();
        let decrypted = transport.decrypt_message(&encrypted).unwrap();
        
        assert_eq!(message, decrypted.as_slice());
    }

    #[test]
    fn test_signing_verification() {
        let transport = SecureTransport::new().unwrap();
        let message = b"Test message for signing";
        
        let signature = transport.sign_message(message);
        let verified = transport.verify_message(message, &signature, transport.get_public_key_bytes());
        
        assert!(verified);
    }
}
