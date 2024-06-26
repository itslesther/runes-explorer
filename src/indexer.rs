use super::log_file::log;
use super::{adapters::sqlite::SQLite, btc_rpc::BTCRPC, rune_updaters::RuneUpdater};
use crate::adapters::db::Block;
use crate::reorg::Reorg;
use crate::runes::Runestone;
use anyhow::Error;
use async_recursion::async_recursion;
use bitcoin::network::constants::Network;
use chrono::Utc;
use rusqlite::Connection;
use std::collections::HashMap;

pub struct Indexer<'a> {
    pub chain: Network,
    pub rpc_url: String,
    // pub pool: Pool<SqliteConnectionManager>,
    pub conn: &'a mut Connection,
    // database: SQLite<'a>,
}

impl<'a> Indexer<'a> {
    #[async_recursion]
    pub async fn index_blocks(&mut self) -> Result<(), Error> {
        log("Indexing blocks")?;
        // let conn = &pool.get().unwrap();
        let mut database = SQLite {};
        database.init_tables(self.conn)?;

        let btc_rpc = &BTCRPC {
            url: self.rpc_url.clone(),
        };

        let halving_block_height: u32 = 2583205;

        let end_block_height: u32 = btc_rpc.get_block_count().await?;
        log(&format!("Current block height: {}", end_block_height))?;

        let start_block_height = if let Some(block) = database.get_latest_block(self.conn)? {
            log(&format!("Resuming from: {}", block.height + 1))?;
            u32::try_from(block.height + 1)?
        } else {
            log(&format!(
                "No blocks indexed yet, starting from the halving block: {}",
                halving_block_height
            ))?;

            halving_block_height
        };

        if start_block_height > end_block_height {
            log("No new blocks to index")?;
            return Ok(());
        }

        let mut reorg_detected = false;

        for block_height in start_block_height..=end_block_height {
            let percentage = ((block_height - start_block_height) as f32
                / (end_block_height - start_block_height) as f32)
                * 100.0;

            log(&format!(
                "{}% completed. Indexing block: {} out of {}",
                format!("{:.1$}", percentage, 2),
                block_height,
                end_block_height
            ))?;

            let start_block_fetch_time = Utc::now();

            let block = btc_rpc.get_block_by_height(block_height).await?;

            let end_block_fetch_time = Utc::now();
            let artifact_tx_count = block
                .txdata
                .iter()
                .filter(|tx| Runestone::decipher(tx).is_some())
                .count();

            let total_tx_count = block.txdata.len();

            log(&format!(
                "Block fetched in: {} seconds. Indexing Artifact txs: {} out of {}",
                end_block_fetch_time
                    .signed_duration_since(start_block_fetch_time)
                    .num_seconds(),
                artifact_tx_count,
                total_tx_count,
            ))?;

            let mut reorg = Reorg {
                database: database.clone(),
                conn: self.conn,
                rpc_url: self.rpc_url.clone(),
            };

            reorg_detected = reorg
                .detect_and_handle_reorg(
                    &block.header.prev_blockhash.to_string().to_lowercase(),
                    block_height,
                )
                .await?
                .is_some();

            if reorg_detected {
                break;
            }

            let mut rune_updater = RuneUpdater {
                database,
                conn: self.conn,
                chain: self.chain,
                burned: HashMap::new(),
                block_height,
                block_time: block.header.time,
                btc_rpc,
            };

            for (tx_index, tx) in block.txdata.iter().enumerate() {
                // let tx_percentage = ((tx_index + 1) as f32 / total_tx_count as f32) * 100.0;
                // log(&format!(
                //     "{}% transactions indexed on block: {}",
                //     format!("{:.1$}", tx_percentage, 2),
                //     block_height
                // ))?;

                rune_updater
                    .index_runes(
                        u32::try_from(tx_index)?,
                        tx,
                        tx.txid().to_string().to_lowercase().as_str(),
                    )
                    .await?;
            }

            rune_updater.update()?;

            database.insert_block(
                self.conn,
                Block {
                    height: block_height.into(),
                    hash: block.block_hash().to_string().to_lowercase(),
                    timestamp: block.header.time,
                },
            )?;
        }

        if reorg_detected {
            log("Resuming indexing")?;
            self.index_blocks().await?;
        } else {
            log("Indexing completed")?;
        }

        Ok(())
    }
}
