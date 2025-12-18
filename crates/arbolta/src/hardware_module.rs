// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::{
  bit::{Bit, BitVec},
  cell::{Cell, CellError, CellFn},
  netlist_wrapper::NetlistWrapper,
  port::{Port, PortDirection, PortError},
  signal::Signals,
  yosys::{Netlist, TopoOrder},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;

#[derive(Default, Clone, Debug, Deserialize, Serialize)] //Decode, Encode
pub struct HardwareModule {
  pub netlist: NetlistWrapper,
  pub signals: Signals,
  pub cells: Box<[Cell]>,
  pub ports: HashMap<String, Port>,
  pub clock_net: Option<(usize, Bit)>, // (net, polarity)
  pub reset_net: Option<(usize, Bit)>, // (net, polarity)
}

// TODO: Refactor these
#[derive(Debug, Error)]
pub enum ModuleError {
  #[error("Module is not flattened or empty")]
  UnFlattened,
  #[error("Missing top module")]
  TopModule,
  #[error("Cell `{0}` doesn't exist")]
  MissingCell(String),
  #[error("No modules given")]
  MissingModule,
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

impl HardwareModule {
  pub fn new(
    netlist: Netlist,
    top_module: Option<&str>,
    torder: TopoOrder,
  ) -> Result<HardwareModule, ModuleError> {
    let netlist = NetlistWrapper::new(netlist, top_module, torder)?;

    let cells = netlist.build_cells()?;

    let global_net_max: usize = netlist
      .nets
      .values()
      .flatten()
      .fold(0, |max, &x| std::cmp::max(max, x));

    let mut signals = Signals::new(global_net_max + 1);
    signals.set_constant(0, Bit::ZERO);
    signals.set_constant(1, Bit::ONE);

    // type doesn't matter here...
    let ports = netlist.find_module_ports::<&str>(None)?;

    Ok(Self {
      netlist,
      signals,
      cells: cells.into(),
      ports,
      ..Default::default()
    })
  }

  // TODO: Fix
  pub fn get_net_bits(&self, name: &str) -> Result<BitVec, ModuleError> {
    let Some(nets) = self.get_net(name) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    let bits: BitVec = nets
      .iter()
      .map(|idx| self.signals.get_net(*idx))
      .collect::<Vec<Bit>>()
      .into();

    Ok(bits)
  }

  pub fn get_net(&self, name: &str) -> Option<&[usize]> {
    self.netlist.names_to_nets.get(name).map(|v| &**v)
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
    match self.ports.get_mut(name) {
      Some(port) => Ok(port.set_shape(shape)?),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_shape(&self, name: &str) -> Result<[usize; 2], ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_shape()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port_direction(&self, name: &str) -> Result<PortDirection, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.direction.clone()),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_port(&self, name: &str) -> Result<BitVec, ModuleError> {
    let (Some(port), Some(nets)) = (self.ports.get(name), self.get_net(name)) else {
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
    let (Some(_port), Some(nets)) = (self.ports.get(name), self.get_net(name)) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    nets
      .to_owned()
      .iter()
      .zip(vals.into_iter().map(Into::into))
      .for_each(|(net, b)| self.signals.set_net(*net, b));

    Ok(())
  }

  // Returns ALL nets in design
  pub fn get_submodule_nets(&self) -> HashMap<&Vec<String>, HashMap<&str, &[usize]>> {
    let mut global_module_nets = HashMap::new();
    for (net_id, nets) in &self.netlist.nets {
      let module_nets: &mut HashMap<&str, &[usize]> =
        global_module_nets.entry(&net_id.parents).or_default();

      module_nets.insert(&net_id.name, nets);
    }

    global_module_nets
  }

  pub fn get_submodule_net_values(&self) -> HashMap<&Vec<String>, HashMap<&str, BitVec>> {
    let global_submodule_nets = self.get_submodule_nets();
    let mut global_net_values = HashMap::new();

    for (parents, module_nets) in global_submodule_nets {
      let submodule_net_values: &mut HashMap<&str, BitVec> =
        global_net_values.entry(parents).or_default();

      for (net_name, nets) in module_nets {
        let bits: BitVec = nets
          .iter()
          .map(|idx| self.signals.get_net(*idx))
          .collect::<Vec<Bit>>()
          .into();

        submodule_net_values.insert(net_name, bits);
      }
    }

    global_net_values
  }

  pub fn get_submodule_toggles_by_net(
    &self,
    category: ToggleCount,
  ) -> HashMap<&Vec<String>, HashMap<&str, u64>> {
    let global_submodule_nets = self.get_submodule_nets();

    let toggle_fn = match category {
      ToggleCount::Falling => Signals::get_toggles_falling,
      ToggleCount::Rising => Signals::get_toggles_rising,
      ToggleCount::Total => Signals::get_toggles_total,
    };

    let mut global_submodule_toggles = HashMap::new();
    for (parents, module_nets) in global_submodule_nets {
      let submodule_toggles: &mut HashMap<&str, u64> =
        global_submodule_toggles.entry(parents).or_default();

      for (net_name, nets) in module_nets {
        let toggles = nets
          .iter()
          .fold(0, |acc, &n| acc + toggle_fn(&self.signals, n));

        submodule_toggles.insert(net_name, toggles);
      }
    }

    global_submodule_toggles
  }

  pub fn get_submodule_toggles_total(&self, category: ToggleCount) -> HashMap<&Vec<String>, u64> {
    let submodule_toggles_by_net = self.get_submodule_toggles_by_net(category);
    let mut submodule_toggles = HashMap::new();

    for (parents, module_nets) in submodule_toggles_by_net {
      let total_toggles = module_nets.values().sum();
      submodule_toggles.insert(parents, total_toggles);
    }

    submodule_toggles
  }
}
