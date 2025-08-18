// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::{
  bit::{Bit, BitVec},
  cell::{Cell, CellError, CellFn, create_cell},
  port::PortError,
  port::{Port, PortDirection, parse_bit},
  signal::Signals,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};
use thiserror::Error;
use yosys_netlist_json as yosys_json;

pub type PortMap = HashMap<String, Port>;
pub type SignalMap = HashMap<String, Box<[usize]>>;
pub type SubmoduleMap = HashMap<String, Vec<String>>; // TODO: Change to box?

#[derive(Default, Clone, Debug, Deserialize, Serialize, Decode, Encode)]
pub struct HardwareModule {
  pub name: String,
  pub ports: PortMap,
  pub signal_map: SignalMap,
  pub signals: Signals,
  pub cell_info: HashMap<String, String>,
  pub cells: Box<[Cell]>,
  pub submodules: SubmoduleMap,        // (name, netnames)
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

impl HardwareModule {
  // TODO: Fix
  #[allow(unused)]
  pub fn load(path: &str) -> anyhow::Result<Self> {
    todo!()
  }

  #[allow(unused)]
  pub fn save(&self, path: &str) -> anyhow::Result<()> {
    todo!()
  }

  pub fn new(
    netlist: &yosys_json::Netlist,
    topo_order: &HashMap<String, Vec<String>>,
    top_module: &str,
  ) -> Result<Self, ModuleError> {
    // Check that top module exists
    let Some(synth_module) = netlist.modules.get(top_module) else {
      return Err(ModuleError::TopModule(top_module.to_string()));
    };

    let mut cells = vec![];
    let mut cell_info: HashMap<String, String> = HashMap::new();
    let Some(module_torder) = topo_order.get(top_module) else {
      return Err(ModuleError::TopoOrder(top_module.to_string()));
    };

    for cell_name in module_torder.iter().rev() {
      // $scopeinfo not in torder

      let Some(synth_cell) = synth_module.cells.get(cell_name) else {
        return Err(CellError::NotFound(cell_name.to_string()).into());
      };

      let cell = create_cell(synth_cell)?;
      cells.push(cell);
      cell_info.insert(cell_name.to_string(), synth_cell.cell_type.to_string());
    }

    let mut signal_map = SignalMap::new();
    let mut max_signal = 0;
    for (name, netname) in &synth_module.netnames {
      let mut nets = vec![];
      for bit in &netname.bits {
        let net = parse_bit(bit)?;
        max_signal = max_signal.max(net);
        nets.push(net);
      }
      signal_map.insert(name.to_string(), nets.into_boxed_slice());
    }

    let mut ports = PortMap::new();
    for (name, port) in &synth_module.ports {
      ports.insert(name.clone(), Port::new(port)?);
    }
    // Temp?
    let mut submodules = SubmoduleMap::default();
    for (name, netname) in &synth_module.netnames {
      if ports.contains_key(name) {
        continue;
      }

      let port = yosys_json::Port {
        direction: yosys_json::PortDirection::Output,
        bits: netname.bits.clone(),
        offset: Default::default(),
        upto: Default::default(),
        signed: Default::default(),
      };

      ports.insert(name.clone(), Port::new(&port)?);

      // Check if net belongs to scopeinfo cell
      let split = name.split(".").collect::<Vec<&str>>();
      if let Some(&scope_name) = split.first()
        && let Some(cell) = synth_module.cells.get(scope_name)
        && cell.cell_type == "$scopeinfo"
      {
        submodules
          .entry(scope_name.to_string())
          .or_default()
          .push(name.to_string());
      }
    }

    let mut signals = Signals::new(max_signal + 1);
    signals.set_constant(0, Bit::ZERO);
    signals.set_constant(1, Bit::ONE);

    Ok(Self {
      name: top_module.to_string(),
      ports,
      signal_map,
      signals,
      cell_info,
      cells: cells.into_boxed_slice(),
      submodules,
      clock_net: None,
      reset_net: None,
    })
  }

  pub fn get_signal_nets(&self, name: &str) -> Result<Box<[usize]>, ModuleError> {
    match self.signal_map.get(name) {
      Some(net) => Ok(net.clone()),
      None => Err(ModuleError::MissingSignal(name.to_string())),
    }
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

      let after = self.signals.nets.clone();

      if before == after {
        break;
      }
    }
  }

  pub fn eval_clocked(&mut self, cycles: Option<u32>) -> Result<(), ModuleError> {
    let Some((clock_net, polarity)) = self.clock_net else {
      return Err(ModuleError::MissingSignal("clock".to_string()));
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
      return Err(ModuleError::MissingSignal("reset".to_string()));
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
    let Some(port) = self.ports.get(name) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    let mut bits: BitVec = port
      .nets
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
    let Some(port) = self.ports.get(name) else {
      return Err(ModuleError::MissingPort(name.to_string()));
    };

    port
      .nets
      .iter()
      .zip(vals.into_iter().map(Into::into))
      .for_each(|(net, b)| self.signals.set_net(*net, b));

    Ok(())
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

  pub fn get_submodule_toggles(
    &self,
    name: &str,
    category: ToggleCount,
  ) -> Result<u64, ModuleError> {
    let Some(net_names) = self.submodules.get(name) else {
      return Err(ModuleError::MissingSubmodule(name.to_string()));
    };

    let mut total: u64 = 0;
    for name in net_names {
      self.ports[name].nets.iter().for_each(|&n| {
        total += match category {
          ToggleCount::Falling => self.signals.get_toggles_falling(n),
          ToggleCount::Rising => self.signals.get_toggles_rising(n),
          ToggleCount::Total => self.signals.get_total_toggles(n),
        }
      });
    }

    Ok(total)
  }

  // #[allow(unused_variables)]
  // pub fn search_module_total_toggle_count(&self, name: &str) -> Result<usize, ModuleError> {
  //   todo!()
  // }
}
