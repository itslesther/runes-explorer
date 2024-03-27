use bitcoin::constants::MAX_SCRIPT_ELEMENT_SIZE;
// use super::*;
// use serde::{Deserialize, Serialize};
use serde::*;
use std::collections::{HashMap, VecDeque};
use crate::runes::edict::Edict;
use crate::runes::etching::Etching;
use crate::runes::rune_id::RuneId;
use crate::runes::terms::Terms;
use crate::runes::tag::Tag;
use crate::runes::varint;
use crate::runes::MAX_DIVISIBILITY;
use crate::runes::MAGIC_NUMBER;
use crate::runes::Flag;
use bitcoin::*;
use bitcoin::blockdata::script::*;
use super::Rune;

pub(crate) trait IntoUsize {
  fn into_usize(self) -> usize;
}

impl IntoUsize for u32 {
  fn into_usize(self) -> usize {
    self.try_into().unwrap()
  }
}


const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Runestone {
  pub cenotaph: bool,
  pub edicts: Vec<Edict>,
  pub etching: Option<Etching>,
  pub mint: Option<RuneId>,
  pub pointer: Option<u32>,
}

struct Message {
  cenotaph: bool,
  edicts: Vec<Edict>,
  fields: HashMap<u128, VecDeque<u128>>,
}

enum Payload {
  Valid(Vec<u8>),
  Invalid,
}

impl Message {
  fn from_integers(tx: &Transaction, payload: &[u128]) -> Self {
    let mut edicts = Vec::new();
    let mut fields = HashMap::<u128, VecDeque<u128>>::new();
    let mut cenotaph = false;

    for i in (0..payload.len()).step_by(2) {
      let tag = payload[i];

      if Tag::Body == tag {
        let mut id = RuneId::default();
        for chunk in payload[i + 1..].chunks(4) {
          if chunk.len() != 4 {
            cenotaph = true;
            break;
          }

          let Some(next) = id.next(chunk[0], chunk[1]) else {
            cenotaph = true;
            break;
          };

          let Some(edict) = Edict::from_integers(tx, next, chunk[2], chunk[3]) else {
            cenotaph = true;
            break;
          };

          id = next;
          edicts.push(edict);
        }
        break;
      }

      let Some(&value) = payload.get(i + 1) else {
        cenotaph = true;
        break;
      };

      fields.entry(tag).or_default().push_back(value);
    }

    Self {
      cenotaph,
      edicts,
      fields,
    }
  }
}

impl Runestone {

  fn default<T: Default>() -> T {
    Default::default()
  }

  pub fn from_transaction(transaction: &Transaction) -> Option<Self> {
    Self::decipher(transaction).ok().flatten()
  }

  fn decipher(transaction: &Transaction) -> Result<Option<Self>, script::Error> {
    let payload = match Runestone::payload(transaction)? {
      Some(Payload::Valid(payload)) => payload,
      Some(Payload::Invalid) => {
        return Ok(Some(Self {
          cenotaph: true,
          ..Self::default()
        }))
      }
      None => return Ok(None),
    };

    let Some(integers) = Runestone::integers(&payload) else {
      return Ok(Some(Self {
        cenotaph: true,
        ..Self::default()
      }));
    };

    let Message {
      cenotaph,
      edicts,
      mut fields,
    } = Message::from_integers(transaction, &integers);

    let mint = Tag::Mint.take(&mut fields, |[block, tx]| {
      RuneId::new(block.try_into().ok()?, tx.try_into().ok()?)
    });

    let pointer = Tag::Pointer.take(&mut fields, |[pointer]| {
      let pointer = u32::try_from(pointer).ok()?;
      (pointer.into_usize() < transaction.output.len()).then_some(pointer)
    });

    let divisibility = Tag::Divisibility.take(&mut fields, |[divisibility]| {
      let divisibility = u8::try_from(divisibility).ok()?;
      (divisibility <= MAX_DIVISIBILITY).then_some(divisibility)
    });

    let limit = Tag::Limit.take(&mut fields, |[limit]| Some(limit));

    let rune = Tag::Rune.take(&mut fields, |[rune]| Some(Rune(rune)));

    let cap = Tag::Cap.take(&mut fields, |[cap]| Some(cap));

    let premine = Tag::Premine.take(&mut fields, |[premine]| Some(premine));

    let spacers = Tag::Spacers.take(&mut fields, |[spacers]| {
      let spacers = u32::try_from(spacers).ok()?;
      (spacers <= MAX_SPACERS).then_some(spacers)
    });

    let symbol = Tag::Symbol.take(&mut fields, |[symbol]| {
      char::from_u32(u32::try_from(symbol).ok()?)
    });

    let offset = (
      Tag::OffsetStart.take(&mut fields, |[start_offset]| {
        u64::try_from(start_offset).ok()
      }),
      Tag::OffsetEnd.take(&mut fields, |[end_offset]| u64::try_from(end_offset).ok()),
    );

    let height = (
      Tag::HeightStart.take(&mut fields, |[start_height]| {
        u64::try_from(start_height).ok()
      }),
      Tag::HeightEnd.take(&mut fields, |[start_height]| {
        u64::try_from(start_height).ok()
      }),
    );

    let mut flags = Tag::Flags
      .take(&mut fields, |[flags]| Some(flags))
      .unwrap_or_default();

    let etching = Flag::Etching.take(&mut flags);

    let terms = Flag::Terms.take(&mut flags);

    let overflow = (|| {
      let premine = premine.unwrap_or_default();
      let cap = cap.unwrap_or_default();
      let limit = limit.unwrap_or_default();
      premine.checked_add(cap.checked_mul(limit)?)
    })()
    .is_none();

    let etching = etching.then_some(Etching {
      divisibility,
      premine,
      rune,
      spacers,
      symbol,
      terms: terms.then_some(Terms {
        cap,
        height,
        limit,
        offset,
      }),
    });

    Ok(Some(Self {
      cenotaph: cenotaph || overflow || flags != 0 || fields.keys().any(|tag| tag % 2 == 0),
      edicts,
      etching,
      mint,
      pointer,
    }))
  }

