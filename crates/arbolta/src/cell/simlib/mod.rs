use super::{Cell, CellFn, CellRegistration};
use crate::{bit::Bit, signal::Signals};
use std::collections::BTreeMap;

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
fn copy_bits<'a>(
  signals: &mut Signals,
  dst_nets: &[usize],
  bits: impl IntoIterator<Item = &'a Bit>,
) {
  dst_nets
    .iter()
    .zip(bits)
    .for_each(|(&n, b)| signals.set_net(n, *b));
}

#[macro_export]
macro_rules! define_arithmetic_cell {
  // Takes `b` by value
  ($name:ident, $op:ident) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $name {
      pub signed: bool,
      pub a_nets: Box<[usize]>,
      pub b_nets: Box<[usize]>,
      pub y_nets: Box<[usize]>,
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
            BitVec::from_int((a.$op(b)) as i64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<i128>(), b.to_int::<i128>());
            BitVec::from_int((a.$op(b)) as i128, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<u64>(), b.to_int::<u64>());
            BitVec::from_int((a.$op(b)) as u64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<u128>(), b.to_int::<u128>());
            BitVec::from_int((a.$op(b)) as u128, Some(output_size))
          }
        };

        copy_bits(signals, &self.y_nets, &y);
      }

      fn reset(&mut self) {}
    }
  };
  // Takes `b` by reference
  ($name:ident, & $op:ident) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
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
            BitVec::from_int((a.$op(&b)) as i64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<i128>(), b.to_int::<i128>());
            BitVec::from_int((a.$op(&b)) as i128, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let (a, b) = (a.to_int::<u64>(), b.to_int::<u64>());
            BitVec::from_int((a.$op(&b)) as u64, Some(output_size))
          } else {
            let (a, b) = (a.to_int::<u128>(), b.to_int::<u128>());
            BitVec::from_int((a.$op(&b)) as u128, Some(output_size))
          }
        };

        copy_bits(signals, &self.y_nets, &y);
      }

      fn reset(&mut self) {}
    }
  };
}

#[allow(unused)]
pub(crate) use define_arithmetic_cell;

// Re-export
pub use arithmetic::*;
pub use bool_ops::*;
pub use compare_ops::*;
pub use logic_reduce_ops::*;
pub use registers::*;
pub use shift_ops::*;
pub use various::*;

// TODO: clean up temp functions
fn make_not(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Not::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_pos(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Pos::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}
fn make_neg(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Neg::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_add(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Add::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_sub(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Sub::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_mul(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Mul::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_div(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Div::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_mod(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Modulus::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_le(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  Le::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_ge(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  Ge::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_gt(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  Gt::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_shl(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Shl::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_shr(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Shr::new(
    parameters["A_SIGNED"] != 0,
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_dff(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Reg::new(
    (parameters["CLK_POLARITY"] != 0).into(),
    connections["CLK"][0],
    connections["D"].clone(),
    connections["Q"].clone(),
  )
  .into()
}

fn make_aldff(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ALDff::new(
    (parameters["CLK_POLARITY"] != 0).into(),
    (parameters["ALOAD_POLARITY"] != 0).into(),
    connections["CLK"][0],
    connections["ALOAD"][0],
    connections["AD"].clone(),
    connections["D"].clone(),
    connections["Q"].clone(),
  )
  .into()
}

fn make_mux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  Mux::new(
    connections["S"][0],
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_bmux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  BMux::new(
    connections["S"].clone(),
    connections["A"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_pmux(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  PMux::new(
    connections["S"].clone(),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_logic_and(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  LogicAnd::new(
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_logic_not(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  LogicNot::new(connections["A"].clone(), connections["Y"].clone()).into()
}

fn make_reduce_or(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ReduceOr::new(connections["A"].clone(), connections["Y"].clone()).into()
}

fn make_reduce_and(
  connections: &BTreeMap<&str, Box<[usize]>>,
  _parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ReduceAnd::new(connections["A"].clone(), connections["Y"].clone()).into()
}

fn make_and(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ProcAnd::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_or(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  ProcOr::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_xor(
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
) -> Cell {
  ProcXor::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_eq(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  Eq::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

fn make_ne(connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>) -> Cell {
  Ne::new(
    (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
    connections["A"].clone(),
    connections["B"].clone(),
    connections["Y"].clone(),
  )
  .into()
}

inventory::submit! {CellRegistration::new(&["$not"], make_not)}
inventory::submit! {CellRegistration::new(&["$pos"], make_pos)}
inventory::submit! {CellRegistration::new(&["$neg"], make_neg)}
inventory::submit! {CellRegistration::new(&["$add"], make_add)}
inventory::submit! {CellRegistration::new(&["$sub"], make_sub)}
inventory::submit! {CellRegistration::new(&["$mul"], make_mul)}
inventory::submit! {CellRegistration::new(&["$div"], make_div)}
inventory::submit! {CellRegistration::new(&["$mod"], make_mod)}
inventory::submit! {CellRegistration::new(&["$le"], make_le)}
inventory::submit! {CellRegistration::new(&["$ge"], make_ge)}
inventory::submit! {CellRegistration::new(&["$gt"], make_gt)}
inventory::submit! {CellRegistration::new(&["$shl"], make_shl)}
inventory::submit! {CellRegistration::new(&["$shr"], make_shr)}
inventory::submit! {CellRegistration::new(&["$dff"], make_dff)}
inventory::submit! {CellRegistration::new(&["$aldff"], make_aldff)}
inventory::submit! {CellRegistration::new(&["$mux"], make_mux)}
inventory::submit! {CellRegistration::new(&["$bmux"], make_bmux)}
inventory::submit! {CellRegistration::new(&["$pmux"], make_pmux)}
inventory::submit! {CellRegistration::new(&["$logic_and"], make_logic_and)}
inventory::submit! {CellRegistration::new(&["$logic_not"], make_logic_not)}
inventory::submit! {CellRegistration::new(&["$reduce_or", "$reduce_bool"], make_reduce_or)}
inventory::submit! {CellRegistration::new(&["$reduce_and"], make_reduce_and)}
inventory::submit! {CellRegistration::new(&["$and"], make_and)}
inventory::submit! {CellRegistration::new(&["$or"], make_or)}
inventory::submit! {CellRegistration::new(&["$xor"], make_xor)}
inventory::submit! {CellRegistration::new(&["$eq"], make_eq)}
inventory::submit! {CellRegistration::new(&["$ne"], make_ne)}
