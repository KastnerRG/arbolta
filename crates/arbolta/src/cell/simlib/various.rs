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
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, Constructor)]
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
      copy_bits(signals, &self.y_nets, a);
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

  #[rstest]
  fn pmux() {
    println!("TODO");
  }
}
