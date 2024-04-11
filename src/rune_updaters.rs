use crate::adapters::db::Database;

use super::adapters::db::{RuneEntry, RuneTransfer, Terms, Transaction as DbTransaction, TXO};
use super::adapters::mock_db::MockDb as Db;
// use super::adapters::sqlite::SQLite as Db;
use super::btc_rpc;
use super::lot::Lot;
use super::runes::*;
use super::utils;

pub struct RuneUpdater {
    pub database: Db,
    pub chain: Network,
    pub burned: HashMap<RuneId, Lot>,
    pub block_height: u32,
    pub block_time: u32,
}

impl RuneUpdater {
    pub fn init(database: Db, chain: Network, block_height: u32, block_time: u32) -> RuneUpdater {
        RuneUpdater {
            database,
            chain,
            burned: HashMap::new(),
            block_height,
            block_time,
        }
    }

    pub async fn index_runes(
        &mut self,
        tx_index: u32,
        tx: &Transaction,
        tx_id: &str,
    ) -> Result<()> {
        let artifact = Runestone::decipher(tx);

        let mut unallocated = self.unallocated(tx)?;
        let mut allocated: Vec<HashMap<RuneId, Lot>> = vec![HashMap::new(); tx.output.len()];

        self.mark_txs_as_spent(tx, tx_id)?;

        if let Some(artifact) = &artifact {
            self.add_transaction(tx_id, &artifact)?;
            self.add_txo(tx, tx_id)?;

            if let Some(id) = artifact.mint() {
                if let Some(amount) = self.mint(id)? {
                    *unallocated.entry(id).or_default() += amount;
                }
            }

            let etched = self.etched(tx_index, tx, artifact).await?;

            if let Artifact::Runestone(runestone) = artifact {
                if let Some((id, ..)) = etched {
                    *unallocated.entry(id).or_default() +=
                        runestone.etching.unwrap().premine.unwrap_or_default();
                }

                for Edict { id, amount, output } in runestone.edicts.iter().copied() {
                    let amount = Lot(amount);

                    // edicts with output values greater than the number of outputs
                    // should never be produced by the edict parser
                    let output = usize::try_from(output).unwrap();
                    assert!(output <= tx.output.len());

                    let id = if id == RuneId::default() {
                        let Some((id, ..)) = etched else {
                            continue;
                        };

                        id
                    } else {
                        id
                    };

                    let Some(balance) = unallocated.get_mut(&id) else {
                        continue;
                    };

                    let mut allocate = |balance: &mut Lot, amount: Lot, output: usize| {
                        if amount > 0 {
                            *balance -= amount;
                            *allocated[output].entry(id).or_default() += amount;
                        }
                    };

                    if output == tx.output.len() {
                        // find non-OP_RETURN outputs
                        let destinations = tx
                            .output
                            .iter()
                            .enumerate()
                            .filter_map(|(output, tx_out)| {
                                (!tx_out.script_pubkey.is_op_return()).then_some(output)
                            })
                            .collect::<Vec<usize>>();

                        if amount == 0 {
                            // if amount is zero, divide balance between eligible outputs
                            let amount = *balance / destinations.len() as u128;
                            let remainder =
                                usize::try_from(*balance % destinations.len() as u128).unwrap();

                            for (i, output) in destinations.iter().enumerate() {
                                allocate(
                                    balance,
                                    if i < remainder { amount + 1 } else { amount },
                                    *output,
                                );
                            }
                        } else {
                            // if amount is non-zero, distribute amount to eligible outputs
                            for output in destinations {
                                allocate(balance, amount.min(*balance), output);
                            }
                        }
                    } else {
                        // Get the allocatable amount
                        let amount = if amount == 0 {
                            *balance
                        } else {
                            amount.min(*balance)
                        };

                        allocate(balance, amount, output);
                    }
                }
            }

            if let Some((id, rune)) = etched {
                self.create_rune_entry(tx_id, artifact, id, rune)?;
            }
        }

        let mut burned: HashMap<RuneId, Lot> = HashMap::new();

        if let Some(Artifact::Cenotaph(_)) = artifact {
            for (id, balance) in unallocated {
                *burned.entry(id).or_default() += balance;
            }
        } else {
            let pointer = artifact
                .map(|artifact| match artifact {
                    Artifact::Runestone(runestone) => runestone.pointer,
                    Artifact::Cenotaph(_) => unreachable!(),
                })
                .unwrap_or_default();

            // assign all un-allocated runes to the default output, or the first non
            // OP_RETURN output if there is no default, or if the default output is
            // too large
            if let Some(vout) = pointer
                .map(|pointer| pointer.into_usize())
                .inspect(|&pointer| assert!(pointer < allocated.len()))
                .or_else(|| {
                    tx.output
                        .iter()
                        .enumerate()
                        .find(|(_vout, tx_out)| !tx_out.script_pubkey.is_op_return())
                        .map(|(vout, _tx_out)| vout)
                })
            {
                for (id, balance) in unallocated {
                    if balance > 0 {
                        *allocated[vout].entry(id).or_default() += balance;
                    }
                }
            } else {
                for (id, balance) in unallocated {
                    if balance > 0 {
                        *burned.entry(id).or_default() += balance;
                    }
                }
            }
        }

        // update outpoint balances

        for (vout, balances) in allocated.into_iter().enumerate() {
            if balances.is_empty() {
                continue;
            }

            // increment burned balances
            if tx.output[vout].script_pubkey.is_op_return() {
                for (id, balance) in &balances {
                    *burned.entry(*id).or_default() += *balance;
                }
                continue;
            }

            let mut balances = balances.into_iter().collect::<Vec<(RuneId, Lot)>>();

            // Sort balances by id so tests can assert balances in a fixed order
            balances.sort();

            self.add_rune_transfers(tx, tx_id, vout, balances)?;
        }

        // increment entries with burned runes
        for (id, amount) in burned {
            *self.burned.entry(id).or_default() += amount;
        }

        Ok(())
    }

