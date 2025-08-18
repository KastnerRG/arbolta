use super::{CellFn, bits_from_nets};
use crate::{
  bit::{Bit, BitVec},
  signal::Signals,
};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
pub struct Not {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Not {
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
      .for_each(|(n, b)| signals.set_net(*n, !*b));
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
pub struct ProcAnd {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for ProcAnd {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let (a, b) = (
      BitVec::from(
        self
          .a_nets
          .iter()
          .map(|n| signals.get_net(*n))
          .collect::<Vec<Bit>>(),
      ),
      BitVec::from(
        self
          .b_nets
          .iter()
          .map(|n| signals.get_net(*n))
          .collect::<Vec<Bit>>(),
      ),
    );

    let output_size = self.y_nets.len();
    let y: BitVec = if self.signed {
      if output_size <= 64 {
        let (a, b) = (a.to_int::<i64>(), b.to_int::<i64>());
        BitVec::from_int(a & b, Some(output_size))
      } else {
        let (a, b) = (a.to_int::<i128>(), b.to_int::<i128>());
        BitVec::from_int(a & b, Some(output_size))
      }
    } else if output_size <= 64 {
      let (a, b) = (a.to_int::<u64>(), b.to_int::<u64>());
      BitVec::from_int(a & b, Some(output_size))
    } else {
      let (a, b) = (a.to_int::<u128>(), b.to_int::<u128>());
      BitVec::from_int(a & b, Some(output_size))
    };

    self
      .y_nets
      .iter()
      .zip(y)
      .for_each(|(&n, bit)| signals.set_net(n, bit));
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_macros::*;
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
  fn and(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case!(ProcAnd, signed, a, b, expected);
  }
}
