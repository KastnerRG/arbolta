// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::*;
use crate::{bit::BitVec, signal::Signals};
use paste::paste;
use serde::{Deserialize, Serialize};

macro_rules! define_shift_cell {
  ($rtl_names:expr, $cell_type:ident { $op_port:ident, $shift_port:ident }, $out_port:ident, $body:expr) => {
    paste! {
      #[derive(Debug, Clone, Serialize, Deserialize)]
      pub struct $cell_type {
        pub signed: bool,
        // Nets
        [<$op_port _nets>]: Box<[usize]>,
        [<$shift_port _nets>]: Box<[usize]>,
        [<$out_port _nets>]: Box<[usize]>,
        // Bits
        [<$op_port _bits>]: BitVec,
        [<$shift_port _bits>]: BitVec,
        [<$out_port _bits>]: BitVec,
      }
    }
    paste! {
      impl $cell_type {
        pub fn new(
          signed: bool,
          [<$op_port _nets>]: Box<[usize]>,
          [<$shift_port _nets>]: Box<[usize]>,
          [<$out_port _nets>]: Box<[usize]>,
        ) -> Self {
          Self {
            signed,
            [<$op_port _bits>]: BitVec::new([<$op_port _nets>].len()),
            [<$op_port _nets>],
            [<$shift_port _bits>]: BitVec::new([<$shift_port _nets>].len()),
            [<$shift_port _nets>],
            [<$out_port _bits>]: BitVec::new([<$out_port _nets>].len()),
            [<$out_port _nets>]
          }
        }
      }
    }

    paste! {
      impl CellFn for $cell_type {
        #[inline]
        fn eval(&mut self, signals: &mut Signals) {
          self.[<$op_port _bits>].set_bits(self.[<$op_port _nets>].iter().map(|&n| signals.get_net(n)));
          self.[<$shift_port _bits>].set_bits(self.[<$shift_port _nets>].iter().map(|&n| signals.get_net(n)));

          let $shift_port = self.[<$shift_port _bits>].to_int::<u32>();
          let output_size = self.[<$out_port _nets>].len();

          if self.signed {
            if output_size <= 64 {
              let $op_port = self.[<$op_port _bits>].to_int::<i64>();
              self.[<$out_port _bits>].set_int( $body as i64 );
            } else {
              let $op_port = self.[<$op_port _bits>].to_int::<i128>();
              self.[<$out_port _bits>].set_int( $body as i64 );
            }
          } else {
            if output_size <= 64 {
              let $op_port = self.[<$op_port _bits>].to_int::<u64>();
              self.[<$out_port _bits>].set_int( $body as i64 );
            } else {
              let $op_port = self.[<$op_port _bits>].to_int::<u128>();
              self.[<$out_port _bits>].set_int( $body as i64 );
            }
          };

          copy_bits(signals, &self.[<$out_port _nets>], &self.[<$out_port _bits>]);
        }

        fn reset(&mut self) {}
      }
    }

    paste! {
      inventory::submit! {CellRegistration::new($rtl_names,
        |connections: &BTreeMap<&str, Box<[usize]>>, parameters: &BTreeMap<&str, usize>| {
          if env::var("ARBOLTA_DEBUG").is_ok() {
            println!("Parsing connections: {:#?}", connections);
          }

          let signed = match parameters.get(stringify!([<$op_port:upper _SIGNED>])) {
            Some(&net_signed) => net_signed != 0,
            None => false
          };

          $cell_type::new(
            signed,
            connections[stringify!([<$op_port:upper>])].clone(),
            connections[stringify!([<$shift_port:upper>])].clone(),
            connections[stringify!([<$out_port:upper>])].clone()
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
