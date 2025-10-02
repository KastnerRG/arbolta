use super::{CellFn, copy_nets};
use crate::{bit::Bit, signal::Signals};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, derive_new::new)]
pub struct Reg {
  polarity: Bit,
  clock_net: usize,
  data_in_nets: Box<[usize]>,
  data_out_nets: Box<[usize]>,
  #[new(default)]
  last_clock: Bit,
}

impl CellFn for Reg {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // Check if clock active for any polarity
    let clock = !(signals.get_net(self.clock_net) ^ self.polarity);

    // Rising edge
    if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      copy_nets(signals, &self.data_in_nets, &self.data_out_nets);
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode, derive_new::new)]
pub struct ALDff {
  clock_polarity: Bit,
  al_polarity: Bit,
  clock_net: usize,
  al_net: usize,
  al_data_in_nets: Box<[usize]>,
  data_in_nets: Box<[usize]>,
  data_out_nets: Box<[usize]>,
  #[new(default)]
  last_clock: Bit,
  #[new(default)]
  last_aload: Bit,
}

impl CellFn for ALDff {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    // TODO: Change this to ==
    let clock = !(signals.get_net(self.clock_net) ^ self.clock_polarity);
    let aload = !(signals.get_net(self.al_net) ^ self.al_polarity);

    // Do asynchronous load
    if aload == Bit::ONE && self.last_aload == Bit::ZERO {
      copy_nets(signals, &self.al_data_in_nets, &self.data_out_nets);
      // Rising edge clock
    } else if clock == Bit::ONE && self.last_clock == Bit::ZERO {
      copy_nets(signals, &self.data_in_nets, &self.data_out_nets);
    }

    self.last_clock = clock;
    self.last_aload = aload;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::ZERO;
  }
}

#[cfg(test)]
mod tests {
  use rstest::rstest;

  #[rstest]
  fn reg() {
    println!("TODO")
  }

  #[rstest]
  fn aldff() {
    println!("TODO")
  }
}
