// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::hardware_module::HardwareModule;

use rstest::rstest;
use std::collections::HashMap;
use yosys_netlist_json as yosys;

fn generate_module(
  cell_type: &str,
  inputs: HashMap<&str, usize>,
  outputs: HashMap<&str, usize>,
  parameters: Option<HashMap<&str, &str>>,
) -> HardwareModule {
  let mut module = yosys::Module::default();
  let mut cell = yosys::Cell::default();
  cell.cell_type = cell_type.to_string();

  let mut num_nets = 2;
  for (name, size) in inputs.iter() {
    let bits: Vec<yosys::BitVal> = (0..*size).map(|i| yosys::BitVal::N(num_nets + i)).collect();
    num_nets += size;

    let mut netname = yosys::Netname::default();
    netname.bits = bits.clone();

    let port = yosys::Port {
      direction: yosys::PortDirection::Input,
      bits: bits.clone(),
      offset: Default::default(),
      upto: Default::default(),
      signed: Default::default(),
    };

    cell.connections.insert(name.to_string(), bits);
    cell
      .port_directions
      .insert(name.to_string(), port.direction);
    module.ports.insert(name.to_string(), port);
    module.netnames.insert(name.to_string(), netname);
  }

  for (name, size) in outputs.iter() {
    let bits: Vec<yosys::BitVal> = (0..*size).map(|i| yosys::BitVal::N(num_nets + i)).collect();
    num_nets += size;

    let mut netname = yosys::Netname::default();
    netname.bits = bits.clone();

    let port = yosys::Port {
      direction: yosys::PortDirection::Output,
      bits: bits.clone(),
      offset: Default::default(),
      upto: Default::default(),
      signed: Default::default(),
    };

    cell.connections.insert(name.to_string(), bits);
    cell
      .port_directions
      .insert(name.to_string(), port.direction);
    module.ports.insert(name.to_string(), port);
    module.netnames.insert(name.to_string(), netname);
  }

  if let Some(parameters) = parameters {
    for (param, val) in parameters.iter() {
      cell
        .parameters
        .insert(param.to_string(), yosys::AttributeVal::S(val.to_string()));
    }
  }
  module.cells.insert(cell_type.to_string(), cell);

  let mut netlist = yosys::Netlist::default();
  netlist.modules.insert("test".to_string(), module);
  let raw_netlist = netlist.to_string().unwrap();

  HardwareModule::new_from_str(&raw_netlist, "test").unwrap()
}

#[rstest]
#[case("$_NOT_", 0, 1)]
#[case("$_NOT_", 1, 0)]
#[case("$_BUF_", 0, 0)]
#[case("$_BUF_", 1, 1)]
fn test_module_1_input_cell(#[case] cell_type: &str, #[case] a: u8, #[case] expected: u8) {
  let inputs = HashMap::from([("A", 1)]);
  let outputs = HashMap::from([("Y", 1)]);
  let mut cell_module = generate_module(cell_type, inputs, outputs, None);
  cell_module.set_port_int("A", a).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("$_AND_", 0, 0, 0)]
#[case("$_AND_", 0, 1, 0)]
#[case("$_AND_", 1, 0, 0)]
#[case("$_AND_", 1, 1, 1)]
#[case("$_NOR_", 0, 0, 1)]
#[case("$_NOR_", 0, 1, 0)]
#[case("$_NOR_", 1, 0, 0)]
#[case("$_NOR_", 1, 1, 0)]
#[case("$_NAND_", 0, 0, 1)]
#[case("$_NAND_", 0, 1, 1)]
#[case("$_NAND_", 1, 0, 1)]
#[case("$_NAND_", 1, 1, 0)]
#[case("$_OR_", 0, 0, 0)]
#[case("$_OR_", 0, 1, 1)]
#[case("$_OR_", 1, 0, 1)]
#[case("$_OR_", 1, 1, 1)]
#[case("$_XOR_", 0, 0, 0)]
#[case("$_XOR_", 0, 1, 1)]
#[case("$_XOR_", 1, 0, 1)]
#[case("$_XOR_", 1, 1, 0)]
#[case("$_XNOR_", 0, 0, 1)]
#[case("$_XNOR_", 0, 1, 0)]
#[case("$_XNOR_", 1, 0, 0)]
#[case("$_XNOR_", 1, 1, 1)]
fn test_module_2_input_cell(
  #[case] cell_type: &str,
  #[case] a: u8,
  #[case] b: u8,
  #[case] expected: u8,
) {
  let inputs = HashMap::from([("A", 1), ("B", 1)]);
  let outputs = HashMap::from([("Y", 1)]);
  let mut cell_module = generate_module(cell_type, inputs, outputs, None);
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

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
  cell_module.set_port_int_vec("A", &[a, b, c]).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case(false, 2, 3)]
#[case(true, -45, 20)]
#[case(false, 0, 0)]
#[case(true, 0, 0)]
#[case(true, -1, 1)]
fn test_module_add(#[case] signed: bool, #[case] a: i32, #[case] b: i32) {
  let signed_param = if signed { "1" } else { "0" };
  let inputs = HashMap::from([("A", 8), ("B", 8)]);
  let outputs = HashMap::from([("Y", 8)]);
  let params = HashMap::from([
    ("A_SIGNED", signed_param),
    ("B_SIGNED", signed_param),
    ("A_WIDTH", "1000"),
    ("B_WIDTH", "1000"),
    ("Y_WIDTH", "1000"),
  ]);

  let mut cell_module = generate_module("$add", inputs, outputs, Some(params));
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: i32 = cell_module.get_port_int("Y").unwrap();
  let expected = a + b;
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
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: i32 = cell_module.get_port_int("Y").unwrap();
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
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: i32 = cell_module.get_port_int("Y").unwrap();
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
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: i16 = cell_module.get_port_int("Y").unwrap();
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
  cell_module.set_port_int("A", a).unwrap();
  cell_module.eval();

  let actual: i16 = cell_module.get_port_int("Y").unwrap();
  let expected = -a;
  assert_eq!(actual, expected);
}
