mod simcells;
mod simlib;
mod test_macros;

// Re-export
use crate::{graph::TopoCell, signal::Signals};
use bincode::{Decode, Encode};
use enum_dispatch::enum_dispatch;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
pub use simcells::*;
pub use simlib::*;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

#[enum_dispatch]
pub trait CellFn {
  fn eval(&mut self, signals: &mut Signals);
  fn reset(&mut self);
}

pub type CellCtor = fn(&BTreeMap<String, Box<[usize]>>, &BTreeMap<String, usize>) -> Cell;
pub type CellDispatchMap = HashMap<&'static str, CellCtor>;

#[derive(derive_more::Constructor)]
pub struct CellRegistration {
  pub aliases: &'static [&'static str],
  pub ctor: CellCtor,
}

inventory::collect!(CellRegistration);

pub static CELL_DISPATCH: Lazy<HashMap<&'static str, CellCtor>> = Lazy::new(|| {
  let mut cell_map = HashMap::new();
  for registration in inventory::iter::<CellRegistration> {
    for name in registration.aliases {
      cell_map.insert(*name, registration.ctor);
    }
  }
  cell_map
});

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
  ALDff,
  Mux,
  BMux,
  PMux,
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
}

// TODO: Make this take topo cell AND move into try_from
// TODO: Move matching to hashmap of function pointers
// will allow for different cell libraries... and reuse of cells in new ways
/// Generate a cell given its Yosys netlist description
/// # Arguments
/// * `cell` - Yosys cell
pub fn create_cell(cell: &TopoCell) -> Result<Cell, CellError> {
  let cell_type = cell.cell_type.as_str();
  let ctor = CELL_DISPATCH
    .get(cell_type)
    .ok_or_else(|| CellError::Unsupported(cell_type.to_string()))?;

  let Some(connections) = &cell.connections else {
    todo!()
  };
  let Some(parameters) = &cell.parameters else {
    todo!()
  };

  Ok(ctor(connections, parameters))
}
