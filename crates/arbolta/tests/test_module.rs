// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::{bit::BitVec, hardware_module::HardwareModule};
use once_cell::sync::Lazy;
use rstest::rstest;
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
  let mut module = HardwareModule::new(cell, &CELL_WRAPPER_NETLIST).unwrap();

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
  let mut module = HardwareModule::new(cell, &CELL_WRAPPER_NETLIST).unwrap();

  for (a, b, expected) in cases {
    module.set_port("A", BitVec::from_int(a, None)).unwrap();
    module.set_port("B", BitVec::from_int(b, None)).unwrap();
    module.eval();
    let actual: u8 = module.get_port("Y").unwrap().to_int();
    assert_eq!(actual, expected, "inputs: `{a}`, `{b}`");
  }
}

// TODO: Expand testing