    pub fn update(mut self) -> Result {
        for (rune_id, burned) in self.burned {
            self.database
                .increase_rune_entry_burned(&rune_id.to_string(), burned.n())?;
        }

        Ok(())
    }

    fn add_transaction(&mut self, tx_id: &str, artifact: &Artifact) -> Result {
        self.database.add_transaction(DbTransaction {
            tx_id: tx_id.to_string(),
            is_artifact: true,
            // is_artifact: artifact.is_some(),
            is_runestone: if let Artifact::Runestone(_) = artifact {
                // is_runestone: if let Some(Artifact::Runestone(_)) = artifact {
                true
            } else {
                false
            },
            is_cenotapth: if let Artifact::Cenotaph(_) = artifact {
                // is_cenotapth: if let Some(Artifact::Cenotaph(_)) = artifact {
                true
            } else {
                false
            },
            cenotapth_messages: if let Artifact::Cenotaph(cenotaph) = artifact {
                // cenotapth_messages: if let Some(Artifact::Cenotaph(cenotaph)) = artifact {
                Some(
                    cenotaph
                        .flaws()
                        .iter()
                        .map(|flaw| flaw.to_string())
                        .collect::<Vec<String>>()
                        .join(","),
                )
            } else {
                None
            },
            timestamp: self.block_time,
        })?;
        Ok(())
    }

    fn add_txo(&mut self, tx: &Transaction, tx_id: &str) -> Result<(), Error> {
        for (vout, _) in tx.output.iter().enumerate() {
            self.database.add_txo(TXO {
                tx_id: tx_id.to_string(),
                output_index: vout as u32,
                value: tx.output[vout].value as u128,
                address: utils::output_to_address(&tx.output[vout], self.chain),
                address_lowercase: utils::output_to_address(&tx.output[vout], self.chain)
                    .map(|s| s.to_lowercase()),
                is_unspent: true,
                spent_tx_id: None,
                timestamp: self.block_time,
            })?;
        }
        Ok(())
    }

