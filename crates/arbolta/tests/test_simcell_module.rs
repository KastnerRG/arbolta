// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::{bit::BitVec, hardware_module::HardwareModule};
use once_cell::sync::Lazy;
use rstest::rstest;
use std::collections::HashMap;
use yosys_netlist_json::Netlist;

static CELL_WRAPPER_NETLIST: Lazy<Netlist> =
  Lazy::new(|| Netlist::from_slice(include_bytes!("deps/simcells_wrappers.json")).unwrap());

#[rstest]
#[case::buffer("$_BUF__WRAPPER", [
    (0, 0),
    (1, 1),
])]
#[case::inverter("$_NOT__WRAPPER", [
    (0, 1),
    (1, 0),
])]
fn test_module_unary_cell(#[case] cell: &str, #[case] cases: [(u8, u8); 2]) {
  let cell_type = cell.strip_suffix("_WRAPPER").unwrap();
  let torder = HashMap::from([(cell, vec![cell_type])]);
  let mut module = HardwareModule::new(CELL_WRAPPER_NETLIST.clone(), Some(cell), torder).unwrap();

  for (a, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "input: `{a}`");
  }
}

#[rstest]
#[case("$_AND__WRAPPER", [
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case("$_ANDNOT__WRAPPER", [
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case("$_NOR__WRAPPER", [
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 0),
])]
#[case("$_NAND__WRAPPER", [
    (0, 0, 1),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case("$_OR__WRAPPER", [
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case("$_XOR__WRAPPER", [
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case("$_XNOR__WRAPPER", [
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
fn test_module_binary_cell(#[case] cell: &str, #[case] cases: [(u8, u8, u8); 4]) {
  let cell_type = cell.strip_suffix("_WRAPPER").unwrap();
  let torder = HashMap::from([(cell, vec![cell_type])]);
  let mut module = HardwareModule::new(CELL_WRAPPER_NETLIST.clone(), Some(cell), torder).unwrap();

  for (a, b, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`");
  }
}

#[rstest]
#[case::andorinvert("$_AOI3__WRAPPER", [
    (0, 0, 0, 1),
    (0, 0, 1, 0),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
#[case::orandinvert("$_OAI3__WRAPPER", [
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 0),
])]
fn test_module_ternary_cell(#[case] cell: &str, #[case] cases: [(u8, u8, u8, u8); 8]) {
  let cell_type = cell.strip_suffix("_WRAPPER").unwrap();
  let torder = HashMap::from([(cell, vec![cell_type])]);
  let mut module = HardwareModule::new(CELL_WRAPPER_NETLIST.clone(), Some(cell), torder).unwrap();

  for (a, b, c, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.set_port("C", BitVec::from_int(c, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`, `{c}`");
  }
}

#[rstest]
#[case::mux2("$_MUX__WRAPPER", [
    (0, 0, 0, 0),
    (0, 0, 1, 0),
    (0, 1, 0, 0),
    (0, 1, 1, 1),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 1),
])]
#[case::nmux2("$_NMUX__WRAPPER", [
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 0),
    (1, 0, 1, 1),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
fn test_module_mux_cell(#[case] cell: &str, #[case] cases: [(u8, u8, u8, u8); 8]) {
  let cell_type = cell.strip_suffix("_WRAPPER").unwrap();
  let torder = HashMap::from([(cell, vec![cell_type])]);
  let mut module = HardwareModule::new(CELL_WRAPPER_NETLIST.clone(), Some(cell), torder).unwrap();

  for (a, b, s, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.set_port("S", BitVec::from_int(s, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`, `{s}`");
  }
}
// TODO: Expand testing
