use anyhow::{Error, Ok};
use bitcoin::{consensus::deserialize, Transaction};
use reqwest::Response;
use serde::*;

// #[derive(Serialize, Deserialize, Debug)]
pub struct TransactionInfo {
    pub raw: RawTxObj,
    pub data: Transaction,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct RawTxObj {
    pub txid: String,
    pub hash: String,
    pub version: u32,
    pub size: u32,
    pub vsize: u32,
    pub weight: u32,
    pub locktime: u64,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub hex: String,
    pub blockhash: String,
    pub confirmations: Option<u32>,
    pub time: u64, // Use i64 for Unix timestamps
    pub blocktime: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    txid: String,
    vout: u32,
    scriptSig: ScriptSig,
    txinwitness: Vec<String>,
    sequence: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vout {
    value: f64,
    n: u32,
    scriptPubKey: ScriptPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScriptPubKey {
    asm: String,
    desc: String,
    hex: String,
    address: Option<String>,
    #[serde(rename = "type")] // Handle the "type" field
    type_field: String,
}


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

pub async fn get_transaction(id: &str) -> Result<TransactionInfo, Error> {
    let raw = get_raw_transaction(id).await?;
    let hex = hex::decode(&raw.hex).unwrap();

    let data: Transaction = deserialize(&hex).unwrap();

    let transaction_info = TransactionInfo {
        raw,
        data,
    };

    Ok(transaction_info)
}

pub async fn get_raw_transaction(id: &str) -> Result<RawTxObj, Error> {
    let response = rpc_request(&RPCRequest::new(
        "getrawtransaction",
        &[RPCValue::Str(id.to_string()), RPCValue::Int(1)],
    ))
    .await?;

    let result = response.json::<RPCResponse<RawTxObj>>().await?.result;
    // let hex = hex::decode(&result.hex).unwrap();

    Ok(result)
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
