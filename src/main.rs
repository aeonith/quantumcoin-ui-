mod network;
mod wallet;
mod blockchain;
mod revstop;

use network::message::NetworkMessage;
use network::Network;
use wallet::Wallet;
use blockchain::{Blockchain, Transaction};
use revstop::RevStop;

use tokio::sync::mpsc;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    println!("🔗 QuantumCoin Node Booting...");

    // Start the P2P networking layer
    let (mut network, mut rx): (Network, mpsc::Receiver<NetworkMessage>) =
        Network::new().await.expect("Failed to start network");

    // Load local wallet and blockchain state
    let mut wallet = Wallet::load_from_files().expect("Could not load wallet");
    let mut blockchain = Blockchain::load_from_disk().unwrap_or_else(Blockchain::new);
    let mut revstop = RevStop::load_status().unwrap_or_else(RevStop::default);

    // Spawn async task to receive network messages
    let blockchain_ref = Arc::new(Mutex::new(blockchain));
    let mempool = Arc::new(Mutex::new(Vec::<Transaction>::new()));

    {
        let blockchain_ref = Arc::clone(&blockchain_ref);
        let mempool = Arc::clone(&mempool);

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    NetworkMessage::NewTransaction(json_tx) => {
                        if let Ok(tx) = serde_json::from_str::<Transaction>(&json_tx) {
                            println!("📡 Received Transaction: {:?}", tx);
                            mempool.lock().unwrap().push(tx);
                        }
                    }
                    NetworkMessage::NewBlock(json_block) => {
                        if let Ok(block) = serde_json::from_str(&json_block) {
                            println!("📦 Received Block: {:?}", block);
                            let mut chain = blockchain_ref.lock().unwrap();
                            if chain.is_valid_new_block(&block) {
                                chain.add_block(block);
                                chain.save_to_disk();
                            }
                        }
                    }
                    NetworkMessage::RequestChain => {
                        println!("🔄 Peer requested chain sync (not implemented yet)");
                    }
                    NetworkMessage::ChainResponse(_blocks) => {
                        println!("⬇️  Received blockchain sync response (not implemented yet)");
                    }
                }
            }
        });
    }

    // Simple CLI loop
    loop {
        println!("\nQuantumCoin CLI:");
        println!("1. Check Balance");
        println!("2. Send Coins");
        println!("3. Mine Transactions");
        println!("4. Connect to Peer");
        println!("5. Exit");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                let chain = blockchain_ref.lock().unwrap();
                let balance = chain.get_balance(&wallet.address);
                println!("🔐 Your balance: {} QTC", balance);
            }
            "2" => {
                if revstop.is_locked {
                    println!("🔒 Wallet is locked by RevStop. Unlock required.");
                    continue;
                }

                let mut to_addr = String::new();
                let mut amount_str = String::new();

                print!("Recipient address: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut to_addr).unwrap();

                print!("Amount to send: ");
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut amount_str).unwrap();

                let amount: f64 = amount_str.trim().parse().unwrap_or(0.0);

                let tx = wallet.create_transaction(&to_addr.trim(), amount);
                println!("📤 Sending transaction: {:?}", tx);

                // Add to local mempool
                mempool.lock().unwrap().push(tx.clone());

                // Broadcast to peers
                let tx_json = serde_json::to_string(&tx).unwrap();
                network.publish_message(NetworkMessage::NewTransaction(tx_json));
            }
            "3" => {
                let mut chain = blockchain_ref.lock().unwrap();
                let mut mempool_locked = mempool.lock().unwrap();

                if mempool_locked.is_empty() {
                    println!("⚠️ No pending transactions to mine.");
                    continue;
                }

                let new_block = chain.mine_pending_transactions(&wallet.address, &mut mempool_locked);
                println!("⛏️ Mined new block!");

                // Save new chain
                chain.save_to_disk();

                // Broadcast mined block
                let block_json = serde_json::to_string(&new_block).unwrap();
                network.publish_message(NetworkMessage::NewBlock(block_json));
            }
            "4" => {
                let mut peer_addr = String::new();
                println!("Enter peer multiaddress (e.g. /ip4/127.0.0.1/tcp/8080): ");
                io::stdin().read_line(&mut peer_addr).unwrap();

                match peer_addr.trim().parse() {
                    Ok(addr) => {
                        match network.swarm.dial(addr) {
                            Ok(_) => println!("🔗 Dialing peer..."),
                            Err(e) => println!("❌ Failed to dial peer: {}", e),
                        }
                    }
                    Err(_) => println!("⚠️ Invalid address format"),
                }
            }
            "5" => {
                println!("👋 Exiting QuantumCoin node...");
                break;
            }
            _ => println!("❓ Invalid option, try again."),
        }
    }
}