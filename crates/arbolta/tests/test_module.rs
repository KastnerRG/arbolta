// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::hardware_module::HardwareModule;
use arbolta::signal::Signal;

use once_cell::sync::Lazy;
use rstest::rstest;
use yosys_netlist_json as yosys;

static VARIABLE_ALPHABET: Lazy<Vec<String>> = Lazy::new(|| {
  (b'A'..=b'Z')
    .map(|x| String::from_utf8(vec![x]).unwrap())
    .collect()
});

fn generate_module(cell_type: &str, num_inputs: usize) -> HardwareModule {
  let mut module = yosys::Module::default();
  let mut cell = yosys::Cell::default();
  cell.cell_type = cell_type.to_string();

  let net_offset = 2; // Offset constants
  for net in 0..num_inputs {
    let name = &VARIABLE_ALPHABET[net];
    let bitval = yosys::BitVal::N(net + net_offset);
    let mut netname = yosys::Netname::default();
    netname.bits.push(bitval);

    let port = yosys::Port {
      direction: yosys::PortDirection::Input,
      bits: vec![bitval],
      offset: Default::default(),
      upto: Default::default(),
      signed: Default::default(),
    };

    cell.connections.insert(name.to_string(), port.bits.clone());
    cell
      .port_directions
      .insert(name.to_string(), port.direction);
    module.ports.insert(name.to_string(), port);
    module.netnames.insert(name.to_string(), netname);
  }

  let bitval = yosys::BitVal::N(num_inputs + net_offset);
  let mut netname = yosys::Netname::default();
  netname.bits.push(bitval);

  let port = yosys::Port {
    direction: yosys::PortDirection::Output,
    bits: vec![bitval],
    offset: Default::default(),
    upto: Default::default(),
    signed: Default::default(),
  };

  cell.connections.insert("Y".to_string(), port.bits.clone());
  cell.port_directions.insert("Y".to_string(), port.direction);
  module.ports.insert("Y".to_string(), port);
  module.netnames.insert("Y".to_string(), netname);

  module.cells.insert(cell_type.to_string(), cell);

  let mut netlist = yosys::Netlist::default();
  netlist.modules.insert(cell_type.to_string(), module);

  HardwareModule::new(netlist, cell_type).unwrap()
}

#[rstest]
#[case("NOT", 0, 1)]
#[case("NOT", 1, 0)]
#[case("BUF", 0, 0)]
#[case("BUF", 1, 1)]
fn test_module_1_input_cell(#[case] cell_type: &str, #[case] a: u8, #[case] expected: u8) {
  let mut cell_module = generate_module(cell_type, 1);
  cell_module.set_port_int("A", a).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("AND", 0, 0, 0)]
#[case("AND", 0, 1, 0)]
#[case("AND", 1, 0, 0)]
#[case("AND", 1, 1, 1)]
#[case("NOR", 0, 0, 1)]
#[case("NOR", 0, 1, 0)]
#[case("NOR", 1, 0, 0)]
#[case("NOR", 1, 1, 0)]
#[case("NAND", 0, 0, 1)]
#[case("NAND", 0, 1, 1)]
#[case("NAND", 1, 0, 1)]
#[case("NAND", 1, 1, 0)]
#[case("OR", 0, 0, 0)]
#[case("OR", 0, 1, 1)]
#[case("OR", 1, 0, 1)]
#[case("OR", 1, 1, 1)]
#[case("XOR", 0, 0, 0)]
#[case("XOR", 0, 1, 1)]
#[case("XOR", 1, 0, 1)]
#[case("XOR", 1, 1, 0)]
#[case("XNOR", 0, 0, 1)]
#[case("XNOR", 0, 1, 0)]
#[case("XNOR", 1, 0, 0)]
#[case("XNOR", 1, 1, 1)]
fn test_module_2_input_cell(
  #[case] cell_type: &str,
  #[case] a: u8,
  #[case] b: u8,
  #[case] expected: u8,
) {
  let mut cell_module = generate_module(cell_type, 2);
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

#[rstest]
#[case("OR", 0, 0, 0, 0)]
#[case("OR", 0, 0, 1, 1)]
#[case("OR", 0, 1, 0, 1)]
#[case("OR", 0, 1, 1, 1)]
#[case("OR", 1, 0, 0, 1)]
#[case("OR", 1, 0, 1, 1)]
#[case("OR", 1, 1, 0, 1)]
#[case("OR", 1, 1, 1, 1)]
fn test_module_3_input_cell(
  #[case] cell_type: &str,
  #[case] a: u8,
  #[case] b: u8,
  #[case] c: u8,
  #[case] expected: u8,
) {
  let mut cell_module = generate_module(cell_type, 3);
  cell_module.set_port_int("A", a).unwrap();
  cell_module.set_port_int("B", b).unwrap();
  cell_module.set_port_int("C", c).unwrap();
  cell_module.eval();

  let actual: u8 = cell_module.get_port_int("Y").unwrap();
  assert_eq!(actual, expected);
}

// #[rstest]
// #[case(Function::DffPosEdge, 0, 0)]
// #[case(Function::DffPosEdge, 1, 1)]
// fn test_module_1_input_cell_clocked(
//   #[case] function: Function,
//   #[case] a: u8,
//   #[case] expected: u8,
// ) {
//   let mut cell_module = cell_module_from_function(function, 2);

//   cell_module.set_port_int("a", 0).unwrap(); // clock
//   cell_module.set_port_int("b", a).unwrap();
//   cell_module.eval();

//   cell_module.set_port_int("a", 1).unwrap();
//   cell_module.eval();

//   cell_module.set_port_int("a", 0).unwrap();
//   cell_module.eval();

//   let actual: u8 = cell_module.get_port_int("Y").unwrap();
//   assert_eq!(actual, expected);
// }
