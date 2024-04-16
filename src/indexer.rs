use std::collections::HashMap;

use super::{adapters::sqlite::SQLite, btc_rpc::BTCRPC, rune_updaters::RuneUpdater};
use crate::adapters::db::Database;
use bitcoin::network::constants::Network;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use super::log_file::log;
use anyhow::Error;
use rusqlite::Connection;

pub struct Indexer<'a> {
    pub chain: Network,
    pub rpc_url: String,
    // pub pool: Pool<SqliteConnectionManager>,
    pub conn: &'a Connection,
    // database: SQLite<'a>,
}

impl <'a> Indexer<'a>  {
    pub async fn index_blocks(&self) -> Result<(), Error> {
        log("Indexing blocks")?;
        // let conn = &pool.get().unwrap();
        let mut database = SQLite { conn: self.conn };
        database.init_tables()?;

        let btc_rpc = &BTCRPC {
            url: self.rpc_url.clone(),
        };

        let halving_block_height: u32 = 2583205;

        let end_block_height: u32 = halving_block_height + 50;
        // let end_block_height: u32 = btc_rpc.get_latest_validated_block_height().await?;
        log(&format!("Current block height: {}", end_block_height))?;

        let mut start_block_height: u32 = u32::try_from(database.get_block_height()?)?;

        if start_block_height == 0 {
            log(&format!(
                "No blocks indexed yet, starting from the halving block: {}",
                halving_block_height
            ))?;

            database.set_block_height(halving_block_height.into())?;
            start_block_height = halving_block_height;
        } else {
            log(&format!("Resuming from block: {}", start_block_height))?;
        }

        if start_block_height >= end_block_height {
            log("No new blocks to index")?;
            return Ok(());
        }

        for block_height in start_block_height..=end_block_height {
            let percentage = ((block_height - start_block_height) as f32
                / (end_block_height - start_block_height) as f32)
                * 100.0;

            log(&format!(
                "{}. Indexing block: {}",
                format!("{:.1$}% completed", percentage, 2),
                block_height
            ))?;

            let block = btc_rpc.get_block_by_height(block_height).await?;

            let mut rune_updater = RuneUpdater {
                database,
                chain: self.chain,
                burned: HashMap::new(),
                block_height,
                block_time: block.header.time,
                btc_rpc,
            };

            for (tx_index, tx) in block.txdata.iter().enumerate() {
                rune_updater
                    .index_runes(
                        u32::try_from(tx_index)?,
                        tx,
                        tx.txid().to_string().to_lowercase().as_str(),
                    )
                    .await?;
            }

            rune_updater.update()?;
            database.set_block_height(block_height.into())?;
        }

        log("Indexing completed")?;

        Ok(())
    }
}
