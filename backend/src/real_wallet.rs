use quantumcoin_wallet::crypto::{generate_keypair, sign_transaction, verify_signature};
use quantumcoin_wallet::address::generate_address;
use rocket::serde::json::Json;
use serde_json::{json, Value};
use anyhow::Result;

#[post("/wallet/generate")]
pub fn generate_wallet() -> Json<Value> {
    // BULLETPROOF: Never fail, always return valid wallet
    match (|| -> Result<_, Box<dyn std::error::Error>> {
        let (public_key, private_key) = generate_keypair()?;
        let address = generate_address(&public_key);
        
        // Validate generated keys before returning
        if public_key.len() != 1312 {
            return Err(format!("Invalid public key size: {}", public_key.len()).into());
        }
        if private_key.len() != 2528 {
            return Err(format!("Invalid private key size: {}", private_key.len()).into());
        }
        if address.is_empty() {
            return Err("Empty address generated".into());
        }
        
        Ok((public_key, private_key, address))
    })() {
        Ok((public_key, private_key, address)) => {
            Json(json!({
                "success": true,
                "address": address,
                "public_key": base64::encode(&public_key),
                "private_key": base64::encode(&private_key),
                "algorithm": "dilithium2",
                "security_level": "NIST Level 2",
                "key_sizes": {
                    "public_key_bytes": public_key.len(),
                    "private_key_bytes": private_key.len()
                }
            }))
        },
        Err(e) => {
            eprintln!("⚠️  Wallet generation error (recovered): {}", e);
            // NEVER fail - always provide a response
            Json(json!({
                "success": false,
                "error": "Wallet generation temporarily unavailable",
                "retry": true,
                "algorithm": "dilithium2"
            }))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct SignRequest {
    message: String,
    private_key: String,
}

#[post("/wallet/sign", data = "<sign_req>")]
pub fn sign_message(sign_req: Json<SignRequest>) -> Json<Value> {
    let private_key_bytes = match base64::decode(&sign_req.private_key) {
        Ok(bytes) => bytes,
        Err(_) => return Json(json!({
            "success": false,
            "error": "Invalid private key encoding"
        }))
    };
    
    match sign_transaction(sign_req.message.as_bytes(), &private_key_bytes) {
        Ok(signature) => {
            Json(json!({
                "success": true,
                "signature": base64::encode(&signature),
                "signature_size": signature.len(),
                "algorithm": "dilithium2"
            }))
        },
        Err(e) => {
            Json(json!({
                "success": false,
                "error": format!("Signing failed: {}", e)
            }))
        }
    }
}

#[derive(serde::Deserialize)]
pub struct VerifyRequest {
    message: String,
    signature: String,
    public_key: String,
}

#[post("/wallet/verify", data = "<verify_req>")]
pub fn verify_signature_endpoint(verify_req: Json<VerifyRequest>) -> Json<Value> {
    let signature_bytes = match base64::decode(&verify_req.signature) {
        Ok(bytes) => bytes,
        Err(_) => return Json(json!({
            "valid": false,
            "error": "Invalid signature encoding"
        }))
    };
    
    let public_key_bytes = match base64::decode(&verify_req.public_key) {
        Ok(bytes) => bytes,
        Err(_) => return Json(json!({
            "valid": false,
            "error": "Invalid public key encoding"
        }))
    };
    
    let is_valid = verify_signature(
        verify_req.message.as_bytes(),
        &signature_bytes,
        &public_key_bytes
    ).unwrap_or(false);
    
    Json(json!({
        "valid": is_valid,
        "algorithm": "dilithium2",
        "security_level": "post_quantum"
    }))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![generate_wallet, sign_message, verify_signature_endpoint]
}
