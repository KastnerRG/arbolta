// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use super::{CellFn, copy_nets};
use crate::{bit::Bit, cell::simlib::copy_bits, signal::Signals};
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
pub struct DffAsyncLoad {
  clock_polarity: Bit,
  load_polarity: Bit,
  clock_net: usize,
  async_load_net: usize,
  async_data_in_nets: Box<[usize]>,
  data_in_nets: Box<[usize]>,
  data_out_nets: Box<[usize]>,
  #[new(default)]
  last_clock: Bit,
  #[new(default)]
  last_load: Bit,
}

impl CellFn for DffAsyncLoad {
  #[inline]
  fn eval(&mut self, signals: &mut Signals) {
    let clock = signals.get_net(self.clock_net);
    let load = signals.get_net(self.async_load_net);

    let pos_clock = clock == self.clock_polarity;
    let pos_load = load == self.load_polarity;
    let clock_rising = pos_clock && self.last_clock == !self.clock_polarity;
    let load_rising = pos_load && self.last_load == !self.load_polarity;

    if clock_rising || load_rising {
      if pos_load {
        copy_nets(signals, &self.async_data_in_nets, &self.data_out_nets);
      } else {
        copy_nets(signals, &self.data_in_nets, &self.data_out_nets);
      }
    }

    self.last_clock = clock;
    self.last_load = load;
  }

  fn reset(&mut self) {
    self.last_clock = !self.clock_polarity;
    self.last_load = !self.load_polarity;
  }
}

#[allow(clippy::too_many_arguments)]
#[derive(Debug, Clone, Serialize, Deserialize, derive_new::new)]
pub struct DffAsyncResetEnable {
  clock_polarity: Bit,
  enable_polarity: Bit,
  reset_polarity: Bit,
  reset_val: Box<[Bit]>,
  clock_net: usize,
  reset_net: usize,
  enable_net: usize,
  data_in_nets: Box<[usize]>,
  data_out_nets: Box<[usize]>,
  #[new(default)]
  last_clock: Bit,
  #[new(default)]
  last_reset: Bit,
}

impl CellFn for DffAsyncResetEnable {
  fn eval(&mut self, signals: &mut Signals) {
    let clock = signals.get_net(self.clock_net);
    let reset = signals.get_net(self.reset_net);
    let enable = signals.get_net(self.enable_net);

    let pos_clock = clock == self.clock_polarity;
    let pos_reset = reset == self.reset_polarity;
    let clock_rising = pos_clock && self.last_clock == !self.clock_polarity;
    let reset_rising = pos_reset && self.last_reset == !self.reset_polarity;

    if clock_rising || reset_rising {
      if pos_reset {
        copy_bits(signals, &self.data_out_nets, &self.reset_val);
      } else if enable == self.enable_polarity {
        copy_nets(signals, &self.data_in_nets, &self.data_out_nets);
      }
    }

    self.last_clock = clock;
    self.last_reset = reset;
  }

