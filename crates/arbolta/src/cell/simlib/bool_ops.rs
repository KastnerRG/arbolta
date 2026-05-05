// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use std::ops::{BitAnd, BitOr, BitXor};

use super::*;
use crate::{bit::BitVec, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Not {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Not {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Need to pad w/ possible sign extension
    let pad_size = self.y_nets.len().saturating_sub(self.a_nets.len());
    let pad_net = self.a_nets.last().copied().unwrap_or(0) * self.signed as usize;

    let a_nets = self
      .a_nets
      .iter()
      .copied()
      .chain(std::iter::repeat_n(pad_net, pad_size));

    for (a_net, &y_net) in a_nets.zip(self.y_nets.iter()) {
      let a = signals.get_net(a_net);
      signals.set_net(y_net, !a);
    }
  }

  fn reset(&mut self) {}
}

define_arithmetic_cell!(&["$and"], ProcAnd { a, b }, y, a.bitand(b));
define_arithmetic_cell!(&["$or"], ProcOr { a, b }, y, a.bitor(b));
define_arithmetic_cell!(&["$xor"], ProcXor { a, b }, y, a.bitxor(b));

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_helpers::*;
  use rstest::rstest;

  #[rstest]
  #[case(false, "1101", "110010")]
  #[case(false, "1010", "110101")]
  #[case(false, "0110", "111001")]
  #[case(false, "11011110", "100001")]
  #[case(false, "11111101", "000010")]
  #[case(false, "1", "0")]
  #[case(false, "0", "1")]
  #[case(false, "1111", "0000")]
  #[case(false, "0000", "1111")]
  #[case(true, "1101", "000010")]
  #[case(true, "1010", "000101")]
  #[case(true, "0110", "111001")]
  #[case(true, "11011110", "100001")]
  #[case(true, "11111101", "000010")]
  #[case(true, "1", "0")]
  #[case(true, "0", "1")]
  #[case(true, "1111", "0000")]
  #[case(true, "0000", "1111")]
  fn not(#[case] signed: bool, #[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case_signed!(Not, signed, a, expected);
  }

  #[rstest]
  #[case(false, "000", "000", "000")]
  #[case(false, "000", "111", "000")]
  #[case(false, "111", "000", "000")]
  #[case(false, "111", "111", "111")]
  #[case(false, "101", "010", "000")]
  #[case(false, "110", "011", "010")]
  #[case(false, "100", "001", "000")]
  #[case(false, "001", "010", "000")]
  #[case(false, "010", "100", "000")]
  #[case(false, "011", "001", "001")]
  #[case(false, "101", "001", "001")]
  #[case(false, "011", "100", "000")]
  #[case(false, "111", "1", "001")]
  #[case(false, "1", "111", "001")]
  #[case(false, "0011", "10", "0010")]
  #[case(false, "10", "0011", "0010")]
  #[case(false, "00000000", "00000000", "00000000")]
  #[case(false, "11111111", "00000000", "00000000")]
  #[case(false, "00000000", "11111111", "00000000")]
  #[case(false, "11111111", "11111111", "11111111")]
  #[case(false, "10101010", "01010101", "00000000")]
  #[case(false, "11001100", "00110011", "00000000")]
  #[case(false, "11110000", "00001111", "00000000")]
  #[case(true, "000", "000", "000")]
  #[case(true, "111", "111", "111")]
  #[case(true, "111", "000", "000")]
  #[case(true, "1", "0", "000000000000")]
  #[case(true, "0", "1", "000000000000")]
  #[case(true, "1", "1", "111111111111")]
  #[case(true, "01", "1", "000000000001")]
  #[case(true, "1", "01", "000000000001")]
  #[case(true, "0111", "111", "000000000111")]
  #[case(true, "111", "0111", "000000000111")]
  #[case(true, "1001", "1111", "111111111001")]
  #[case(true, "1001", "0111", "000000000001")]
  fn and(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(ProcAnd, signed, a, b, expected);
  }

  #[rstest]
  #[case(false, "000", "000", "000")]
  #[case(false, "000", "111", "111")]
  #[case(false, "111", "000", "111")]
  #[case(false, "101", "010", "111")]
  #[case(false, "110", "011", "111")]
  #[case(false, "100", "001", "101")]
  #[case(false, "001", "010", "011")]
  #[case(false, "010", "100", "110")]
  #[case(false, "011", "001", "011")]
  #[case(false, "101", "001", "101")]
  #[case(false, "011", "100", "111")]
  #[case(false, "111", "1", "111")]
  #[case(false, "1", "111", "111")]
  #[case(false, "0011", "10", "0011")]
  #[case(false, "10", "0011", "0011")]
  #[case(false, "00000000", "00000000", "00000000")]
  #[case(false, "11111111", "00000000", "11111111")]
  #[case(false, "00000000", "11111111", "11111111")]
  #[case(false, "11111111", "11111111", "11111111")]
  #[case(false, "10101010", "01010101", "11111111")]
  #[case(false, "11001100", "00110011", "11111111")]
  #[case(false, "11110000", "00001111", "11111111")]
  #[case(true, "000", "000", "000")]
  #[case(true, "000", "111", "111")]
  #[case(true, "111", "000", "111")]
  #[case(true, "101", "010", "111")]
  #[case(true, "110", "011", "111")]
  #[case(true, "1", "0", "111111111111")]
  #[case(true, "0", "1", "111111111111")]
  #[case(true, "1", "1", "111111111111")]
  #[case(true, "01", "1", "111111111111")]
  #[case(true, "1", "01", "111111111111")]
  #[case(true, "0111", "111", "111111111111")]
  #[case(true, "111", "0111", "111111111111")]
  fn or(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(ProcOr, signed, a, b, expected);
  }

  #[rstest]
  #[case(false, "000", "000", "000")]
  #[case(false, "000", "111", "111")]
  #[case(false, "111", "000", "111")]
  #[case(false, "111", "111", "000")]
  #[case(false, "101", "010", "111")]
  #[case(false, "110", "011", "101")]
  #[case(false, "100", "001", "101")]
  #[case(false, "001", "010", "011")]
  #[case(false, "010", "100", "110")]
  #[case(false, "011", "001", "010")]
  #[case(false, "101", "001", "100")]
  #[case(false, "011", "100", "111")]
  #[case(false, "111", "1", "110")]
  #[case(false, "1", "111", "110")]
  #[case(false, "0011", "10", "0001")]
  #[case(false, "10", "0011", "0001")]
  #[case(false, "00000000", "00000000", "00000000")]
  #[case(false, "11111111", "00000000", "11111111")]
  #[case(false, "00000000", "11111111", "11111111")]
  #[case(false, "11111111", "11111111", "00000000")]
  #[case(false, "10101010", "01010101", "11111111")]
  #[case(false, "11001100", "00110011", "11111111")]
  #[case(false, "11110000", "00001111", "11111111")]
  #[case(true, "000", "000", "000")]
  #[case(true, "111", "111", "000")]
  #[case(true, "111", "000", "111")]
  #[case(true, "1", "0", "111111111111")]
  #[case(true, "0", "1", "111111111111")]
  #[case(true, "1", "1", "000000000000")]
  #[case(true, "01", "1", "111111111110")]
  #[case(true, "1", "01", "111111111110")]
  #[case(true, "0111", "111", "111111111000")]
  #[case(true, "111", "0111", "111111111000")]
  fn xor(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(ProcXor, signed, a, b, expected);
  }
}
