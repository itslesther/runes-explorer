use std::collections::HashMap;

use crate::adapters::db::*;
use serde::*;
use utoipa::{IntoParams, ToSchema};

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SimpleStatus {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuneEntryListResponse {
    pub data: Vec<RuneEntry>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct RuneEntryDetailsParams {
    pub rune_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RuneEntryDetailsResponse {
    pub data: Option<RuneEntry>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct AddressBalanceParams {
    pub rune_id: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AddressBalanceResponse {
    pub data: u128,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct AddressBalanceListParams {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AddressBalanceListResponse {
    pub data: HashMap<String, u128>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct RunesTXOByOutputIndexParams {
    pub tx_id: String,
    pub index: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RunesTXOByOutputIndexResponse {
    pub data: Vec<RuneTXO>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct AddressRunesUTXOByRuneIdParams {
    pub address: String,
    pub rune_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct AddressRunesUTXOByRuneIdResponse {
    pub data: Vec<RuneTXO>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct TransactionWithRunesParams {
    pub tx_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TransactionWithRunesTXO {
    pub tx_id: String,
    pub block_height: u64,
    pub inputs: Vec<RuneTXO>,
    pub outputs: Vec<RuneTXO>,
    // pub is_artifact: bool,
    pub is_runestone: bool,
    pub is_cenotapth: bool,
    pub cenotapth_message: Option<String>,
    pub timestamp: u32,
    pub etched_rune_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TransactionWithRunesResponse {
    pub data: TransactionWithRunesTXO,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct TransactionListResponse {
    pub data: Vec<Transaction>,
}

