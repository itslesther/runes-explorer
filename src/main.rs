mod adapters;
mod btc_rpc;
mod indexer;
mod log_file;
mod lot;
mod rune_updaters;
mod runes;
mod server;
mod utils;

use anyhow::Error;
use actix_web::{get, web::Data, App, HttpResponse, HttpServer, Responder};
use adapters::db;
use bitcoin::network::constants::Network;
use indexer::Indexer;
use r2d2::Pool;
use server::{schemas, services};
use r2d2_sqlite::SqliteConnectionManager;
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
    let manager = SqliteConnectionManager::file("./runes.db");
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Error building a connection pool");

    let indexer = Indexer {
            chain: Network::Testnet,
            rpc_url: "https://powerful-cool-bush.btc-testnet.quiknode.pro/cf40fbe86ac4d435ce4799c8aae18c1dc65b96c8".to_string(),
            conn: &pool.clone().get().unwrap(),
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
        ),
        components(schemas(
            schemas::SimpleStatus,
            schemas::RuneEntryListResponse,
            schemas::RuneEntryDetailsResponse,
            schemas::AddressBalanceResponse,
            schemas::AddressBalanceListResponse,
            schemas::RunesTXOByOutputIndexResponse,
            schemas::AddressRunesUTXOByRuneIdResponse,
            db::RuneEntry,
            db::Terms,
            db::RuneTXO,
        ),)
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
            .service(
                SwaggerUi::new("/swagger/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )

        // .configure(services::config)
    })
    .bind(("localhost", 8080))?
    .run()
    .await?;

//     let BTCRPC = btc_rpc::BTCRPC {
//         url: "https://powerful-cool-bush.btc-testnet.quiknode.pro/cf40fbe86ac4d435ce4799c8aae18c1dc65b96c8".to_string()
//     };

//    let tx =  BTCRPC.get_transaction("8b37b98cce0e4f7a6210823986f7c2b528ca0c93ac091dbb7d9a7f920daf3179").await?;
//     let artifact = runes::Runestone::decipher(&tx.data);
//     println!("Artifact: {:?}", artifact);

    // // println!(
    // //     "Address {:?}",
    // //     Address::from_script(
    // //         tx.data.output[0].script_pubkey.as_script(),
    // //         bitcoin::Network::Bitcoin
    // //     )?.to_string()
    // // );

    // // println!("Best Block hash: {:?}", btc_rpc::get_best_block_hash().await?);

    // //     let mut script_pubkey: Vec<u8> = bitcoin::script::Builder::new()
    // //     .push_opcode(bitcoin::opcodes::all::OP_RETURN)
    // //     .push_opcode(Runestone::MAGIC_NUMBER)
    // //     .into_script()
    // //     .into_bytes();

    // // script_pubkey.push(bitcoin::opcodes::all::OP_PUSHBYTES_4.to_u8());

    // let tx = &bitcoin::Transaction {
    //     input: Vec::new(),
    //     // output: vec![
    //     //     bitcoin::TxOut {
    //     //     script_pubkey: bitcoin::script::Builder::new()
    //     //       .push_opcode(bitcoin::opcodes::all::OP_RETURN)
    //     //       .push_opcode(runes::Runestone::MAGIC_NUMBER)
    //     //       .push_opcode(bitcoin::opcodes::all::OP_VERIFY)
    //     //       .push_slice([0])
    //     //       .push_slice::<&bitcoin::blockdata::script::PushBytes>(runes::varint::encode(1).as_slice().try_into().unwrap())
    //     //       .push_slice::<&bitcoin::blockdata::script::PushBytes>(runes::varint::encode(1).as_slice().try_into().unwrap())
    //     //       .push_slice([2, 0])
    //     //       .into_script(),
    //     //     value: 0,
    //     //   },
    //     // ],
    //     output: vec![bitcoin::TxOut {
    //         script_pubkey: runes::Runestone {
    //             edicts: vec![
    //                 runes::Edict {
    //                     id: runes::RuneId::new(0, 0).unwrap(),
    //                     amount: 2,
    //                     output: 0,
    //                 },
    //                 runes::Edict {
    //                     id: runes::RuneId::new(0, 0).unwrap(),
    //                     amount: 5,
    //                     output: 0,
    //                 },
    //             ],
    //             etching: Some(runes::Etching {
    //                 divisibility: Some(18),
    //                 premine: Some(100),
    //                 rune: ("LESTHER").parse::<runes::Rune>().ok(),
    //                 symbol: Some('L'),
    //                 // symbol: None,
    //                 terms: Some(runes::Terms {
    //                     amount: Some(2),
    //                     cap: Some(200),
    //                     height: (None, None),
    //                     offset: (Some(10), Some(20)),
    //                 }),
    //                 // spacers: Some(63),
    //                 spacers: ("L.E.S.T.H.E.R")
    //                     .parse::<runes::SpacedRune>()
    //                     .ok()
    //                     .map(|rune| rune.spacers),
    //             }),
    //             mint: runes::RuneId::new(0, 0),
    //             pointer: Some(0), // ..default()
    //         }
    //         .encipher(),
    //         // script_pubkey: bitcoin::ScriptBuf::from_bytes(script_pubkey),
    //         value: 0,
    //     }],
    //     lock_time: bitcoin::blockdata::locktime::absolute::LockTime::ZERO,
    //     version: 2,
    // };

    // let artifact = runes::Runestone::decipher(tx);
    // let runestone = if let Some(runes::Artifact::Runestone(_runestone)) = &artifact {
    //     _runestone
    // } else {
    //     println!("No Runestone found in transaction");
    //     return Ok(());
    // };

    // println!("Runestone: {:?}", runestone);

    Ok(())
}
