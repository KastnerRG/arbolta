use super::CellFn;
use crate::{bit::Bit, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

macro_rules! reduce_nets {
  ($signals:expr, $nets:expr, $initial:expr, $op:tt) => {
    $nets
      .iter()
      .fold($initial, |acc, i| acc $op $signals.get_net(*i))
  };
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct ReduceAnd {
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for ReduceAnd {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    signals.set_net(
      self.y_nets[0],
      reduce_nets!(signals, self.a_nets, Bit::ZERO, &),
    );
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct ReduceOr {
  pub a_nets: Box<[usize]>,
  pub y_nets: Box<[usize]>,
}

impl CellFn for ReduceOr {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    signals.set_net(
      self.y_nets[0],
      reduce_nets!(signals, self.a_nets, Bit::ZERO, |),
    );
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct LogicAnd {
  pub a_nets: Box<[usize]>,
  pub b_nets: Box<[usize]>,
  pub y_nets: Box<[usize]>,
}

impl CellFn for LogicAnd {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a: Bit = reduce_nets!(signals, self.a_nets, Bit::ZERO, |);
    let b: Bit = reduce_nets!(signals, self.b_nets, Bit::ZERO, |);
    signals.set_net(self.y_nets[0], a & b);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct LogicNot {
  pub a_nets: Box<[usize]>,
  pub y_nets: Box<[usize]>,
}

impl CellFn for LogicNot {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a: Bit = reduce_nets!(signals, self.a_nets, Bit::ZERO, |);
    signals.set_net(self.y_nets[0], !a);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicOr {
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for LogicOr {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a: Bit = reduce_nets!(signals, self.a_nets, Bit::ZERO, |);
    let b: Bit = reduce_nets!(signals, self.b_nets, Bit::ZERO, |);
    signals.set_net(self.y_nets[0], a | b);
  }

  fn reset(&mut self) {}
}

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
  fn logic_or() {
    println!("TODO")
  }
}
