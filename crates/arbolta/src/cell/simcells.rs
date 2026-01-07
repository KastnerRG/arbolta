use super::CellFn;
use crate::{bit::Bit, cell::CellRegistration, signal::Signals};
use derive_more::Constructor;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! create_cell {
  // 1-output
  ($rtl_name:expr, $cell_type:ident { $($in_netn:ident),* }, $out_net:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $(
        $in_netn: usize,
      )*

      $out_net: usize
    }

    impl CellFn for $cell_type {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn: Bit = signals.get_net(self.$in_netn);
        )*
        signals.set_net(self.$out_net, $body($($in_netn,)*));
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new(&[$rtl_name],
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          $cell_type::new(
            $(
              connections[stringify!([<$in_netn:upper>])][0],
            )*
            connections[stringify!([<$out_net:upper>])][0]
          ).into()
      })}
    }
  };

  // N-output
  ($rtl_name:expr, $cell_type:ident { $($in_netn:ident),* $(,)?}, { $($out_netn:ident),+ $(,)?}, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
    pub struct $cell_type {
      $(
        $in_netn: usize,
      )*

      $(
        $out_netn: usize,
      )*
    }

    impl CellFn for $cell_type {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        $(
          let $in_netn: Bit = signals.get_net(self.$in_netn);
        )*

        let ( $($out_netn),+ ) = $body($($in_netn,)*);

        $(
          signals.set_net(self.$out_netn, $out_netn);
        )+
      }

      fn reset(&mut self) {}
    }

    paste! {
      inventory::submit! {CellRegistration::new(&[$rtl_name],
        |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
          $cell_type::new(
            $(
              connections[stringify!([<$in_netn:upper>])][0],
            )*
            $(
              connections[stringify!([<$out_netn:upper>])][0],
            )*
          ).into()
      })}
    }
  };
}

create_cell!("$_BUF_", Buffer { a }, y, |a: Bit| a);
create_cell!("$_NOT_", Inverter { a }, y, |a: Bit| !a);
create_cell!("$_AND_", And { a, b }, y, |a: Bit, b: Bit| a & b);
create_cell!("$_AND3_", And3 { a, b, c }, y, |a, b, c| a & b & c);
create_cell!("$_AND4_", And4 { a, b, c, d }, y, |a, b, c, d| a
  & b
  & c
  & d);
create_cell!("$_NAND_", Nand { a, b }, y, |a: Bit, b: Bit| !(a & b));
create_cell!("$_OR_", Or { a, b }, y, |a: Bit, b: Bit| a | b);
create_cell!("$_OR3_", Or3 { a, b, c }, y, |a: Bit, b: Bit, c: Bit| a
  | b
  | c);
