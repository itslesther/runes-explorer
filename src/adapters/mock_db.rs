use anyhow::Error;
use bitcoin::Txid;
use serde::*;

pub struct MockDb {
    pub rune_entries: Vec<RuneEntry>,
    pub transactions: Vec<Transaction>,
    pub rune_transfers: Vec<RuneTransfer>,
    pub txos: Vec<TXO>,
}

pub struct RuneEntry {
    pub etching_tx_id: String,
    pub block_height: u64,
    pub rune_id: String,
    pub name: String,
    pub raw_name: String,
    pub symbol: Option<char>,
    pub divisibility: u8,
    pub premine: u128,
    pub terms: Option<Terms>,
    pub burned: u128,
    pub mint_count: u128, // validation for Terms::cap
    pub timestamp: u32,
    pub is_cenotapth: bool,
}
pub struct Transaction {
    pub tx_id: String,
    // pub inputs: Vec<TXO>,
    // pub outputs: Vec<TXO>,
    pub is_artifact: bool,
    pub is_runestone: bool,
    pub is_cenotapth: bool,
    pub cenotapth_messages: Option<String>,
    pub timestamp: u32,
}

pub struct Terms {
    pub amount: Option<u128>,
    pub cap: Option<u128>,
    pub height_start: Option<u64>,
    pub height_end: Option<u64>,
    pub offset_start: Option<u64>,
    pub offset_end: Option<u64>,
}

pub struct TXO {
    pub tx_id: String,
    pub output_index: u32,
    pub value: u64,
    pub address: Option<String>,
    // pub rune_transfers: Vec<RuneTransfer>,
    pub is_unspent: bool,
    pub timestamp: u32,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuneTransfer {
    pub tx_id: String,
    pub output_index: u32,
    pub rune_id: String,
    pub amount: u128,
    pub address: Option<String>,
    pub is_unspent: bool,
    pub timestamp: u32,
}

impl MockDb {
    pub fn init() -> MockDb {
        MockDb {
            rune_entries: Vec::new(),
            transactions: Vec::new(),
            rune_transfers: Vec::new(),
            txos: Vec::new(),
        }
    }

    pub fn get_runes_transfers(
        &mut self,
        tx_id: &str,
        tx_index: u32,
    ) -> Result<Vec<&mut RuneTransfer>, Error> {
        Ok(self
            .rune_transfers
            .iter_mut()
            .filter(|rt| rt.tx_id == tx_id.to_string() && rt.output_index == tx_index)
            // .cloned() // Change: Add cloned() to create a new iterator with cloned elements
            .collect())
    }

    pub fn get_rune_by_id(&mut self, rune_id: &str) -> Result<Option<&mut RuneEntry>, Error> {
        Ok(self
            .rune_entries
            .iter_mut()
            .find(|rune| rune.rune_id == rune_id))
    }

    pub fn update_rune_entry_mint_count(&mut self, rune_id: &str) {
        if let Some(mut rune) = self.get_rune_by_id(rune_id).unwrap() {
            rune.mint_count += 1;
        }
    }
    pub fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error> {
        if let Some(mut rune) = self.get_rune_by_id(rune_id).unwrap() {
            rune.burned += amount;
        }
        Ok(())
    }

    pub fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<&RuneEntry>, Error> {
        Ok(self.rune_entries.iter().find(|rune| rune.raw_name == name))
    }

    pub fn add_transaction(&mut self, transaction: Transaction) {
        self.transactions.push(transaction);
    }

    pub fn add_rune_entry(&mut self, rune_entry: RuneEntry) -> Result<(), Error> {
        self.rune_entries.push(rune_entry);
        Ok(())
    }

    pub fn add_rune_transfer(&mut self, rune_transfer: RuneTransfer) {
        self.rune_transfers.push(rune_transfer);
    }

    pub fn get_txo(&mut self, tx_id: &str, output_index: u32) -> Result<Option<&mut TXO>, Error> {
        Ok(self
            .txos
            .iter_mut()
            .find(|txo| txo.tx_id == tx_id && txo.output_index == output_index))
    }

    pub fn mark_utxo_as_spent(&mut self, tx_id: &str, output_index: u32) -> Result<(), Error> {
        if let Some(txo) = self.get_txo(tx_id, output_index)? {
            txo.is_unspent = false;
        }

        let mut rune_transfers = self.get_runes_transfers(tx_id, output_index)?;
        rune_transfers.iter_mut().for_each(|rt| rt.is_unspent = false);

        Ok(())
    }

    pub fn add_txo(&mut self, txo: TXO) {
        self.txos.push(txo);
    }

    // pub fn get_rune(&self, etching_tx_id: &str) -> Option<&RuneEntry> {
    //     self.rune_entries
    //         .iter()
    //         .find(|rune| rune.etching_tx_id == etching_tx_id)
    // }

    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| tx.tx_id == tx_id)
    }

    pub fn get_rune_by_name(&self, name: &str) -> Option<&RuneEntry> {
        self.rune_entries.iter().find(|rune| rune.name == name)
    }

    pub fn get_runes(&self) -> &Vec<RuneEntry> {
        &self.rune_entries
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}