    fn add_rune_transfers(
        &mut self,
        tx: &Transaction,
        tx_id: &str,
        vout: usize,
        balances: Vec<(RuneId, Lot)>,
    ) -> Result {
        for (id, balance) in balances {
            self.database.add_rune_transfer(RuneTransfer {
                tx_id: tx_id.to_string(),
                output_index: vout as u32,
                rune_id: id.to_string(),
                amount: balance.n(),
                address: utils::output_to_address(&tx.output[vout], self.chain),
                address_lowercase: utils::output_to_address(&tx.output[vout], self.chain)
                    .map(|s| s.to_lowercase()),
                is_unspent: true,
                spent_tx_id: None,
                timestamp: self.block_time,
            })?;
        }

        Ok(())
    }

    fn create_rune_entry(
        &mut self,
        tx_id: &str,
        artifact: &Artifact,
        id: RuneId,
        rune: Rune,
    ) -> Result {
        let rune_count = self.database.get_rune_count()? + 1;

        let rune_entry = match artifact {
            Artifact::Cenotaph(_) => RuneEntry {
                etching_tx_id: tx_id.to_string(),
                block_height: id.block,
                rune_id: id.to_string(),
                name: rune.to_string(),
                raw_name: rune.to_string(),
                symbol: None,
                burned: 0,
                divisibility: 0,
                terms: None,
                mint_count: 0,
                premine: 0,
                timestamp: self.block_time,
                is_cenotapth: true,
                rune_number: rune_count,
            },
            Artifact::Runestone(Runestone { etching, .. }) => {
                let Etching {
                    divisibility,
                    terms,
                    premine,
                    spacers,
                    symbol,
                    ..
                } = etching.unwrap();

                RuneEntry {
                    etching_tx_id: tx_id.to_string(),
                    block_height: id.block,
                    rune_id: id.to_string(),
                    name: SpacedRune {
                        rune,
                        spacers: spacers.unwrap_or_default(),
                    }
                    .to_string(),
                    raw_name: rune.to_string(),
                    symbol,
                    divisibility: divisibility.unwrap_or_default(),
                    premine: premine.unwrap_or_default(),
                    terms: if terms.is_none() {
                        None
                    } else {
                        let terms = terms.unwrap();

                        Some(Terms {
                            amount: terms.amount,
                            cap: terms.cap,
                            height_start: terms.height.0,
                            height_end: terms.height.1,
                            offset_start: terms.offset.0,
                            offset_end: terms.offset.1,
                        })
                    },
                    burned: 0,
                    mint_count: 0,
                    timestamp: self.block_time,
                    is_cenotapth: false,
                    rune_number: rune_count,
                }
            }
        };

        self.database.add_rune_entry(rune_entry)
    }

    async fn etched(
        &mut self,
        tx_index: u32,
        tx: &Transaction,
        artifact: &Artifact,
    ) -> Result<Option<(RuneId, Rune)>> {
        let rune = match artifact {
            Artifact::Runestone(runestone) => match runestone.etching {
                Some(etching) => etching.rune,
                None => return Ok(None),
            },
            Artifact::Cenotaph(cenotaph) => match cenotaph.etching {
                Some(rune) => Some(rune),
                None => return Ok(None),
            },
        };

        let minimum = Rune::minimum_at_height(self.chain, Height(self.block_height));

        let rune = if let Some(rune) = rune {
            if rune < minimum
                || rune.is_reserved()
                || self
                    .database
                    .get_rune_by_raw_name(&rune.to_string())?
                    .is_some()
                || !self.tx_commits_to_rune(tx, rune).await?
            {
                return Ok(None);
            }
            rune
        } else {
            // let reserved_runes = self
            //     .statistic_to_count
            //     .get(&Statistic::ReservedRunes.into())?
            //     .map(|entry| entry.value())
            //     .unwrap_or_default();

            // self.statistic_to_count
            //     .insert(&Statistic::ReservedRunes.into(), reserved_runes + 1)?;

            Rune::reserved(self.block_height.into(), tx_index)
        };

        Ok(Some((
            RuneId {
                block: self.block_height.into(),
                tx: tx_index,
            },
            rune,
        )))
    }

