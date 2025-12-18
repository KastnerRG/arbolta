use super::{CellFn, copy_nets};
use crate::{bit::Bit, signal::Signals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct Reg {
  pub polarity: Bit,
  pub clock_net: usize,
  pub data_in_nets: Box<[usize]>,
  pub data_out_nets: Box<[usize]>,
  #[new(default)]
  pub last_clock: Bit,
}

impl CellFn for Reg {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let clock = signals.get_net(self.clock_net);

    if clock == self.polarity && self.last_clock == !self.polarity {
      copy_nets(signals, &self.data_in_nets, &self.data_out_nets);
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = !self.polarity;
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
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
  use super::super::{bits_from_nets, copy_bits};
  use super::*;
  use crate::{bit::BitVec, cell::test_helpers::*};
  use rstest::rstest;

  #[rstest]
  #[case::zero(Bit::ONE, "000000000000")] // 0
  #[case::one(Bit::ONE, "000000000001")] // 1
  #[case(Bit::ONE, "1101111001101100")] // 56940
  #[case(Bit::ZERO, "000000000000")] // 0
  #[case(Bit::ZERO, "000000000001")] // 1
  #[case(Bit::ZERO, "1101111001101100")] // 56940
  fn reg(#[case] polarity: Bit, #[case] data_in: BitVec) {
    let reg_size = data_in.len();
    let nets = allocate_nets(Some(1), &[&data_in, &data_in]);

    let clock_net: usize = 0;
    let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = Reg::new(
      polarity,
      clock_net,
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    signals.set_net(clock_net, !polarity); // Reset
    cell.eval(&mut signals);
    let mut actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual.to_int::<i32>(), 0);
    assert_eq!(cell.last_clock, !polarity);

    copy_bits(&mut signals, data_in_nets, &data_in);
    cell.eval(&mut signals);
    actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual.to_int::<i32>(), 0);
    assert_eq!(cell.last_clock, !polarity);

    signals.set_net(clock_net, polarity); // Rising edge
    cell.eval(&mut signals);
    actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual, data_in);
    assert_eq!(cell.last_clock, polarity);

    signals.set_net(clock_net, !polarity); // Falling edge
    cell.eval(&mut signals);
    actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual, data_in);
    assert_eq!(cell.last_clock, !polarity);

    copy_bits(&mut signals, data_in_nets, &vec![Bit::ZERO; reg_size]); // Zero
    cell.eval(&mut signals);
    actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual, data_in);
    assert_eq!(cell.last_clock, !polarity);

    signals.set_net(clock_net, polarity); // Rising edge
    cell.eval(&mut signals);
    actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
    assert_eq!(actual, BitVec::from(vec![Bit::ZERO; reg_size]));
    assert_eq!(cell.last_clock, polarity);
  }

  #[rstest]
  fn aldff() {
    println!("TODO")
  }
}
