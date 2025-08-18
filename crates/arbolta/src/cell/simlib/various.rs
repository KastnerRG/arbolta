use super::{CellFn, bits_from_nets, copy_nets};
use crate::{bit::Bit, signal::Signals};
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
    let mut a = bits_from_nets(signals, &self.a_nets);

    // Have to pad
    if a.len() < self.y_nets.len() {
      // Signed and sign-bit set
      let sign_bit_set = a.last().is_some_and(|&b| b.into());
      let pad_bit: Bit = (self.signed && sign_bit_set).into();

      let pad_size = self.y_nets.len() - a.len();
      a.extend(std::iter::repeat_n(pad_bit, pad_size));
    }

    self
      .y_nets
      .iter()
      .zip(a.iter())
      .for_each(|(n, b)| signals.set_net(*n, *b));
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
}
