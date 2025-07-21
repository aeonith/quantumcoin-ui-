mod transaction;
mod block;
mod blockchain;
mod wallet;
mod revstop;
mod cli;            // keep your existing interactive CLI

use wallet::Wallet;
use blockchain::Blockchain;
use revstop::RevStop;

fn main() -> anyhow::Result<()> {
    let wallet_path = "data";
    let mut wallet = Wallet::load_from_files(wallet_path)
        .unwrap_or_else(|_| Wallet::generate());

    let mut blockchain = Blockchain::load("blockchain.json")
        .unwrap_or_else(|_| Blockchain::new_with_genesis(&wallet.get_address()));

    let mut revstop = RevStop::load_status("revstop.json")?;

    cli::start_cli(&mut wallet, &mut blockchain, &mut revstop)?;

    wallet.save_to_files(wallet_path)?;
    blockchain.save("blockchain.json")?;
    revstop.save_status("revstop.json")?;
    Ok(())
}