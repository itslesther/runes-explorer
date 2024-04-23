use super::log_file::log;
use super::Error;
use super::{adapters::sqlite::SQLite, btc_rpc::BTCRPC};
use rusqlite::Connection;

pub struct Recoverable {
    pub height: u32,
    pub depth: u32,
}

pub struct Reorg<'a> {
    pub database: SQLite,
    pub conn: &'a mut Connection,
    pub rpc_url: String,
}

impl<'a> Reorg<'a> {
    pub async fn detect_and_handle_reorg(
        &mut self,
        bitcoind_prev_blockhash: &str,
        height: u32,
    ) -> Result<Option<Recoverable>, Error> {
        let max_recoverable_reorg_depth: u32 = 20;

        match self
            .database
            .get_block_by_height(self.conn, u64::from(height.checked_sub(1).unwrap()))?
        {
            Some(prev_block) => {
                if prev_block.hash == bitcoind_prev_blockhash.to_string() {
                    return Ok(None);
                } else {
                    log(&format!("Reorg detected at height: {}", height))?;

                    let btc_rpc = &BTCRPC {
                        url: self.rpc_url.clone(),
                    };

                    for depth in 1..=max_recoverable_reorg_depth {
                        let index_block = self
                            .database
                            .get_block_by_height(self.conn, u64::from(height - depth))?;

                        let bitcoind_block_hash = btc_rpc
                            .get_block_hash_by_height(height.saturating_sub(depth))
                            .await
                            .ok();

                        if index_block.map(|b| b.hash) == bitcoind_block_hash {
                            let reorg_from_height = height.saturating_sub(depth);

                            log(&format!(
                                "Recovering at depth: {}. Rolling back to indexed height: {}",
                                depth, reorg_from_height
                            ))?;

                            self.database
                                .reorg_blocks(self.conn, u64::from(reorg_from_height))?;

                            log(&format!("Roll back finished"))?;

                            return Ok(Some(Recoverable { height, depth }));
                        }
                    }

                    panic!("Reorg depth exceeded the recoverable limit");
                }
            }
            None => {
                return Ok(None);
            }
        }
    }
}
