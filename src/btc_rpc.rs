use anyhow::{Error, Ok};
use bitcoin::{consensus::deserialize, Transaction};
use reqwest::Response;
use serde::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCResponse<T> {
    result: T,
    error: Option<String>,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RPCValue {
    Str(String),
    Int(usize),
    Bool(bool),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCRequest {
    jsonrpc: String,
    id: String,
    method: String,
    params: Vec<RPCValue>,
}

impl RPCRequest {
    fn new(method: &str, params: &[RPCValue]) -> RPCRequest {
        RPCRequest {
            jsonrpc: "1.0".to_string(),
            id: "rm".to_string(),
            method: method.to_string(),
            params: Vec::from(params),
        }
    }
}

pub const BTC_RPC_URL: &str =
    "https://clean-light-violet.btc.quiknode.pro/c22960fe7aa43d1cfaf1e8a2b8cf60a1a430b7cb";

pub async fn get_transaction(id: &str) -> Result<Transaction, Error> {
    let raw_tx = get_raw_transaction(id).await?;

    let tx: Transaction = deserialize(&raw_tx).unwrap();

    Ok(tx)
}

pub async fn get_raw_transaction(id: &str) -> Result<Vec<u8>, Error> {
    let response = rpc_request(&RPCRequest::new(
        "getrawtransaction",
        &[RPCValue::Str(id.to_string())],
    ))
    .await?;

    let result = response.json::<RPCResponse<String>>().await?.result;
    let raw_tx = hex::decode(&result).unwrap();

    Ok(raw_tx)
}

// pub async fn get_latest_validated_block() -> Result<(), Error> {
//     let block_hash = get_best_block_hash().await?;

//     let response = rpc_request(&RPCRequest::new(
//         "getblock",
//         &[RPCValue::Str(block_hash)],
//     ))
//     .await?;

//     let result = response.json::<RPCResponse<String>>().await?.result;

//     Ok(result)
// }

pub async fn get_best_block_hash() -> Result<String, Error> {
    let response = rpc_request(&RPCRequest::new("getbestblockhash", &[])).await?;

    let result = response.json::<RPCResponse<String>>().await?.result;

    Ok(result)
}

async fn rpc_request(request: &RPCRequest) -> Result<Response, Error> {
    let client = reqwest::Client::new();

    let response = client
        .post(BTC_RPC_URL)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(request).unwrap())
        // .body(json_data.to_owned())
        .send()
        .await?;

    Ok(response)
}
