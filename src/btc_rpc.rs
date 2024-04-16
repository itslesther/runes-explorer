use anyhow::{Error, Ok};
use bitcoin::{consensus::deserialize, Block, Transaction};
use reqwest::Response;
use serde::*;
#[derive(Clone, Debug)]
pub struct TransactionInfo {
    pub raw: RawTxObj,
    pub data: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawBlockHeader {
    pub hash: String,
    pub confirmations: u32,
    pub height: u32,
    pub time: u64,
    #[serde(rename = "nTx")]
    pub n_tx: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
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
    pub blockhash: Option<String>,
    pub confirmations: Option<u32>,
    pub time: Option<u64>, // Use i64 for Unix timestamps
    pub blocktime: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawBlockObj<Tx> {
    pub hash: String,
    pub confirmations: u32,
    pub height: u32,
    pub version: u64,
    #[serde(rename = "versionHex")]
    pub version_hex: String,
    pub merkleroot: String,
    pub time: u64,
    pub mediantime: u64,
    pub nonce: u64,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    #[serde(rename = "nTx")]
    pub n_tx: u32,
    pub previousblockhash: String,
    pub strippedsize: u64,
    pub size: u64,
    pub weight: u64,
    pub tx: Vec<Tx>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vin {
    txid: Option<String>,
    coinbase: Option<String>,
    vout: Option<u32>,
    #[serde(rename = "scriptSig")]
    script_sig: Option<ScriptSig>,
    txinwitness: Option<Vec<String>>,
    sequence: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScriptSig {
    asm: String,
    hex: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vout {
    value: f64,
    n: u32,
    #[serde(rename = "scriptPubKey")] // Handle the "type" field
    script_pubkey: ScriptPubKey,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

// TESTNET
pub const BTC_RPC_URL: &str =
    "https://powerful-cool-bush.btc-testnet.quiknode.pro/cf40fbe86ac4d435ce4799c8aae18c1dc65b96c8";

// MAINNET
// pub const BTC_RPC_URL: &str =
// "https://clean-light-violet.btc.quiknode.pro/c22960fe7aa43d1cfaf1e8a2b8cf60a1a430b7cb";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BTCRPC {
    pub url: String,
}

impl BTCRPC {
    pub async fn get_transaction(&self, id: &str) -> Result<TransactionInfo, Error> {
        let raw = self.get_raw_transaction(id).await?;
        let hex = hex::decode(&raw.hex).unwrap();

        let data: Transaction = deserialize(&hex).unwrap();

        let transaction_info = TransactionInfo { raw, data };

        Ok(transaction_info)
    }

    pub async fn get_raw_transaction(&self, id: &str) -> Result<RawTxObj, Error> {
        let response: Response = self.rpc_request(&RPCRequest::new(
            "getrawtransaction",
            &[RPCValue::Str(id.to_string()), RPCValue::Int(1)],
        ))
        .await?;

        let result = response.json::<RPCResponse<RawTxObj>>().await?.result;
        Ok(result)
    }

    pub async fn get_latest_validated_block_height(&self) -> Result<u32, Error> {
        let block_hash = self.get_best_block_hash().await?;

        let response = self.rpc_request(&RPCRequest::new(
            "getblockheader",
            &[RPCValue::Str(block_hash), RPCValue::Bool(true)],
        ))
        .await?;
        let block_header = response.json::<RPCResponse<RawBlockHeader>>().await?.result;

        Ok(block_header.height)
    }

    pub async fn get_latest_validated_block(&self) -> Result<Block, Error> {
        let block_hash = self.get_best_block_hash().await?;

        let block = self.get_block_by_hash(&block_hash).await?;

        Ok(block)
    }

    pub async fn get_block_by_height(&self, block_height: u32) -> Result<Block, Error> {
        let response = self.rpc_request(&RPCRequest::new(
            "getblockhash",
            &[RPCValue::Int(block_height as usize)],
        ))
        .await?;

        let block_hash = response.json::<RPCResponse<String>>().await?.result;

        let block = self.get_block_by_hash(&block_hash).await?;

        Ok(block)
    }

    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<Block, Error> {
        let response = self.rpc_request(&RPCRequest::new(
            "getblock",
            &[RPCValue::Str(block_hash.to_string()), RPCValue::Int(0)],
        ))
        .await?;

        let result = response.json::<RPCResponse<String>>().await?.result;

        let hex = hex::decode(&result).unwrap();

        let data: bitcoin::Block = deserialize(&hex).unwrap();

        Ok(data)
    }

    // pub async fn get_block_by_hash(block_hash: &str) -> Result<RawBlockObj<TransactionInfo>, Error> {
    //     let response = rpc_request(&RPCRequest::new(
    //         "getblock",
    //         &[RPCValue::Str(block_hash.to_string()), RPCValue::Int(2)],
    //     ))
    //     .await?;

    //     let result = response
    //         .json::<RPCResponse<RawBlockObj<RawTxObj>>>()
    //         .await?
    //         .result;

    //     let block: RawBlockObj<TransactionInfo> = RawBlockObj {
    //         hash: result.hash,
    //         confirmations: result.confirmations,
    //         height: result.height,
    //         version: result.version,
    //         version_hex: result.version_hex,
    //         merkleroot: result.merkleroot,
    //         time: result.time,
    //         mediantime: result.mediantime,
    //         nonce: result.nonce,
    //         bits: result.bits,
    //         difficulty: result.difficulty,
    //         chainwork: result.chainwork,
    //         n_tx: result.n_tx,
    //         previousblockhash: result.previousblockhash,
    //         strippedsize: result.strippedsize,
    //         size: result.size,
    //         weight: result.weight,
    //         tx: result
    //             .tx
    //             .iter()
    //             .map(|raw_temp| {
    //                 let hex = hex::decode(&raw_temp.hex).unwrap();

    //                 let mut raw = raw_temp.clone();
    //                 raw.confirmations = Some(result.confirmations);
    //                 raw.time = Some(result.time);
    //                 raw.blocktime = Some(result.time);

    //                 let data: Transaction = deserialize(&hex).unwrap();

    //                 let transaction_info = TransactionInfo { raw, data };
    //                 // data.confirmations = Some(result.confirmations);
    //                 transaction_info
    //             })
    //             .collect(),
    //     };

    //     Ok(block)
    // }

    pub async fn get_best_block_hash(&self) -> Result<String, Error> {
        let response = self.rpc_request(&RPCRequest::new("getbestblockhash", &[])).await?;

        let result = response.json::<RPCResponse<String>>().await?.result;

        Ok(result)
    }

    async fn rpc_request(&self,request: &RPCRequest) -> Result<Response, Error> {
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
}
