use crate::bit::{Bit, BitVec};
use crate::signal::Signal;
use bincode::{Decode, Encode};
use derive_more::Constructor;
use enum_dispatch::enum_dispatch;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use yosys_netlist_json as yosys;

#[enum_dispatch]
pub trait CellFn {
  fn eval(&mut self, signals: &mut [Signal]);
  fn reset(&mut self);
}

#[enum_dispatch(CellFn)]
#[derive(Debug, Serialize, Deserialize, Clone, Encode, Decode)]
/// Proxy for a standard-cell and basic unit of 'compute'.
pub enum Cell {
  Inverter,
  Buf,
  Nand,
  And,
  AndNot,
  Or,
  Nor,
  Xor,
  Xnor,
  OrNot,
  Dff,
  DffReset,
  Reg,
  Mux,
  Add,
  Sub,
  Mul,
  Pos,
  Neg,
  Shl,
  LogicAnd,
  LogicNot,
}

#[derive(Debug, Error)]
pub enum CellError {
  #[error("unsupported cell `{0}`")]
  Unsupported(String),
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
    "BUF" | "$_BUF_" => Cell::Buf(Buf::new(
      input_connections["A"][0],
      output_connections["Y"][0],
    )),
    "NOT" | "$_NOT_" | "$not" => Cell::Inverter(Inverter::new(
      input_connections["A"][0],
      output_connections["Y"][0],
    )),
    "$_NAND_" | "NAND" => Cell::Nand(Nand::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "OR" | "$_OR_" | "$reduce_or" => Cell::Or(Or::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "ORNOT" | "$_ORNOT_" => Cell::OrNot(OrNot::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "$_NOR_" | "NOR" => Cell::Nor(Nor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "XOR" | "$_XOR_" => Cell::Xor(Xor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "$_XNOR_" | "XNOR" => Cell::Xnor(Xnor::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "AND" | "$_AND_" => Cell::And(And::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "ANDNOT" | "$_ANDNOT_" => Cell::AndNot(AndNot::new(
      input_connections
        .into_values()
        .flatten()
        .collect::<Vec<usize>>()
        .into_boxed_slice(),
      output_connections["Y"][0],
    )),
    "DFF" | "$_DFF_P_" => Cell::Dff(Dff::new(
      Bit::One,
      input_connections["C"][0],
      input_connections["D"][0],
      output_connections["Q"][0],
    )),
    "$_SDFF_PP0_" => Cell::DffReset(DffReset::new(
      Bit::One,
      input_connections["D"][0],
      input_connections["C"][0],
      input_connections["R"][0],
      output_connections["Q"][0],
    )),
    "$pos" => Cell::Pos(Pos::new(
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$neg" => Cell::Neg(Neg::new(
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    // Proc Cells
    "$mux" | "$_MUX_" => Cell::Mux(Mux::new(
      input_connections["S"][0],
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$dff" => Cell::Reg(Reg::new(
      Bit::One,
      input_connections["CLK"][0],
      input_connections["D"].clone().into_boxed_slice(),
      output_connections["Q"].clone().into_boxed_slice(),
    )),
    "$add" => Cell::Add(Add::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$sub" => Cell::Sub(Sub::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$mul" => Cell::Mul(Mul::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap()
        == cell.parameters["B_SIGNED"].to_number().unwrap(),
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$shl" => Cell::Shl(Shl::new(
      // Hacky, fix later
      cell.parameters["A_SIGNED"].to_number().unwrap() == 1,
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$logic_and" => Cell::LogicAnd(LogicAnd::new(
      // Hacky, fix later
      input_connections["A"].clone().into_boxed_slice(),
      input_connections["B"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    "$logic_not" => Cell::LogicNot(LogicNot::new(
      // Hacky, fix later
      input_connections["A"].clone().into_boxed_slice(),
      output_connections["Y"].clone().into_boxed_slice(),
    )),
    _ => return Err(CellError::Unsupported(cell.cell_type.to_string())),
  };

  Ok(new_cell)
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
pub struct Inverter {
  input_net: usize,
  output_net: usize,
}

impl CellFn for Inverter {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(!signals[self.input_net].get_value());
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
pub struct Buf {
  input_net: usize,
  output_net: usize,
}

impl CellFn for Buf {
  fn eval(&mut self, signals: &mut [Signal]) {
    signals[self.output_net].set_value(signals[self.input_net].get_value());
  }

  fn reset(&mut self) {}
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct DffReset {
  polarity: Bit,
  data_in_net: usize,
  clock_net: usize,
  reset_net: usize,
  data_out_net: usize,
  last_clock: Bit,
}

impl DffReset {
  pub fn new(
    polarity: Bit,
    data_in_net: usize,
    clock_net: usize,
    reset_net: usize,
    data_out_net: usize,
  ) -> Self {
    Self {
      polarity,
      data_in_net,
      clock_net,
      reset_net,
      data_out_net,
      last_clock: Bit::Zero,
    }
  }
}

impl CellFn for DffReset {
  fn eval(&mut self, signals: &mut [Signal]) {
    let clock = !(signals[self.clock_net].get_value() ^ self.polarity);

    if clock == Bit::One && self.last_clock == Bit::Zero {
      if signals[self.reset_net].get_value() == Bit::One {
        // Reset
        signals[self.data_out_net].set_value(Bit::Zero);
      } else {
        signals[self.data_out_net].set_value(signals[self.data_in_net].get_value());
      }
    }

    self.last_clock = clock;
  }

  fn reset(&mut self) {
    self.last_clock = Bit::Zero;
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
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
}

// +++ Proc Cells +++
#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}

#[derive(Debug, Clone, Constructor, Serialize, Deserialize, Encode, Decode)]
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
}
