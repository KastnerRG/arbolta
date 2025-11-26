// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::{
  bit::{Bit, BitVec},
  cell::{Cell, CellError, CellFn, create_cell},
  port::PortError,
  port::{Port, PortDirection, parse_bit},
  signal::Signals,
  temp2::*,
  yosys::Netlist,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub enum CellType {
  Submodule(String),
  Primitive(String),
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub struct SubModule {
  ports: HashMap<String, Port>,
  nets: HashMap<String, Box<[usize]>>,
  // TODO: make pointer to cells in hardware module?
  // TODO: make pointer to cells in hardware module?
  cells: HashMap<String, CellType>, // cell, cell type?
}

// Names
pub type SubmoduleMap = HashMap<Box<[String]>, SubModule>;

#[derive(Default, Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub struct HardwareModule {
  pub top_module: String,
  pub signals: Signals,
  pub cell_info: HashMap<String, String>,
  pub cells: Box<[Cell]>,
  pub submodules: SubmoduleMap,
  pub clock_net: Option<(usize, Bit)>, // (net, polarity)
  pub reset_net: Option<(usize, Bit)>, // (net, polarity)
}

#[derive(Debug, Error)]
pub enum ModuleError {
  #[error("Module is not flattened or empty")]
  UnFlattened,
  #[error("Missing top module `{0}`")]
  TopModule(String),
  #[error("Module `{0}` missing from topological order")]
  TopoOrder(String),
  #[error("Cell error `{0}`")]
  Cell(#[from] CellError),
  #[error("Port error `{0}`")]
  Port(#[from] PortError),
  #[error("No reset configured")]
  MissingReset,
  #[error("No clock configured")]
  MissingClock,
  #[error("Signal `{0}` doesn't exist")]
  MissingSignal(String),
  #[error("Net `{0}` doesn't exist")]
  MissingNet(usize),
  #[error("Port `{0}` doesn't exist")]
  MissingPort(String),
  #[error("Submodule `{0}` doesn't exist")]
  MissingSubmodule(String),
}

#[derive(Clone, Debug)]
pub enum ToggleCount {
  Rising,
  Falling,
  Total,
}
fn get_submodules(
  parents: &[TopoCellParent],
  hierarchy: &TopoHierarchy,
  global_nets: &TopoNetMap,
  netlist: &Netlist,
) -> Result<SubmoduleMap, ModuleError> {
  let name = parents
    .iter()
    .map(|p| p.name.clone())
    .collect::<Vec<String>>();
  let top_parent = parents.last().unwrap(); // Assume top is last parent...

  let mut submodules = SubmoduleMap::new();

  // Create new submodule
  let mut submodule = SubModule::default();
  if let Some(synth_module) = netlist.modules.get(&top_parent.cell_type.to_string()) {
    // TODO: Error checking?
    for (cell_name, cell_info) in &synth_module.cells {
      // TODO: Make pointer to cell in HardwareModule cells
      // TODO: Use hierarchy to tell if cell is submodule or not
      let cell_type = if netlist.modules.contains_key(&cell_info.cell_type) {
        CellType::Submodule(cell_info.cell_type.to_owned())
      } else {
        CellType::Primitive(cell_info.cell_type.to_owned())
      };
      submodule.cells.insert(cell_name.to_owned(), cell_type);
    }

    // Add nets
    for (net_name, net_info) in &synth_module.netnames {
      let nets: Vec<usize> = net_info
        .bits
        .iter()
        .map(parse_bit)
        .collect::<Result<_, _>>()?;
      // Translate
      let nets: Vec<usize> = nets.iter().map(|n| global_nets[top_parent][n]).collect();

      // Add ports
      if let Some(port_info) = synth_module.ports.get(net_name) {
        let port = Port {
          direction: PortDirection::try_from(&port_info.direction)?,
          shape: [1, nets.len()],
        };

        submodule.ports.insert(net_name.to_owned(), port);
      }

      submodule
        .nets
        .insert(net_name.to_owned(), nets.into_boxed_slice());
    }
  }

  submodules.insert(name.into(), submodule);

  if let Some(entry) = hierarchy.get(top_parent) {
    for child in entry {
      let mut parents = parents.to_vec();
      parents.push(child.clone());

      submodules.extend(get_submodules(&parents, hierarchy, global_nets, netlist)?);
    }
  }

  Ok(submodules)
}

impl HardwareModule {
  pub fn new(top_module: &str, netlist: &Netlist) -> Result<HardwareModule, ModuleError> {
    let top_module_parent = TopoCellParent::new(top_module.to_string(), top_module.to_string());
    let mut topo_cells = collect_cells(
      top_module,
      std::slice::from_ref(&top_module_parent),
      netlist,
    )?;
    let hierarchy = get_cell_hierarchy(&topo_cells);

    let mut global_nets = TopoNetMap::new();
    let mut global_net_max = 0;
    collect_nets(
      &top_module_parent,
      &hierarchy,
      netlist,
      &mut global_nets,
      &mut global_net_max,
    )?;

    update_cells(topo_cells.as_mut_slice(), &global_nets, netlist)?;

    let graph = build_graph(&top_module_parent, &topo_cells, &global_nets, netlist)?;
    let topo_order = get_topo_cell_order(&graph);

    let submodules = get_submodules(&[top_module_parent], &hierarchy, &global_nets, netlist)?;

    let cells: Vec<Cell> = topo_order
      .into_iter()
      .map(create_cell)
      .collect::<Result<Vec<_>, _>>()?;

    let mut signals = Signals::new(global_net_max + 1);
    signals.set_constant(0, Bit::ZERO);
    signals.set_constant(1, Bit::ONE);

    Ok(Self {
      top_module: top_module.to_owned(),
      signals,
      cell_info: HashMap::new(),
      cells: cells.into_boxed_slice(),
      submodules,
      clock_net: None,
      reset_net: None,
    })
  }

  pub fn get_signal_nets(&self, name: &str) -> Option<Box<[usize]>> {
    let submodule = &self.submodules[std::slice::from_ref(&self.top_module)];
    submodule.nets.get(name).cloned()
  }

  pub fn stick_signal(&mut self, net: usize, value: Bit) -> Result<(), ModuleError> {
    if net >= self.signals.size {
      Err(ModuleError::MissingNet(net))
    } else {
      self.signals.set_constant(net, value);
      Ok(())
    }
  }

  pub fn unstick_signal(&mut self, net: usize) -> Result<(), ModuleError> {
    if net >= self.signals.size {
      Err(ModuleError::MissingNet(net))
    } else {
      self.signals.unset_constant(net);
      Ok(())
    }
  }

  pub fn set_clock(&mut self, net: usize, polarity: Bit) -> Result<(), ModuleError> {
    if net >= self.signals.size {
      Err(ModuleError::MissingNet(net))
    } else {
      self.clock_net = Some((net, polarity));
      Ok(())
    }
  }

  pub fn set_reset(&mut self, net: usize, polarity: Bit) -> Result<(), ModuleError> {
    if net >= self.signals.size {
      Err(ModuleError::MissingNet(net))
    } else {
      self.reset_net = Some((net, polarity));
      Ok(())
    }
  }

  // Eval until all signals have settled
  // TODO: Make this more efficient
  pub fn eval(&mut self) {
    loop {
      let before = self.signals.nets.clone();

      self
        .cells
        .iter_mut()
        .for_each(|cell| cell.eval(&mut self.signals));

      if before == self.signals.nets {
        break;
      }
    }
  }

  pub fn eval_clocked(&mut self, cycles: Option<u32>) -> Result<(), ModuleError> {
    let Some((clock_net, polarity)) = self.clock_net else {
      return Err(ModuleError::MissingClock);
    };

    let cycles = cycles.unwrap_or(1);

    for _ in 0..cycles {
      self.eval();
      self.signals.set_net(clock_net, polarity);
      self.eval();
      self.signals.set_net(clock_net, !polarity);
      self.eval();
    }

    Ok(())
  }

  pub fn eval_reset_clocked(&mut self, cycles: Option<u32>) -> Result<(), ModuleError> {
    let Some((reset_net, polarity)) = self.reset_net else {
      return Err(ModuleError::MissingReset);
    };

    self.signals.set_net(reset_net, polarity);
    self.eval_clocked(cycles)?;
    self.signals.set_net(reset_net, !polarity);
    self.eval();

    Ok(())
  }

  pub fn reset(&mut self) {
    self.cells.iter_mut().for_each(|c| c.reset());
    self.signals.reset();
    self.signals.set_constant(0, Bit::ZERO);
    self.signals.set_constant(1, Bit::ONE);
  }

  pub fn set_port_shape(&mut self, name: &str, shape: &[usize; 2]) -> Result<(), ModuleError> {
    let submodule = self
      .submodules
      .get_mut(std::slice::from_ref(&self.top_module))
      .unwrap();

    match submodule.ports.get_mut(name) {
      Some(port) => Ok(port.set_shape(shape)?),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_shape(&self, name: &str) -> Result<[usize; 2], ModuleError> {
    let submodule = self
      .submodules
      .get(std::slice::from_ref(&self.top_module))
      .unwrap();

    match submodule.ports.get(name) {
      Some(port) => Ok(port.get_shape()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_direction(&self, name: &str) -> Result<PortDirection, ModuleError> {
    let submodule = self
      .submodules
      .get(std::slice::from_ref(&self.top_module))
      .unwrap();

    match submodule.ports.get(name) {
      Some(port) => Ok(port.direction.clone()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port(&self, name: &str) -> Result<BitVec, ModuleError> {
    let submodule = self
      .submodules
      .get(std::slice::from_ref(&self.top_module))
      .unwrap();

    let (Some(port), Some(nets)) = (submodule.ports.get(name), submodule.nets.get(name)) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    let mut bits: BitVec = nets
      .iter()
      .map(|idx| self.signals.get_net(*idx))
      .collect::<Vec<Bit>>()
      .into();
    bits.shape = port.shape;

    Ok(bits)
  }

  pub fn set_port<I, B>(&mut self, name: &str, vals: I) -> Result<(), ModuleError>
  where
    I: IntoIterator<Item = B>,
    B: Into<Bit>,
  {
    let submodule = self
      .submodules
      .get_mut(std::slice::from_ref(&self.top_module))
      .unwrap();

    let (Some(_port), Some(nets)) = (submodule.ports.get(name), submodule.nets.get(name)) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    nets
      .iter()
      .zip(vals.into_iter().map(Into::into))
      .for_each(|(net, b)| self.signals.set_net(*net, b));

    Ok(())
  }

  pub fn get_all_signal_nets(&self) -> HashMap<String, Vec<usize>> {
    let mut all_nets = HashMap::new();
    for (parents, submodule) in &self.submodules {
      let parent_name = parents.join(".");

      for (net_name, nets) in &submodule.nets {
        let name = format!("{parent_name}.{net_name}");

        all_nets.insert(name, nets.to_vec());
      }
    }

    all_nets
  }

  pub fn get_all_signal_values(&self) -> HashMap<String, BitVec> {
    let mut all_nets = HashMap::new();
    for (parents, submodule) in &self.submodules {
      let parent_name = parents.join(".");

      for (net_name, nets) in &submodule.nets {
        let name = format!("{parent_name}.{net_name}");

        let bits: BitVec = nets
          .iter()
          .map(|idx| self.signals.get_net(*idx))
          .collect::<Vec<Bit>>()
          .into();

        all_nets.insert(name, bits);
      }
    }

    all_nets
  }

  // pub fn get_cell_breakdown(&self) -> HashMap<String, usize> {
  //   todo!()
  // }

  // #[allow(unused_variables)]
  // pub fn search_module_cell_breakdown(
  //   &self,
  //   name: &str,
  // ) -> Result<HashMap<String, usize>, ModuleError> {
  //   todo!()
  // }

  // TODO: Add tests for these

  pub fn get_toggles(&self, category: ToggleCount) -> u64 {
    let mut total: u64 = 0;
    (0..self.signals.size).for_each(|i| {
      total += match category {
        ToggleCount::Falling => self.signals.get_toggles_falling(i),
        ToggleCount::Rising => self.signals.get_toggles_rising(i),
        ToggleCount::Total => self.signals.get_total_toggles(i),
      }
    });
    total
  }

  // pub fn get_submodule_toggles(
  //   &self,
  //   name: &str,
  //   category: ToggleCount,
  // ) -> Result<u64, ModuleError> {
  //   let Some(net_names) = self.submodules.get(name) else {
  //     return Err(ModuleError::MissingSubmodule(name.to_string()));
  //   };

  //   let mut total: u64 = 0;
  //   for name in net_names {
  //     self.ports[name].nets.iter().for_each(|&n| {
  //       total += match category {
  //         ToggleCount::Falling => self.signals.get_toggles_falling(n),
  //         ToggleCount::Rising => self.signals.get_toggles_rising(n),
  //         ToggleCount::Total => self.signals.get_total_toggles(n),
  //       }
  //     });
  //   }

  //   Ok(total)
  // }

  // #[allow(unused_variables)]
  // pub fn search_module_total_toggle_count(&self, name: &str) -> Result<usize, ModuleError> {
  //   todo!()
  // }
  // TODO: Fix
  #[allow(unused)]
  pub fn load(path: &str) -> anyhow::Result<Self> {
    todo!()
  }

  #[allow(unused)]
  pub fn save(&self, path: &str) -> anyhow::Result<()> {
    todo!()
  }
}
