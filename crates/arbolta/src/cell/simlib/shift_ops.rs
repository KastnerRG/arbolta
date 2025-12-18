use super::*;
use crate::{bit::BitVec, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

// define_arithmetic_cell!(Shl, <<);
#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Shl {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Shl {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a = BitVec::from(bits_from_nets(signals, &self.a_nets));
    let shift: u32 = BitVec::from(bits_from_nets(signals, &self.b_nets)).to_int();

    let output_size = self.y_nets.len();
    let y: BitVec = if self.signed {
      if output_size <= 64 {
        BitVec::from_int(
          (a.to_int::<i64>().wrapping_shl(shift)) as i64,
          Some(output_size),
        )
      } else {
        BitVec::from_int(
          (a.to_int::<i128>().wrapping_shl(shift)) as i128,
          Some(output_size),
        )
      }
    } else if output_size <= 64 {
      BitVec::from_int(
        (a.to_int::<u64>().wrapping_shl(shift)) as u64,
        Some(output_size),
      )
    } else {
      BitVec::from_int(
        (a.to_int::<u128>().wrapping_shl(shift)) as u128,
        Some(output_size),
      )
    };

    copy_bits(signals, &self.y_nets, &y);
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Shr {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Shr {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let a = BitVec::from(bits_from_nets(signals, &self.a_nets));
    let shift: u32 = BitVec::from(bits_from_nets(signals, &self.b_nets)).to_int();

    let output_size = self.y_nets.len();
    let y: BitVec = if self.signed {
      if output_size <= 64 {
        BitVec::from_int(
          (a.to_int::<i64>().wrapping_shr(shift)) as i64,
          Some(output_size),
        )
      } else {
        BitVec::from_int(
          (a.to_int::<i128>().wrapping_shr(shift)) as i128,
          Some(output_size),
        )
      }
    } else if output_size <= 64 {
      BitVec::from_int(
        (a.to_int::<u64>().wrapping_shr(shift)) as u64,
        Some(output_size),
      )
    } else {
      BitVec::from_int(
        (a.to_int::<u128>().wrapping_shr(shift)) as u128,
        Some(output_size),
      )
    };

    copy_bits(signals, &self.y_nets, &y);
  }

  fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  #[rstest]
  fn shl() {
    println!("TODO")
  }

  #[rstest]
  fn shr() {
    println!("TODO")
  }
}
