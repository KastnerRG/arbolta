// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

mod helpers;

use arbolta::{
  bit::{Bit, BitVec},
  hardware_module::HardwareModule,
  netlist_wrapper::NetlistWrapper,
  yosys::PortDirection,
};
use helpers::{build_netlist, int_to_attr};
use indexmap::indexmap;
use rstest::rstest;

#[rstest]
// 37738 + 4365 = 42103
#[case::unsigned_normal(false, "1001001101101010", "0001000100001101", "1010010001110111")]
// 155 + 7 = 162
#[case::unsigned_normal(false, "10011011", "111", "0000000010100010")]
// 54 + 4234 = 4288
#[case::unsigned_normal(false, "00110110", "0001000010001010", "1000011000000")]
//  37738 + 4365 = 1143
#[case::unsigned_overflow(false, "1001001101101010", "0001000100001101", "0010001110111")]
#[case(false, "00000111", "00000111", "00001110")] // 7 + 7 = 49
#[case(false, "00000111", "111", "01110")] // 7 + 7 = 49
#[case(false, "111", "111", "1110")] // 7 + 7 = 14
#[case(false, "111", "111", "10")] // 7 + 7 = 4 (overflow)
#[case(true, "00000111", "00000111", "00001110")] // 7 + 7 = 49
#[case(true, "00000111", "1001", "00000")] // 7 + -7 = 0
#[case(true, "1001", "11001", "11110010")] // -7 + -7 = -14
#[case(true, "1001", "1001", "10")] // -7 + -7 = -4 (overflow)
#[case(true, "111", "111", "10")] // 7 + 7 = 4 (overflow)
#[case(true, "1", "0", "111111111111")] // sign extend
#[case(true, "0", "1", "111111111111")] // sign extend
#[case(true, "1", "1", "111111111110")] // -1 + -1 = -2
#[case(true, "01", "1", "000000000000")] // 1 + -1 = 0
fn test_add(#[case] signed: bool, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
  let (netlist, torder) = build_netlist(
    "$add",
    "$add_wrapper",
    indexmap! {
      "A_SIGNED" => int_to_attr(signed as u32),
      "B_SIGNED" => int_to_attr(signed as u32),
      "Y_SIGNED" => int_to_attr(signed as u32),
      "A_WIDTH" => int_to_attr(a.len() as u32),
      "B_WIDTH" => int_to_attr(b.len() as u32),
      "Y_WIDTH" => int_to_attr(expected.len() as u32),
    },
    indexmap! {
      "A" => (PortDirection::Input, a.len()),
      "B" => (PortDirection::Input, b.len()),
      "Y" => (PortDirection::Output, expected.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  module.set_port("A", a).unwrap();
  module.set_port("B", b).unwrap();
  module.eval();

  let actual = module.get_port("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(Bit::ZERO, "000", "001", "000")]
#[case(Bit::ONE, "000", "001", "001")]
#[case(Bit::ZERO, "111", "000", "111")]
#[case(Bit::ONE, "111", "000", "000")]
fn test_mux(#[case] select: Bit, #[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
  let (netlist, torder) = build_netlist(
    "$mux",
    "$mux_wrapper",
    indexmap! {
      "WIDTH" => int_to_attr(a.len() as u32),
    },
    indexmap! {
      "A" => (PortDirection::Input, a.len()),
      "B" => (PortDirection::Input, b.len()),
      "S" => (PortDirection::Input, 1),
      "Y" => (PortDirection::Output, expected.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  module.set_port("S", [select]).unwrap();
  module.set_port("A", a).unwrap();
  module.set_port("B", b).unwrap();
  module.eval();

  let actual = module.get_port("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case::zero(Bit::ONE, "000000000000")] // 0
#[case::one(Bit::ONE, "000000000001")] // 1
#[case(Bit::ONE, "1101111001101100")] // 56940
#[case(Bit::ZERO, "000000000000")] // 0
#[case(Bit::ZERO, "000000000001")] // 1
#[case(Bit::ZERO, "1101111001101100")] // 56940
fn test_reg(#[case] polarity: Bit, #[case] data_in: BitVec) {
  let (netlist, torder) = build_netlist(
    "$dff",
    "$dff_wrapper",
    indexmap! {
      "CLK_POLARITY" => int_to_attr(polarity.to_int()),
      "WIDTH" => int_to_attr(data_in.len() as u32),
    },
    indexmap! {
      "CLK" => (PortDirection::Input, 1),
      "D" => (PortDirection::Input, data_in.len()),
      "Q" => (PortDirection::Output, data_in.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  let clock_net = module.get_net("CLK").unwrap()[0];
  module.set_clock(clock_net, polarity).unwrap();

  module.set_port("CLK", [!polarity]).unwrap(); // Reset
  module.eval();
  assert_eq!(module.get_port("Q").unwrap().to_int::<u64>(), 0);

  module.set_port("D", &data_in).unwrap();
  assert_eq!(module.get_port("Q").unwrap().to_int::<u64>(), 0);

  module.eval_clocked(Some(1)).unwrap();
  assert_eq!(module.get_port("Q").unwrap(), data_in);

  module.set_port("D", BitVec::from_int(0, None)).unwrap(); // Zero
  assert_eq!(module.get_port("Q").unwrap(), data_in);

  module.eval_clocked(Some(1)).unwrap();
  assert_eq!(module.get_port("Q").unwrap().to_int::<u64>(), 0);
}

#[rstest]
#[case("0", "0", "0")]
#[case("0", "1", "0")]
#[case("1", "0", "0")]
#[case("1", "1", "1")]
#[case("00000000", "0000000", "0000")]
#[case("10000000", "0000001", "0001")]
#[case("1", "0000001", "01")]
#[case("1111111", "01", "01")]
#[case("1010100", "0000", "00000")]
fn test_logic_and(#[case] a: BitVec, #[case] b: BitVec, #[case] expected: BitVec) {
  let (netlist, torder) = build_netlist(
    "$logic_and",
    "$logic_and_wrapper",
    indexmap! {
      "A_SIGNED" => int_to_attr(false as u32),
      "A_WIDTH" => int_to_attr(a.len() as u32),
      "B_SIGNED" => int_to_attr(false as u32),
      "B_WIDTH" => int_to_attr(b.len() as u32),
      "Y_WIDTH" => int_to_attr(expected.len() as u32),
    },
    indexmap! {
      "A" => (PortDirection::Input, a.len()),
      "B" => (PortDirection::Input, b.len()),
      "Y" => (PortDirection::Output, expected.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  module.set_port("A", a).unwrap();
  module.set_port("B", b).unwrap();
  module.eval();

  let actual = module.get_port("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0", "1")]
#[case("1", "0")]
#[case("00000000", "0000001")]
#[case("10000000", "0000000")]
#[case("1", "0000000")]
#[case("1111111", "00")]
#[case("1010100", "00")]
fn test_logic_not(#[case] a: BitVec, #[case] expected: BitVec) {
  let (netlist, torder) = build_netlist(
    "$logic_not",
    "$logic_not_wrapper",
    indexmap! {
      "A_SIGNED" => int_to_attr(false as u32),
      "A_WIDTH" => int_to_attr(a.len() as u32),
      "Y_WIDTH" => int_to_attr(expected.len() as u32),
    },
    indexmap! {
      "A" => (PortDirection::Input, a.len()),
      "Y" => (PortDirection::Output, expected.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  module.set_port("A", a).unwrap();
  module.eval();

  let actual = module.get_port("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("0", "0")]
#[case("00000000", "0000000")]
#[case("10000000", "0000001")]
#[case("1", "0000001")]
#[case("1111111", "01")]
#[case("1010100", "01")]
fn test_reduce_or(#[case] a: BitVec, #[case] expected: BitVec) {
  let (netlist, torder) = build_netlist(
    "$reduce_or",
    "$reduce_or_wrapper",
    indexmap! {
      "A_SIGNED" => int_to_attr(false as u32),
      "A_WIDTH" => int_to_attr(a.len() as u32),
      "Y_WIDTH" => int_to_attr(expected.len() as u32),
    },
    indexmap! {
      "A" => (PortDirection::Input, a.len()),
      "Y" => (PortDirection::Output, expected.len()),
    },
  );

  let netlist_wrapper = NetlistWrapper::new(None, netlist, torder, None).unwrap();
  let mut module = HardwareModule::new(netlist_wrapper, None).unwrap();

  module.set_port("A", a).unwrap();
  module.eval();

  let actual = module.get_port("Y").unwrap();
  assert_eq!(actual, expected);
}
