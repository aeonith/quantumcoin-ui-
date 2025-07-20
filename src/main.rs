mod wallet;
use wallet::Wallet;

fn main() {
    println!("🚀 QuantumCoin Engine Initialized");

    // Load existing wallet or create a new one
    let wallet = Wallet::load_from_files().unwrap_or_else(|| {
        let w = Wallet::new();
        w.save_to_files();
        println!("🔐 New wallet generated and saved.");
        w
    });

    println!("🔑 Wallet Address: {}", wallet.get_address());

    // Test signing a message
    let message = b"Test transaction";
    let signature = wallet.sign_message(message);
    let verified = wallet.verify_signature(message, &signature);

    if verified {
        println!("✅ Signature verified.");
    } else {
        println!("❌ Signature failed verification.");
    }
}