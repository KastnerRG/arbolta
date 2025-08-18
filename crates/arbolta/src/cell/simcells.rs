use super::CellFn;
use crate::{bit::Bit, signal::Signals};
use bincode::{Decode, Encode};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

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
      let reset = !(signals.get_net(self.reset_net) & self.reset_polarity);

      if reset == Bit::ONE {
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
