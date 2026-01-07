mod simcells;
mod simlib;
// mod temp;
mod test_helpers;

// Re-export
use crate::signal::Signals;
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

// connections, parameters
pub type CellCtor = fn(&BTreeMap<&str, Box<[usize]>>, &BTreeMap<&str, usize>) -> Cell;
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
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Proxy for a standard-cell and basic unit of 'compute'.
pub enum Cell {
  // ASAP7
  HalfAdder,
  HalfAdderInv,
  FullAdder,
  FullAdderInv,
  AndOrReduce,
  DffInv,
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
  And3,
  AndOr,
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

pub type CellMapping = HashMap<String, (String, Option<HashMap<String, String>>)>;

pub fn create_cell(
  cell_type: &str,
  connections: &BTreeMap<&str, Box<[usize]>>,
  parameters: &BTreeMap<&str, usize>,
  mapping: Option<&CellMapping>,
) -> Result<Cell, CellError> {
  let (cell_type, mut connections) = if let Some(mapping) = mapping
    && let Some((mapped_cell_type, mapped_connections)) = mapping.get(cell_type)
  {
    let connections = match mapped_connections {
      Some(mapped_connections) => connections
        .iter()
        .map(|(&port_name, nets)| (mapped_connections[port_name].as_str(), nets.clone()))
        .collect(),
      None => connections.clone(),
    };

    (mapped_cell_type.as_str(), connections)
  } else {
    (cell_type, connections.clone())
  };

  let ctor = CELL_DISPATCH
    .get(cell_type)
    .ok_or_else(|| CellError::Unsupported(cell_type.to_string()))?;

  // Special case, buffer with no output
  if cell_type == "$_BUF_" && !connections.contains_key("Y") {
    connections.insert("Y", Box::from([0]));
  }

  Ok(ctor(&connections, parameters))
}