  pub fn encipher(&self) -> ScriptBuf {
    let mut payload = Vec::new();

    if let Some(etching) = self.etching {
      let mut flags = 0;
      Flag::Etching.set(&mut flags);

      if etching.terms.is_some() {
        Flag::Terms.set(&mut flags);
      }

      Tag::Flags.encode([flags], &mut payload);

      Tag::Rune.encode_option(etching.rune.map(|rune| rune.0), &mut payload);
      Tag::Divisibility.encode_option(etching.divisibility, &mut payload);
      Tag::Spacers.encode_option(etching.spacers, &mut payload);
      Tag::Symbol.encode_option(etching.symbol, &mut payload);
      Tag::Premine.encode_option(etching.premine, &mut payload);

      if let Some(terms) = etching.terms {
        Tag::Limit.encode_option(terms.limit, &mut payload);
        Tag::Cap.encode_option(terms.cap, &mut payload);
        Tag::HeightStart.encode_option(terms.height.0, &mut payload);
        Tag::HeightEnd.encode_option(terms.height.1, &mut payload);
        Tag::OffsetStart.encode_option(terms.offset.0, &mut payload);
        Tag::OffsetEnd.encode_option(terms.offset.1, &mut payload);
      }
    }

    if let Some(RuneId { block, tx }) = self.mint {
      Tag::Mint.encode([block.into(), tx.into()], &mut payload);
    }

    Tag::Pointer.encode_option(self.pointer, &mut payload);

    if self.cenotaph {
      Tag::Cenotaph.encode([0], &mut payload);
    }

    if !self.edicts.is_empty() {
      varint::encode_to_vec(Tag::Body.into(), &mut payload);

      let mut edicts = self.edicts.clone();
      edicts.sort_by_key(|edict| edict.id);

      let mut previous = RuneId::default();
      for edict in edicts {
        let (block, tx) = previous.delta(edict.id).unwrap();
        varint::encode_to_vec(block, &mut payload);
        varint::encode_to_vec(tx, &mut payload);
        varint::encode_to_vec(edict.amount, &mut payload);
        varint::encode_to_vec(edict.output.into(), &mut payload);
        previous = edict.id;
      }
    }

    let mut builder = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .push_opcode(MAGIC_NUMBER);

    for chunk in payload.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
      let push: &script::PushBytes = chunk.try_into().unwrap();
      builder = builder.push_slice(push);
    }

    builder.into_script()
  }

  fn payload(transaction: &Transaction) -> Result<Option<Payload>, script::Error> {
    // search transaction outputs for payload
    for output in &transaction.output {
      let mut instructions = output.script_pubkey.instructions();

      // payload starts with OP_RETURN
      if instructions.next().transpose()? != Some(Instruction::Op(opcodes::all::OP_RETURN)) {
        continue;
      }

      // followed by the protocol identifier
      if instructions.next().transpose()? != Some(Instruction::Op(MAGIC_NUMBER)) {
        continue;
      }

      // construct the payload by concatinating remaining data pushes
      let mut payload = Vec::new();

      for result in instructions {
        if let Instruction::PushBytes(push) = result? {
          payload.extend_from_slice(push.as_bytes());
        } else {
          return Ok(Some(Payload::Invalid));
        }
      }

      return Ok(Some(Payload::Valid(payload)));
    }

    Ok(None)
  }

  fn integers(payload: &[u8]) -> Option<Vec<u128>> {
    let mut integers = Vec::new();
    let mut i = 0;

    while i < payload.len() {
      let (integer, length) = varint::decode(&payload[i..])?;
      integers.push(integer);
      i += length;
    }

    Some(integers)
  }
}
