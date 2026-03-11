use super::*;
use crate::{bit::BitVec, signal::Signals};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

macro_rules! define_shift_cell {
  ($rtl_names:expr, $cell_type:ident { $op_net:ident, $shift_net:ident }, $out_net:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      pub signed: bool,
      $op_net: Box<[usize]>,
      $shift_net: Box<[usize]>,
      $out_net: Box<[usize]>,
    }

    impl CellFn for $cell_type {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let $op_net = BitVec::from(bits_from_nets(signals, &self.$op_net));
        let $shift_net = BitVec::from(bits_from_nets(signals, &self.$shift_net)).to_int::<u32>();
        let output_size = self.$out_net.len();

        let $out_net: BitVec = if self.signed {
          if output_size <= 64 {
            let $op_net = $op_net.to_int::<i64>();
            BitVec::from_int($body as i64, Some(output_size))
          } else {
            let $op_net = $op_net.to_int::<i128>();
            BitVec::from_int($body as i128, Some(output_size))
          }
        } else {
          if output_size <= 64 {
            let $op_net = $op_net.to_int::<u64>();
            BitVec::from_int($body as u64, Some(output_size))
          } else {
            let $op_net = $op_net.to_int::<u128>();
            BitVec::from_int($body as u128, Some(output_size))
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

          let signed = match parameters.get(stringify!([<$op_net:upper _SIGNED>])) {
            Some(&net_signed) => net_signed != 0,
            None => false
          };

          $cell_type::new(
            signed,
            connections[stringify!([<$op_net:upper>])].clone(),
            connections[stringify!([<$shift_net:upper>])].clone(),
            connections[stringify!([<$out_net:upper>])].clone()
          ).into()
      })}
    }
  };
}
define_shift_cell!(&["$shl"], Shl { a, b }, y, a.wrapping_shl(b));
define_shift_cell!(&["$shr"], Shr { a, b }, y, a.wrapping_shr(b));

#[cfg(test)]
mod tests {
  use super::*;
  use crate::cell::test_helpers::*;
  use rstest::rstest;

  #[rstest]
  #[case(false, "10011011", "0", "0000000010011011")] // 155 << 0
  #[case(true, "10011011", "0", "1111111110011011")] // -101 << 0
  #[case(false, "10011011", "111", "0100110110000000")] // 155 << 7
  #[case(true, "10011011", "111", "1100110110000000")] // -101 << 7
  fn shl(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Shl, signed, a, b, expected);
  }

  #[rstest]
  #[case(false, "10011011", "0", "0000000010011011")] // 155 >> 0
  #[case(true, "10011011", "0", "1111111110011011")] // -101 >> 0
  #[case(false, "10011011", "111", "0000000000000001")] // 155 >> 7
  #[case(true, "10011011", "111", "1111111111111111")] // -101 >> 7
  fn shr(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
    run_binary_cell_case_signed!(Shr, signed, a, b, expected);
  }
}
