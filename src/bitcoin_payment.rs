use std::time::Duration;
use serde::Deserialize;
use reqwest::Client;
use std::collections::HashSet;
use tokio::time::sleep;

const BTC_ADDRESS: &str = "bc1qv7tpdxqvgwutfrhf53nhwgp77j5lv7whnk433y";
const MIN_BTC_AMOUNT: f64 = 0.000083;
const QTC_PRICE_USD: f64 = 0.25;

#[derive(Debug, Deserialize)]
struct Transaction {
    txid: String,
    status: TxStatus,
    vin: Vec<TxInput>,
    vout: Vec<TxOutput>,
}

#[derive(Debug, Deserialize)]
struct TxStatus {
    confirmed: bool,
}

#[derive(Debug, Deserialize)]
struct TxInput {
    prevout: Option<TxOutput>,
}

#[derive(Debug, Deserialize)]
struct TxOutput {
    value: u64, // in satoshis
    scriptpubkey_address: Option<String>,
}

pub async fn run_btc_listener_loop() {
    let client = Client::new();
    let mut seen_txids: HashSet<String> = HashSet::new();

    loop {
        match check_for_payment(&client, &mut seen_txids).await {
            Ok(_) => {}
            Err(e) => eprintln!("Error checking payment: {}", e),
        }
        sleep(Duration::from_secs(30)).await;
    }
}

async fn check_for_payment(
    client: &Client,
    seen_txids: &mut HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://blockstream.info/api/address/{}/txs",
        BTC_ADDRESS
    );

    let resp = client.get(&url).send().await?;
    let txs: Vec<Transaction> = resp.json().await?;

    for tx in txs {
        if !seen_txids.contains(&tx.txid) && tx.status.confirmed {
            seen_txids.insert(tx.txid.clone());

            let mut btc_received = 0.0;
            for out in tx.vout.iter() {
                if let Some(address) = &out.scriptpubkey_address {
                    if address == BTC_ADDRESS {
                        btc_received += out.value as f64 / 100_000_000.0;
                    }
                }
            }

            println!("✅ New confirmed BTC tx {}: received {} BTC", tx.txid, btc_received);

            if btc_received < MIN_BTC_AMOUNT {
                println!("⚠️ Rejected: amount below 0.000083 BTC minimum.");
                continue;
            }

            let btc_price = fetch_btc_price_usd(client).await?;
            let usd_value = btc_received * btc_price;
            let qtc_to_send = (usd_value / QTC_PRICE_USD).floor() as u64;

            println!(
                "💰 BTC = ${:.2} | USD = ${:.2} | Issuing {} QTC...",
                btc_price, usd_value, qtc_to_send
            );

            // TODO: Replace with actual call to your Rust wallet/blockchain:
            // issue_qtc_to_wallet(qtc_to_send, user_wallet_address_from_context).await?;
            println!("🚀 [Mock] Issued {} QTC to user's wallet ✅", qtc_to_send);
        }
    }

    Ok(())
}

async fn fetch_btc_price_usd(client: &Client) -> Result<f64, Box<dyn std::error::Error>> {
    #[derive(Debug, Deserialize)]
    struct PriceResp {
        bitcoin: CoinPrice,
    }

    #[derive(Debug, Deserialize)]
    struct CoinPrice {
        usd: f64,
    }

    let resp = client
        .get("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd")
        .send()
        .await?
        .json::<PriceResp>()
        .await?;

    Ok(resp.bitcoin.usd)
}