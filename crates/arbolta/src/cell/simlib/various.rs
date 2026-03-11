// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::{CellFn, bits_from_nets_pad, copy_bits, copy_nets};
use crate::{
  bit::{Bit, BitVec},
  cell::simlib::bits_from_nets,
  signal::Signals,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Pos {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Pos {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Passthrough with padding
    let a = bits_from_nets_pad(self.signed, signals, &self.a_nets, self.y_nets.len());
    copy_bits(signals, &self.y_nets, &a);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct Mux {
  pub select_net: usize,
  pub a_nets: Box<[usize]>,
  pub b_nets: Box<[usize]>,
  pub y_nets: Box<[usize]>,
}

impl CellFn for Mux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let src_nets = match signals.get_net(self.select_net) {
      Bit::ZERO => &self.a_nets,
      Bit::ONE => &self.b_nets,
    };
    copy_nets(signals, src_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct BMux {
  select_nets: Box<[usize]>,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

// "Selects between 'slices' of A where each value of S corresponds to a unique"
impl CellFn for BMux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let select = BitVec::from(bits_from_nets(signals, &self.select_nets));
    let start_net = select.to_int::<usize>() * self.y_nets.len();
    let end_net = start_net + self.y_nets.len();

    // TODO: Don't read all bits
    let a = bits_from_nets(signals, &self.a_nets);
    (start_net..end_net)
      .zip(a)
      .for_each(|(n, b)| signals.set_net(n, b));
  }

  fn reset(&mut self) {}
}
// "Selects between 'slices' of B where each slice corresponds to a single bit
// of S. Outputs A when all bits of S are low."
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct PMux {
  select_nets: Box<[usize]>,
  a_nets: Box<[usize]>, // output when S all low
  b_nets: Box<[usize]>, // slices source
  y_nets: Box<[usize]>, // slice size
}

impl CellFn for PMux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Should be power of 2?
    // 0, 2, 4, 8
    let select = BitVec::from(bits_from_nets(signals, &self.select_nets)).to_int::<usize>();
    // let src_nets = &self.a_nets;
    if select == 0 {
      let a = bits_from_nets(signals, &self.a_nets);
      copy_bits(signals, &self.y_nets, &a);
    } else {
      let start_net = (select.ilog2() as usize) * self.y_nets.len();
      let end_net = start_net + self.y_nets.len();

      // TODO: Don't read all bits
      let a = bits_from_nets(signals, &self.b_nets);
      (start_net..end_net)
        .zip(a)
        .for_each(|(n, b)| signals.set_net(n, b));
    }
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_helpers::*;
  use rstest::rstest;

  #[rstest]
  fn pos() {
    println!("TODO");
  }

  #[rstest]
  #[case(Bit::ZERO, "000", "001", "000")]
  #[case(Bit::ONE, "000", "001", "001")]
  #[case(Bit::ZERO, "111", "000", "111")]
  #[case(Bit::ONE, "111", "000", "000")]
  fn mux(#[case] select: Bit, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    let nets = allocate_nets(Some(1), &[&a, &b, &expected]);

    let select_net: usize = 0;
    let (a_nets, b_nets, y_nets) = (&nets[0], &nets[1], &nets[2]);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);
    let mut cell = Mux::new(select_net, a_nets.clone(), b_nets.clone(), y_nets.clone());

    signals.set_net(select_net, select);
    copy_bits(&mut signals, a_nets, &a);
    copy_bits(&mut signals, b_nets, &b);

    cell.eval(&mut signals);
    let actual = BitVec::from(bits_from_nets(&mut signals, y_nets));

    assert_eq!(actual, expected);
  }

  #[rstest]
  fn bmux() {
    println!("TODO");
  }

  #[rstest]
  fn pmux() {
    println!("TODO");
  }
}
