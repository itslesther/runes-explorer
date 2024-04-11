use super::db::*;
use anyhow::Error;
use rusqlite::{params, Connection, Result};

pub struct SQLite {
    conn: Connection,
}

impl SQLite {
    fn init() -> Result<SQLite, Error> {
        let conn = Connection::open("./runes.db")?;
        // let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rune_entries (
        etching_tx_id TEXT NOT NULL,
        block_height INTEGER,
        rune_id TEXT NOT NULL,
        name TEXT NOT NULL,
        raw_name TEXT NOT NULL,
        symbol TEXT,
        divisibility INTEGER NOT NULL,
        premine INTEGER NOT NULL,
        terms_id TEXT,
        burned INTEGER NOT NULL,
        mint_count INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        is_cenotapth BOOLEAN
      )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS terms (
          amount INTEGER,
          cap INTEGER,
          height_start INTEGER,
          height_end INTEGER,
          offset_start INTEGER,
          offset_end INTEGER
        )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
        tx_id TEXT PRIMARY KEY,
        is_artifact BOOLEAN,
        is_runestone BOOLEAN,
        is_cenotapth BOOLEAN,
        cenotapth_messages TEXT,
        timestamp INTEGER
      )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rune_transfers (
        tx_id TEXT NOT NULL,
        output_index INTEGER,
        rune_id TEXT NOT NULL,
        amount INTEGER NOT NULL,
        address TEXT,
        address_lowercase TEXT,
        is_unspent BOOLEAN,
        spent_tx_id TEXT,
        timestamp INTEGER
      )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS txos (
        tx_id TEXT NOT NULL,
        output_index INTEGER,
        value INTEGER NOT NULL,
        address TEXT,
        address_lowercase TEXT,
        is_unspent BOOLEAN,
        spent_tx_id TEXT,
        spent_tx_id TEXT,
        timestamp INTEGER
      )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS statistics (
        tx_id TEXT NOT NULL,
        rune_count INTEGER,
        block_height INTEGER NOT NULL
      )",
            (), // empty list of parameters.
        )?;

        Ok(SQLite { conn })
    }
}

impl Database for SQLite {
    fn get_runes_transfers_by_tx(
        &mut self,
        tx_id: &str,
        output_index: u32,
    ) -> Result<Vec<RuneTransfer>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM rune_transfers WHERE tx_id = ?1 AND output_index = ?2")?;
        let result_iter = stmt.query_map(params![tx_id, output_index], |row| {
            let amount: u64 = row.get(3)?;

            Ok(RuneTransfer {
                tx_id: row.get(0)?,
                output_index: row.get(1)?,
                rune_id: row.get(2)?,
                amount: amount as u128,
                address: row.get(4)?,
                address_lowercase: row.get(5)?,
                is_unspent: row.get(6)?,
                spent_tx_id: row.get(7)?,
                timestamp: row.get(8)?,
            })
        })?;

        Ok(result_iter
            .map(|r| r.unwrap())
            .collect::<Vec<RuneTransfer>>())
    }

    fn get_rune_by_id(&mut self, rune_id: &str) -> Result<Option<&mut RuneEntry>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM rune_entries WHERE rune_id = ?1")?;

        let result_iter = stmt.query_map(params![rune_id], |row| {
          let symbol: Option<String> = row.get(5)?;
          let premine: u64 = row.get(7)?;
          let burned: u64 = row.get(9)?;

            Ok(RuneEntry {
                etching_tx_id: row.get(0)?,
                block_height: row.get(1)?,
                rune_id: row.get(2)?,
                name: row.get(3)?,
                raw_name: row.get(4)?,
                symbol: symbol.map(|s| s.chars().next().unwrap()),
                divisibility: row.get(6)?,
                premine: premine as u128,
                terms_id: row.get(8)?,
                burned: row.get(9)?,
                mint_count: row.get(10)?,
                timestamp: row.get(11)?,
                is_cenotapth: row.get(12)?,
            })
        })?;

        Ok(result_iter.map(|r| r.unwrap()).next())
    }

    fn update_rune_entry_mint_count(&mut self, rune_id: &str) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE rune_entries SET mint_count = mint_count + 1 WHERE rune_id = ?1",
            params![rune_id],
        )?;

        Ok(())
    }

    fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE rune_entries SET burned = burned + ?1 WHERE rune_id = ?2",
            params![u64::try_from(amount).unwrap(), rune_id],
        )?;

        Ok(())
    }

    fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<&RuneEntry>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM rune_entries WHERE raw_name = ?1")?;
        let result_iter = stmt.query_map(params![name], |row| {
            Ok(RuneEntry {
                etching_tx_id: row.get(0)?,
                block_height: row.get(1)?,
                rune_id: row.get(2)?,
                name: row.get(3)?,
                raw_name: row.get(4)?,
                symbol: row.get(5)?,
                divisibility: row.get(6)?,
                premine: row.get(7)?,
                terms_id: row.get(8)?,
                burned: row.get(9)?,
                mint_count: row.get(10)?,
                timestamp: row.get(11)?,
                is_cenotapth: row.get(12)?,
            })
        })?;

        Ok(result_iter.map(|r| r.unwrap()).next())
    }
}
