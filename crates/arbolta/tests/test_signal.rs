// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::signal::Signal;

#[test]
fn test_signal_net_init() {
  let x = Signal::default();

  assert_eq!(x.get_value(), Bit::ZERO);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
  assert_eq!(x.get_total_toggle_count(), 0);
}

#[test]
fn test_signal_net_set_value() {
  let mut x = Signal::default();

  assert_eq!(x.get_value(), Bit::ZERO);
  x.set_value(Bit::ONE);
  assert_eq!(x.get_value(), Bit::ONE);
}

#[test]
fn test_signal_net_toggle_rising() {
  let mut x = Signal::default();

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::ONE);

  assert_eq!(x.get_total_toggle_count(), 1);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 1);
}

#[test]
fn test_signal_net_toggle_falling() {
  let mut x = Signal {
    value: Bit::ONE,
    ..Default::default()
  };

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::ZERO);

  assert_eq!(x.get_total_toggle_count(), 1);
  assert_eq!(x.get_toggle_count_falling(), 1);
  assert_eq!(x.get_toggle_count_rising(), 0);
}

#[test]
fn test_signal_net_toggle_same_zero() {
  let mut x = Signal::default();

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::ZERO);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
}

#[test]
fn test_signal_net_toggle_same_one() {
  let mut x = Signal {
    value: Bit::ONE,
    ..Default::default()
  };

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);

  x.set_value(Bit::ONE);

  assert_eq!(x.get_total_toggle_count(), 0);
  assert_eq!(x.get_toggle_count_falling(), 0);
  assert_eq!(x.get_toggle_count_rising(), 0);
}
