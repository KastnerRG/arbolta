// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::*;
use crate::{bit::Bit, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

macro_rules! define_reduce_cell {
  ($rtl_names:expr, $cell_type:ident { $($in_netn:ident),* $(,)?}, $out_net:ident, $op:tt, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $(
        $in_netn: Box<[usize]>,
      )*

      $out_net: Box<[usize]>
    }

    impl CellFn for $cell_type {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn: Bit = self
            .$in_netn
            .iter()
            .fold(Bit::ZERO, |acc, &n| acc $op signals.get_net(n));
        )*

        signals.set_net(self.$out_net[0], $body );
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          $cell_type::new(
            $(
              connections[stringify!([<$in_netn:upper>])].clone(),
            )*
            connections[stringify!([<$out_net:upper>])].clone()
          ).into()
      })}
    }
  };
}

define_reduce_cell!(&["$reduce_and"], ReduceAnd { a }, y, &, a);
define_reduce_cell!(&["$reduce_or", "$reduce_bool"], ReduceOr { a }, y, |, a);
define_reduce_cell!(&["$logic_and"], LogicAnd { a, b }, y, |, a & b);
define_reduce_cell!(&["$logic_not"], LogicNot { a }, y, |, !a);
define_reduce_cell!(&["$logic_or"], LogicOr { a, b }, y, |, a | b);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bit::BitVec;
  use crate::cell::test_helpers::*;
  use rstest::rstest;

  #[rstest]
  #[case("0", "0")]
  #[case("00000000", "0000000")]
  #[case("10000000", "0000001")]
  #[case("1", "0000001")]
  #[case("1111111", "01")]
  #[case("1010100", "01")]
  fn reduce_or(#[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case!(ReduceOr, a, expected);
  }

  #[rstest]
  fn reduce_and() {
    println!("TODO")
  }

  #[rstest]
  #[case("0", "0", "0")]
  #[case("0", "1", "0")]
  #[case("1", "0", "0")]
  #[case("1", "1", "1")]
  #[case("00000000", "0000000", "0000")]
  #[case("10000000", "0000001", "0001")]
  #[case("1", "0000001", "01")]
  #[case("1111111", "01", "01")]
  #[case("1010100", "0000", "00000")]
  fn logic_and(#[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case!(LogicAnd, a, b, expected);
  }

  #[rstest]
  #[case("0", "1")]
  #[case("1", "0")]
  #[case("00000000", "0000001")]
  #[case("10000000", "0000000")]
  #[case("1", "0000000")]
  #[case("1111111", "00")]
  #[case("1010100", "00")]
  fn logic_not(#[case] a: BitVec, #[case] expected: BitVec) {
    run_unary_cell_case!(LogicNot, a, expected);
  }

  #[rstest]
  #[case("0", "0", "0")]
  #[case("0", "1", "1")]
  #[case("1", "0", "1")]
  #[case("1", "1", "1")]
  #[case("00000000", "0000000", "0000")]
  #[case("10000000", "0000001", "0001")]
  #[case("1", "0000001", "01")]
  #[case("1111111", "01", "01")]
  #[case("1010100", "0000", "00001")]
  fn logic_or(#[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case!(LogicOr, a, b, expected);
  }
}
