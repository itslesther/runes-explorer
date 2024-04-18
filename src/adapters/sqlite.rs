use std::collections::HashMap;

use super::db::*;
use crate::log_file::log;
use anyhow::Error;
use rusqlite::{params, Connection, Result};

#[derive(Debug, Clone, Copy)]
pub struct SQLite<'a> {
    pub conn: &'a Connection,
    // pub log_file: LogFile,
}

impl<'a> SQLite<'a> {
    // pub fn init(conn: &'a Connection) -> SQLite<'a> {
    //     // SQLite { conn, log_file: LogFile::new() }
    //     SQLite { conn }
    // }
    pub fn init_tables(&self) -> Result<(), Error> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rune_entries (
            etching_tx_id TEXT NOT NULL,
            block_height INTEGER NOT NULL,
            rune_id TEXT NOT NULL,
            name TEXT NOT NULL,
            raw_name TEXT NOT NULL,
            symbol TEXT,
            divisibility INTEGER NOT NULL,
            premine TEXT NOT NULL,
            burned TEXT NOT NULL,
            mint_count TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            is_cenotapth BOOLEAN,
            cenotapth_message TEXT,
            rune_number TEXT NOT NULL,
            turbo BOOLEAN NOT NULL
        )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS terms (
            rune_id TEXT NOT NULL,
            amount TEXT,
            cap TEXT,
            height_start INTEGER,
            height_end INTEGER,
            offset_start INTEGER,
            offset_end INTEGER
        )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
            tx_id TEXT PRIMARY KEY,
            block_height INTEGER NOT NULL,
            is_artifact BOOLEAN,
            is_runestone BOOLEAN,
            is_cenotapth BOOLEAN,
            cenotapth_message TEXT,
            timestamp INTEGER
      )",
            (),
        )?;

        // event_type: etch, mint, burn, transfer
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS rune_events (
            tx_id TEXT NOT NULL,
            rune_id TEXT NOT NULL,
            block_height INTEGER NOT NULL,
            timestamp INTEGER,
            amount TEXT NOT NULL,
            event_type TEXT NOT NULL,
            output_index INTEGER,
            address TEXT
      )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS runes_txos (
            tx_id TEXT NOT NULL,
            output_index INTEGER NOT NULL,
            block_height INTEGER NOT NULL,
            rune_id TEXT NOT NULL,
            amount TEXT NOT NULL,
            address TEXT,
            is_unspent BOOLEAN,
            spent_tx_id TEXT,
            timestamp INTEGER
      )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS txos (
            tx_id TEXT NOT NULL,
            output_index INTEGER NOT NULL,
            block_height INTEGER NOT NULL,
            value TEXT NOT NULL,
            address TEXT,
            is_unspent BOOLEAN,
            spent_tx_id TEXT,
            timestamp INTEGER
      )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS statistics (
            block_height INTEGER NOT NULL
            )",
            (),
        )?;

        self.create_db_indexes()?;

        log("Tables initialized")?;

        Ok(())
    }
}

impl<'a> Database for SQLite<'a> {
    fn get_rune_by_id(&self, rune_id: &str) -> Result<Option<RuneEntry>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM rune_entries WHERE rune_id = ?1")?;

        let result_iter = stmt.query_map(params![rune_id], |row| {
            let symbol: Option<String> = row.get("symbol")?;
            let premine: String = row.get("premine")?;
            let burned: String = row.get("burned")?;
            let mint_count: String = row.get("mint_count")?;
            let rune_number: String = row.get("rune_number")?;

            let mut terms_stmt = self
                .conn
                .prepare("SELECT * FROM terms WHERE rune_id = ?1")?;

            let terms_result_iter = terms_stmt.query_map(params![rune_id], |row| {
                let amount: Option<String> = row.get("amount")?;
                let cap: Option<String> = row.get("cap")?;

                Ok(Terms {
                    amount: amount.map(|v| v.parse::<u128>().unwrap()),
                    cap: cap.map(|v| v.parse::<u128>().unwrap()),
                    height_start: row.get("height_start")?,
                    height_end: row.get("height_end")?,
                    offset_start: row.get("offset_start")?,
                    offset_end: row.get("offset_end")?,
                })
            })?;

            let terms: Option<Terms> = terms_result_iter.map(|t| t.unwrap()).next();

            Ok(RuneEntry {
                etching_tx_id: row.get("etching_tx_id")?,
                block_height: row.get("block_height")?,
                rune_id: row.get("rune_id")?,
                name: row.get("name")?,
                raw_name: row.get("raw_name")?,
                symbol: symbol.map(|s| s.chars().next().unwrap()),
                divisibility: row.get("divisibility")?,
                premine: premine.parse().unwrap(),
                terms: terms,
                burned: burned.parse().unwrap(),
                mint_count: mint_count.parse().unwrap(),
                timestamp: row.get("timestamp")?,
                is_cenotapth: row.get("is_cenotapth")?,
                cenotapth_message: row.get("cenotapth_message")?,
                rune_number: rune_number.parse().unwrap(),
                turbo: row.get("turbo")?,
            })
        })?;

