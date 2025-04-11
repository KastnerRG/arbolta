use crate::bit::{Bit, BitVec};
use crate::signal::Signal;
use derive_more::Constructor;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use yosys_netlist_json as yosys;

/// Proxy for a standard-cell and basic unit of 'compute'.
pub type Cell = Box<dyn CellFn>;

pub trait CellFn: Debug + Send + Sync + erased_serde::Serialize {
  fn eval(&mut self, signals: &mut [Signal]);
  fn reset(&mut self);
  fn input_connections(&self) -> Vec<&usize>;
  fn output_connections(&self) -> Vec<&usize>;
  fn clone_box(&self) -> Cell;
}

#[derive(Debug, Error)]
pub enum CellError {
  #[error("unsupported cell `{0}`")]
  Unsupported(String),
}

impl Default for Cell {
  fn default() -> Self {
    Box::new(NoneCell)
  }
}

impl Clone for Cell {
  fn clone(&self) -> Self {
    self.clone_box()
  }
}

/// Generate a cell given its Yosys netlist description
/// # Arguments
/// * `cell` - Yosys cell
pub fn create_cell(cell: &yosys::Cell) -> Result<Cell, CellError> {
  let mut input_connections: IndexMap<String, Vec<usize>> = IndexMap::new();
  let mut output_connections: IndexMap<String, Vec<usize>> = IndexMap::new();

  for (port_name, port_bits) in cell.connections.iter() {
    // Skip ports with no direction
    let Some(direction) = cell.port_directions.get(port_name) else {
      continue;
    };

    if *direction == yosys::PortDirection::InOut {
      todo!("Inout not supported.")
    };

    for bit in port_bits {
      let net = match bit {
        yosys::BitVal::N(net) => *net,
        yosys::BitVal::S(constant) => match constant {
          yosys::SpecialBit::_0 => 0, // Global 0
          yosys::SpecialBit::_1 => 1, // Global 1
          yosys::SpecialBit::X => todo!("X not supported."),
          yosys::SpecialBit::Z => todo!("Z not supported."),
        },
      };

      if *direction == yosys::PortDirection::Input {
        input_connections
          .entry(port_name.to_string())
          .or_default()
          .push(net);
      } else {
        output_connections
          .entry(port_name.to_string())
          .or_default()
          .push(net);
      }
    }
  }

  let new_cell: Cell = match cell.cell_type.as_str() {
    "BUF" | "$_BUF_" => Box::new(Buf::new(
      input_connections["A"][0],
      output_connections["Y"][0],
    )),
    "NOT" | "$_NOT_" | "$not" => Box::new(Inverter::new(
      input_connections["A"][0],
      output_connections["Y"][0],
    )),
    "$_NAND_" | "NAND" => Box::new(Nand::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "OR" | "$_OR_" | "$reduce_or" => Box::new(Or::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "ORNOT" | "$_ORNOT_" => Box::new(OrNot::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "$_NOR_" | "NOR" => Box::new(Nor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "XOR" | "$_XOR_" => Box::new(Xor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "$_XNOR_" | "XNOR" => Box::new(Xnor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "AND" | "$_AND_" => Box::new(And::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "ANDNOT" | "$_ANDNOT_" => Box::new(AndNot::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "DFF" | "$_DFF_P_" => Box::new(Dff::new(
      Bit::One,
      input_connections["C"][0],
      input_connections["D"][0],
      output_connections["Q"][0],
    )),
    "$_SDFF_PP0_" => Box::new(DffPosedgeReset::new(
      input_connections["D"][0],
      input_connections["C"][0],
      input_connections["R"][0],
      output_connections["Q"][0],
    )),
    "$pos" => Box::new(Pos::new(
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$neg" => Box::new(Neg::new(
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    // Proc Cells
    "$mux" | "$_MUX_" => Box::new(Mux::new(
      input_connections["S"][0],
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$dff" => Box::new(Reg::new(
      Bit::One,
      input_connections["CLK"][0],
      input_connections["D"].clone().into_boxed_slice(),
      output_connections["Q"].clone().into_boxed_slice(),
    )),
    "$add" => Box::new(Add::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$sub" => Box::new(Sub::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$mul" => Box::new(Mul::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap()
        == cell.parameters["B_SIGNED"].to_number().unwrap(),
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$shl" => Box::new(Shl::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$logic_and" => Box::new(LogicAnd::new(
      // Hacky, fix later
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$logic_not" => Box::new(LogicNot::new(
      // Hacky, fix later
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    _ => return Err(CellError::Unsupported(cell.cell_type.to_string())),
  };

  Ok(new_cell)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoneCell;

#[allow(unused_variables)]
impl CellFn for NoneCell {
  fn eval(&mut self, signals: &mut [Signal]) {} // Do nothing

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    vec![]
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Inverter {
  input_net: usize,
  output_net: usize,
}

impl CellFn for Inverter {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(!signals[self.input_net].get_value());
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    vec![&self.input_net]
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Buf {
  input_net: usize,
  output_net: usize,
}

impl CellFn for Buf {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(signals[self.input_net].get_value());
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    vec![&self.input_net]
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Nand {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for Nand {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      !self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc & signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct And {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for And {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc & signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct AndNot {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for AndNot {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(self.input_nets.iter().rev().skip(1).fold(
      !signals[*self.input_nets.last().unwrap()].get_value(),
      |acc, net| acc & signals[*net].get_value(),
    ));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Or {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for Or {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc | signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Nor {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for Nor {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      !self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc | signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Xor {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for Xor {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc ^ signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Xnor {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for Xnor {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(
      !self
        .input_nets
        .iter()
        .skip(1)
        .fold(signals[self.input_nets[0]].get_value(), |acc, net| {
          acc ^ signals[*net].get_value()
        }),
    );
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct OrNot {
  input_nets: Box<[usize]>,
  output_net: usize,
}

impl CellFn for OrNot {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(self.input_nets.iter().rev().skip(1).fold(
      !signals[*self.input_nets.last().unwrap()].get_value(),
      |acc, net| acc | signals[*net].get_value(),
    ));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.input_nets.as_ref().iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.output_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DffPosedgeReset {
  data_in_net: usize,
  clock_net: usize,
  reset_net: usize,
  data_out_net: usize,
  last_clock: Bit,
  last_data: Bit,
}

impl DffPosedgeReset {
  pub fn new(data_in_net: usize, clock_net: usize, reset_net: usize, data_out_net: usize) -> Self {
    Self {
      data_in_net,
      clock_net,
      reset_net,
      data_out_net,
      last_clock: Bit::Zero,
      last_data: Bit::Zero,
    }
  }
}

impl CellFn for DffPosedgeReset {
  fn eval(&mut self, signals: &mut [Signal]) {
    let (data, clock, reset) = (
      signals[self.data_in_net].get_value(),
      signals[self.clock_net].get_value(),
      signals[self.reset_net].get_value(),
    );

    // Detect rising edge
    let output_bit = if clock == Bit::One && self.last_clock == Bit::Zero {
      match reset {
        Bit::Zero => data,
        Bit::One => Bit::Zero, // Do reset
      }
    } else {
      self.last_data
    };

    signals[self.data_out_net].set_value(output_bit);
    (self.last_data, self.last_clock) = (output_bit, clock);
  }

  fn reset(&mut self) {
    self.last_clock = Bit::Zero;
    self.last_data = Bit::Zero;
  }

  fn input_connections(&self) -> Vec<&usize> {
    vec![&self.clock_net, &self.data_in_net, &self.reset_net]
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.data_out_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dff {
  polarity: Bit,
  data_in_net: usize,
  clock_net: usize,
  data_out_net: usize,
  last_clock: Bit,
}

impl Dff {
  pub fn new(polarity: Bit, clock_net: usize, data_in_net: usize, data_out_net: usize) -> Self {
    Self {
      polarity,
      data_in_net,
      clock_net,
      data_out_net,
      last_clock: Bit::Zero,
    }
  }
}

impl CellFn for Dff {
  fn eval(&mut self, signals: &mut [Signal]) {
    let clock = !(signals[self.clock_net].get_value() ^ self.polarity);

    if clock == Bit::One && self.last_clock == Bit::Zero {
      signals[self.data_out_net].set_value(signals[self.data_in_net].get_value());
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::Zero;
  }

  fn input_connections(&self) -> Vec<&usize> {
    vec![&self.clock_net, &self.data_in_net]
  }

  fn output_connections(&self) -> Vec<&usize> {
    vec![&self.data_out_net]
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reg {
  polarity: Bit,
  data_in_nets: Box<[usize]>,
  clock_net: usize,
  data_out_nets: Box<[usize]>,
  last_clock: Bit,
}

impl Reg {
  pub fn new(
    polarity: Bit,
    clock_net: usize,
    data_in_nets: Box<[usize]>,
    data_out_nets: Box<[usize]>,
  ) -> Self {
    Self {
      polarity,
      data_in_nets,
      clock_net,
      data_out_nets,
      last_clock: Bit::Zero,
    }
  }
}

impl CellFn for Reg {
  fn eval(&mut self, signals: &mut [Signal]) {
    let clock = !(signals[self.clock_net].get_value() ^ self.polarity);

    if clock == Bit::One && self.last_clock == Bit::Zero {
      self
        .data_out_nets
        .iter()
        .zip(self.data_in_nets.iter())
        .for_each(|(out_net, in_net)| signals[*out_net].set_value(signals[*in_net].get_value()));
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::Zero;
  }

  fn input_connections(&self) -> Vec<&usize> {
    self.data_in_nets.iter().chain([&self.clock_net]).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.data_out_nets.as_ref().iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

// +++ Proc Cells +++
#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Mux {
  select_net: usize,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Mux {
  fn eval(&mut self, signals: &mut [Signal]) {
    if signals[self.select_net].get_value() == Bit::One {
      // Select B
      self
        .y_nets
        .iter()
        .enumerate()
        .for_each(|(i, net)| signals[*net].set_value(signals[self.b_nets[i]].get_value()));
    } else {
      // Select A
      self
        .y_nets
        .iter()
        .enumerate()
        .for_each(|(i, net)| signals[*net].set_value(signals[self.a_nets[i]].get_value()));
    }
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self
      .a_nets
      .iter()
      .chain(self.b_nets.iter().chain([&self.select_net]))
      .collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.as_ref().iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Add {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Add {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let b = BitVec::from(
      self
        .b_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(a.to_int::<i64>() + b.to_int::<i64>(), self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(a.to_int::<u64>() + b.to_int::<u64>(), self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().chain(self.b_nets.iter()).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.as_ref().iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Sub {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Sub {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let b = BitVec::from(
      self
        .b_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(a.to_int::<i64>() - b.to_int::<i64>(), self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(a.to_int::<u64>() - b.to_int::<u64>(), self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().chain(self.b_nets.iter()).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.as_ref().iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Mul {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Mul {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let b = BitVec::from(
      self
        .b_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(a.to_int::<i64>() * b.to_int::<i64>(), self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(a.to_int::<u64>() * b.to_int::<u64>(), self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().chain(self.b_nets.iter()).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.as_ref().iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Pos {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Pos {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(a.to_int::<i64>(), self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(a.to_int::<u64>(), self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Neg {
  signed: bool,
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Neg {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(-a.to_int::<i64>(), self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(!(a.to_int::<u64>()) + 1, self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct Shl {
  signed: bool,
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for Shl {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let b = BitVec::from(
      self
        .b_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let shift = b.to_int::<u64>();
    // Hard code as i64 add, fix later
    let y = if self.signed {
      BitVec::from_int_sized(a.to_int::<i64>() << shift, self.y_nets.len()).unwrap()
    } else {
      BitVec::from_int_sized(a.to_int::<u64>() << shift, self.y_nets.len()).unwrap()
    };

    self
      .y_nets
      .iter()
      .enumerate()
      .for_each(|(i, net)| signals[*net].set_value(y.bits[i]));
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().chain(self.b_nets.iter()).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct LogicAnd {
  a_nets: Box<[usize]>,
  b_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for LogicAnd {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    let b = BitVec::from(
      self
        .b_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as u64 add, fix later
    // Assume only need to set LSB
    let a = Bit::from(a.to_int::<u64>() != 0);
    let b = Bit::from(b.to_int::<u64>() != 0);
    signals[self.y_nets[0]].set_value(a & b);
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().chain(self.b_nets.iter()).collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize)]
pub struct LogicNot {
  a_nets: Box<[usize]>,
  y_nets: Box<[usize]>,
}

impl CellFn for LogicNot {
  fn eval(&mut self, signals: &mut [Signal]) {
    let a = BitVec::from(
      self
        .a_nets
        .iter()
        .map(|net| signals[*net].get_value())
        .collect::<Vec<Bit>>(),
    );

    // Hard code as u64 add, fix later
    // Assume only need to set LSB
    let a = Bit::from(a.to_int::<u64>() != 0);
    signals[self.y_nets[0]].set_value(!a);
  }

  fn reset(&mut self) {}

  fn input_connections(&self) -> Vec<&usize> {
    self.a_nets.iter().collect()
  }

  fn output_connections(&self) -> Vec<&usize> {
    self.y_nets.iter().collect()
  }

  fn clone_box(&self) -> Cell {
    Box::new(self.clone())
  }
}
