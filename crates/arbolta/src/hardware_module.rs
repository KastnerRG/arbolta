// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::{
  bit::{Bit, BitVec},
  cell::{CELL_DISPATCH, Cell, CellError, CellFn, create_cell},
  graph::*,
  port::PortError,
  port::{Port, PortDirection, parse_bit},
  signal::Signals,
  yosys::Netlist,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{
  collections::{HashMap, HashSet},
  fmt::Debug,
};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub enum CellType {
  Submodule(String),
  Scopeinfo(String), // TODO: Convert this to submodule
  Primitive(String),
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub struct SubModule {
  pub ports: HashMap<String, Port>,
  pub nets: HashMap<String, Box<[usize]>>,
  pub cells: HashMap<String, CellType>,
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
  pub graph: String, // TODO: Don't store this as a dot string...
  // pub topo_cells: Vec<TopoCell>,
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

#[derive(Clone, Debug, Copy)]
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
    for (cell_name, cell_info) in &synth_module.cells {
      let cell_type = cell_info.cell_type.to_owned();

      let cell_type = if CELL_DISPATCH.contains_key(&cell_type.as_str()) {
        CellType::Primitive(cell_type)
      } else if cell_type == "$scopeinfo" {
        CellType::Scopeinfo(cell_type)
      } else {
        CellType::Submodule(cell_type)
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

    // let graph = build_graph(&top_module_parent, &topo_cells, &global_nets, netlist)?;
    let graph = build_graph(&topo_cells)?;
    let topo_order = get_topo_cell_order(&topo_cells);

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
      graph,
      clock_net: None,
      reset_net: None,
    })
  }

  pub fn get_nets(&self, net_name: &str, parents: Option<&[String]>) -> Option<Box<[usize]>> {
    let parents = parents.unwrap_or(std::slice::from_ref(&self.top_module));
    let submodule = &self.submodules[parents];
    submodule.nets.get(net_name).cloned()
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
  pub fn eval(&mut self) {
    loop {
      self.signals.clear_dirty();
      self
        .cells
        .iter_mut()
        .for_each(|cell| cell.eval(&mut self.signals));

      if !self.signals.is_dirty() {
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

  // TODO: Add tests for these
  pub fn get_toggles(&self, category: ToggleCount) -> u64 {
    let toggle_fn = match category {
      ToggleCount::Falling => Signals::get_toggles_falling,
      ToggleCount::Rising => Signals::get_toggles_rising,
      ToggleCount::Total => Signals::get_toggles_total,
    };

    (0..self.signals.size).fold(0, |acc, i| acc + toggle_fn(&self.signals, i))
  }

  // Returns all child_nets
  fn find_nets_helper(
    &self,
    parents: Option<&[String]>,
    filter_inputs: Option<bool>,
    global_nets: &mut HashMap<Vec<String>, HashSet<usize>>,
  ) -> Result<HashSet<usize>, ModuleError> {
    // First time calling, parse top module
    let parents = parents.unwrap_or(std::slice::from_ref(&self.top_module));

    let Some(submodule) = self.submodules.get(parents) else {
      return Err(ModuleError::MissingSubmodule(parents.join(".")));
    };

    // Collect all nets in submodule
    let mut submodule_nets =
      HashSet::<usize>::from_iter(submodule.nets.values().flatten().copied());

    // First time calling, don't filter input nets
    if filter_inputs.unwrap_or(false) {
      // Collect all input nets
      let mut input_nets = HashSet::<usize>::new();
      for (port_name, port_info) in &submodule.ports {
        if port_info.direction == PortDirection::Input
          && let Some(nets) = submodule.nets.get(port_name)
        {
          input_nets.extend(nets);
        }
      }

      submodule_nets = submodule_nets.difference(&input_nets).copied().collect();
    }

    // Parse children, collect all nets
    let mut total_nets = HashSet::<usize>::new();
    for (cell_name, cell_type) in &submodule.cells {
      if let CellType::Submodule(_) = cell_type {
        // Submodule child
        let child_parents = [parents, &[cell_name.to_owned()]].concat();
        let child_nets = self.find_nets_helper(Some(&child_parents), Some(true), global_nets)?;

        total_nets.extend(child_nets);
      }
    }
    // Filter out child nets
    submodule_nets = submodule_nets.difference(&total_nets).copied().collect();

    global_nets.insert(parents.to_vec(), submodule_nets.clone());
    total_nets.extend(submodule_nets); // Add current submodule nets to total

    Ok(total_nets)
  }

  pub fn get_submodule_nets(&self) -> Result<HashMap<Vec<String>, HashSet<usize>>, ModuleError> {
    let mut submodule_nets = HashMap::new();
    self.find_nets_helper(None, None, &mut submodule_nets)?;
    Ok(submodule_nets)
  }

  pub fn get_submodule_toggles(
    &self,
    category: ToggleCount,
  ) -> Result<HashMap<Vec<String>, HashMap<usize, u64>>, ModuleError> {
    let mut total_toggles = HashMap::new();
    let submodule_nets = self.get_submodule_nets()?;

    let toggle_fn = match category {
      ToggleCount::Falling => Signals::get_toggles_falling,
      ToggleCount::Rising => Signals::get_toggles_rising,
      ToggleCount::Total => Signals::get_toggles_total,
    };

    for (submodule_name, nets) in submodule_nets {
      let toggles = nets
        .into_iter()
        .map(|n| (n, toggle_fn(&self.signals, n)))
        .collect::<Vec<(usize, u64)>>();

      total_toggles.insert(submodule_name, HashMap::from_iter(toggles));
    }

    Ok(total_toggles)
  }

  pub fn get_submodule_net_map(&self) -> HashMap<Vec<String>, HashMap<String, Vec<usize>>> {
    HashMap::from_iter(self.submodules.iter().map(|(sub_name, sub)| {
      (
        sub_name.to_vec(),
        HashMap::from_iter(
          sub
            .nets
            .iter()
            .map(|(net_name, nets)| (net_name.to_owned(), nets.to_vec())),
        ),
      )
    }))
  }

  pub fn get_all_signal_nets_reverse(&self) -> HashMap<usize, Vec<String>> {
    let signal_net_map = self.get_all_signal_nets();

    let mut reverse_map = HashMap::<usize, Vec<String>>::new();
    for (net_name, nets) in &signal_net_map {
      if nets.len() > 1 {
        for (i, &n) in nets.iter().enumerate() {
          let entry = reverse_map.entry(n).or_default();
          entry.push(format!("{net_name}[{i}]"));
        }
      } else {
        let entry = reverse_map.entry(nets[0]).or_default();
        entry.push(net_name.to_owned())
      }
    }

    reverse_map
  }

  // fn find_cells_helper(
  //   &self,
  //   parents: Option<&[String]>,
  //   global_cells: &mut HashMap<Vec<String>, HashSet<String>>,
  // ) -> Result<HashSet<String>, ModuleError> {
  //   let parents = parents.unwrap_or(std::slice::from_ref(&self.top_module));

  //   let Some(submodule) = self.submodules.get(parents) else {
  //     return Err(ModuleError::MissingSubmodule(parents.join(".")));
  //   };

  //   // let submodule_cells = vec![]
  //   for (cell_name, cell_type) in &submodule.cells {
  //     if let CellType::Primitive(cell_type) = cell_type {
  //       todo!()
  //     }
  //   }

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
