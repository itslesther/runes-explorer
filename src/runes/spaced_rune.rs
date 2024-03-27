use serde_with::*;

use super::*;
use std::fmt::Display;
use std::fmt;
use std::str::FromStr;
// use std::error::Error;

// use super::rune::Rune;

#[derive(
  Copy, Clone, Debug, PartialEq, Ord, PartialOrd, Eq, Default, DeserializeFromStr, SerializeDisplay,
)]
pub struct SpacedRune {
  pub rune: Rune,
  pub spacers: u32,
}

impl SpacedRune {
  pub fn new(rune: Rune, spacers: u32) -> Self {
    Self { rune, spacers }
  }
}

impl FromStr for SpacedRune {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut rune = String::new();
    let mut spacers = 0u32;

    for c in s.chars() {
      match c {
        'A'..='Z' => rune.push(c),
        '.' | '•' => {
          let flag = 1 << rune.len().checked_sub(1).ok_or("leading spacer").unwrap();
          if spacers & flag != 0 {
            panic!("double spacer");
          }
          spacers |= flag;
        }
        _ => panic!("invalid character"),
      }
    }

    if 32 - spacers.leading_zeros() >= rune.len().try_into().unwrap() {
      panic!("trailing spacer")
    }

    Ok(SpacedRune {
      rune: rune.parse()?,
      spacers,
    })
  }
}

impl Display for SpacedRune {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let rune = self.rune.to_string();

    for (i, c) in rune.chars().enumerate() {
      write!(f, "{c}")?;

      if i < rune.len() - 1 && self.spacers & 1 << i != 0 {
        write!(f, "•")?;
      }
    }

    Ok(())
  }
}
