use crate::adapters::sqlite::SQLite;
use crate::server::schemas::*;
use crate::{adapters::db::Database, AppState};
use actix_web::{get, web, HttpResponse, Responder};

#[utoipa::path(
    responses(
        (status = 200, description = "Returns created runes", body = RuneEntryListResponse)
    )
)]
#[get("/runes")]
async fn get_runes(state: web::Data<AppState>) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite { };

    let response = RuneEntryListResponse {
        data: database.get_runes(conn).unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns rune details", body = RuneEntryDetailsResponse)),
    params(RuneEntryDetailsParams)
)]
#[get("/runes/{rune_id}")]
async fn get_rune_by_id(
    state: web::Data<AppState>,
    path_params: web::Path<RuneEntryDetailsParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite { };

    let rune_id = &path_params.rune_id;

    let response = RuneEntryDetailsResponse {
        data: database.get_rune_by_id(conn, rune_id).unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns balance for a the specified address and rune", body = AddressBalanceResponse)),
    params(AddressBalanceParams)
)]
#[get("/address/{address}/runes/{rune_id}/balance")]
async fn get_address_balance_by_rune_id(
    state: web::Data<AppState>,
    path_params: web::Path<AddressBalanceParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite { };

    let address = &path_params.address.to_lowercase();
    let rune_id = &path_params.rune_id;

    let response = AddressBalanceResponse {
        data: database
            .get_address_balance_by_rune_id(conn, address, rune_id)
            .unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns balances object for a the specified address", body = AddressBalanceListResponse)),
    params(AddressBalanceListParams)
)]
#[get("/address/{address}/runes/balance-list")]
async fn get_address_balance_list(
    state: web::Data<AppState>,
    path_params: web::Path<AddressBalanceListParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite {  };

    let address = &path_params.address.to_lowercase();

    let response = AddressBalanceListResponse {
        data: database.get_address_balance_list(conn, address).unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns runes utxo details", body = RunesTXOByOutputIndexResponse)),
    params(RunesTXOByOutputIndexParams)
)]
#[get("/runes/utxo/{tx_id}/{index}")]
async fn get_runes_txo_by_output_index(
    state: web::Data<AppState>,
    path_params: web::Path<RunesTXOByOutputIndexParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite {  };

    let tx_id = &path_params.tx_id.to_lowercase();
    let index = path_params.index;

    let response = RunesTXOByOutputIndexResponse {
        data: database
            .get_runes_txo_by_output_index(conn, tx_id, index)
            .unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns utxo for the specified address and rune", body = AddressRunesUTXOByRuneIdResponse)),
    params(AddressRunesUTXOByRuneIdParams)
)]
#[get("/address/{address}/runes/{rune_id}/utxo")]
async fn get_address_runes_utxo_by_rune_id(
    state: web::Data<AppState>,
    path_params: web::Path<AddressRunesUTXOByRuneIdParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite { };

    let address = &path_params.address.to_lowercase();
    let rune_id = &path_params.rune_id;

    let response = AddressRunesUTXOByRuneIdResponse {
        data: database
            .get_address_runes_utxo_by_rune_id(conn,address, rune_id)
            .unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns transaction list", body = TransactionListResponse)),
)]
#[get("/transactions")]
async fn get_transaction_list(state: web::Data<AppState>) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite {  };

    let response = TransactionListResponse {
        data: database.get_transactions(conn).unwrap(),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    responses((status = 200, description = "Returns transaction details with runes inputs and outputs", body = TransactionWithRunesResponse)),
    params(TransactionWithRunesParams)
)]
#[get("/transactions/{tx_id}")]
async fn get_transaction_with_runes_txo(
    state: web::Data<AppState>,
    path_params: web::Path<TransactionWithRunesParams>,
) -> impl Responder {
    let conn = &mut state.pool.get().unwrap();

    let database = SQLite {  };

    let tx_id = &path_params.tx_id.to_lowercase();

    let Some(transaction) = database.get_transaction(conn, tx_id).unwrap() else {
        return HttpResponse::NotFound().finish();
    };

    let runes_txo = database.get_transaction_runes_txo(conn, tx_id).unwrap();

    let rune_entry = database.get_rune_by_etched_tx_id(conn, tx_id).unwrap();

    let response = TransactionWithRunesResponse {
        data: TransactionWithRunesTXO {
            tx_id: tx_id.to_string(),
            block_height: transaction.block_height,
            inputs: runes_txo
                .clone()
                .iter()
                .filter(|rt| rt.spent_tx_id == Some(tx_id.to_string()))
                .cloned()
                .collect(),
            outputs: runes_txo
                .clone()
                .iter()
                .filter(|rt| rt.tx_id == tx_id.to_string())
                .cloned()
                .collect(),
            is_runestone: transaction.is_runestone,
            is_cenotapth: transaction.is_cenotapth,
            cenotapth_message: transaction.cenotapth_message,
            timestamp: transaction.timestamp,
            etched_rune_id: rune_entry.map(|r| r.rune_id),
        },
    };

    HttpResponse::Ok().json(response)
}
