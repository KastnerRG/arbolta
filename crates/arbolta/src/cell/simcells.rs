use super::{Cell, CellFn};
use crate::{bit::Bit, cell::CellRegistration, signal::Signals};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! define_unary_cell {
  ($name:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
    pub struct $name {
      a_net: usize,
      y_net: usize,
    }

    impl CellFn for $name {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let a: Bit = signals.get_net(self.a_net);
        signals.set_net(self.y_net, $body(a));
      }

      fn reset(&mut self) {}
    }
  };
}

define_unary_cell!(Buffer, |x: Bit| x);
define_unary_cell!(Inverter, |x: Bit| !x);

macro_rules! define_binary_cell {
  ($name:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
    pub struct $name {
      a_net: usize,
      b_net: usize,
      y_net: usize,
    }

    impl CellFn for $name {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let inputs: [Bit; 2] = [signals.get_net(self.a_net), signals.get_net(self.b_net)];
        signals.set_net(self.y_net, $body(inputs));
      }

      fn reset(&mut self) {}
    }
  };
}

define_binary_cell!(And, |x: [Bit; 2]| x[0] & x[1]);
define_binary_cell!(Nand, |x: [Bit; 2]| !(x[0] & x[1]));
define_binary_cell!(Or, |x: [Bit; 2]| x[0] | x[1]);
define_binary_cell!(Nor, |x: [Bit; 2]| !(x[0] | x[1]));
define_binary_cell!(Xor, |x: [Bit; 2]| x[0] ^ x[1]);
define_binary_cell!(Xnor, |x: [Bit; 2]| !(x[0] ^ x[1]));
define_binary_cell!(AndNot, |x: [Bit; 2]| x[0] & !x[1]);
define_binary_cell!(OrNot, |x: [Bit; 2]| x[0] | !x[1]);

macro_rules! define_ternary_cell {
  ($name:ident, $op0_net:ident, $op1_net:ident, $op2_net:ident, $body:expr) => {
    #[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
    pub struct $name {
      $op0_net: usize,
      $op1_net: usize,
      $op2_net: usize,
      y_net: usize,
    }

    impl CellFn for $name {
      #[inline]
      fn eval(&mut self, signals: &mut Signals) {
        let inputs: [Bit; 3] = [
          signals.get_net(self.$op0_net),
          signals.get_net(self.$op1_net),
          signals.get_net(self.$op2_net),
        ];
        signals.set_net(self.y_net, $body(inputs));
      }

      fn reset(&mut self) {}
    }
  };
}

define_ternary_cell!(AndOrInvert, a_net, b_net, c_net, |x: [Bit; 3]| !((x[0]
  & x[1])
  | x[2]));
define_ternary_cell!(OrAndInvert, a_net, b_net, c_net, |x: [Bit; 3]| !((x[0]
  | x[1])
  & x[2]));

define_ternary_cell!(
  Mux2,
  a_net,
  b_net,
  select_net,
  |x: [Bit; 3]| if x[2].into() { x[1] } else { x[0] }
);
define_ternary_cell!(
  NMux2,
  a_net,
  b_net,
  select_net,
  |x: [Bit; 3]| if x[2].into() { !x[1] } else { !x[0] }
);

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, derive_new::new)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, derive_new::new)]
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

// TODO: Create with macro...
// Cell Constructor functions
fn make_buf(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Buffer::new(connections["A"][0], connections["Y"][0]).into()
}

fn make_not(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Inverter::new(connections["A"][0], connections["Y"][0]).into()
}

fn make_and(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  And::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_nand(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Nand::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_or(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Or::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_nor(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Nor::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_xor(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Xor::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_xnor(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Xnor::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_andnot(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  AndNot::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_ornot(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  OrNot::new(
    connections["A"][0],
    connections["B"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_mux(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Mux2::new(
    connections["A"][0],
    connections["B"][0],
    connections["S"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_nmux(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  NMux2::new(
    connections["A"][0],
    connections["B"][0],
    connections["S"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_andorinvert(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  AndOrInvert::new(
    connections["A"][0],
    connections["B"][0],
    connections["C"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_orandinvert(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  OrAndInvert::new(
    connections["A"][0],
    connections["B"][0],
    connections["C"][0],
    connections["Y"][0],
  )
  .into()
}

fn make_dff(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  Dff::new(
    Bit::ONE,
    connections["C"][0],
    connections["D"][0],
    connections["Q"][0],
  )
  .into()
}

fn make_dffreset(
  connections: &BTreeMap<String, Box<[usize]>>,
  _parameters: &BTreeMap<String, usize>,
) -> Cell {
  DffReset::new(
    Bit::ONE,
    Bit::ONE,
    Bit::ZERO,
    connections["C"][0],
    connections["R"][0],
    connections["D"][0],
    connections["Q"][0],
  )
  .into()
}

inventory::submit! {CellRegistration::new(&["BUF", "$_BUF_"], make_buf)}
inventory::submit! {CellRegistration::new(&["NOT", "$_NOT_"], make_not)}
inventory::submit! {CellRegistration::new(&["AND", "$_AND_"], make_and)}
inventory::submit! {CellRegistration::new(&["NAND", "$_NAND_"], make_nand)}
inventory::submit! {CellRegistration::new(&["OR", "$_OR_"], make_or)}
inventory::submit! {CellRegistration::new(&["NOR", "$_NOR_"], make_nor)}
inventory::submit! {CellRegistration::new(&["XOR", "$_XOR_"], make_xor)}
inventory::submit! {CellRegistration::new(&["XNOR", "$_XNOR_"], make_xnor)}
inventory::submit! {CellRegistration::new(&["ANDNOT", "$_ANDNOT_"], make_andnot)}
inventory::submit! {CellRegistration::new(&["ORNOT", "$_ORNOT_"], make_ornot)}
inventory::submit! {CellRegistration::new(&["$_MUX_"], make_mux)}
inventory::submit! {CellRegistration::new(&["$_NMUX_"], make_nmux)}
inventory::submit! {CellRegistration::new(&["$_AOI3_"], make_andorinvert)}
inventory::submit! {CellRegistration::new(&["$_OAI3_"], make_orandinvert)}
inventory::submit! {CellRegistration::new(&["DFF", "$_DFF_P_"], make_dff)}
inventory::submit! {CellRegistration::new(&["$_SDFF_PP0_"], make_dffreset)}
