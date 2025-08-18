// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::{
  bit::BitVec,
  hardware_module::HardwareModule,
  yosys::{SynthConfig, YosysClient},
};
use once_cell::sync::Lazy;
use rstest::rstest;
use std::{collections::HashMap, path::PathBuf};
use yosys_netlist_json::Netlist;

const CELL_WRAPPER_NETLIST: &str = include_str!("deps/simcells_wrappers.json");

static CELL_WRAPPER: Lazy<Netlist> =
  Lazy::new(|| Netlist::from_slice(CELL_WRAPPER_NETLIST.as_bytes()).unwrap());

#[rstest]
#[case::buffer("$_BUF_", [
    (0, 0),
    (1, 1),
])]
#[case::inverter("$_NOT_", [ // (A, Y)
    (0, 1),
    (1, 0),
])]
fn test_module_unary_cell(#[case] cell: &str, #[case] cases: [(u8, u8); 2]) {
  let mut module = HardwareModule::new(
    &CELL_WRAPPER,
    &HashMap::from([(cell.to_string(), vec![cell.to_string()])]),
    cell,
  )
  .unwrap();

  for (a, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "input: `{a}`");
  }
}

#[rstest]
#[case("$_AND_", [
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case("$_NOR_", [
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 0),
])]
#[case("$_NAND_", [
    (0, 0, 1),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case("$_OR_", [
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case("$_XOR_", [
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case("$_XNOR_", [
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
fn test_module_binary_cell(#[case] cell: &str, #[case] cases: [(u8, u8, u8); 4]) {
  let mut module = HardwareModule::new(
    &CELL_WRAPPER,
    &HashMap::from([(cell.to_string(), vec![cell.to_string()])]),
    cell,
  )
  .unwrap();

  for (a, b, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`");
  }
}

#[rstest]
#[tokio::test]
#[case::add_signed("add_wrapper", true, vec![
    (0, 0, 0),
    (-45, 20, -25),
    (0, 0, 0),
    (-1, 1, 0)
])]
#[case::add_unsigned("add_wrapper", false, vec![
    (2, 3, 5),
    (0, 0, 0)
])]
async fn test_module_simcell_2_input_signed(
  #[case] module_name: &str,
  #[case] signed: bool,
  #[case] cases: Vec<(i32, i32, i32)>,
) {
  let client = YosysClient {
    // TODO: Fix
    yosys_server_path: "/home/alexander/research/arbolta/scopefix/target/debug/yosys_server".into(),
    ..Default::default()
  };

  let signed_param = if signed { "1" } else { "0" };

  let synth_config = SynthConfig {
    defer: true,
    defines: Some(vec!["SIMLIB_NOCHECKS".to_string()]),
    parameters: Some(HashMap::from([
      ("A_SIGNED".to_string(), signed_param.to_string()),
      ("B_SIGNED".to_string(), signed_param.to_string()),
      ("A_WIDTH".to_string(), "8".to_string()),
      ("B_WIDTH".to_string(), "8".to_string()),
      ("Y_WIDTH".to_string(), "8".to_string()),
    ])),
    ..Default::default()
  };

  let (netlist, torder) = client
    .simple_synth(
      &PathBuf::from(
        "/home/alexander/research/arbolta/scopefix/crates/arbolta/tests/deps/simlib_wrappers.sv",
      ),
      Some(module_name.to_string()),
      synth_config,
    )
    .await
    .unwrap();

  let mut module = HardwareModule::new(&netlist, &torder, module_name).unwrap();

  for (a, b, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.eval();
    let actual: i32 = module.get_port("Y").unwrap().to_int();

    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`")
  }
}

/*
#[rstest]
#[case("$reduce_or", 0, 0, 0, 0)]
#[case("$reduce_or", 0, 0, 1, 1)]
#[case("$reduce_or", 0, 1, 0, 1)]
#[case("$reduce_or", 0, 1, 1, 1)]
#[case("$reduce_or", 1, 0, 0, 1)]
#[case("$reduce_or", 1, 0, 1, 1)]
#[case("$reduce_or", 1, 1, 0, 1)]
#[case("$reduce_or", 1, 1, 1, 1)]
fn test_module_3_input_cell(
  #[case] cell_type: &str,
  #[case] a: u8,
  #[case] b: u8,
  #[case] c: u8,
  #[case] expected: u8,
) {
  let inputs = HashMap::from([("A", 3)]);
  let outputs = HashMap::from([("Y", 1)]);
  let params = HashMap::from([("A_SIGNED", "0"), ("A_WIDTH", "11"), ("Y_WIDTH", "1")]);
  let mut cell_module = generate_module(cell_type, inputs, outputs, Some(params));
  cell_module.set_port_shape("A", &[3, 1]).unwrap();
  cell_module
    .set_port("A", BitVec::from_ints([a, b, c], Some(1)))
    .unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port("Y").unwrap().to_int();
  assert_eq!(actual, expected);
}



#[rstest]
#[case(true, 2, 3)]
#[case(true, -45, 20)]
#[case(false, 0, 0)]
#[case(true, 0, 0)]
#[case(true, -1, 1)]
fn test_module_sub(#[case] signed: bool, #[case] a: i32, #[case] b: i32) {
  let signed_param = if signed { "1" } else { "0" };
  let params = HashMap::from([
    ("A_SIGNED", signed_param),
    ("B_SIGNED", signed_param),
    ("A_WIDTH", "1000"),
    ("B_WIDTH", "1000"),
    ("Y_WIDTH", "1000"),
  ]);
  let inputs = HashMap::from([("A", 8), ("B", 8)]);
  let outputs = HashMap::from([("Y", 8)]);

  let mut cell_module = generate_module("$sub", inputs, outputs, Some(params));
  cell_module
    .set_port("A", BitVec::from_int(a, None))
    .unwrap();
  cell_module
    .set_port("B", BitVec::from_int(b, None))
    .unwrap();
  cell_module.eval();

  let actual: i32 = cell_module.get_port("Y").unwrap().to_int();
  let expected = a - b;
  assert_eq!(actual, expected);
}

#[rstest]
#[case(true, 2, 3)]
#[case(true, -45, 20)]
#[case(false, 0, 0)]
#[case(true, 0, 0)]
#[case(true, -1, 1)]
fn test_module_mul(#[case] signed: bool, #[case] a: i32, #[case] b: i32) {
  let signed_param = if signed { "1" } else { "0" };
  let params = HashMap::from([
    ("A_SIGNED", signed_param),
    ("B_SIGNED", signed_param),
    ("A_WIDTH", "10000"),
    ("B_WIDTH", "10000"),
    ("Y_WIDTH", "10000"),
  ]);
  let inputs = HashMap::from([("A", 16), ("B", 16)]);
  let outputs = HashMap::from([("Y", 16)]);

  let mut cell_module = generate_module("$mul", inputs, outputs, Some(params));
  cell_module
    .set_port("A", BitVec::from_int(a, None))
    .unwrap();
  cell_module
    .set_port("B", BitVec::from_int(b, None))
    .unwrap();
  cell_module.eval();

  let actual: i32 = cell_module.get_port("Y").unwrap().to_int();
  let expected = a * b;
  assert_eq!(actual, expected);
}

#[rstest]
#[case(true, 2, 3)]
#[case(true, -45, 1)]
#[case(false, 0, 0)]
#[case(true, 0, 0)]
#[case(true, -1, 1)]
fn test_module_shl(#[case] signed: bool, #[case] a: i16, #[case] b: i16) {
  let signed_param = if signed { "1" } else { "0" };
  let params = HashMap::from([
    ("A_SIGNED", signed_param),
    ("B_SIGNED", "0"),
    ("A_WIDTH", "10000"),
    ("B_WIDTH", "10000"),
    ("Y_WIDTH", "10000"),
  ]);
  let inputs = HashMap::from([("A", 16), ("B", 16)]);
  let outputs = HashMap::from([("Y", 16)]);

  let mut cell_module = generate_module("$shl", inputs, outputs, Some(params));
  cell_module
    .set_port("A", BitVec::from_int(a, None))
    .unwrap();
  cell_module
    .set_port("B", BitVec::from_int(b, None))
    .unwrap();
  cell_module.eval();

  let actual: i16 = cell_module.get_port("Y").unwrap().to_int();
  let expected = a << b;
  assert_eq!(actual, expected);
}

#[rstest]
#[case(true, 2)]
#[case(true, -45)]
#[case(true, 0)]
#[case(true, -1)]
fn test_module_neg(#[case] signed: bool, #[case] a: i16) {
  let signed_param = if signed { "1" } else { "0" };
  let params = HashMap::from([
    ("A_SIGNED", signed_param),
    ("A_WIDTH", "10000"),
    ("Y_WIDTH", "10000"),
  ]);
  let inputs = HashMap::from([("A", 16)]);
  let outputs = HashMap::from([("Y", 16)]);

  let mut cell_module = generate_module("$neg", inputs, outputs, Some(params));
  cell_module
    .set_port("A", BitVec::from_int(a, None))
    .unwrap();
  cell_module.eval();

  let actual: i16 = cell_module.get_port("Y").unwrap().to_int();
  let expected = -a;
  assert_eq!(actual, expected);
}

#[rstest]
#[case(vec![30, -44], "1101010000011110")]
#[case(vec![-19, -42], "1101011011101101")]
#[case(vec![3, -4], "1111110000000011")]
#[case(vec![1, -1], "1111111100000001")]
fn test_module_i8_port_array(#[case] vals: Vec<i8>, #[case] expected: BitVec) {
  let params = HashMap::from([
    ("A_SIGNED", "1"),
    ("A_WIDTH", "10000"),
    ("Y_WIDTH", "10000"),
  ]);
  let inputs = HashMap::from([("A", 16)]);
  let outputs = HashMap::from([("Y", 16)]);
  // Don't actually eval $neg
  let mut cell_module = generate_module("$neg", inputs, outputs, Some(params));

  cell_module
    .set_port_shape("A", &[vals.len(), i8::BITS as usize])
    .unwrap();
  cell_module
    .set_port("A", BitVec::from_ints(vals, Some(i8::BITS as usize)))
    .unwrap();

  let actual = cell_module.get_port("A").unwrap();

  assert_eq!(actual.bits, expected.bits)
}
*/
