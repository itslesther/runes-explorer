mod adapters;
mod btc_rpc;
mod indexer;
mod log_file;
mod lot;
mod reorg;
mod rune_updaters;
mod runes;
mod server;
mod utils;

use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use adapters::db;
use anyhow::Error;
use bitcoin::network::constants::Network;
use indexer::Indexer;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use server::{schemas, services};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

struct AppState {
    pub pool: Pool<SqliteConnectionManager>,
}

#[utoipa::path(
    responses(
        (status = 200, description = "API is alive and well!", body = SimpleStatus)
    )
)]
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json(schemas::SimpleStatus {
        message: "Hello World".to_string(),
    })
}

// #[tokio::main]
#[actix_web::main]
async fn main() -> Result<(), Error> {
    let manager = SqliteConnectionManager::file("./indexer.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Error building a connection pool");

    let mut indexer = Indexer {
            chain: Network::Testnet,
            rpc_url: "https://powerful-cool-bush.btc-testnet.quiknode.pro/cf40fbe86ac4d435ce4799c8aae18c1dc65b96c8".to_string(),
            conn: &mut pool.clone().get().unwrap(),
    };

    indexer.index_blocks().await?;

    #[derive(OpenApi)]
    #[openapi(
        paths(
            hello,
            services::get_runes,
            services::get_rune_by_id,
            services::get_address_balance_by_rune_id,
            services::get_address_balance_list,
            services::get_runes_txo_by_output_index,
            services::get_address_runes_utxo_by_rune_id,
            services::get_transaction_list,
            services::get_transaction_with_runes_txo,
        ),
        components(schemas(
            schemas::SimpleStatus,
            schemas::RuneEntryListResponse,
            schemas::RuneEntryDetailsResponse,
            schemas::AddressBalanceResponse,
            schemas::AddressBalanceListResponse,
            schemas::RunesTXOByOutputIndexResponse,
            schemas::AddressRunesUTXOByRuneIdResponse,
            schemas::TransactionListResponse,
            schemas::TransactionWithRunesResponse,
            schemas::TransactionWithRunesTXO,
            db::RuneEntry,
            db::Terms,
            db::RuneTXO,
            db::Transaction,
        ))
    )]
    struct ApiDoc;
    let openapi = ApiDoc::openapi();

    log_file::log("HTTP Server started on http://localhost:8080")?;

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { pool: pool.clone() }))
            .service(hello)
            .service(services::get_runes)
            .service(services::get_rune_by_id)
            .service(services::get_address_balance_by_rune_id)
            .service(services::get_address_balance_list)
            .service(services::get_runes_txo_by_output_index)
            .service(services::get_address_runes_utxo_by_rune_id)
            .service(services::get_transaction_list)
            .service(services::get_transaction_with_runes_txo)
            .service(
                SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind(("localhost", 8080))?
    .run()
    .await?;

    Ok(())
}
