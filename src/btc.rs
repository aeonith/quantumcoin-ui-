use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct TxStatus {
    confirmations: u64,
}

/// Checks if the Bitcoin transaction is confirmed (>= 1 confirmation).
pub async fn get_btc_payment_status(txid: &str) -> bool {
    let url = format!("https://api.blockcypher.com/v1/btc/main/txs/{}", txid);
    let client = Client::new();

    match client.get(&url).send().await {
        Ok(resp) => {
            if let Ok(tx_status) = resp.json::<TxStatus>().await {
                tx_status.confirmations >= 1
            } else {
                false
            }
        }
        Err(_) => false,
    }
}