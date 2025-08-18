use super::CellFn;
use crate::{
  bit::{Bit, BitVec},
  signal::Signals,
};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

macro_rules! define_arithmetic_cell {
  ($name:ident, $op:tt) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
    pub struct $name {
      signed: bool,
      a_nets: Box<[usize]>,
      b_nets: Box<[usize]>,
      y_nets: Box<[usize]>,
    }

    impl CellFn for $name {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let (a, b) = (
          BitVec::from(
            self
              .a_nets
              .iter()
              .map(|n| signals.get_net(*n))
              .collect::<Vec<Bit>>(),
          ),
          BitVec::from(
            self
              .b_nets
              .iter()
              .map(|n| signals.get_net(*n))
              .collect::<Vec<Bit>>(),
          ),
        );
        let output_size = self.y_nets.len();

        let y: BitVec = if self.signed {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<i64>(), b.to_int::<i64>());
            BitVec::from_int(a $op b, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<i128>(), b.to_int::<i128>());
            BitVec::from_int(a $op b, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<u64>(), b.to_int::<u64>());
            BitVec::from_int(a $op b, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<u128>(), b.to_int::<u128>());
            BitVec::from_int(a $op b, Some(output_size))
          }
        };

        self
          .y_nets
          .iter()
          .zip(y.into_iter())
          .for_each(|(n, bit)| signals.set_net(*n, bit));
      }

      fn reset(&mut self) {}
    }
  };
}

define_arithmetic_cell!(Add, +);
define_arithmetic_cell!(Sub, -);
define_arithmetic_cell!(Mul, *);
define_arithmetic_cell!(Div, /);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_macros::*;
  use rstest::rstest;

  #[rstest]
  // 37738 + 4365 = 42103
  #[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "1010010001110111")]
  // 155 + 7 = 162
  #[case::unsigned_normal(false, "10011011", "111", "0000000010100010")]
  // 54 + 4234 = 4288
  #[case::unsigned_normal(false, "00110110", "0001000010001010", "1000011000000")]
  //  37738 + 4365 = 1143
  #[case::unsigned_overflow(false, "1001001101101010", "0001000100001101", "0010001110111")]
  #[case(false, "00000111", "00000111", "00001110")] // 7 + 7 = 49
  #[case(false, "00000111", "111", "01110")] // 7 + 7 = 49
  #[case(false, "111", "111", "1110")] // 7 + 7 = 14
  #[case(false, "111", "111", "10")] // 7 + 7 = 4 (overflow)
  #[case(true, "00000111", "00000111", "00001110")] // 7 + 7 = 49
  #[case(true, "00000111", "1001", "00000")] // 7 + -7 = 0
  #[case(true, "1001", "11001", "11110010")] // -7 + -7 = -14
  #[case(true, "1001", "1001", "10")] // -7 + -7 = -4 (overflow)
  #[case(true, "111", "111", "10")] // 7 + 7 = 4 (overflow)
  #[case(true, "1", "0", "111111111111")] // sign extend
  #[case(true, "0", "1", "111111111111")] // sign extend
  #[case(true, "1", "1", "111111111110")] // -1 + -1 = -2
  #[case(true, "01", "1", "000000000000")] // 1 + -1 = 0
  fn add(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case!(Add, signed, a, b, expected);
  }

  #[rstest]
  fn sub() {
    println!("TODO")
  }

  #[rstest]
  // 8712 * 5366 = 46748592
  #[case::unsigned_normal(
    false,
    "0010001000001000",
    "0001010011110110",
    "00000010110010010101001110110000"
  )]
  // 155 * 7 = 1085
  #[case::unsigned_normal(false, "10011011", "111", "0000010000111101")]
  // 54 * 4234 = 228636
  #[case::unsigned_normal(false, "00110110", "0001000010001010", "00110111110100011100")]
  // 37738 + 4365 = 99938
  #[case::unsigned_overflow(false, "1001001101101010", "0001000100001101", "00011000011001100010")]
  fn mul(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case!(Mul, signed, a, b, expected);
  }

  #[rstest]
  fn div() {
    println!("TODO")
  }
}
