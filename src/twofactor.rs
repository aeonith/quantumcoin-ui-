use oath::{totp_raw_now, HashType};
use rand::Rng;
use base32::{Alphabet, encode};

pub fn generate_2fa_secret() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 10] = rng.gen();
    encode(Alphabet::RFC4648 { padding: false }, &bytes)
}

pub fn verify_2fa_code(secret: &str, code: &str) -> bool {
    if let Ok(secret_bytes) = base32::decode(Alphabet::RFC4648 { padding: false }, secret) {
        let totp = totp_raw_now(&secret_bytes, 6, 0, 30, &HashType::SHA1);
        if let Ok(current_code) = totp {
            return format!("{:06}", current_code) == code;
        }
    }
    false
}