use anyhow::Error;

mod adapters;
mod btc_rpc;
mod indexer;
mod log_file;
mod lot;
mod rune_updaters;
mod runes;
mod utils;

// use crate::adapters::db::Database;
use bitcoin::network::constants::Network;
use indexer::Indexer;

// async fn get_raw_transaction() -> Result<(), Error> {
//     let mut script_pubkey: Vec<u8> = bitcoin::script::Builder::new()
//         .push_opcode(bitcoin::opcodes::all::OP_RETURN)
//         .push_opcode(Runestone::MAGIC_NUMBER)
//         .into_script()
//         .into_bytes();

//     script_pubkey.push(bitcoin::opcodes::all::OP_PUSHBYTES_4.to_u8());

//     let some_runestone = Runestone::from_transaction(&bitcoin::Transaction {
//         input: Vec::new(),
//         // output: vec![
//         //     bitcoin::TxOut {
//         //     script_pubkey: bitcoin::script::Builder::new()
//         //       .push_opcode(bitcoin::opcodes::all::OP_RETURN)
//         //       .push_opcode(Runestone::MAGIC_NUMBER)
//         //       .push_opcode(bitcoin::opcodes::all::OP_VERIFY)
//         //       .push_slice([0])
//         //       .push_slice::<&bitcoin::blockdata::script::PushBytes>(runes::varint::encode(1).as_slice().try_into().unwrap())
//         //       .push_slice::<&bitcoin::blockdata::script::PushBytes>(runes::varint::encode(1).as_slice().try_into().unwrap())
//         //       .push_slice([2, 0])
//         //       .into_script(),
//         //     value: 0,
//         //   },
//         // ],
//         output: vec![bitcoin::TxOut {
//             script_pubkey: Runestone {
//                 // edicts: Vec::new(),
//                 edicts: vec![
//                     Edict {
//                         id: RuneId::new(0, 0).unwrap(),
//                         amount: 2,
//                         output: 0,
//                     },
//                     Edict {
//                         id: RuneId::new(0, 0).unwrap(),
//                         amount: 5,
//                         output: 0,
//                     },
//                 ],
//                 etching: Some(Etching {
//                     divisibility: Some(18),
//                     premine: Some(100),
//                     rune: Some(Rune::from_str("LESTHER")?),
//                     symbol: Some('L'),
//                     // symbol: None,
//                     terms: Some(Terms {
//                         amount: Some(2),
//                         cap: Some(200),
//                         height: (None, None),
//                         offset: (Some(10), Some(20)),
//                     }),
//                     // spacers: Some(63),
//                     spacers: Some(SpacedRune::from_str("L.E.S.T.H.E.R").unwrap().spacers),
//                 }),
//                 cenotaph: 0,
//                 mint: RuneId::new(0, 0),
//                 pointer: Some(0), // ..default()
//             }
//             .encipher(),
//             // script_pubkey: bitcoin::ScriptBuf::from_bytes(script_pubkey),
//             value: 0,
//         }],
//         lock_time: bitcoin::blockdata::locktime::absolute::LockTime::ZERO,
//         version: 2,
//     });

//     if some_runestone.is_none() {
//         println!("No Runestone found in transaction");
//         return Ok(());
//     }

//     let runestone = some_runestone.unwrap();
//     println!("Runestone: {:?}", runestone);

//     let rune_name = SpacedRune {
//         rune: runestone.etching.unwrap().rune.unwrap(),
//         spacers: runestone.etching.unwrap().spacers.unwrap_or(0),
//     };

//     let rune_symbol = runestone.etching.unwrap().symbol.unwrap_or('¤');