  fn reset(&mut self) {
    self.last_clock = !self.clock_polarity;
    self.last_reset = !self.reset_polarity;
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::super::copy_bits;
  use super::*;
  use crate::{bit::BitVec, cell::test_helpers::*};
  use rstest::rstest;

  fn read_bits(signals: &Signals, nets: &[usize]) -> BitVec {
    BitVec::from_iter(nets.iter().map(|&n| signals.get_net(n)))
  }

  fn write_bits(signals: &mut Signals, nets: &[usize], value: &BitVec) {
    copy_bits(signals, nets, value);
  }

  fn zeros(width: usize) -> BitVec {
    BitVec::from(vec![Bit::ZERO; width])
  }

  #[rstest]
  #[case(Bit::ONE, "1")]
  #[case(Bit::ZERO, "1")]
  #[case(Bit::ONE, "000000000001")]
  #[case(Bit::ZERO, "000000000001")]
  #[case(Bit::ONE, "1101111001101100")]
  #[case(Bit::ZERO, "1101111001101100")]
  fn reg_captures_only_on_active_edge(#[case] polarity: Bit, #[case] first_value: BitVec) {
    let width = first_value.len();
    let nets = allocate_nets(Some(NET_OFFSET + 1), &[&first_value, &first_value]);
    let clock_net = NET_OFFSET;

    let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = Reg::new(
      polarity,
      clock_net,
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    // Start on inactive clock level.
    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), zeros(width));
    assert_eq!(cell.last_clock, !polarity);

    // Change D while clock is inactive: Q must not change.
    write_bits(&mut signals, data_in_nets, &first_value);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), zeros(width));
    assert_eq!(cell.last_clock, !polarity);

    // Active edge: Q captures D.
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), first_value);
    assert_eq!(cell.last_clock, polarity);

    // Hold active level, change D: Q must not change without a new edge.
    let second_value = zeros(width);
    write_bits(&mut signals, data_in_nets, &second_value);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), first_value);
    assert_eq!(cell.last_clock, polarity);

    // Inactive edge: Q must still not change.
    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), first_value);
    assert_eq!(cell.last_clock, !polarity);

    // Hold inactive level, still no change.
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), first_value);
    assert_eq!(cell.last_clock, !polarity);

    // Next active edge captures new D.
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), second_value);
    assert_eq!(cell.last_clock, polarity);
  }

  #[rstest]
  #[case(Bit::ONE, "0001", "1010", "0110")]
  #[case(Bit::ZERO, "0001", "1010", "0110")]
  #[case(Bit::ONE, "1", "0", "1")]
  #[case(Bit::ZERO, "1", "0", "1")]
  fn reg_multiple_cycles(
    #[case] polarity: Bit,
    #[case] v0: BitVec,
    #[case] v1: BitVec,
    #[case] v2: BitVec,
  ) {
    let nets = allocate_nets(Some(NET_OFFSET + 1), &[&v0, &v0]);
    let clock_net = NET_OFFSET;
    let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = Reg::new(
      polarity,
      clock_net,
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);

    write_bits(&mut signals, data_in_nets, &v0);
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), v0);

    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);

    write_bits(&mut signals, data_in_nets, &v1);
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), v1);

    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);

    write_bits(&mut signals, data_in_nets, &v2);
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), v2);
  }

  #[rstest]
  #[case(Bit::ONE, "10101100")]
  #[case(Bit::ZERO, "10101100")]
  fn reg_reset_restores_edge_detector_state(#[case] polarity: Bit, #[case] value: BitVec) {
    let nets = allocate_nets(Some(NET_OFFSET + 1), &[&value, &value]);
    let clock_net = NET_OFFSET;
    let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = Reg::new(
      polarity,
      clock_net,
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    // Drive clock active and evaluate once so internal last_clock changes.
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(cell.last_clock, polarity);

    // Reset should re-arm the edge detector.
    cell.reset();
    assert_eq!(cell.last_clock, !polarity);

    // With clock still active, eval will look like an active edge and capture.
    write_bits(&mut signals, data_in_nets, &value);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), value);
    assert_eq!(cell.last_clock, polarity);
  }

  #[rstest]
  #[case(Bit::ONE)]
  #[case(Bit::ZERO)]
  fn reg_does_not_recapture_without_transition(#[case] polarity: Bit) {
    let init = BitVec::from_str("0011").unwrap();
    let next = BitVec::from_str("1100").unwrap();
    let nets = allocate_nets(Some(NET_OFFSET + 1), &[&init, &init]);
    let clock_net = NET_OFFSET;
    let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = Reg::new(
      polarity,
      clock_net,
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    // Initial capture.
    signals.set_net(clock_net, !polarity);
    cell.eval(&mut signals);
    write_bits(&mut signals, data_in_nets, &init);
    signals.set_net(clock_net, polarity);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), init);

    // Change D while clock remains active; repeated evals must not change Q.
    write_bits(&mut signals, data_in_nets, &next);
    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), init);

    cell.eval(&mut signals);
    assert_eq!(read_bits(&signals, data_out_nets), init);
  }
  // #[rstest]
  // #[case::zero(Bit::ONE, "000000000000")] // 0
  // #[case::one(Bit::ONE, "000000000001")] // 1
  // #[case(Bit::ONE, "1101111001101100")] // 56940
  // #[case(Bit::ZERO, "000000000000")] // 0
  // #[case(Bit::ZERO, "000000000001")] // 1
  // #[case(Bit::ZERO, "1101111001101100")] // 56940
  // fn reg(#[case] polarity: Bit, #[case] data_in: BitVec) {
  //   let reg_size = data_in.len();
  //   let nets = allocate_nets(Some(1), &[&data_in, &data_in]);

  //   let clock_net: usize = 0;
  //   let (data_in_nets, data_out_nets) = (&nets[0], &nets[1]);

  //   let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
  //   let mut cell = Reg::new(
  //     polarity,
  //     clock_net,
  //     data_in_nets.clone(),
  //     data_out_nets.clone(),
  //   );

  //   signals.set_net(clock_net, !polarity); // Reset
  //   cell.eval(&mut signals);
  //   let mut actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual.to_int::<i32>(), 0);
  //   assert_eq!(cell.last_clock, !polarity);

  //   copy_bits(&mut signals, data_in_nets, &data_in);
  //   cell.eval(&mut signals);
  //   actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual.to_int::<i32>(), 0);
  //   assert_eq!(cell.last_clock, !polarity);

  //   signals.set_net(clock_net, polarity); // Rising edge
  //   cell.eval(&mut signals);
  //   actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual, data_in);
  //   assert_eq!(cell.last_clock, polarity);

  //   signals.set_net(clock_net, !polarity); // Falling edge
  //   cell.eval(&mut signals);
  //   actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual, data_in);
  //   assert_eq!(cell.last_clock, !polarity);

  //   copy_bits(&mut signals, data_in_nets, &vec![Bit::ZERO; reg_size]); // Zero
  //   cell.eval(&mut signals);
  //   actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual, data_in);
  //   assert_eq!(cell.last_clock, !polarity);

  //   signals.set_net(clock_net, polarity); // Rising edge
  //   cell.eval(&mut signals);
  //   actual = BitVec::from(bits_from_nets(&mut signals, data_out_nets));
  //   assert_eq!(actual, BitVec::from(vec![Bit::ZERO; reg_size]));
  //   assert_eq!(cell.last_clock, polarity);
  // }

  #[rstest]
  #[case::zero(Bit::ONE, Bit::ZERO, "000000000000")] // 0
  #[case::zero(Bit::ONE, Bit::ONE, "000000000000")] // 0
  #[case::one(Bit::ONE, Bit::ZERO, "000000000001")] // 1
  #[case::one(Bit::ONE, Bit::ONE, "000000000001")] // 1
  #[case(Bit::ONE, Bit::ZERO, "1101111001101100")] // 56940
  #[case(Bit::ONE, Bit::ONE, "1101111001101100")] // 56940
  #[case(Bit::ZERO, Bit::ZERO, "000000000000")] // 0
  #[case(Bit::ZERO, Bit::ONE, "000000000000")] // 0
  #[case(Bit::ZERO, Bit::ZERO, "000000000001")] // 1
  #[case(Bit::ZERO, Bit::ONE, "000000000001")] // 1
  #[case(Bit::ZERO, Bit::ZERO, "1101111001101100")] // 56940
  #[case(Bit::ZERO, Bit::ONE, "1101111001101100")] // 56940

  fn aldff(#[case] clock_polarity: Bit, #[case] load_polarity: Bit, #[case] data_in: BitVec) {
    let _reg_size = data_in.len();
    let nets = allocate_nets(Some(2), &[&data_in, &data_in, &data_in]);
    let (clock_net, load_net) = (0_usize, 1_usize);
    let (data_in_nets, async_data_in_nets, data_out_nets) = (&nets[0], &nets[1], &nets[2]);

    let mut signals = Signals::new(data_out_nets.last().unwrap() + 1);
    let mut cell = DffAsyncLoad::new(
      clock_polarity,
      load_polarity,
      clock_net,
      load_net,
      async_data_in_nets.clone(),
      data_in_nets.clone(),
      data_out_nets.clone(),
    );

    cell.eval(&mut signals);
    let actual = read_bits(&signals, data_out_nets);
    assert_eq!(actual.to_int::<i32>(), 0); // Should start w/ 0
    // TODO: Check other cases
  }
}
