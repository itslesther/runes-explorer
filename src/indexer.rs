use std::collections::HashMap;

use super::{adapters::sqlite::SQLite, btc_rpc, rune_updaters::RuneUpdater};
use crate::adapters::db::Database;
use bitcoin::network::constants::Network;

use anyhow::Error;

pub struct Indexer {
    database: SQLite,
}

impl Indexer {
    pub fn init() -> Indexer {
        Indexer {
            database: SQLite::init().unwrap(),
        }
    }

    pub async fn index_blocks(&self) -> Result<(), Error> {
        let database = &self.database;
        let start_block_height: u32 = u32::try_from(database.get_block_height()?)?;
        let latest_block_height: u32 = btc_rpc::get_latest_validated_block_height().await?;

        for block_height in start_block_height..=latest_block_height {
            println!("Indexing block: {}", block_height);

            let block = btc_rpc::get_block_by_height(block_height).await?;

            println!("Block Transaction count: {:?}", block.n_tx);

            let txs = block.tx;

            for (tx_index, tx) in txs.iter().enumerate() {
              // let database = self.database;

              // let rune_updater = RuneUpdater::init(database,  Network::Testnet, block_height, u32::try_from(block.time)?);
                // let rune_updater = RuneUpdater {
                //     database,
                //     chain: Network::Testnet,
                //     burned: HashMap::new(),
                //     block_height,
                //     block_time: u32::try_from(block.time)?,
                // };

                // rune_updater
                //     .index_runes(
                //         u32::try_from(tx_index)?,
                //         &tx.data,
                //         &tx.raw.txid.to_lowercase(),
                //     )
                //     .await?;
            }
        }

        // self.database.increase_block_height()?;
        Ok(())
    }
}
