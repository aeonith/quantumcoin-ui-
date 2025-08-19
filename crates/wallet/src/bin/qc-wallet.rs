use clap::{Parser, Subcommand};
use qc_wallet::{new_seed_32, address_from_seed, generate_mnemonic, WalletSeed, test_wallet_recovery};
use anyhow::Result;

#[derive(Parser)]
#[command(name="qc-wallet")]
#[command(about = "QuantumCoin Wallet - Bitcoin-level security")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate new wallet with mnemonic
    New,
    /// Generate address from seed
    Addr { 
        #[arg(default_value_t=0)] 
        index: u32 
    },
    /// Restore wallet from mnemonic  
    Restore {
        #[arg(long)]
        mnemonic: String,
    },
    /// Test wallet recovery
    TestRecover,
    /// Generate multiple addresses
    Generate {
        #[arg(short, long, default_value_t=5)]
        count: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.cmd {
        Cmd::New => {
            println!("🔑 Generating new QuantumCoin wallet...");
            
            let wallet = WalletSeed::generate()?;
            
            println!("✅ Wallet generated successfully:");
            println!("Mnemonic: {}", wallet.mnemonic);
            println!("Address[0]: {}", wallet.derive_address(0));
            println!("Address[1]: {}", wallet.derive_address(1));
            println!("Address[2]: {}", wallet.derive_address(2));
            println!("");
            println!("⚠️  SECURITY WARNING:");
            println!("- Store mnemonic securely (write it down offline)");
            println!("- Never share mnemonic with anyone");
            println!("- Mnemonic can recover all addresses and funds");
        }
        
        Cmd::Addr { index } => {
            eprintln!("Enter seed hex on stdin:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            let mut seed = [0u8; 32]; 
            let bytes = hex::decode(input.trim())?;
            seed.copy_from_slice(&bytes[..32]);
            
            let address = address_from_seed(&seed, index);
            println!("addr[{}]={}", index, address);
        }
        
        Cmd::Restore { mnemonic } => {
            println!("🔄 Restoring wallet from mnemonic...");
            
            let wallet = WalletSeed::from_mnemonic(&mnemonic, "")?;
            
            println!("✅ Wallet restored successfully:");
            println!("Address[0]: {}", wallet.derive_address(0));
            println!("Address[1]: {}", wallet.derive_address(1));
            println!("Address[2]: {}", wallet.derive_address(2));
            println!("");
            println!("🔍 To see more addresses: qc-wallet generate --count 10");
        }
        
        Cmd::TestRecover => {
            test_wallet_recovery()?;
        }
        
        Cmd::Generate { count } => {
            println!("🔑 Generating {} addresses...", count);
            
            let wallet = WalletSeed::generate()?;
            
            println!("Mnemonic: {}", wallet.mnemonic);
            println!("Addresses:");
            
            for i in 0..count {
                let address = wallet.derive_address(i);
                println!("  [{}]: {}", i, address);
            }
        }
    }
    
    Ok(())
}
