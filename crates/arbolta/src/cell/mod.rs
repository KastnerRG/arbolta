mod simcells;
mod simlib;
mod test_macros;

// Re-export
pub use simcells::*;
pub use simlib::*;

use crate::{bit::Bit, signal::Signals};
use bincode::{Decode, Encode};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use yosys_netlist_json as yosys_json;

#[enum_dispatch]
pub trait CellFn {
  fn eval(&mut self, signals: &mut Signals);
  fn reset(&mut self);
}

#[enum_dispatch(CellFn)]
#[derive(Debug, Serialize, Deserialize, Clone, Decode, Encode)]
/// Proxy for a standard-cell and basic unit of 'compute'.
pub enum Cell {
  // Sim Cells
  Buffer,
  Inverter,
  And,
  Nand,
  Or,
  Nor,
  Xor,
  Xnor,
  AndNot,
  OrNot,
  Mux2,
  NMux2,
  AndOrInvert,
  OrAndInvert,
  Dff,
  DffReset,
  // Sim Lib
  Not,
  Neg,
  Pos,
  Add,
  Sub,
  Mul,
  Div,
  Modulus,
  Le,
  Ge,
  Gt,
  Shl,
  Shr,
  Reg,
  Mux,
  BMux,
  LogicAnd,
  LogicNot,
  ReduceOr,
  ReduceAnd,
  ProcAnd,
  Eq,
  Ne,
  ProcOr,
  ProcXor,
}

#[derive(Debug, Error)]
pub enum CellError {
  #[error("Unsupported cell `{0}`")]
  Unsupported(String),
  #[error("Cell `{0}` not found in netlist")]
  NotFound(String),
  #[error("Direction `{0}` not supported")]
  Direction(String),
  #[error("Bad parameter `{0}` = `{1:?}`")]
  Parameter(String, yosys_json::AttributeVal),
}

/// Parse global net from `BitVal`.
/// Errors if bit direction is not supported.
fn parse_bit(bit: &yosys_json::BitVal) -> Result<usize, CellError> {
  match bit {
    yosys_json::BitVal::N(net) => Ok(*net),
    yosys_json::BitVal::S(constant) => match constant {
      yosys_json::SpecialBit::_0 => Ok(0), // Global 0
      yosys_json::SpecialBit::_1 => Ok(1), // Global 1
      yosys_json::SpecialBit::X => Err(CellError::Direction("X".to_string())),
      yosys_json::SpecialBit::Z => Err(CellError::Direction("Z".to_string())),
    },
  }
}

