use super::*;
use crate::{bit::BitVec, signal::Signals};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

define_arithmetic_cell!(Eq, ==);
define_arithmetic_cell!(Ne, !=);

#[cfg(test)]
mod tests {
  use rstest::rstest;

  #[rstest]
  fn eq() {
    println!("TODO")
  }

  #[rstest]
  fn ne() {
    println!("TODO")
  }
}
