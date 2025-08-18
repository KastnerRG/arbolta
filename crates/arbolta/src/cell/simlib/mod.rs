use super::CellFn;
use crate::{bit::Bit, signal::Signals};

mod arithmetic;
mod bool_ops;
mod logic_reduce_ops;
mod registers;
mod various;

#[inline(always)]
fn bits_from_nets(signals: &mut Signals, nets: &[usize]) -> Vec<Bit> {
  nets.iter().map(|&n| signals.get_net(n)).collect()
}

#[inline(always)]
fn copy_nets(signals: &mut Signals, src_nets: &[usize], dst_nets: &[usize]) {
  src_nets.iter().zip(dst_nets.iter()).for_each(|(src, dst)| {
    signals.set_net(*dst, signals.get_net(*src));
  })
}

// Re-export
pub use arithmetic::*;
pub use bool_ops::*;
pub use logic_reduce_ops::*;
pub use registers::*;
pub use various::*;
