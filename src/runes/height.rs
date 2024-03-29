use super::*;
use core::ops::{Add, Sub};


#[derive(Copy, Clone, Debug, Ord, Eq, Serialize, PartialEq, PartialOrd)]
pub struct Height(pub u32);

impl Height {
  pub fn n(self) -> u32 {
    self.0
  }

  // pub fn subsidy(self) -> u64 {
  //   Epoch::from(self).subsidy()
  // }

  // pub fn starting_sat(self) -> Sat {
  //   let epoch = Epoch::from(self);
  //   let epoch_starting_sat = epoch.starting_sat();
  //   let epoch_starting_height = epoch.starting_height();
  //   epoch_starting_sat + u64::from(self.n() - epoch_starting_height.n()) * epoch.subsidy()
  // }

  pub fn period_offset(self) -> u32 {
    self.0 % DIFFCHANGE_INTERVAL
  }
}

impl Add<u32> for Height {
  type Output = Self;

  fn add(self, other: u32) -> Height {
    Self(self.0 + other)
  }
}

impl Sub<u32> for Height {
  type Output = Self;

  fn sub(self, other: u32) -> Height {
    Self(self.0 - other)
  }
}

impl PartialEq<u32> for Height {
  fn eq(&self, other: &u32) -> bool {
    self.0 == *other
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn n() {
    assert_eq!(Height(0).n(), 0);
    assert_eq!(Height(1).n(), 1);
  }

  #[test]
  fn add() {
    assert_eq!(Height(0) + 1, 1);
    assert_eq!(Height(1) + 100, 101);
  }

  #[test]
  fn sub() {
    assert_eq!(Height(1) - 1, 0);
    assert_eq!(Height(100) - 50, 50);
  }

  #[test]
  fn eq() {
    assert_eq!(Height(0), 0);
    assert_eq!(Height(100), 100);
  }
  #[test]
  fn period_offset() {
    assert_eq!(Height(0).period_offset(), 0);
    assert_eq!(Height(1).period_offset(), 1);
    assert_eq!(Height(DIFFCHANGE_INTERVAL - 1).period_offset(), 2015);
    assert_eq!(Height(DIFFCHANGE_INTERVAL).period_offset(), 0);
    assert_eq!(Height(DIFFCHANGE_INTERVAL + 1).period_offset(), 1);
  }
}
