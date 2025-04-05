use crate::bit::Bit;
use crate::signal::Signal;
use derive_more::Constructor;
use indexmap::IndexMap;
use std::fmt::Debug;
use thiserror::Error;
use yosys_netlist_json as yosys;

/// Proxy for a standard-cell and basic unit of 'compute'.
pub type Cell = Box<dyn CellFn>;

pub trait CellFn: Debug + Send + Sync {
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
    "BUF" => Box::new(Buf::new(
      input_connections["A"][0],
      output_connections["Y"][0],
    )),
    "NOT" => Box::new(Inverter::new(
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
    "OR" | "$_OR_" => Box::new(Or::new(
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
    "$_SDFF_PP0_" => Box::new(DffPosedgeReset::new(
      input_connections["D"][0],
      input_connections["C"][0],
      input_connections["R"][0],
      output_connections["Q"][0],
    )),
    _ => return Err(CellError::Unsupported(cell.cell_type.to_string())),
  };

  Ok(new_cell)
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone, Constructor)]
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

#[derive(Debug, Clone)]
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
