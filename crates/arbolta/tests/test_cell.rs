// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::{create_cell, Cell, CellFn, DffPosedgeReset};
use arbolta::signal::Signal;

use once_cell::sync::Lazy;
use rstest::rstest;
use yosys_netlist_json as yosys;

static VARIABLE_ALPHABET: Lazy<Vec<String>> = Lazy::new(|| {
  (b'A'..=b'Z')
    .map(|x| String::from_utf8(vec![x]).unwrap())
    .collect()
});

fn generate_cell(cell_type: &str, num_inputs: usize) -> Cell {
  let mut cell = yosys::Cell::default();
  cell.cell_type = cell_type.to_string();

  for net in 0..num_inputs {
    let name = &VARIABLE_ALPHABET[net];
    cell
      .connections
      .insert(name.to_string(), vec![yosys::BitVal::N(net)]);
    cell
      .port_directions
      .insert(name.to_string(), yosys::PortDirection::Input);
  }

  cell
    .connections
    .insert("Y".to_string(), vec![yosys::BitVal::N(num_inputs)]);
  cell
    .port_directions
    .insert("Y".to_string(), yosys::PortDirection::Output);

  create_cell(&cell).unwrap()
}

#[rstest]
#[case("NOT", Bit::Zero, Bit::One)]
#[case("NOT", Bit::One, Bit::Zero)]
#[case("BUF", Bit::Zero, Bit::Zero)]
#[case("BUF", Bit::One, Bit::One)]
fn test_cell_1_input(#[case] cell_type: &str, #[case] a: Bit, #[case] expected: Bit) {
  let mut cell = generate_cell(cell_type, 1);
  let mut signals = vec![Signal::new_constant(a), Signal::default()].into_boxed_slice();
  cell.eval(&mut signals);
  assert_eq!(signals[1].get_value(), expected)
}

#[rstest]
#[case("AND", Bit::Zero, Bit::Zero, Bit::Zero)]
#[case("AND", Bit::Zero, Bit::One, Bit::Zero)]
#[case("AND", Bit::One, Bit::Zero, Bit::Zero)]
#[case("AND", Bit::One, Bit::One, Bit::One)]
#[case("ANDNOT", Bit::Zero, Bit::Zero, Bit::Zero)]
#[case("ANDNOT", Bit::Zero, Bit::One, Bit::Zero)]
#[case("ANDNOT", Bit::One, Bit::Zero, Bit::One)]
#[case("ANDNOT", Bit::One, Bit::One, Bit::Zero)]
#[case("NOR", Bit::Zero, Bit::Zero, Bit::One)]
#[case("NOR", Bit::Zero, Bit::One, Bit::Zero)]
#[case("NOR", Bit::One, Bit::Zero, Bit::Zero)]
#[case("NOR", Bit::One, Bit::One, Bit::Zero)]
#[case("NAND", Bit::Zero, Bit::Zero, Bit::One)]
#[case("NAND", Bit::Zero, Bit::One, Bit::One)]
#[case("NAND", Bit::One, Bit::Zero, Bit::One)]
#[case("NAND", Bit::One, Bit::One, Bit::Zero)]
#[case("OR", Bit::Zero, Bit::Zero, Bit::Zero)]
#[case("OR", Bit::Zero, Bit::One, Bit::One)]
#[case("OR", Bit::One, Bit::Zero, Bit::One)]
#[case("OR", Bit::One, Bit::One, Bit::One)]
#[case("XOR", Bit::Zero, Bit::Zero, Bit::Zero)]
#[case("XOR", Bit::Zero, Bit::One, Bit::One)]
#[case("XOR", Bit::One, Bit::Zero, Bit::One)]
#[case("XOR", Bit::One, Bit::One, Bit::Zero)]
#[case("XNOR", Bit::Zero, Bit::Zero, Bit::One)]
#[case("XNOR", Bit::Zero, Bit::One, Bit::Zero)]
#[case("XNOR", Bit::One, Bit::Zero, Bit::Zero)]
#[case("XNOR", Bit::One, Bit::One, Bit::One)]
#[case("ORNOT", Bit::Zero, Bit::Zero, Bit::One)]
#[case("ORNOT", Bit::Zero, Bit::One, Bit::Zero)]
#[case("ORNOT", Bit::One, Bit::Zero, Bit::One)]
#[case("ORNOT", Bit::One, Bit::One, Bit::One)]
fn test_cell_2_input(
  #[case] cell_type: &str,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] expected: Bit,
) {
  let mut cell = generate_cell(cell_type, 2);
  let mut signals = vec![
    Signal::new_constant(a),
    Signal::new_constant(b),
    Signal::default(),
  ]
  .into_boxed_slice();
  cell.eval(&mut signals);

  assert_eq!(signals[2].get_value(), expected);
}

#[rstest]
#[case("OR", Bit::Zero, Bit::Zero, Bit::Zero, Bit::Zero)]
#[case("OR", Bit::Zero, Bit::Zero, Bit::One, Bit::One)]
#[case("OR", Bit::Zero, Bit::One, Bit::Zero, Bit::One)]
#[case("OR", Bit::Zero, Bit::One, Bit::One, Bit::One)]
#[case("OR", Bit::One, Bit::Zero, Bit::Zero, Bit::One)]
#[case("OR", Bit::One, Bit::Zero, Bit::One, Bit::One)]
#[case("OR", Bit::One, Bit::One, Bit::Zero, Bit::One)]
#[case("OR", Bit::One, Bit::One, Bit::One, Bit::One)]
fn test_cell_3_input(
  #[case] cell_type: &str,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] c: Bit,
  #[case] expected: Bit,
) {
  let mut cell = generate_cell(cell_type, 3);
  let mut signals = vec![
    Signal::new_constant(a),
    Signal::new_constant(b),
    Signal::new_constant(c),
    Signal::default(),
  ]
  .into_boxed_slice();
  cell.eval(&mut signals);

  assert_eq!(signals[3].get_value(), expected);
}

#[rstest]
fn test_cell_sdff_pp() {
  // D, C, R, Q
  let (data_in, clock, reset, data_out) = (0, 1, 2, 3);
  let mut cell = DffPosedgeReset::new(data_in, clock, reset, data_out);
  let mut signals = vec![Signal::default(); 4].into_boxed_slice();

  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::Zero);

  signals[data_in].set_value(Bit::One);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::Zero);

  signals[clock].set_value(Bit::One); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::One);

  signals[clock].set_value(Bit::Zero); // Falling edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::One);

  signals[reset].set_value(Bit::One);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::One);

  signals[clock].set_value(Bit::One); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::Zero);
}