//     println!("Rune name: {:?}", rune_name.to_string());
//     println!("Rune symbol: {:?}", rune_symbol);
//     println!("Is cenotaph: {:?}", runestone.is_cenotaph());
//     println!(
//         "Cenotaph reasons: {:?}",
//         runestone
//             .cenotaph_reasons()
//             .iter()
//             .map(|cenotaph| cenotaph.to_string())
//             .collect::<Vec<String>>()
//             .join(",")
//     );

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    let indexer = Indexer {
        chain: Network::Bitcoin,
    };

    indexer.index_blocks().await?;

    // // let tx = btc_rpc::get_transaction(
    // //     "e279cb8e09983e63117f7879f2393e3fbc1d132f5c3c8f4adae3bce7799556c4",
    // // )
    // // .await?;

    // // println!("Transaction raw: {:?}", tx.raw);

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

    // let mut db: adapters::mock_db::MockDb = adapters::mock_db::MockDb {
    //     rune_entries: Vec::new(),
    //     transactions: Vec::new(),
    //     rune_transfers: Vec::new(),
    //     txos: Vec::new(),
    //     statistics: adapters::db::Statistics {
    //         block_height: 0,
    //     }
    // };

    // db.add_rune_entry(adapters::db::RuneEntry {
    //     etching_tx_id: "e279cb8".to_string(),
    //     block_height: 1,
    //     rune_id: "1:2".to_string(),
    //     name: "L.E.S.T.H.E.R".to_string(),
    //     raw_name: "LESTHER".to_string(),
    //     symbol: Some('L'),
    //     divisibility: 18,
    //     premine: 100,
    //     terms: Some(adapters::db::Terms {
    //         amount: Some(2),
    //         cap: Some(200),
    //         height_start: None,
    //         height_end: None,
    //         offset_start: Some(10),
    //         offset_end: Some(20),
    //     }),
    //     burned: 0,
    //     mint_count: 0,
    //     timestamp: 0,
    //     is_cenotapth: false,
    //     rune_number: 0,
    // })?;

    // // println!("Rune entries: {:?}", db.get_runes()?);

    // // println!("mint count: {:?}", db.get_runes()?[0].mint_count);
    // // db.update_rune_entry_mint_count("1:2")?;
    // // println!("mint count: {:?}", db.get_runes()?[0].mint_count);

    // // println!("burn count: {:?}", db.get_runes()?[0].burned);
    // // db.increase_rune_entry_burned("1:2", 5)?;
    // // println!("burn count: {:?}", db.get_runes()?[0].burned);

    // let latest_block_height = btc_rpc::get_latest_validated_block_height().await?;
    // let start_block_height = 2583205;
    // // let start_block_height = 2583205;

    // println!("Latest height: {:?}", latest_block_height);

    // let mut artifact_txs: Vec<String> = Vec::new();

    // for block_height in start_block_height..=latest_block_height {
    //     println!("\nIndexing Block height: {:?}", block_height);

    //     let block = btc_rpc::get_block_by_height(block_height).await?;

    //     println!("Tx count: {:?}", block.n_tx);

    //     let txs = block.tx;

    //     for tx in txs.iter() {
    //         let artifact = runes::Runestone::decipher(&tx.data);

    //         let is_artifact = artifact.is_some();

    //         let is_runestone = if let Some(runes::Artifact::Runestone(_)) = artifact {
    //             true
    //         } else {
    //             false
    //         };

    //         let is_cenotapth = if let Some(runes::Artifact::Cenotaph(_)) = artifact {
    //             true
    //         } else {
    //             false
    //         };

    //         let cenotapth_messages = if let Some(runes::Artifact::Cenotaph(cenotaph)) = artifact {
    //             Some(
    //                 cenotaph
    //                     .flaws()
    //                     .iter()
    //                     .map(|flaw| flaw.to_string())
    //                     .collect::<Vec<String>>()
    //                     .join(","),
    //             )
    //         } else {
    //             None
    //         };

    //         // println!("Tx: {:?} is artifact: {:?}", &tx.raw.txid, is_artifact);

    //         if is_artifact {
    //             artifact_txs.push(tx.raw.txid.clone());

    //             let mut data_file = OpenOptions::new()
    //                 .append(true)
    //                 .open("data.txt")
    //                 .expect("cannot open file");

    //             // Write to a file
    //             data_file
    //                 .write((tx.raw.txid).as_bytes())
    //                 .expect("write failed");

    //             data_file
    //             .write(("\n").as_bytes())
    //             .expect("write failed");

    //             println!(
    //                 "Tx: {:?} is runestone: {:?}, is cenotapth: {:?}, cenotapth messages: {:?}",
    //                 tx.raw.txid, is_runestone, is_cenotapth, cenotapth_messages
    //             );
    //         }
    //     }
    // }

    // println!("\n\nArtifact txs: {:?}", artifact_txs);
    // // println!("Runestone: {:?}", runestone);

    Ok(())
}
