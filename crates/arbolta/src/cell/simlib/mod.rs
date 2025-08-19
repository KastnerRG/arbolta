use super::CellFn;
use crate::{bit::Bit, signal::Signals};

mod arithmetic;
mod bool_ops;
mod compare_ops;
mod logic_reduce_ops;
mod registers;
mod shift_ops;
mod various;

#[inline(always)]
fn bits_from_nets(signals: &mut Signals, nets: &[usize]) -> Vec<Bit> {
  nets.iter().map(|&n| signals.get_net(n)).collect()
}

#[inline(always)]
fn bits_from_nets_pad(
  signed: bool,
  signals: &mut Signals,
  nets: &[usize],
  out_size: usize,
) -> Vec<Bit> {
  let mut bits = bits_from_nets(signals, nets);
  if bits.len() < out_size {
    let sign_bit_set = bits.last().is_some_and(|&b| b.into());
    let pad_bit: Bit = (signed && sign_bit_set).into();
    let pad_size = out_size - bits.len();

    bits.extend(std::iter::repeat_n(pad_bit, pad_size));
  }

  bits
}

#[inline(always)]
fn copy_nets(signals: &mut Signals, src_nets: &[usize], dst_nets: &[usize]) {
  src_nets.iter().zip(dst_nets.iter()).for_each(|(src, dst)| {
    signals.set_net(*dst, signals.get_net(*src));
  })
}

#[inline(always)]
fn copy_bits(signals: &mut Signals, dst_nets: &[usize], bits: impl IntoIterator<Item = Bit>) {
  dst_nets
    .iter()
    .zip(bits)
    .for_each(|(&n, b)| signals.set_net(n, b));
}

#[macro_export]
macro_rules! define_arithmetic_cell {
  ($name:ident, $op:tt) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
    pub struct $name {
      signed: bool,
      a_nets: Box<[usize]>,
      b_nets: Box<[usize]>,
      y_nets: Box<[usize]>,
    }

    impl CellFn for $name {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let a = BitVec::from(bits_from_nets(signals, &self.a_nets));
        let b = BitVec::from(bits_from_nets(signals, &self.b_nets));

        let output_size = self.y_nets.len();

        let y: BitVec = if self.signed {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<i64>(), b.to_int::<i64>());
            BitVec::from_int((a $op b) as i64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<i128>(), b.to_int::<i128>());
            BitVec::from_int((a $op b) as i128, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<u64>(), b.to_int::<u64>());
            BitVec::from_int((a $op b) as u64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<u128>(), b.to_int::<u128>());
            BitVec::from_int((a $op b) as u128, Some(output_size))
          }
        };

        copy_bits(signals, &self.y_nets, y);
      }

      fn reset(&mut self) {}
    }
  }
}

#[allow(unused)]
pub(crate) use define_arithmetic_cell;

// #[macro_export]
// macro_rules! define_unary_arithmetic_cell {
//   ($name:ident, $op:tt) => {
//     #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
//     pub struct $name {
//       signed: bool,
//       a_nets: Box<[usize]>,
//       y_nets: Box<[usize]>,
//     }

//     impl CellFn for $name {
//       #[inline]
//       fn eval(&mut self, signals: &mut Signals) {
//         let a = BitVec::from(bits_from_nets(signals, &self.a_nets));
//         let output_size = self.y_nets.len();

//         let y: BitVec = if self.signed {
//           if output_size <= 64 {
//             let a = a.to_int::<i64>();
//             BitVec::from_int(($op a) as i64, Some(output_size))
//           } else {
//             let a = a.to_int::<i128>();
//             BitVec::from_int(($op a) as i128, Some(output_size))
//           }
//         } else if output_size <= 64 {
//             let a = a.to_int::<u64>();
//             BitVec::from_int(($op a) as u64, Some(output_size))
//         } else {
//             let a = a.to_int::<u128>();
//             BitVec::from_int(($op a) as u128, Some(output_size))
//         };

//         copy_bits(signals, &self.y_nets, y);
//       }

//       fn reset(&mut self) {}
//     }
//   }
// }

// #[allow(unused)]
// pub(crate) use define_unary_arithmetic_cell;

// Re-export
pub use arithmetic::*;
pub use bool_ops::*;
pub use compare_ops::*;
pub use logic_reduce_ops::*;
pub use registers::*;
pub use shift_ops::*;
pub use various::*;
