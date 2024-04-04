use {
    self::{flag::Flag, tag::Tag},
    super::*,
};
pub use {
    anyhow::{anyhow, bail, ensure, Context, Error},
    bitcoin::{
        address::{Address, NetworkUnchecked},
        blockdata::{
            constants::{DIFFCHANGE_INTERVAL, MAX_SCRIPT_ELEMENT_SIZE, SUBSIDY_HALVING_INTERVAL},
            locktime::absolute::LockTime,
        },
        consensus::{self, Decodable, Encodable},
        hash_types::{BlockHash, TxMerkleNode},
        hashes::Hash,
        opcodes,
        script::{self, Instruction},
        Amount, Block, Network, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
        Txid, Witness,
    },
    cenotaph::Cenotaph,
    edict::Edict,
    etching::Etching,
    height::Height,
    into_usize::IntoUsize,
    message::Message,
    pile::Pile,
    rune::Rune,
    rune_id::RuneId,
    runestone::Runestone,
    serde::*,
    serde_with::{DeserializeFromStr, SerializeDisplay},
    spaced_rune::SpacedRune,
    std::{
        collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
        fmt,
        fmt::{Display, Formatter},
        path::{Path, PathBuf},
        str::FromStr,
    },
    terms::Terms,
};

pub type Result<T = (), E = Error> = std::result::Result<T, E>;

mod cenotaph;
mod edict;
mod etching;
mod flag;
mod height;
mod into_usize;
mod message;
mod pile;
mod rune;
mod rune_id;
mod runestone;
mod spaced_rune;
mod tag;
mod terms;
pub mod varint;

pub fn default<T: Default>() -> T {
    Default::default()
}

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
