use ring::signature::{Ed25519KeyPair, KeyPair};
use ring::rand::SystemRandom;
use base64::{Engine as _, engine::general_purpose};

pub fn get_keys() -> (String, String) {
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).unwrap();
    
    let public_key = general_purpose::STANDARD.encode(key_pair.public_key().as_ref());
    let private_key = general_purpose::STANDARD.encode(pkcs8_bytes.as_ref());
    
    (public_key, private_key)
}

pub fn generate_wallet_address() -> String {
    let (public_key, _) = get_keys();
    format!("qtc_{}", &public_key[..20])
}

pub fn validate_address(address: &str) -> bool {
    address.starts_with("qtc_") && address.len() == 24
}
