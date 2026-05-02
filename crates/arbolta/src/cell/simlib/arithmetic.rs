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
  #[case::unsigned_basic(false, "1000", "0010", "0100")] // 8 / 2 = 4
  #[case::unsigned_trunc(false, "111", "10", "11")] // 7 / 2 = 3
  #[case::unsigned_equal(false, "1010", "1010", "1")] // 10 / 10 = 1
  #[case::unsigned_less(false, "0010", "1000", "0")] // 2 / 8 = 0
  #[case::unsigned_zero(false, "0000", "1010", "0")] // 0 / 10 = 0
  #[case::unsigned_widen(false, "00110110", "00000010", "00011011")] // 54 / 2 = 27
  #[case::unsigned_mismatch(false, "111", "001", "111")] // 7 / 1 = 7
  #[case::unsigned_overflow(false, "1111", "0001", "1111")] // 15 / 1 = 15
  #[case::signed_basic(true, "01000", "00010", "00100")] // 8 / 2 = 4
  #[case::signed_trunc(true, "00111", "00010", "00011")] // 7 / 2 = 3 (truncate toward 0)
  #[case::signed_neg_pos(true, "11001", "00010", "11101")] // -7 / 2 = -3
  #[case::signed_pos_neg(true, "00111", "11110", "11101")] // 7 / -2 = -3
  #[case::signed_neg_neg(true, "11001", "11110", "00011")] // -7 / -2 = 3
  #[case::signed_equal(true, "11001", "11001", "1")] // -7 / -7 = 1
  #[case::signed_less(true, "00010", "11000", "0")] // 2 / -8 = 0 (trunc toward 0)
  #[case::signed_zero(true, "00000", "11001", "0")] // 0 / -7 = 0
  #[case::signed_extend(true, "1", "1", "1")] // -1 / -1 = 1
  #[case::signed_extend2(true, "1", "01", "111111111111")]
  // #[case::div_by_zero_u(false, "1010", "0000", "0")]
  // #[case::div_by_zero_s(true, "1010", "0000", "0")]
  fn div(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Div, signed, a, b, expected);
  }

  #[rstest]
  #[case(false, "00000111", "00000111", "1")] // 7 == 7
  #[case(false, "00000111", "111", "1")] // 7 == 7
  #[case(false, "111", "111", "1")] // 7 == 7
  #[case(false, "111", "10", "0")] // 7 != 2
  #[case(false, "00000000", "0", "1")] // 0 == 0
  #[case(false, "00000001", "1", "1")] // 1 == 1
  #[case(false, "00000001", "0", "0")] // 1 != 0
  #[case(false, "1001001101101010", "1001001101101010", "1")] // exact equal
  #[case(false, "1001001101101010", "0001000100001101", "0")] // different values
  #[case(false, "00110110", "000000110110", "1")] // zero-extend equal
  #[case(false, "00110110", "0001000010001010", "0")] // 54 != 4234
  #[case(true, "00000111", "00000111", "1")] // 7 == 7
  #[case(true, "00000111", "111", "0")] // 7 == -1
  #[case(true, "111", "111", "1")] // -1 == -1
  #[case(true, "1", "111111111111", "1")] // sign extend: -1 == -1
  #[case(true, "0", "000000000000", "1")] // 0 == 0
  #[case(true, "01", "1", "0")] // 1 != -1
  #[case(true, "01", "0001", "1")] // 1 == 1
  #[case(true, "1001", "11001", "1")] // -7 == -7
  #[case(true, "1001", "1001", "1")] // -7 == -7
  #[case(true, "1001", "0111", "0")] // -7 != 7
  #[case(true, "1", "0", "0")] // -1 != 0
  #[case(true, "0111", "111", "0")] // 7 != -1
  fn equal(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Equal, signed, a, b, expected);
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
  #[case(false, "1000", "0010", "1")] // 8 > 2
  #[case(false, "0010", "1000", "0")] // 2 > 8
  #[case(false, "0111", "0111", "0")] // 7 > 7
  #[case(false, "111", "10", "1")] // 7 > 2
  #[case(false, "10", "111", "0")] // 2 > 7
  #[case(false, "00000000", "0", "0")] // 0 > 0
  #[case(false, "00000001", "0", "1")] // 1 > 0
  #[case(false, "0", "00000001", "0")] // 0 > 1
  #[case(false, "00110110", "000000110101", "1")] // 54 > 53
  #[case(false, "00110110", "000000110110", "0")] // 54 > 54
  #[case(false, "00110110", "000000110111", "0")] // 54 > 55
  #[case(true, "01000", "00010", "1")] // 8 > 2
  #[case(true, "00010", "01000", "0")] // 2 > 8
  #[case(true, "00111", "00111", "0")] // 7 > 7
  #[case(true, "11001", "00010", "0")] // -7 > 2
  #[case(true, "00010", "11001", "1")] // 2 > -7
  #[case(true, "11001", "11110", "0")] // -7 > -2
  #[case(true, "11110", "11001", "1")] // -2 > -7
  #[case(true, "11111", "11111", "0")] // -1 > -1
  #[case(true, "1", "0", "0")] // -1 > 0
  #[case(true, "0", "1", "1")] // 0 > -1
  #[case(true, "1", "111111111111", "0")] // -1 > -1
  #[case(true, "01", "1", "1")] // 1 > -1
  #[case(true, "1", "01", "0")] // -1 > 1
  #[case(true, "1001", "11001", "0")] // -7 > -7
  #[case(true, "11001", "1001", "0")] // -7 > -7
  #[case(true, "0111", "111", "1")] // 7 > -1
  #[case(true, "111", "0111", "0")] // -1 > 7
  fn greater_than(
    #[case] signed: bool,
    #[case] a: BitVec,
    #[case] b: BitVec,
    #[case] expected: BitVec,
  ) {
    run_binary_cell_case_signed!(GreaterThan, signed, a, b, expected);
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
  #[case(false, "1000", "0010", "0000")] // 8 % 2 = 0
  #[case(false, "111", "10", "1")] // 7 % 2 = 1
  #[case(false, "1010", "1010", "0")] // 10 % 10 = 0
  #[case(false, "0010", "1000", "10")] // 2 % 8 = 2
  #[case(false, "0000", "1010", "0")] // 0 % 10 = 0
  #[case(false, "00110110", "00000010", "00000000")] // 54 % 2 = 0
  #[case(false, "00110111", "00000010", "00000001")] // 55 % 2 = 1
  #[case(false, "111", "001", "000")] // 7 % 1 = 0
  #[case(true, "01000", "00010", "00000")] // 8 % 2 = 0
  #[case(true, "00111", "00010", "00001")] // 7 % 2 = 1
  #[case(true, "11001", "00010", "11111")] // -7 % 2 = -1
  #[case(true, "11001", "00011", "11111")] // -7 % 3 = -1
  #[case(true, "00111", "11110", "00001")] // 7 % -2 = 1
  #[case(true, "11001", "11110", "11111")] // -7 % -2 = -1
  #[case(true, "11001", "11001", "0")] // -7 % -7 = 0
  #[case(true, "00010", "11000", "00010")] // 2 % -8 = 2
  #[case(true, "00000", "11001", "0")] // 0 % -7 = 0
  #[case(true, "1", "1", "0")] // -1 % -1 = 0
  #[case(true, "1", "01", "000000000000")] // -1 % 1 = 0
  // #[case(false, "1010", "0000", "0")]
  // #[case(true, "1010", "0000", "0")]
  fn modulus(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Modulus, signed, a, b, expected);
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
  #[case(false, "00000111", "00000111", "0")] // 7 != 7
  #[case(false, "00000111", "111", "0")] // 7 != 7
  #[case(false, "111", "111", "0")] // 7 != 7
  #[case(false, "111", "10", "1")] // 7 != 2
  #[case(false, "00000000", "0", "0")] // 0 != 0
  #[case(false, "00000001", "1", "0")] // 1 != 1
  #[case(false, "00000001", "0", "1")] // 1 != 0
  #[case(false, "1001001101101010", "1001001101101010", "0")]
  #[case(false, "1001001101101010", "0001000100001101", "1")]
  #[case(false, "00110110", "000000110110", "0")] // zero-extend equal
  #[case(false, "00110110", "0001000010001010", "1")] // 54 != 4234
  #[case(true, "00000111", "00000111", "0")] // 7 != 7
  #[case(true, "111", "111", "0")] // -1 != -1
  #[case(true, "1", "111111111111", "0")] // -1 != -1 (sign extend)
  #[case(true, "0", "000000000000", "0")] // 0 != 0
  #[case(true, "01", "1", "1")] // 1 != -1
  #[case(true, "01", "0001", "0")] // 1 != 1
  #[case(true, "1001", "11001", "0")] // -7 != -7
  #[case(true, "1001", "1001", "0")] // -7 != -7
  #[case(true, "1001", "0111", "1")] // -7 != 7
  #[case(true, "1", "0", "1")] // -1 != 0
  #[case(true, "0111", "111", "1")] // 7 != -1
  fn not_equal(
    #[case] signed: bool,
    #[case] a: BitVec,
    #[case] b: BitVec,
    #[case] expected: BitVec,
  ) {
    run_binary_cell_case_signed!(NotEqual, signed, a, b, expected);
  }

  #[rstest]
  #[case::unsigned_basic(false, "1000", "0010", "0110")] // 8 - 2 = 6
  #[case::unsigned_zero(false, "0010", "0010", "0000")] // 2 - 2 = 0
  #[case::unsigned_less(false, "0010", "1000", "1010")] // 2 - 8 = -6 (wrap)
  #[case::unsigned_one(false, "0001", "0001", "0000")] // 1 - 1 = 0
  #[case::unsigned_underflow(false, "0000", "0001", "1111")] // 0 - 1 = -1 (wrap)
  #[case::unsigned_widen(false, "00110110", "00000010", "00110100")] // 54 - 2 = 52
  #[case::unsigned_mismatch(false, "111", "001", "110")] // 7 - 1 = 6
  #[case::unsigned_overflow(false, "0000", "1111", "0001")] // 0 - 15 = -15 (wrap)
  #[case::signed_basic(true, "01000", "00010", "00110")] // 8 - 2 = 6
  #[case::signed_zero(true, "00111", "00111", "00000")] // 7 - 7 = 0
  #[case::signed_neg_pos(true, "11001", "00010", "10111")] // -7 - 2 = -9
  #[case::signed_pos_neg(true, "00111", "11110", "01001")] // 7 - (-2) = 9
  #[case::signed_neg_neg(true, "11001", "11110", "11011")] // -7 - (-2) = -5
  #[case::signed_equal(true, "11001", "11001", "00000")] // -7 - -7 = 0
  #[case::signed_less(true, "00010", "11000", "01010")] // 2 - (-8) = 10
  #[case::signed_zero2(true, "00000", "11001", "00111")] // 0 - (-7) = 7
  #[case::signed_extend(true, "1", "1", "0")] // -1 - -1 = 0
  #[case::signed_extend2(true, "1", "01", "111111111110")] // -1 - 1 = -2
  #[case::signed_overflow(true, "1000", "0001", "0111")] // -8 - 1 = 7 (wrap)
  #[case::signed_overflow2(true, "0111", "1111", "1000")] // 7 - (-1) = -8 (wrap)
  fn sub(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Sub, signed, a, b, expected);
  }
}
