pub struct MockDb {
  pub runes: Vec<Runes>,
  pub transactions: Vec<Transaction>,
}

struct Runes {
  etching_tx_id: String,
  name: String,
  raw_name: String,
  symbol: Option<char>,
  divisibility: Option<u8>,
  premine: Option<u128>,
  terms: Option<Terms>,
  burned: u128,
  mint_count: u128, // validation for Terms::cap
}

struct Transaction {
  tx_id: String,
  inputs: Vec<TXO>,
  outputs: Vec<TXO>,
  is_cenotapth: bool,
  is_runestone: bool,
}

struct Terms {
  amount: Option<u128>,
  cap: Option<u128>,
  height_start: Option<u64>,
  height_end: Option<u64>,
  offset_start: Option<u64>,
  offset_end: Option<u64>,
}

struct TXO {
  tx_id: String,
  value: u32,
  address: String,
  output_index: u32,
  rune_transfers: Vec<RuneTransfer>,
  is_unspent: bool,
}

struct RuneTransfer {
  tx_id: String,
  output_index: u32,
  rune_id: String,
  amount: u32,
  address: String,
  is_unspent: bool,
}

impl MockDb {
  pub fn init() -> MockDb {
      MockDb {
          runes: Vec::new(),
          transactions: Vec::new(),
      }
  }

  pub fn add_rune(&mut self, rune: Runes) {
      self.runes.push(rune);
  }

  pub fn add_transaction(&mut self, transaction: Transaction) {
      self.transactions.push(transaction);
  }

  pub fn get_rune(&self, etching_tx_id: &str) -> Option<&Runes> {
      self.runes
          .iter()
          .find(|rune| rune.etching_tx_id == etching_tx_id)
  }

  pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
      self.transactions.iter().find(|tx| tx.tx_id == tx_id)
  }

  pub fn get_rune_by_name(&self, name: &str) -> Option<&Runes> {
      self.runes.iter().find(|rune| rune.name == name)
  }

  pub fn get_runes(&self) -> &Vec<Runes> {
      &self.runes
  }

  pub fn get_transactions(&self) -> &Vec<Transaction> {
      &self.transactions
  }
}
