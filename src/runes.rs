use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use {
    self::{flag::Flag, tag::Tag},
    super::*,
};
pub use {
    edict::Edict, etching::Etching, pile::Pile, rune::Rune, rune_id::RuneId, runestone::Runestone,
    spaced_rune::SpacedRune, terms::Terms,
};

use bitcoin::blockdata::opcodes;

pub const MAX_DIVISIBILITY: u8 = 38;

const MAGIC_NUMBER: opcodes::Opcode = opcodes::all::OP_PUSHNUM_13;
const RESERVED: u128 = 6402364363415443603228541259936211926;

mod edict;
mod etching;
mod flag;
mod pile;
mod rune;
mod rune_id;
mod runestone;
mod spaced_rune;
mod tag;
mod terms;
pub mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq)]
pub enum MintError {
    Cap(u128),
    End(u64),
    Start(u64),
    Unmintable,
}

impl Display for MintError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MintError::Cap(cap) => write!(f, "limited to {cap} mints"),
            MintError::End(end) => write!(f, "mint ended on block {end}"),
            MintError::Start(start) => write!(f, "mint starts on block {start}"),
            MintError::Unmintable => write!(f, "not mintable"),
        }
    }
}
