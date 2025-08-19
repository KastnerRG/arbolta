use super::*;
use crate::{bit::BitVec, signal::Signals};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

define_arithmetic_cell!(Shl, <<);
define_arithmetic_cell!(Shr, >>);

#[cfg(test)]
mod tests {
  use rstest::rstest;

  #[rstest]
  fn shl() {
    println!("TODO")
  }

  #[rstest]
  fn shr() {
    println!("TODO")
  }
}
