use super::*;
use crate::{bit::BitVec, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::ops::Rem;

define_arithmetic_cell!(Add, wrapping_add);
define_arithmetic_cell!(Sub, wrapping_sub);
define_arithmetic_cell!(Mul, wrapping_mul);
define_arithmetic_cell!(Div, wrapping_div);
define_arithmetic_cell!(Modulus, rem);
define_arithmetic_cell!(Lt, &lt);
define_arithmetic_cell!(Le, &le);
define_arithmetic_cell!(Gt, &gt);
define_arithmetic_cell!(Ge, &ge);

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Neg {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Neg {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a = BitVec::from(bits_from_nets(signals, &self.a_nets));
    let output_size = self.y_nets.len();

    let y: BitVec = if output_size <= 64 {
      let a = -a.to_int::<i64>();
      if self.signed {
        BitVec::from_int(a, Some(output_size))
      } else {
        BitVec::from_int(a as u64, Some(output_size))
      }
    } else {
      let a = -a.to_int::<i128>();
      if self.signed {
        BitVec::from_int(a, Some(output_size))
      } else {
        BitVec::from_int(a as u128, Some(output_size))
      }
    };

    copy_bits(signals, &self.y_nets, &y);
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_helpers::*;
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
    run_binary_cell_case_signed!(Add, signed, a, b, expected);
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
    run_binary_cell_case_signed!(Mul, signed, a, b, expected);
  }

  #[rstest]
  fn div() {
    println!("TODO")
  }

  #[rstest]
  fn modulus() {
    println!("TODO")
  }

  #[rstest]
  // 37738 < 4365 = 0
  #[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "0")]
  #[case::signed_normal(true, "1001001101101010", "0001000100001101", "1")]
  #[case::unsigned_equal(false, "101010", "00101010", "1")]
  fn le(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Le, signed, a, b, expected);
  }

  #[rstest]
  // 37738 < 4365 = 0
  #[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "0")]
  #[case::signed_normal(true, "1001001101101010", "0001000100001101", "1")]
  #[case::unsigned_equal(false, "101010", "00101010", "0")]
  fn lt(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Lt, signed, a, b, expected);
  }

  #[rstest]
  fn ge() {
    println!("TODO")
  }

  #[rstest]
  fn gt() {
    println!("TODO")
  }

  #[rstest]
  // -37738 = 27798
  #[case::unsigned_normal(false, "1001001101101010", "0110110010010110")]
  // -37738 = -37738
  #[case::signed_normal(true, "001001001101101010", "11110110110010010110")]
  // -(-27798) = 27798
  #[case::signed_normal(true, "1001001101101010", "0110110010010110")]
  fn neg(#[case] signed: bool, #[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case_signed!(Neg, signed, a, expected);
  }
}