        let rune_entry = result_iter.map(|r| r.unwrap()).next();
        Ok(rune_entry)
    }

    fn get_rune_by_etched_tx_id(&self, etching_tx_id: &str) -> Result<Option<RuneEntry>, Error> {
        let mut stmt: rusqlite::Statement<'_> = self
            .conn
            .prepare("SELECT * FROM rune_entries WHERE etching_tx_id = ?1")?;

        let result_iter = stmt.query_map(params![etching_tx_id], |row| {
            let symbol: Option<String> = row.get("symbol")?;
            let premine: String = row.get("premine")?;
            let burned: String = row.get("burned")?;
            let mint_count: String = row.get("mint_count")?;
            let rune_number: String = row.get("rune_number")?;
            let rune_id: String = row.get("rune_id")?;

            let mut terms_stmt = self
                .conn
                .prepare("SELECT * FROM terms WHERE rune_id = ?1")?;

            let terms_result_iter = terms_stmt.query_map(params![rune_id], |row| {
                let amount: Option<String> = row.get("amount")?;
                let cap: Option<String> = row.get("cap")?;

                Ok(Terms {
                    amount: amount.map(|v| v.parse::<u128>().unwrap()),
                    cap: cap.map(|v| v.parse::<u128>().unwrap()),
                    height_start: row.get("height_start")?,
                    height_end: row.get("height_end")?,
                    offset_start: row.get("offset_start")?,
                    offset_end: row.get("offset_end")?,
                })
            })?;

            let terms: Option<Terms> = terms_result_iter.map(|t| t.unwrap()).next();

            Ok(RuneEntry {
                etching_tx_id: row.get("etching_tx_id")?,
                block_height: row.get("block_height")?,
                rune_id,
                name: row.get("name")?,
                raw_name: row.get("raw_name")?,
                symbol: symbol.map(|s| s.chars().next().unwrap()),
                divisibility: row.get("divisibility")?,
                premine: premine.parse().unwrap(),
                terms,
                burned: burned.parse().unwrap(),
                mint_count: mint_count.parse().unwrap(),
                timestamp: row.get("timestamp")?,
                is_cenotapth: row.get("is_cenotapth")?,
                cenotapth_message: row.get("cenotapth_message")?,
                rune_number: rune_number.parse().unwrap(),
                turbo: row.get("turbo")?,
            })
        })?;

        let rune_entry = result_iter.map(|r| r.unwrap()).next();
        Ok(rune_entry)
    }

    fn update_rune_entry_mint_count(
        &mut self,
        rune_id: &str,
        tx_id: &str,
        amount: u128,
        block_height: u64,
        timestamp: u32,
    ) -> Result<(), Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT mint_count FROM rune_entries WHERE rune_id = ?1")?;

        let result_iter = stmt.query_map(params![rune_id], |row| {
            let mint_count: String = row.get("mint_count")?;
            let new_mint_count: String = (mint_count.parse::<u128>().unwrap() + 1).to_string();

            Ok(new_mint_count)
        })?;

        let new_mint_count: String = result_iter.map(|r| r.unwrap()).next().unwrap();

        self.conn.execute(
            "
                BEGIN TRANSACTION;

            UPDATE rune_entries SET mint_count = ?1
                WHERE rune_id = ?2;

            INSERT INTO rune_events (tx_id, rune_id, block_height, timestamp, amount, event_type)
                VALUES (?3, ?4, ?5, ?6, ?7, ?8);

                COMMIT;
            ",
            params![new_mint_count, rune_id, tx_id, rune_id, block_height, timestamp, amount.to_string(), "mint"],
        )?;


        log(&format!(
            "Mint count for rune id {} updated to: {}",
            rune_id, new_mint_count
        ))?;

        Ok(())
    }

    fn increase_rune_entry_burned(&mut self, rune_id: &str, amount: u128) -> Result<(), Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT burned FROM rune_entries WHERE rune_id = ?1")?;

        let result_iter = stmt.query_map(params![rune_id], |row| {
            let burned: String = row.get("burned")?;
            let new_burned: String = (burned.parse::<u128>().unwrap() + amount).to_string();

            Ok(new_burned)
        })?;

        let new_burned = result_iter.into_iter().next().unwrap().unwrap();

        self.conn.execute(
            "UPDATE rune_entries SET burned = ?1 WHERE rune_id = ?2",
            params![new_burned, rune_id],
        )?;

        log(&format!(
            "Burned amount for rune id {} updated to: {}",
            new_burned, rune_id
        ))?;

        Ok(())
    }

    fn add_rune_burn_event(
        &mut self,
        rune_id: &str,
        tx_id: &str,
        amount: u128,
        block_height: u64,
        timestamp: u32,
    ) -> Result<(), Error> {
        self.conn.execute(
            "INSERT INTO rune_events (tx_id, rune_id, block_height, timestamp, amount, event_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![tx_id, rune_id, block_height, timestamp, amount.to_string(), "burn"],
        )?;

        // println!("Burn event for rune {} added: {:?}", rune_id, tx_id);
        log(&format!(
            "Burn event for rune {} added: {:?}",
            rune_id, tx_id
        ))?;

        Ok(())
    }

    fn get_rune_by_raw_name(&self, name: &str) -> Result<Option<RuneEntry>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM rune_entries WHERE raw_name = ?1")?;

        let result_iter = stmt.query_map(params![name], |row| {
            let symbol: Option<String> = row.get("symbol")?;
            let premine: String = row.get("premine")?;
            let burned: String = row.get("burned")?;
            let mint_count: String = row.get("mint_count")?;
            let rune_number: String = row.get("rune_number")?;
            let rune_id: String = row.get("rune_id")?;

            let mut terms_stmt = self
                .conn
                .prepare("SELECT * FROM terms WHERE rune_id = ?1")?;

            let terms_result_iter = terms_stmt.query_map(params![rune_id], |row| {
                let amount: Option<String> = row.get("amount")?;
                let cap: Option<String> = row.get("cap")?;

                Ok(Terms {
                    amount: amount.map(|v| v.parse::<u128>().unwrap()),
                    cap: cap.map(|v| v.parse::<u128>().unwrap()),
                    height_start: row.get("height_start")?,
                    height_end: row.get("height_end")?,
                    offset_start: row.get("offset_start")?,
                    offset_end: row.get("offset_end")?,
                })
            })?;

            let terms: Option<Terms> = terms_result_iter.map(|t| t.unwrap()).next();

            Ok(RuneEntry {
                etching_tx_id: row.get("etching_tx_id")?,
                block_height: row.get("block_height")?,
                rune_id,
                name: row.get("name")?,
                raw_name: row.get("raw_name")?,
                symbol: symbol.map(|s| s.chars().next().unwrap()),
                divisibility: row.get("divisibility")?,
                premine: premine.parse().unwrap(),
                terms,
                burned: burned.parse().unwrap(),
                mint_count: mint_count.parse().unwrap(),
                timestamp: row.get("timestamp")?,
                is_cenotapth: row.get("is_cenotapth")?,
                cenotapth_message: row.get("cenotapth_message")?,
                rune_number: rune_number.parse().unwrap(),
                turbo: row.get("turbo")?,
            })
        })?;

        let rune_entry = result_iter.map(|r| r.unwrap()).next();
        Ok(rune_entry)
    }

    fn add_transaction(&mut self, transaction: Transaction) -> Result<(), Error> {
        self.conn.execute(
            "INSERT INTO transactions (tx_id, is_artifact, is_runestone, is_cenotapth, cenotapth_message, timestamp, block_height) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                transaction.tx_id,
                transaction.is_artifact,
                transaction.is_runestone,
                transaction.is_cenotapth,
                transaction.cenotapth_message,
                transaction.timestamp,
                transaction.block_height
            ],
        )?;

        log(&format!("Transaction added: {:?}", transaction.tx_id))?;

        Ok(())
    }

    fn add_rune_entry(&mut self, rune_entry: RuneEntry) -> Result<(), Error> {
        self.conn.execute(
            "
            BEGIN TRANSACTION;

            INSERT INTO rune_entries (etching_tx_id, block_height, rune_id, name, raw_name, symbol, divisibility, premine, burned, mint_count, timestamp, is_cenotapth, cenotapth_message, rune_number, turbo)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15);

            INSERT INTO rune_events (tx_id, rune_id, block_height, timestamp, amount, event_type)
                VALUES (?16, ?17, ?18, ?19, ?20, ?21);
            
            COMMIT;
            ",
            params![
                rune_entry.etching_tx_id,
                rune_entry.block_height,
                rune_entry.rune_id,
                rune_entry.name,
                rune_entry.raw_name,
                rune_entry.symbol.map(|s| s.to_string()),
                rune_entry.divisibility,
                rune_entry.premine.to_string(),
                rune_entry.burned.to_string(),
                rune_entry.mint_count.to_string(),
                rune_entry.timestamp,
                rune_entry.is_cenotapth,
                rune_entry.cenotapth_message,
                rune_entry.rune_number.to_string(),
                rune_entry.turbo,
                rune_entry.etching_tx_id,
                rune_entry.rune_id,
                rune_entry.block_height,
                rune_entry.timestamp,
                rune_entry.premine.to_string(),
                "etch"
            ],
        )?;

        if let Some(terms) = rune_entry.terms {
            self.conn.execute(
                "INSERT INTO terms (rune_id, amount, cap, height_start, height_end, offset_start, offset_end) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    rune_entry.rune_id,
                    terms.amount.map(|a| a.to_string()),
                    terms.cap.map(|c| c.to_string()),
                    terms.height_start,
                    terms.height_end,
                    terms.offset_start,
                    terms.offset_end
                ],
            )?;
        }

        log(&format!("Rune entry added: {:?}", rune_entry.rune_id))?;

        Ok(())
    }

    fn add_rune_txo(&mut self, rune_txo: RuneTXO) -> Result<(), Error> {
        self.conn.execute("
            BEGIN TRANSACTION;
        
        INSERT INTO runes_txos (tx_id, output_index, rune_id, amount, address, is_unspent, spent_tx_id, timestamp, block_height)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9);

        INSERT INTO rune_events (tx_id, rune_id, block_height, timestamp, amount, event_type, output_index, address)
            VALUES (?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17);
            
            COMMIT;
        ",
        params![
            rune_txo.tx_id,
            rune_txo.output_index,
            rune_txo.rune_id,
            rune_txo.amount.to_string(),
            rune_txo.address,
            rune_txo.is_unspent,
            rune_txo.spent_tx_id,
            rune_txo.timestamp,
            rune_txo.block_height,
            rune_txo.tx_id,
            rune_txo.rune_id,
            rune_txo.block_height,
            rune_txo.timestamp,
            rune_txo.amount.to_string(),
            "transfer",
            rune_txo.output_index,
            rune_txo.address
        ])?;

        log(&format!(
            "Rune transfer for rune {} added: {:?}",
            rune_txo.rune_id, rune_txo.tx_id
        ))?;

        Ok(())
    }

    fn get_txo(&mut self, tx_id: &str, output_index: u32) -> Result<Option<TXO>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM txos WHERE tx_id = ?1 AND output_index = ?2")?;

        let result_iter = stmt.query_map(params![tx_id, output_index], |row| {
            let value: String = row.get("value")?;

            Ok(TXO {
                tx_id: row.get("tx_id")?,
                output_index: row.get("output_index")?,
                value: value.parse().unwrap(),
                address: row.get("address")?,
                is_unspent: row.get("is_unspent")?,
                spent_tx_id: row.get("spent_tx_id")?,
                timestamp: row.get("timestamp")?,
                block_height: row.get("block_height")?,
            })
        })?;

        let txo = result_iter.map(|r| r.unwrap()).next();

        Ok(txo)
    }

    fn mark_utxo_as_spent(
        &mut self,
        tx_id: &str,
        output_index: u32,
        spent_tx_id: &str,
    ) -> Result<(), Error> {
        self.conn.execute(
            "UPDATE txos SET is_unspent = FALSE, spent_tx_id = ?1 WHERE tx_id = ?2 AND output_index = ?3",
            params![spent_tx_id, tx_id, output_index],
        )?;

        self.conn.execute(
            "UPDATE runes_txos SET is_unspent = FALSE, spent_tx_id = ?1 WHERE tx_id = ?2 AND output_index = ?3",
            params![spent_tx_id, tx_id, output_index],
        )?;

        // log(&format!("UTXO marked as spent: {}:{} -> {}", tx_id, output_index, spent_tx_id))?;

        Ok(())
    }

    fn add_txo(&mut self, txo: TXO) -> Result<(), Error> {
        self.conn.execute(
            "INSERT INTO txos (tx_id, output_index, value, address, is_unspent, spent_tx_id, timestamp, block_height) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                txo.tx_id,
                txo.output_index,
                txo.value.to_string(),
                txo.address,
                txo.is_unspent,
                txo.spent_tx_id,
                txo.timestamp,
                txo.block_height
            ],
        )?;

        log(&format!(
            "TXO added: {}:{} -> {}",
            txo.tx_id, txo.output_index, txo.value
        ))?;

        Ok(())
    }

    fn get_address_balance_by_rune_id(&self, address: &str, rune_id: &str) -> Result<u128, Error> {
        let mut stmt = self
        .conn
        .prepare("SELECT amount FROM runes_txos WHERE address = ?1 AND rune_id = ?2 AND is_unspent = TRUE")?;

        let result_iter = stmt.query_map(params![address.to_lowercase(), rune_id], |row| {
            let amount: String = row.get("amount")?;

            Ok(amount.parse::<u128>().unwrap_or_default())
        })?;

        Ok(result_iter.map(|r| r.unwrap()).sum())
    }

    fn get_address_balance_list(&self, address: &str) -> Result<HashMap<String, u128>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT rune_id, amount FROM runes_txos WHERE address = ?1 AND is_unspent = TRUE",
        )?;

        let result_iter = stmt.query_map(params![address.to_lowercase()], |row| {
            let rune_id: String = row.get("rune_id")?;
            let amount: String = row.get("amount")?;

            Ok((rune_id, amount.parse::<u128>().unwrap_or_default()))
        })?;

        let mut balance_list: HashMap<String, u128> = HashMap::new();

        for balance in result_iter {
            let (rune_id, amount) = balance.unwrap();
            *balance_list.entry(rune_id).or_default() += amount;
        }

        Ok(balance_list)
    }

    fn get_runes_txo_by_output_index(
        &self,
        tx_id: &str,
        output_index: u32,
    ) -> Result<Vec<RuneTXO>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM runes_txos WHERE tx_id = ?1 AND output_index = ?2")?;

        let result_iter = stmt.query_map(params![tx_id, output_index], |row| {
            let amount: String = row.get("amount")?;

            Ok(RuneTXO {
                tx_id: row.get("tx_id")?,
                output_index: row.get("output_index")?,
                rune_id: row.get("rune_id")?,
                amount: amount.parse().unwrap(),
                address: row.get("address")?,
                is_unspent: row.get("is_unspent")?,
                spent_tx_id: row.get("spent_tx_id")?,
                timestamp: row.get("timestamp")?,
                block_height: row.get("block_height")?,
            })
        })?;

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn get_transaction_runes_txo(&self, tx_id: &str) -> Result<Vec<RuneTXO>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM runes_txos WHERE tx_id = ?1 OR spent_tx_id = ?1")?;

        let result_iter = stmt.query_map(params![tx_id], |row| {
            let amount: String = row.get("amount")?;

            Ok(RuneTXO {
                tx_id: row.get("tx_id")?,
                output_index: row.get("output_index")?,
                rune_id: row.get("rune_id")?,
                amount: amount.parse().unwrap(),
                address: row.get("address")?,
                is_unspent: row.get("is_unspent")?,
                spent_tx_id: row.get("spent_tx_id")?,
                timestamp: row.get("timestamp")?,
                block_height: row.get("block_height")?,
            })
        })?;

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn get_address_runes_txo(&self, address: &str) -> Result<Vec<RuneTXO>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM runes_txos WHERE address = ?1")?;

        let result_iter = stmt
            .query_map(params![address.to_lowercase()], |row| {
                let amount: String = row.get("amount")?;

                Ok(RuneTXO {
                    tx_id: row.get("tx_id")?,
                    output_index: row.get("output_index")?,
                    rune_id: row.get("rune_id")?,
                    amount: amount.parse().unwrap(),
                    address: row.get("address")?,
                    is_unspent: row.get("is_unspent")?,
                    spent_tx_id: row.get("spent_tx_id")?,
                    timestamp: row.get("timestamp")?,
                    block_height: row.get("block_height")?,
                })
            })
            .unwrap();

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn get_address_runes_utxo_by_rune_id(
        &self,
        address: &str,
        rune_id: &str,
    ) -> Result<Vec<RuneTXO>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM runes_txos WHERE address = ?1 AND rune_id = ?2 AND is_unspent = TRUE",
        )?;

        let result_iter = stmt
            .query_map(params![address.to_lowercase(), rune_id], |row| {
                let amount: String = row.get("amount")?;

                Ok(RuneTXO {
                    tx_id: row.get("tx_id")?,
                    output_index: row.get("output_index")?,
                    rune_id: row.get("rune_id")?,
                    amount: amount.parse().unwrap(),
                    address: row.get("address")?,
                    is_unspent: row.get("is_unspent")?,
                    spent_tx_id: row.get("spent_tx_id")?,
                    timestamp: row.get("timestamp")?,
                    block_height: row.get("block_height")?,
                })
            })
            .unwrap();

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn get_rune_count(&self) -> Result<u128, Error> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM rune_entries")?;
        let result_iter = stmt.query_map([], |row| {
            let count: u64 = row.get(0)?;

            Ok(count)
        })?;

        let result = result_iter.map(|r| r.unwrap()).next().unwrap_or_default() as u128;

        Ok(result)
    }

    fn get_block_height(&self) -> Result<u64, Error> {
        let mut stmt = self.conn.prepare("SELECT block_height FROM statistics")?;
        let result_iter = stmt.query_map([], |row| {
            let block_height: u64 = row.get("block_height")?;

            Ok(block_height)
        })?;

        let block_height = result_iter.map(|r| r.unwrap()).next().unwrap_or_default();

        Ok(block_height)
    }

    fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>, Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM transactions WHERE tx_id = ?1")?;
        let result_iter = stmt
            .query_map(params![tx_id], |row| {
                Ok(Transaction {
                    tx_id: row.get("tx_id")?,
                    is_artifact: row.get("is_artifact")?,
                    is_runestone: row.get("is_runestone")?,
                    is_cenotapth: row.get("is_cenotapth")?,
                    cenotapth_message: row.get("cenotapth_message")?,
                    timestamp: row.get("timestamp")?,
                    block_height: row.get("block_height")?,
                })
            })
            .unwrap();

        let result = result_iter.map(|r| r.unwrap()).next();

        Ok(result)
    }

    fn get_transactions(&self) -> Result<Vec<Transaction>, Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM transactions")?;
        let result_iter = stmt
            .query_map([], |row| {
                Ok(Transaction {
                    tx_id: row.get("tx_id")?,
                    is_artifact: row.get("is_artifact")?,
                    is_runestone: row.get("is_runestone")?,
                    is_cenotapth: row.get("is_cenotapth")?,
                    cenotapth_message: row.get("cenotapth_message")?,
                    timestamp: row.get("timestamp")?,
                    block_height: row.get("block_height")?,
                })
            })
            .unwrap();

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn get_runes(&self) -> Result<Vec<RuneEntry>, Error> {
        let mut stmt = self.conn.prepare("SELECT * FROM rune_entries")?;

        let result_iter = stmt.query_map([], |row| {
            let symbol: Option<String> = row.get("symbol")?;
            let premine: String = row.get("premine")?;
            let burned: String = row.get("burned")?;
            let mint_count: String = row.get("mint_count")?;
            let rune_number: String = row.get("rune_number")?;
            let rune_id: String = row.get("rune_id")?;

            let mut terms_stmt = self
                .conn
                .prepare("SELECT * FROM terms WHERE rune_id = ?1")?;

            let terms_result_iter = terms_stmt.query_map(params![rune_id], |row| {
                let amount: Option<String> = row.get("amount")?;
                let cap: Option<String> = row.get("cap")?;

                Ok(Terms {
                    amount: amount.map(|a| a.parse::<u128>().unwrap()),
                    cap: cap.map(|c| c.parse::<u128>().unwrap()),
                    height_start: row.get("height_start")?,
                    height_end: row.get("height_end")?,
                    offset_start: row.get("offset_start")?,
                    offset_end: row.get("offset_end")?,
                })
            })?;

            let terms: Option<Terms> = terms_result_iter.into_iter().next().map(|t| t.unwrap());

            Ok(RuneEntry {
                etching_tx_id: row.get("etching_tx_id")?,
                block_height: row.get("block_height")?,
                rune_id,
                name: row.get("name")?,
                raw_name: row.get("raw_name")?,
                symbol: symbol.map(|s| s.chars().next().unwrap()),
                divisibility: row.get("divisibility")?,
                premine: premine.parse().unwrap(),
                terms,
                burned: burned.parse().unwrap(),
                mint_count: mint_count.parse().unwrap(),
                timestamp: row.get("timestamp")?,
                is_cenotapth: row.get("is_cenotapth")?,
                cenotapth_message: row.get("cenotapth_message")?,
                rune_number: rune_number.parse().unwrap(),
                turbo: row.get("turbo")?,
            })
        })?;

        Ok(result_iter.map(|r| r.unwrap()).collect())
    }

    fn set_block_height(&mut self, new_block_height: u64) -> Result<(), Error> {
        let mut stmt = self.conn.prepare("SELECT block_height FROM statistics")?;
        let result_iter = stmt.query_map([], |row| {
            let block_height: u64 = row.get("block_height")?;

            Ok(block_height)
        })?;

        let block_height = result_iter.map(|r| r.unwrap()).next();

        if let Some(_) = block_height {
            self.conn.execute(
                "UPDATE statistics SET block_height = ?1 WHERE TRUE",
                params![new_block_height],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO statistics (block_height) VALUES (?1)",
                params![new_block_height],
            )?;
        }

        Ok(())
    }

    fn get_db_indexes(&self) -> Result<Vec<SQLiteIndex>, Error> {
        let mut stmt = self.conn.prepare(
            "SELECT
                type, 
                name, 
                tbl_name
            FROM
                sqlite_master
            WHERE
                type= 'index';",
        )?;

        let result_iter = stmt.query_map([], |row| {
            Ok(SQLiteIndex {
                name: row.get("name")?,
                tbl_name: row.get("tbl_name")?,
            })
        })?;

        let indexes: Vec<SQLiteIndex> = result_iter.map(|r| r.unwrap()).collect();

        Ok(indexes)
    }

    fn create_db_indexes(&self) -> Result<(), Error> {
        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_rune_entries_rune_id
                ON rune_entries(rune_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_rune_entries_etching_tx_id
                ON rune_entries(etching_tx_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_rune_entries_raw_name
                ON rune_entries(raw_name);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_terms_rune_id
                ON terms(rune_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_txos_tx_id_output_index
                ON txos(tx_id, output_index);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_tx_id_output_index
                ON runes_txos(tx_id, output_index);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_address_rune_id_is_unspent
                ON runes_txos(address, rune_id, is_unspent);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_address_is_unspent
                ON runes_txos(address, is_unspent);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_tx_id
                ON runes_txos(tx_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_spent_tx_id
                ON runes_txos(spent_tx_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_runes_txos_address
                ON runes_txos(address);
        ",
            (),
        )?;

        self.conn.execute(
            "
                CREATE INDEX IF NOT EXISTS idx_transactions_tx_id
                ON transactions(tx_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_rune_events_tx_id
            ON rune_events(tx_id);
        ",
            (),
        )?;

        self.conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_rune_events_address
            ON rune_events(address);
        ",
            (),
        )?;

        self.conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_rune_events_block_height
            ON rune_events(block_height);
        ",
            (),
        )?;

        self.conn.execute(
            "
            CREATE INDEX IF NOT EXISTS idx_rune_events_rune_id
            ON rune_events(rune_id);
        ",
            (),
        )?;

        Ok(())
    }
}
