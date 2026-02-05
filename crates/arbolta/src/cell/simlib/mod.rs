use super::{Cell, CellFn, CellRegistration};
use crate::{bit::Bit, signal::Signals};
use paste::paste;
use std::{collections::BTreeMap, env};
mod arithmetic;
mod bool_ops;
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
  ($rtl_names:expr, $cell_type:ident { $($in_netn:ident),* $(,)?}, $out_net:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      pub signed: bool,

      $(
        $in_netn: Box<[usize]>,
      )*

      $out_net: Box<[usize]>
    }

    impl CellFn for $cell_type {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn = BitVec::from(bits_from_nets(signals, &self.$in_netn));
        )*

        let output_size = self.$out_net.len();

        let $out_net: BitVec = if self.signed {
          if output_size <= 64 {
            let ($($in_netn,)*) = ($($in_netn.to_int::<i64>(),)*);
            BitVec::from_int( $body as i64, Some(output_size))
          } else {
            let ($($in_netn,)*) = ($($in_netn.to_int::<i128>(),)*);
            BitVec::from_int( $body as i128, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let ($($in_netn,)*) = ($($in_netn.to_int::<u64>(),)*);
            BitVec::from_int( $body as u64, Some(output_size))
          } else {
            let ($($in_netn,)*) = ($($in_netn.to_int::<u128>(),)*);
            BitVec::from_int( $body as u128, Some(output_size))
          }
        };

        copy_bits(signals, &self.$out_net, &$out_net);
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          let mut signed = false;
          $(
            if let Some(&net_signed) = parameters.get(stringify!([<$in_netn:upper _SIGNED>])) {
              signed |= (net_signed != 0);
            }
          )*

          $cell_type::new(
            signed,
            $(
              connections[stringify!([<$in_netn:upper>])].clone(),
            )*
            connections[stringify!([<$out_net:upper>])].clone()
          ).into()
      })}
    }
  };
}

#[allow(unused)]
pub(crate) use define_arithmetic_cell;

// Re-export
pub use arithmetic::*;
pub use bool_ops::*;
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

inventory::submit! {CellRegistration::new(&["$not"], make_not)}
inventory::submit! {CellRegistration::new(&["$pos"], make_pos)}
inventory::submit! {CellRegistration::new(&["$dff"], make_dff)}
inventory::submit! {CellRegistration::new(&["$aldff"], make_aldff)}
inventory::submit! {CellRegistration::new(&["$mux"], make_mux)}
inventory::submit! {CellRegistration::new(&["$bmux"], make_bmux)}
inventory::submit! {CellRegistration::new(&["$pmux"], make_pmux)}
