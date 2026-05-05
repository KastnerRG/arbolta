// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use arbolta::{bit::Bit, signal::Signals};
use rstest::rstest;

const NET_OFFSET: usize = Signals::NET_CONST1 + 1;

#[test]
fn test_signal_init() {
  let x = Signals::new(1);

  assert_eq!(x.size, 3, "size = 2 (constants) + 1");
  assert!(!x.is_dirty());

  // Check constants
  assert_eq!(x.get_net(Signals::NET_CONST0), Bit::ZERO);
  assert!(x.is_constant(Signals::NET_CONST0));
  assert_eq!(x.get_toggles_falling(Signals::NET_CONST0), 0);
  assert_eq!(x.get_toggles_rising(Signals::NET_CONST0), 0);
  assert_eq!(x.get_toggles_total(Signals::NET_CONST0), 0);

  assert_eq!(x.get_net(Signals::NET_CONST1), Bit::ONE);
  assert!(x.is_constant(Signals::NET_CONST1));
  assert_eq!(x.get_toggles_falling(Signals::NET_CONST1), 0);
  assert_eq!(x.get_toggles_rising(Signals::NET_CONST1), 0);
  assert_eq!(x.get_toggles_total(Signals::NET_CONST1), 0);

  // Check new net
  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
}

#[test]
fn test_signal_toggle_rising() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);

  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);
}

#[test]
fn test_signal_reset() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);

  x.set_net(NET_OFFSET, Bit::ONE);
  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());

  x.reset();
  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_constant(NET_OFFSET));
  assert!(!x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);
}

#[test]
fn test_signal_toggle_falling() {
  let mut x = Signals::new(1);
  x.set_net(NET_OFFSET, Bit::ONE);

  assert!(x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);

  x.set_net(NET_OFFSET, Bit::ZERO);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 2);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);
}

#[test]
fn test_signal_clear_dirty() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);

  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);

  x.clear_dirty();
  assert!(!x.is_dirty());
  assert!(!x.is_constant(NET_OFFSET));
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);
}

#[test]
fn test_signal_dirty_write_same() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());

  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());

  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());
}

#[test]
fn test_signal_toggle_same_zero() {
  let mut x = Signals::new(1);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);

  x.set_net(NET_OFFSET, Bit::ZERO);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ZERO);
  assert!(!x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 0);
}

#[test]
fn test_signal_toggle_same_one() {
  let mut x = Signals::new(1);
  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(x.is_dirty());
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);

  x.clear_dirty();

  x.set_net(NET_OFFSET, Bit::ONE);

  assert_eq!(x.get_net(NET_OFFSET), Bit::ONE);
  assert!(!x.is_dirty()); // no change
  assert_eq!(x.get_toggles_total(NET_OFFSET), 1);
  assert_eq!(x.get_toggles_falling(NET_OFFSET), 0);
  assert_eq!(x.get_toggles_rising(NET_OFFSET), 1);
}

#[rstest]
#[case::net0_set0(Signals::NET_CONST0, Bit::ZERO)]
#[case::net0_set1(Signals::NET_CONST0, Bit::ONE)]
#[case::net1_set0(Signals::NET_CONST1, Bit::ZERO)]
#[case::net1_set1(Signals::NET_CONST1, Bit::ONE)]
fn test_signal_set_constant_fail(#[case] net: usize, #[case] val: Bit) {
  let mut x = Signals::new(1);

  assert!(x.set_constant(net, val).is_err());
}

#[rstest]
#[case::net0(Signals::NET_CONST0)]
#[case::net1(Signals::NET_CONST1)]
fn test_signal_unset_constant_fail(#[case] net: usize) {
  let mut x = Signals::new(1);
  assert!(x.unset_constant(net).is_err());
}

#[rstest]
#[case(NET_OFFSET, Bit::ZERO)]
#[case(NET_OFFSET, Bit::ONE)]
fn test_signal_set_constant_pass(#[case] net: usize, #[case] val: Bit) {
  let mut x = Signals::new(1);

  x.set_constant(net, val).unwrap();
  assert_eq!(x.get_net(net), val);

  // set opposite
  x.set_net(net, !val);
  assert_eq!(x.get_net(net), val);
}

#[rstest]
#[case(NET_OFFSET, Bit::ZERO)]
#[case(NET_OFFSET, Bit::ONE)]
fn test_signal_unset_constant_pass(#[case] net: usize, #[case] val: Bit) {
  let mut x = Signals::new(1);

  x.set_constant(net, val).unwrap();
  assert_eq!(x.get_net(net), val);

  // set opposite
  x.set_net(net, !val);
  assert_eq!(x.get_net(net), val);

  x.unset_constant(net).unwrap();
  x.set_net(net, !val);
  assert_eq!(x.get_net(net), !val);
}
