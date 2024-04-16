use std::collections::HashMap;

use anyhow::Error;
use serde::*;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Statistics {
    pub block_height: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuneEntry {
    pub etching_tx_id: String,
    pub block_height: u64,
    pub rune_id: String,
    pub name: String,
    pub raw_name: String,
    pub symbol: Option<char>,
    pub divisibility: u8,
    pub premine: u128,
    pub terms: Option<Terms>,
    pub burned: u128,
    pub mint_count: u128, // validation for Terms::cap
    pub timestamp: u32,
    pub is_cenotapth: bool,
    pub cenotapth_messages: Option<String>,
    pub rune_number: u128,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Transaction {
    pub tx_id: String,
    pub block_height: u64,
    // pub inputs: Vec<TXO>,
    // pub outputs: Vec<TXO>,
    pub is_artifact: bool,
    pub is_runestone: bool,
    pub is_cenotapth: bool,
    pub cenotapth_messages: Option<String>,
    pub timestamp: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Terms {
    pub amount: Option<u128>,
    pub cap: Option<u128>,
    pub height_start: Option<u64>,
    pub height_end: Option<u64>,
    pub offset_start: Option<u64>,
    pub offset_end: Option<u64>,
}
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TXO {
    pub tx_id: String,
    pub output_index: u32,
    pub block_height: u64,
    pub value: u128,
    pub address: Option<String>,
    pub address_lowercase: Option<String>,
    // pub rune_transfers: Vec<RuneTransfer>,
    pub is_unspent: bool,
    pub spent_tx_id: Option<String>,
    pub timestamp: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuneTXO {
    pub tx_id: String,
    pub output_index: u32,
    pub block_height: u64,
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
    fn get_runes_txo_by_output_index(
        &self,
        tx_id: &str,
        output_index: u32,
    ) -> Result<Vec<RuneTXO>, Error>;
    fn get_rune_by_id(&self, rune_id: &str) -> Result<Option<RuneEntry>, Error>;
    fn update_rune_entry_mint_count(&mut self, rune_id: &str) -> Result<(), Error>;
    fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error>;
    fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<RuneEntry>, Error>;
    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Error>;
    fn add_rune_entry(&mut self, rune_entry: RuneEntry) -> Result<(), Error>;
    fn add_rune_txo(&mut self, rune_txo: RuneTXO) -> Result<(), Error>;
    fn get_txo(&mut self, tx_id: &str, output_index: u32) -> Result<Option<TXO>, Error>;
    fn mark_utxo_as_spent(
        &mut self,
        tx_id: &str,
        output_index: u32,
        spent_tx_id: &str,
    ) -> Result<(), Error>;
    fn add_txo(&mut self, txo: TXO) -> Result<(), Error>;
    fn get_address_balance_by_rune_id(&self, address: &str, rune_id: &str) -> Result<u128, Error>;
    fn get_address_balance_list(&self, address: &str) -> Result<HashMap<String, u128>, Error>;
    fn get_address_runes_txo(&self, address: &str) -> Result<Vec<RuneTXO>, Error>;
    fn get_address_runes_utxo_by_rune_id(
        &self,
        address: &str,
        rune_id: &str,
    ) -> Result<Vec<RuneTXO>, Error>;
    fn get_rune_count(&self) -> Result<u128, Error>;
    fn get_block_height(&self) -> Result<u64, Error>;
    fn set_block_height(&mut self, block_height: u64) -> Result<(), Error>;
    fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, Error>;
    fn get_runes(&self) -> Result<Vec<RuneEntry>, Error>;
}