create_cell!(
  "$_OR4_",
  Or4 { a, b, c, d },
  y,
  |a: Bit, b: Bit, c: Bit, d: Bit| a | b | c | d
);
create_cell!("$_NOR_", Nor { a, b }, y, |a: Bit, b: Bit| !(a | b));
create_cell!(
  "$_NOR3_",
  Nor3 { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| !(a | b | c)
);
create_cell!("$_XOR_", Xor { a, b }, y, |a: Bit, b: Bit| a ^ b);
create_cell!("$_XNOR_", Xnor { a, b }, y, |a: Bit, b: Bit| !(a ^ b));
create_cell!("$_ANDNOT_", AndNot { a, b }, y, |a: Bit, b: Bit| a & !b);
create_cell!("$_ORNOT_", OrNot { a, b }, y, |a: Bit, b: Bit| a | !b);
create_cell!(
  "$_ANDOR21_",
  AndOr21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (a1 & a2) | b
);
create_cell!(
  "$_ANDOR211_",
  AndOr211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| (a1 & a2) | b | c
);
create_cell!(
  "$_ANDOR32_",
  AndOr32 { a1, a2, a3, b1, b2 },
  y,
  |a1, a2, a3, b1, b2| (a1 & a2 & a3) | (b1 & b2)
);
create_cell!(
  "$_ANDORREDUCE_",
  AndOrReduce { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| (a & c) | (b & c)
);
create_cell!(
  "$_OAI21_",
  OrAnd21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| (!a1 & !a2) | !b
);
create_cell!(
  "$_AOI21_",
  AOI21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| { (!a1 & !b) | (!a2 & !b) }
);
create_cell!(
  "$_OA21_",
  OA21 { a1, a2, b },
  y,
  |a1: Bit, a2: Bit, b: Bit| { (a1 & b) | (a2 & b) }
);
create_cell!(
  "$_OA211_",
  OA211 { a1, a2, b, c },
  y,
  |a1: Bit, a2: Bit, b: Bit, c: Bit| { (a1 & b & c) | (a2 & b & c) }
);
create_cell!(
  "$_AOI3_",
  AndOrInvert { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| { !((a & b) | c) }
);
create_cell!(
  "$_OAI3_",
  OrAndInvert { a, b, c },
  y,
  |a: Bit, b: Bit, c: Bit| { !((a | b) & c) }
);
create_cell!(
  "$_MUX_",
  Mux2 { a, b, s },
  y,
  |a: Bit, b: Bit, select: Bit| if select.into() { b } else { a }
);
create_cell!(
  "$_NMUX_",
  NMux2 { a, b, s },
  y,
  |a: Bit, b: Bit, select: Bit| if select.into() { !b } else { !a }
);
create_cell!("$_HA_", HalfAdder { a, b }, {s, c}, |a: Bit, b: Bit| (
  a ^ b,
  a & b
));
create_cell!("$_HAI_", HalfAdderInv { a, b }, {sn, con}, |a: Bit, b: Bit| (
  !(a ^ b),
  !(a & b)
));
create_cell!(
  "$_FA_", FullAdder {a, b, ci}, {s, co},
  |a: Bit, b: Bit, carry_in: Bit| {
    let sum = a ^ b;
    (sum ^ carry_in, (a & b) | (sum & carry_in))
  }
);
create_cell!(
  "$_FAI_", FullAdderInv {a, b, ci}, {sn, con},
  |a: Bit, b: Bit, carry_in: Bit| {
    let sum = a ^ b;
    (!(sum ^ carry_in), !((a & b) | (sum & carry_in)))
  }
);

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct Dff {
  polarity: Bit,
  clock_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for Dff {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      let data_in = signals.get_net(self.data_in_net);
      signals.set_net(self.data_out_net, data_in);
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&["$_DFF_P_"],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    Dff::new(
      Bit::ONE,
      connections["CLK"][0],
      connections["D"][0],
      connections["Q"][0],
    ).into()
  })
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct DffInv {
  polarity: Bit,
  clock_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for DffInv {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      let data_in = signals.get_net(self.data_in_net);
      signals.set_net(self.data_out_net, !data_in);
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&["$_DFF_PI_"],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    DffInv::new(
      Bit::ONE,
      connections["CLK"][0],
      connections["D"][0],
      connections["QN"][0],
    ).into()
  })
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct DffReset {
  clock_polarity: Bit,
  reset_polarity: Bit,
  reset_val: Bit,
  clock_net: usize,
  reset_net: usize,
  data_in_net: usize,
  data_out_net: usize,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for DffReset {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.clock_polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      // Check if reset active for any polarity
      if signals.get_net(self.reset_net) == self.reset_polarity {
        signals.set_net(self.data_out_net, self.reset_val);
      } else {
        let data_in = signals.get_net(self.data_in_net);
        signals.set_net(self.data_out_net, data_in);
      }
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

inventory::submit! {
  CellRegistration::new(&["$_SDFF_PP0_"],
  |connections: &BTreeMap<&str, Box<[usize]>>, _parameters: &BTreeMap<&str, usize>| {
    DffReset::new(
      Bit::ONE,
      Bit::ONE,
      Bit::ZERO,
      connections["C"][0],
      connections["R"][0],
      connections["D"][0],
      connections["Q"][0],
    ).into()
  })
}
