// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::*;
use crate::{bit::BitVec, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::ops::Rem;

define_arithmetic_cell!(&["$add"], Add { a, b }, y, a.wrapping_add(b));
define_arithmetic_cell!(&["$div"], Div { a, b }, y, a.wrapping_div(b));
define_arithmetic_cell!(&["$eq"], Equal { a, b }, y, a.eq(&b));
define_arithmetic_cell!(&["$ge"], GreaterEqual { a, b }, y, a.ge(&b));
define_arithmetic_cell!(&["$gt"], GreaterThan { a, b }, y, a.gt(&b));
define_arithmetic_cell!(&["$le"], LessEqual { a, b }, y, a.le(&b));
define_arithmetic_cell!(&["$lt"], LessThan { a, b }, y, a.lt(&b));
define_arithmetic_cell!(&["$mod"], Modulus { a, b }, y, a.rem(b));
define_arithmetic_cell!(&["$mul"], Mul { a, b }, y, a.wrapping_mul(b));
define_arithmetic_cell!(&["$neg"], Negate { a }, y, a.wrapping_neg());
define_arithmetic_cell!(&["$ne"], NotEqual { a, b }, y, a.ne(&b));
define_arithmetic_cell!(&["$sub"], Sub { a, b }, y, a.wrapping_sub(b));

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
  fn div() {
    println!("TODO")
  }

  #[rstest]
  fn equal() {
    println!("TODO")
  }

  #[rstest]
  #[case::zero_unsigned(false, "0000", "0000", "0001")] // 0 >= 0
  #[case::zero_signed(true, "0000", "0000", "0001")] // 0 >= 0
  #[case(false, "0000", "1111", "0000")] // 0 >= 15
  #[case(true, "0000", "1111", "0001")] // 0 >= -1
  #[case(false, "0111", "000011", "0001")] // 7 >= 3
  #[case(true, "0111", "000011", "0001")] // 7 >= 3
  fn greater_equal(
    #[case] signed: bool,
    #[case] a: BitVec,
    #[case] b: BitVec,
    #[case] expected: BitVec,
  ) {
    run_binary_cell_case_signed!(GreaterEqual, signed, a, b, expected);
  }

  #[rstest]
  fn greater_than() {
    println!("TODO")
  }

  #[rstest]
  #[case::zero_unsigned(false, "0000", "0000", "1")]
  #[case::zero_signed(true, "0000", "0000", "1")]
  // 37738 < 4365 = 0
  #[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "0")]
  #[case::signed_normal(true, "1001001101101010", "0001000100001101", "1")]
  #[case::unsigned_equal(false, "101010", "00101010", "1")]
  fn less_equal(
    #[case] signed: bool,
    #[case] a: BitVec,
    #[case] b: BitVec,
    #[case] expected: BitVec,
  ) {
    run_binary_cell_case_signed!(LessEqual, signed, a, b, expected);
  }

  #[rstest]
  // 37738 < 4365 = 0
  #[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "0")]
  #[case::signed_normal(true, "1001001101101010", "0001000100001101", "1")]
  #[case::unsigned_equal(false, "101010", "00101010", "0")]
  fn less_than(
    #[case] signed: bool,
    #[case] a: BitVec,
    #[case] b: BitVec,
    #[case] expected: BitVec,
  ) {
    run_binary_cell_case_signed!(LessThan, signed, a, b, expected);
  }

  #[rstest]
  fn modulus() {
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
  // -37738 = 27798
  #[case::unsigned_normal(false, "1001001101101010", "0110110010010110")]
  // -37738 = -37738
  #[case::signed_normal(true, "001001001101101010", "11110110110010010110")]
  // -(-27798) = 27798
  #[case::signed_normal(true, "1001001101101010", "0110110010010110")]
  fn negate(#[case] signed: bool, #[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case_signed!(Negate, signed, a, expected);
  }

  #[rstest]
  fn not_equal() {
    println!("TODO")
  }
  #[rstest]
  fn sub() {
    println!("TODO")
  }
}