/// Generate a cell given its Yosys netlist description
/// # Arguments
/// * `cell` - Yosys cell
pub fn create_cell(cell: &yosys_json::Cell) -> Result<Cell, CellError> {
  // Port name -> net
  let mut connections: HashMap<String, Vec<usize>> = HashMap::new();
  for (port_name, port_bits) in cell.connections.iter() {
    for bit in port_bits {
      let net = parse_bit(bit)?;
      connections
        .entry(port_name.to_string())
        .or_default()
        .push(net);
    }
  }

  let mut parameters: HashMap<String, usize> = HashMap::new();
  for (param_name, param) in cell.parameters.iter() {
    let Some(param) = param.to_number() else {
      return Err(CellError::Parameter(param_name.to_string(), param.clone()));
    };
    parameters.insert(param_name.to_string(), param);
  }

  let new_cell: Cell = match cell.cell_type.as_str() {
    // Sim cells
    "BUF" | "$_BUF_" => Cell::Buffer(Buffer::new(connections["A"][0], connections["Y"][0])),
    "NOT" | "$_NOT_" => Cell::Inverter(Inverter::new(connections["A"][0], connections["Y"][0])),
    "AND" | "$_AND_" => Cell::And(And::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "NAND" | "$_NAND_" => Cell::Nand(Nand::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "OR" | "$_OR_" => Cell::Or(Or::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "NOR" | "$_NOR_" => Cell::Nor(Nor::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "XOR" | "$_XOR_" => Cell::Xor(Xor::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "XNOR" | "$_XNOR_" => Cell::Xnor(Xnor::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "ANDNOT" | "$_ANDNOT_" => Cell::AndNot(AndNot::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "ORNOT" | "$_ORNOT_" => Cell::OrNot(OrNot::new(
      connections["A"][0],
      connections["B"][0],
      connections["Y"][0],
    )),
    "$_MUX_" => Cell::Mux2(Mux2::new(
      connections["A"][0],
      connections["B"][0],
      connections["S"][0],
      connections["Y"][0],
    )),
    "$_NMUX_" => Cell::NMux2(NMux2::new(
      connections["A"][0],
      connections["B"][0],
      connections["S"][0],
      connections["Y"][0],
    )),
    "$_AOI3_" => Cell::AndOrInvert(AndOrInvert::new(
      connections["A"][0],
      connections["B"][0],
      connections["C"][0],
      connections["Y"][0],
    )),
    "$_OAI3_ " => Cell::OrAndInvert(OrAndInvert::new(
      connections["A"][0],
      connections["B"][0],
      connections["C"][0],
      connections["Y"][0],
    )),
    "DFF" | "$_DFF_P_" => Cell::Dff(Dff::new(
      Bit::ONE,
      connections["C"][0],
      connections["D"][0],
      connections["Q"][0],
    )),
    "$_SDFF_PP0_ " => Cell::DffReset(DffReset::new(
      Bit::ONE,
      Bit::ONE,
      Bit::ZERO,
      connections["C"][0],
      connections["R"][0],
      connections["D"][0],
      connections["Q"][0],
    )),
    // Sim lib
    "$not" => Cell::Not(Not::new(
      parameters["A_SIGNED"] != 0,
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$pos" => Cell::Pos(Pos::new(
      parameters["A_SIGNED"] != 0,
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$neg" => Cell::Neg(Neg::new(
      parameters["A_SIGNED"] != 0,
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$add" => Cell::Add(Add::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$sub" => Cell::Sub(Sub::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$mul" => Cell::Mul(Mul::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$div" => Cell::Div(Div::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$mod" => Cell::Modulus(Modulus::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$le" => Cell::Le(Le::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$ge" => Cell::Ge(Ge::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$gt" => Cell::Gt(Gt::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$shl" => Cell::Shl(Shl::new(
      parameters["A_SIGNED"] != 0,
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$shr" => Cell::Shr(Shr::new(
      parameters["A_SIGNED"] != 0,
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$dff" => Cell::Reg(Reg::new(
      (parameters["CLK_POLARITY"] != 0).into(),
      connections["CLK"][0],
      connections["D"].clone().into(),
      connections["Q"].clone().into(),
    )),
    "$mux" => Cell::Mux(Mux::new(
      connections["S"][0],
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$bmux" => Cell::BMux(BMux::new(
      connections["S"].clone().into(),
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$logic_and" => Cell::LogicAnd(LogicAnd::new(
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$logic_not" => Cell::LogicNot(LogicNot::new(
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$reduce_or" | "$reduce_bool" => Cell::ReduceOr(ReduceOr::new(
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$reduce_and" => Cell::ReduceAnd(ReduceAnd::new(
      connections["A"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$and" => Cell::ProcAnd(ProcAnd::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$or" => Cell::ProcOr(ProcOr::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$xor" => Cell::ProcXor(ProcXor::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$eq" => Cell::Eq(Eq::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    "$ne" => Cell::Ne(Ne::new(
      (parameters["A_SIGNED"] != 0) && (parameters["B_SIGNED"] != 0),
      connections["A"].clone().into(),
      connections["B"].clone().into(),
      connections["Y"].clone().into(),
    )),
    _ => return Err(CellError::Unsupported(cell.cell_type.to_string())),
  };

  Ok(new_cell)
}
