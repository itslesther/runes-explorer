use super::db::*;
use anyhow::Error;

pub struct MockDb {
    pub rune_entries: Vec<RuneEntry>,
    pub transactions: Vec<Transaction>,
    pub rune_transfers: Vec<RuneTransfer>,
    pub txos: Vec<TXO>,
    pub statistics: Statistics,
}

impl Database for MockDb {
    // fn init() -> Result<(), Error> {
    //     Ok(())
    // }
    // fn init() -> MockDb {
    //     MockDb {
    //         rune_entries: Vec::new(),
    //         transactions: Vec::new(),
    //         rune_transfers: Vec::new(),
    //         txos: Vec::new(),
    //         statistics: Statistics {
    //             rune_count: 0,
    //             block_height: 0,
    //         },
    //     }
    // }

    fn get_runes_transfers_by_tx(
        &mut self,
        tx_id: &str,
        output_index: u32,
    ) -> Result<Vec<RuneTransfer>, Error> {
        Ok(self
            .rune_transfers
            .iter()
            .filter(|&rt| rt.tx_id == tx_id.to_string() && rt.output_index == output_index)
            .cloned() // Change: Add cloned() to create a new iterator with cloned elements
            .collect())
    }

    fn get_rune_by_id(&mut self, rune_id: &str) -> Result<Option<&mut RuneEntry>, Error> {
        Ok(self
            .rune_entries
            .iter_mut()
            .find(|rune| rune.rune_id == rune_id))
    }

    fn update_rune_entry_mint_count(&mut self, rune_id: &str) -> Result<(), Error> {
        if let Some(rune) = self.get_rune_by_id(rune_id).unwrap() {
            rune.mint_count += 1;
        }
        Ok(())
    }

    fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error> {
        if let Some(rune) = self.get_rune_by_id(rune_id).unwrap() {
            rune.burned += amount;
        }
        Ok(())
    }

    fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<&RuneEntry>, Error> {
        Ok(self.rune_entries.iter().find(|rune| rune.raw_name == name))
    }

    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        self.transactions.push(transaction);
        Ok(())
    }

    fn add_rune_entry(&mut self, rune_entry: RuneEntry) -> Result<(), Error> {
        self.rune_entries.push(rune_entry);
        self.increase_rune_count()?;
        Ok(())
    }

    fn add_rune_transfer(&mut self, rune_transfer: RuneTransfer) -> Result<(), Error> {
        self.rune_transfers.push(rune_transfer);
        Ok(())
    }

    fn get_txo(&mut self, tx_id: &str, output_index: u32) -> Result<Option<&mut TXO>, Error> {
        Ok(self
            .txos
            .iter_mut()
            .find(|txo| txo.tx_id == tx_id && txo.output_index == output_index))
    }

    fn mark_utxo_as_spent(
        &mut self,
        tx_id: &str,
        output_index: u32,
        spent_tx_id: &str,
    ) -> Result<(), Error> {
        if let Some(txo) = self.get_txo(tx_id, output_index)? {
            txo.is_unspent = false;
            txo.spent_tx_id = Some(spent_tx_id.to_string());
        }

        self.rune_transfers
            .iter_mut()
            .filter(|rt| rt.tx_id == tx_id.to_string() && rt.output_index == output_index)
            .for_each(|rt| {
                rt.is_unspent = false;
                rt.spent_tx_id = Some(spent_tx_id.to_string());
            });

        Ok(())
    }

    fn add_txo(&mut self, txo: TXO) -> Result<(), Error> {
        self.txos.push(txo);
        Ok(())
    }

    fn get_address_balance_by_rune_id(&self, address: &str, rune_id: &str) -> u128 {
        self.rune_transfers
            .iter()
            .filter(|rt| {
                rt.address == Some(address.to_string()) && rt.rune_id == rune_id && rt.is_unspent
            })
            .map(|rt| rt.amount)
            .sum()
    }

    fn get_address_transfer(&self, address: &str) -> Vec<&RuneTransfer> {
        self.rune_transfers
            .iter()
            .filter(|rt| rt.address == Some(address.to_string()))
            .collect()
    }

    fn get_rune_count(&self) -> Result<u128, Error> {
        Ok(self.statistics.rune_count)
    }

    fn increase_rune_count(&mut self) -> Result<(), Error> {
        self.statistics.rune_count += 1;
        Ok(())
    }

    fn get_block_height(&self) -> Result<u64, Error> {
        Ok(self.statistics.block_height)
    }

    fn increase_block_height(&mut self) -> Result<(), Error> {
        self.statistics.block_height += 1;
        Ok(())
    }

    // pub fn get_address_transfers

    // pub fn get_rune(&self, etching_tx_id: &str) -> Option<&RuneEntry> {
    //     self.rune_entries
    //         .iter()
    //         .find(|rune| rune.etching_tx_id == etching_tx_id)
    // }

    fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.tx_id == tx_id)
    }

    fn get_rune_by_name(&self, name: &str) -> Option<&RuneEntry> {
        self.rune_entries.iter().find(|rune| rune.name == name)
    }

    fn get_runes(&self) -> Result<&Vec<RuneEntry>, Error> {
        Ok(&self.rune_entries)
    }

    fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}
