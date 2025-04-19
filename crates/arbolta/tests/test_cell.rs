// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::{create_cell, Cell, CellFn, Dff, DffReset};
use arbolta::signal::Signal;

use rstest::rstest;
use std::collections::HashMap;
use yosys_netlist_json as yosys;

fn generate_cell(
  cell_type: &str,
  inputs: HashMap<&str, usize>,
  outputs: HashMap<&str, usize>,
  parameters: Option<HashMap<&str, &str>>,
) -> Cell {
  let mut cell = yosys::Cell::default();
  cell.cell_type = cell_type.to_string();

  let mut num_nets = 0;
  for (name, size) in inputs.iter() {
    let bits: Vec<yosys::BitVal> = (0..*size).map(|i| yosys::BitVal::N(num_nets + i)).collect();
    num_nets += size;
    cell.connections.insert(name.to_string(), bits);
    cell
      .port_directions
      .insert(name.to_string(), yosys::PortDirection::Input);
  }

  for (name, size) in outputs.iter() {
    let bits: Vec<yosys::BitVal> = (0..*size).map(|i| yosys::BitVal::N(num_nets + i)).collect();
    num_nets += size;
    cell.connections.insert(name.to_string(), bits);
    cell
      .port_directions
      .insert(name.to_string(), yosys::PortDirection::Output);
  }

  if let Some(parameters) = parameters {
    for (param, val) in parameters.iter() {
      cell
        .parameters
        .insert(param.to_string(), yosys::AttributeVal::S(val.to_string()));
    }
  }

  create_cell(&cell).unwrap()
}

#[rstest]
#[case("NOT", Bit::ZERO, Bit::ONE)]
#[case("NOT", Bit::ONE, Bit::ZERO)]
#[case("BUF", Bit::ZERO, Bit::ZERO)]
#[case("BUF", Bit::ONE, Bit::ONE)]
fn test_cell_1_input(#[case] cell_type: &str, #[case] a: Bit, #[case] expected: Bit) {
  let inputs = HashMap::from([("A", 1)]);
  let outputs = HashMap::from([("Y", 1)]);
  let mut cell = generate_cell(cell_type, inputs, outputs, None);
  let mut signals = vec![Signal::new_constant(a), Signal::default()].into_boxed_slice();
  cell.eval(&mut signals);
  assert_eq!(signals[1].get_value(), expected)
}

#[rstest]
#[case("AND", Bit::ZERO, Bit::ZERO, Bit::ZERO)]
#[case("AND", Bit::ZERO, Bit::ONE, Bit::ZERO)]
#[case("AND", Bit::ONE, Bit::ZERO, Bit::ZERO)]
#[case("AND", Bit::ONE, Bit::ONE, Bit::ONE)]
#[case("ANDNOT", Bit::ZERO, Bit::ZERO, Bit::ZERO)]
#[case("ANDNOT", Bit::ZERO, Bit::ONE, Bit::ZERO)]
#[case("ANDNOT", Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("ANDNOT", Bit::ONE, Bit::ONE, Bit::ZERO)]
#[case("NOR", Bit::ZERO, Bit::ZERO, Bit::ONE)]
#[case("NOR", Bit::ZERO, Bit::ONE, Bit::ZERO)]
#[case("NOR", Bit::ONE, Bit::ZERO, Bit::ZERO)]
#[case("NOR", Bit::ONE, Bit::ONE, Bit::ZERO)]
#[case("NAND", Bit::ZERO, Bit::ZERO, Bit::ONE)]
#[case("NAND", Bit::ZERO, Bit::ONE, Bit::ONE)]
#[case("NAND", Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("NAND", Bit::ONE, Bit::ONE, Bit::ZERO)]
#[case("OR", Bit::ZERO, Bit::ZERO, Bit::ZERO)]
#[case("OR", Bit::ZERO, Bit::ONE, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ONE, Bit::ONE)]
#[case("XOR", Bit::ZERO, Bit::ZERO, Bit::ZERO)]
#[case("XOR", Bit::ZERO, Bit::ONE, Bit::ONE)]
#[case("XOR", Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("XOR", Bit::ONE, Bit::ONE, Bit::ZERO)]
#[case("XNOR", Bit::ZERO, Bit::ZERO, Bit::ONE)]
#[case("XNOR", Bit::ZERO, Bit::ONE, Bit::ZERO)]
#[case("XNOR", Bit::ONE, Bit::ZERO, Bit::ZERO)]
#[case("XNOR", Bit::ONE, Bit::ONE, Bit::ONE)]
#[case("ORNOT", Bit::ZERO, Bit::ZERO, Bit::ONE)]
#[case("ORNOT", Bit::ZERO, Bit::ONE, Bit::ZERO)]
#[case("ORNOT", Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("ORNOT", Bit::ONE, Bit::ONE, Bit::ONE)]
fn test_cell_2_input(
  #[case] cell_type: &str,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] expected: Bit,
) {
  let inputs = HashMap::from([("A", 1), ("B", 1)]);
  let outputs = HashMap::from([("Y", 1)]);
  let mut cell = generate_cell(cell_type, inputs, outputs, None);
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
#[case("OR", Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ZERO)]
#[case("OR", Bit::ZERO, Bit::ZERO, Bit::ONE, Bit::ONE)]
#[case("OR", Bit::ZERO, Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("OR", Bit::ZERO, Bit::ONE, Bit::ONE, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ZERO, Bit::ZERO, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ONE, Bit::ZERO, Bit::ONE)]
#[case("OR", Bit::ONE, Bit::ONE, Bit::ONE, Bit::ONE)]
fn test_cell_3_input(
  #[case] cell_type: &str,
  #[case] a: Bit,
  #[case] b: Bit,
  #[case] c: Bit,
  #[case] expected: Bit,
) {
  let inputs = HashMap::from([("A", 1), ("B", 1), ("C", 1)]);
  let outputs = HashMap::from([("Y", 1)]);
  let mut cell = generate_cell(cell_type, inputs, outputs, None);
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
fn test_cell_dff_p() {
  // D, C, Q
  let (data_in, clock, data_out) = (0, 1, 2);
  let mut cell = Dff::new(Bit::ONE, clock, data_in, data_out);
  let mut signals = vec![Signal::default(); 3].into_boxed_slice();

  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[data_in].set_value(Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ZERO); // Falling edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[data_in].set_value(Bit::ZERO);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);
}

#[rstest]
fn test_cell_sdff_pp() {
  // D, C, R, Q
  let (data_in, clock, reset, data_out) = (0, 1, 2, 3);
  let mut cell = DffReset::new(Bit::ONE, data_in, clock, reset, data_out);
  let mut signals = vec![Signal::default(); 4].into_boxed_slice();

  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[data_in].set_value(Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ZERO); // Falling edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[reset].set_value(Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);
}
