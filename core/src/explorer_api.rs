use std::time::Duration;

use derive_more::From;
use ergo_lib::chain::transaction::Transaction;
use ergo_lib::chain::transaction::TxId;
use ergo_lib::ergotree_ir::chain::address::NetworkPrefix;
use reqwest::blocking::RequestBuilder;
use reqwest::blocking::Response;
use reqwest::header::CONTENT_TYPE;
use thiserror::Error;
use url::ParseError;

use crate::oracle_config::ORACLE_CONFIG;

#[derive(Debug, From, Error)]
pub enum ExplorerApiError {
    #[error("reqwest error: {0}")]
    RequestError(reqwest::Error),
    #[error("serde error: {0}")]
    SerdeError(serde_json::Error),
    #[error("invalid explorer url: {0}")]
    InvalidExplorerUrl(ParseError),
}

pub struct ExplorerApi {
    pub url: url::Url,
}

impl ExplorerApi {
    pub fn new(url: &str) -> Result<Self, ExplorerApiError> {
        Ok(Self {
            url: url::Url::parse(url)?,
        })
    }

    /// Sets required headers for a request
    fn set_req_headers(&self, rb: RequestBuilder) -> RequestBuilder {
        rb.header("accept", "application/json")
            .header(CONTENT_TYPE, "application/json")
    }

    /// Sends a GET request to the Ergo node
    fn send_get_req(&self, endpoint: &str) -> Result<Response, ExplorerApiError> {
        let url = self.url.join(endpoint)?;
        let client = reqwest::blocking::Client::new().get(url);
        Ok(self.set_req_headers(client).send()?)
    }

    /// GET /transactions/{id}
    pub fn get_transaction(&self, tx_id: TxId) -> Result<Transaction, ExplorerApiError> {
        let endpoint = "/transactions/".to_owned() + &tx_id.to_string();
        let response = self.send_get_req(&endpoint)?;
        let text = response.text()?;
        Ok(serde_json::from_str(&text)?)
    }
}

pub fn wait_for_tx_confirmation(tx_id: TxId) {
    wait_for_txs_confirmation(vec![tx_id]);
}

pub fn wait_for_txs_confirmation(tx_ids: Vec<TxId>) {
    let network = ORACLE_CONFIG.oracle_address.network();
    let timeout = Duration::from_secs(600);
    let explorer_api = match network {
        NetworkPrefix::Mainnet => ExplorerApi::new("https://api.ergoplatform.com/api/v1/").unwrap(),
        NetworkPrefix::Testnet => {
            ExplorerApi::new("https://api-testnet.ergoplatform.com/api/v1/").unwrap()
        }
    };
    let start_time = std::time::Instant::now();
    println!("Waiting for block confirmation from ExplorerApi ...");
    let mut remaining_txs = tx_ids.clone();
    loop {
        for tx_id in remaining_txs.clone() {
            if let Ok(tx) = explorer_api.get_transaction(tx_id) {
                assert_eq!(tx.id(), tx_id);
                log::info!("Transaction found: {tx_id}");
                remaining_txs.retain(|id| *id != tx_id);
            }
        }
        if remaining_txs.is_empty() {
            break;
        }
        if start_time.elapsed() > timeout {
            println!("Timeout waiting for transactions");
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
