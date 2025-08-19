use super::{CellFn, bits_from_nets_pad, copy_bits, copy_nets};
use crate::{
  bit::{Bit, BitVec},
  cell::simlib::bits_from_nets,
  signal::Signals,
};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
    copy_bits(signals, &self.y_nets, a);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Constructor)]
pub struct Mux {
  select_net: usize,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Mux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let select = signals.get_net(self.select_net) == Bit::ONE;
    let src_nets = if select { &self.b_nets } else { &self.a_nets };
    copy_nets(signals, src_nets, &self.y_nets);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Constructor)]
pub struct BMux {
  select_nets: Box<[usize]>,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for BMux {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let select = BitVec::from(bits_from_nets(signals, &self.select_nets));
    let start_net: usize = select.to_int::<usize>() * self.y_nets.len();
    let end_net = start_net + self.y_nets.len();

    let a = bits_from_nets(signals, &self.a_nets);
    (start_net..end_net)
      .zip(a)
      .for_each(|(n, b)| signals.set_net(n, b));
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  #[rstest]
  fn pos() {
    println!("TODO");
  }

  #[rstest]
  fn mux() {
    println!("TODO");
  }
  #[rstest]
  fn bmux() {
    println!("TODO");
  }
}
