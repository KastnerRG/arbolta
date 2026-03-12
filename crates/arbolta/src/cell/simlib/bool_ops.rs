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
    let a = bits_from_nets_pad(self.signed, signals, &self.a_nets, self.y_nets.len());

    self
      .y_nets
      .iter()
      .zip(a)
      .for_each(|(&n, b)| signals.set_net(n, !b));
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
  #[case(false, "1111", "1111", "1111")]
  #[case(false, "0000", "1111", "0000")]
  fn and(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(ProcAnd, signed, a, b, expected);
  }

  #[rstest]
  fn or() {
    println!("TODO")
  }

  #[rstest]
  fn xor() {
    println!("TODO")
  }
}
