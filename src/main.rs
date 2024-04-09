use anyhow::Error;
use bitcoin::Address;
// use serde::{Deserialize, Serialize};

mod adapters;
mod btc_rpc;
mod runes;
mod runes_decoder;
mod rune_updaters;
mod lot;

// #[derive(Serialize, Deserialize, Debug)]
// struct RPCResponse {
//     result: ResultObj,
//     error: Option<String>,
//     id: String,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
// enum RPCValue {
//     Str(String),
//     Int(usize),
//     Bool(bool),
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct RPCRequest {
//     jsonrpc: String,
//     id: String,
//     method: String,
//     params: Vec<RPCValue>,
// }

// impl RPCRequest {
//     fn new(method: &str, params: &[RPCValue]) -> RPCRequest {
//         return RPCRequest {
//             jsonrpc: "1.0".to_string(),
//             id: "rm".to_string(),
//             method: method.to_string(),
//             params: Vec::from(params),
//         };
//     }
// }

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
    let tx = btc_rpc::get_transaction(
        "0922b1308df414934b73d46ffdafba547b5c732c549f138d2430c0469d905534",
    )
    .await?;
    // println!(
    //     "Address {:?}",
    //     Address::from_script(
    //         tx.input[0].previous_output.as_script(),
    //         bitcoin::Network::Bitcoin
    //     )?.to_string()
    // );
    println!(
        "Address {:?}",
        Address::from_script(
            tx.data.output[0].script_pubkey.as_script(),
            bitcoin::Network::Bitcoin
        )?.to_string()
    );

    println!("Best Block hash: {:?}", btc_rpc::get_best_block_hash().await?);

    println!("Runestone: {:?}", runes::Runestone::decipher(&tx.data));
    Ok(())
}
