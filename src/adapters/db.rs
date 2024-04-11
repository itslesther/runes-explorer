use anyhow::Error;
use serde::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Statistics {
    pub block_height: u64,
    pub rune_count: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuneEntry {
    pub etching_tx_id: String,
    pub block_height: u64,
    pub rune_id: String,
    pub name: String,
    pub raw_name: String,
    pub symbol: Option<char>,
    pub divisibility: u8,
    pub premine: u128,
    // pub terms_id: Option<String>,
    pub terms: Option<Terms>,
    pub burned: u128,
    pub mint_count: u128, // validation for Terms::cap
    pub timestamp: u32,
    pub is_cenotapth: bool,
    pub rune_number: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub tx_id: String,
    // pub inputs: Vec<TXO>,
    // pub outputs: Vec<TXO>,
    pub is_artifact: bool,
    pub is_runestone: bool,
    pub is_cenotapth: bool,
    pub cenotapth_messages: Option<String>,
    pub timestamp: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Terms {
    pub amount: Option<u128>,
    pub cap: Option<u128>,
    pub height_start: Option<u64>,
    pub height_end: Option<u64>,
    pub offset_start: Option<u64>,
    pub offset_end: Option<u64>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXO {
    pub tx_id: String,
    pub output_index: u32,
    pub value: u64,
    pub address: Option<String>,
    pub address_lowercase: Option<String>,
    // pub rune_transfers: Vec<RuneTransfer>,
    pub is_unspent: bool,
    pub spent_tx_id: Option<String>,
    pub timestamp: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuneTransfer {
    pub tx_id: String,
    pub output_index: u32,
    pub rune_id: String,
    pub amount: u128,
    pub address: Option<String>,
    pub address_lowercase: Option<String>,
    pub is_unspent: bool,
    pub spent_tx_id: Option<String>,
    pub timestamp: u32,
}

pub trait Database {
    // fn init() -> Result<(), Error>;
    fn get_runes_transfers_by_tx(
        &mut self,
        tx_id: &str,
        tx_index: u32,
    ) -> Result<Vec<RuneTransfer>, Error>;

    fn get_rune_by_id(&mut self, rune_id: &str) -> Result<Option<&mut RuneEntry>, Error>;
    fn update_rune_entry_mint_count(&mut self, rune_id: &str) -> Result<(), Error>;
    fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error>;
    fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<&RuneEntry>, Error>;
    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Error>;
    fn add_rune_entry(&mut self, rune_entry: RuneEntry) -> Result<(), Error>;
    fn add_rune_transfer(&mut self, rune_transfer: RuneTransfer) -> Result<(), Error>;
    fn get_txo(&mut self, tx_id: &str, output_index: u32) -> Result<Option<&mut TXO>, Error>;
    fn mark_utxo_as_spent(
        &mut self,
        tx_id: &str,
        output_index: u32,
        spent_tx_id: &str,
    ) -> Result<(), Error>;
    fn add_txo(&mut self, txo: TXO) -> Result<(), Error>;
    fn get_address_balance_by_rune_id(&self, address: &str, rune_id: &str) -> u128;
    fn get_address_transfer(&self, address: &str) -> Vec<&RuneTransfer>;
    fn get_rune_count(&self) -> Result<u128, Error>;
    fn increase_rune_count(&mut self) -> Result<(), Error>;
    fn get_block_height(&self) -> Result<u64, Error>;
    fn increase_block_height(&mut self) -> Result<(), Error>;
    fn get_transaction(&self, tx_id: &str) -> Option<&Transaction>;
    fn get_rune_by_name(&self, name: &str) -> Option<&RuneEntry>;
    fn get_runes(&self) -> Result<&Vec<RuneEntry>, Error>;
    fn get_transactions(&self) -> &Vec<Transaction>;
}
