// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::signal::Signals;

#[test]
fn test_signal_net_init() {
  let x = Signals::new(1);

  assert_eq!(x.get_net(0), Bit::ZERO);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);
  assert_eq!(x.get_toggles_total(0), 0);
}

#[test]
fn test_signal_net_set_value() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(0), Bit::ZERO);
  x.set_net(0, Bit::ONE);
  assert_eq!(x.get_net(0), Bit::ONE);
}

#[test]
fn test_signal_net_toggle_rising() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);

  x.set_net(0, Bit::ONE);

  assert_eq!(x.get_toggles_total(0), 1);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 1);
}

#[test]
fn test_signal_net_toggle_falling() {
  let mut x = Signals::new(1);
  x.nets[0] = Bit::ONE;

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);

  x.set_net(0, Bit::ZERO);

  assert_eq!(x.get_toggles_total(0), 1);
  assert_eq!(x.get_toggles_falling(0), 1);
  assert_eq!(x.get_toggles_rising(0), 0);
}

#[test]
fn test_signal_net_toggle_same_zero() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);

  x.set_net(0, Bit::ZERO);

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);
}

#[test]
fn test_signal_net_toggle_same_one() {
  let mut x = Signals::new(1);
  x.nets[0] = Bit::ONE;

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);

  x.set_net(0, Bit::ONE);

  assert_eq!(x.get_toggles_total(0), 0);
  assert_eq!(x.get_toggles_falling(0), 0);
  assert_eq!(x.get_toggles_rising(0), 0);
}
