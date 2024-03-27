

use reqwest::Error;
// use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ResultObj {
    txid: String,
    hash: String,
    version: i32,
    size: i32,
    vsize: i32,
    weight: i32,
    locktime: i32,
    vin: Vec<Vin>,
    vout: Vec<Vout>,
    hex: String,
    blockhash: String,
    confirmations: i32,
    time: i64, // Use i64 for Unix timestamps
    blocktime: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vin {
    txid: String,
    vout: i32,
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
    n: i32,
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
struct RPCResponse {
    result: ResultObj,
    error: Option<String>,
    id: String,
}

async fn get_raw_transaction() -> Result<(), Error> {
    let url =
        "https://clean-light-violet.btc.quiknode.pro/c22960fe7aa43d1cfaf1e8a2b8cf60a1a430b7cb";
    let json_data = r#"{
        "jsonrpc": "1.0", 
        "id": "curltest", 
        "method": "getrawtransaction", 
        "params": ["98a509a0b3fb9068a66ebd8f8c4ff8ef4b8b40827401a708ec5e32536192bb05", true]
    }"#;

    let client = reqwest::Client::new();

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(json_data.to_owned())
        .send()
        .await?
        .json::<RPCResponse>()
        .await?;

    let raw_tx = hex::decode(&response.result.hex).unwrap();
    
    let tx: bitcoin::Transaction = bitcoin::consensus::deserialize(&raw_tx).unwrap();
    print!("Tx: {:?}", tx);

    Ok(())
}

async fn decode_tx_input(vin: Vin) {

}

async fn decode_tx_output(vout: Vout) {

}

#[tokio::main]
async fn main() -> Result<(), Error> {
    get_raw_transaction().await?;
    Ok(())
}