    fn mint(&mut self, id: RuneId) -> Result<Option<Lot>> {
        let Some(rune_entry) = self.database.get_rune_by_id(&id.to_string())? else {
            return Ok(None);
        };

        let Ok(amount) = mintable(&rune_entry, self.block_height.into()) else {
            return Ok(None);
        };

        self.database
            .update_rune_entry_mint_count(&id.to_string())?;

        Ok(Some(Lot(amount)))
    }

    fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<RuneId, Lot>> {
        // map of rune ID to un-allocated balance of that rune
        let mut unallocated: HashMap<RuneId, Lot> = HashMap::new();

        // increment unallocated runes with the runes in tx inputs
        for input in &tx.input {
            let rune_transfers = self.database.get_runes_transfers_by_output_index(
                &input.previous_output.txid.to_string().to_lowercase(),
                input.previous_output.vout,
            )?;

            for rt in rune_transfers {
                *unallocated
                    .entry(rt.rune_id.parse::<RuneId>()?)
                    .or_default() += rt.amount;
            }
        }

        Ok(unallocated)
    }

    fn mark_txs_as_spent(&mut self, tx: &Transaction, tx_id: &str) -> Result<()> {
        for input in &tx.input {
            self.database.mark_utxo_as_spent(
                &input.previous_output.txid.to_string().to_lowercase(),
                input.previous_output.vout,
                tx_id,
            )?;
        }

        Ok(())
    }

    async fn tx_commits_to_rune(&mut self, tx: &Transaction, rune: Rune) -> Result<bool> {
        let commitment = rune.commitment();

        for input in &tx.input {
            // extracting a tapscript does not indicate that the input being spent
            // was actually a taproot output. this is checked below, when we load the
            // output's entry from the database
            let Some(tapscript) = input.witness.tapscript() else {
                continue;
            };

            for instruction in tapscript.instructions() {
                // ignore errors, since the extracted script may not be valid
                let Ok(instruction) = instruction else {
                    break;
                };

                let Some(pushbytes) = instruction.push_bytes() else {
                    continue;
                };

                if pushbytes.as_bytes() != commitment {
                    continue;
                }

                let Ok(tx_info) =
                    btc_rpc::get_transaction(&input.previous_output.txid.to_string()).await
                // .into_option()?
                else {
                    panic!("input not in UTXO set: {}", input.previous_output);
                };

                let taproot = tx_info.data.output[input.previous_output.vout.into_usize()]
                    .script_pubkey
                    .is_v1_p2tr();

                let mature = tx_info
                    .raw
                    .confirmations
                    .map(|confirmations| confirmations >= Runestone::COMMIT_INTERVAL.into())
                    .unwrap_or_default();

                if taproot && mature {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}

pub fn mintable(rune_entry: &RuneEntry, block_height: u64) -> Result<u128, MintError> {
    let Some(terms) = rune_entry.terms.clone() else {
        return Err(MintError::Unmintable);
    };

    if let Some(start) = mint_start(rune_entry) {
        if block_height < start {
            return Err(MintError::Start(start));
        }
    }

    if let Some(end) = mint_end(rune_entry) {
        if block_height >= end {
            return Err(MintError::End(end));
        }
    }

    let cap = terms.cap.unwrap_or_default();

    if rune_entry.mint_count >= cap {
        return Err(MintError::Cap(cap));
    }

    Ok(terms.amount.unwrap_or_default())
}

pub fn mint_start(rune_entry: &RuneEntry) -> Option<u64> {
    let terms = rune_entry.terms.clone()?;

    let relative = terms
        .offset_start
        .map(|offset| rune_entry.block_height.saturating_add(offset));

    let absolute = terms.height_start;

    // return the maximum of the relative (offset) and absolute start heights
    relative
        .zip(absolute)
        .map(|(relative, absolute)| relative.max(absolute))
        .or(relative)
        .or(absolute)
}

pub fn mint_end(rune_entry: &RuneEntry) -> Option<u64> {
    let terms = rune_entry.terms.clone()?;

    let relative = terms
        .offset_end
        .map(|offset| rune_entry.block_height.saturating_add(offset));

    let absolute = terms.height_end;

    relative
        .zip(absolute)
        .map(|(relative, absolute)| relative.min(absolute))
        .or(relative)
        .or(absolute)
}
