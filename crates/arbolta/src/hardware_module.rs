// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use super::port::{Port, PortDirection};
use crate::bit::{Bit, BitVec};
use crate::cell::{Cell, CellError, CellFn, create_cell};
use crate::signal::Signal;
use anyhow::Result;
use bincode::{Decode, Encode};
use ndarray::{Array1, ArrayView1};
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{Read, Write};
use std::process::Command;
use tempfile::NamedTempFile;
use thiserror::Error;
use yosys_netlist_json as yosys;

pub type PortMap = HashMap<String, Port>;
pub type SignalMap = HashMap<String, Box<[usize]>>;

#[derive(Default, Clone, Debug, Deserialize, Serialize, Encode, Decode)]
pub struct HardwareModule {
  pub name: String,
  pub ports: PortMap,
  pub signal_map: SignalMap,
  pub signals: Box<[Signal]>,
  pub cell_info: HashMap<String, String>,
  pub cells: Box<[Cell]>,
  pub clock_net: Option<(usize, Bit)>, // TODO: Add polarity
  pub reset_net: Option<(usize, Bit)>, // TODO: Add polarity
}

#[derive(Debug, Error)]
pub enum ModuleError {
  #[error("couldn't load netlist `{0}`")]
  Netlist(String),
  #[error("module does not have port `{0}`")]
  MissingPort(String),
  #[error("module does not have signal `{0}`")]
  MissingSignal(String),
  #[error("module does not have net `{0}`")]
  MissingNet(usize),
}

impl HardwareModule {
  pub fn load(path: &str) -> Result<Self> {
    let serialized = std::fs::read(path)?;
    let reader = flexbuffers::Reader::get_root(serialized.as_slice())?;
    Ok(Self::deserialize(reader)?)
  }

  pub fn save(&self, path: &str) -> Result<()> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    self.serialize(&mut serializer)?;
    let mut file_output = std::fs::File::create(path)?;
    _ = file_output.write(serializer.view())?;
    Ok(())
  }

  pub fn new_from_str(raw_netlist: &str, top_module: &str) -> Result<Self> {
    let mut temp_netlist = NamedTempFile::new()?;

    // TODO: Better error handling
    let _ = temp_netlist.write(raw_netlist.as_bytes())?;

    Self::new_from_path(temp_netlist.path().to_str().unwrap(), top_module)
  }

  pub fn new_from_path(netlist_path: &str, top_module: &str) -> Result<Self> {
    let temp_flattened = NamedTempFile::new()?;
    let mut temp_torder = NamedTempFile::new()?;

    _ = Command::new("yosys")
      .arg("-f")
      .arg("json")
      .arg(netlist_path)
      .arg("-p")
      .arg(format!(
        "flatten; write_json {}; tee -o {} torder",
        temp_flattened.path().display(),
        temp_torder.path().display()
      ))
      .output()?;

    let netlist = yosys::Netlist::from_reader(temp_flattened)?;

    let mut topo_order = String::new();
    _ = temp_torder.read_to_string(&mut topo_order)?;

    let mut clean_topo_order = vec![];

    // Don't need these lines
    for line in topo_order.lines().skip(3) {
      let split: Vec<&str> = line.split_whitespace().collect();
      if split[0] == "cell" {
        clean_topo_order.push(split[1]);
      }
    }

    Self::new(netlist, &clean_topo_order, top_module)
  }

  pub fn new(netlist: yosys::Netlist, topo_order: &[&str], top_module: &str) -> Result<Self> {
    // Design must be flattened
    let Some(synth_module) = netlist.modules.get(top_module) else {
      return Err(CellError::Unsupported(top_module.to_string()))?;
    };

    let mut cells = vec![];
    let mut cell_info: HashMap<String, String> = HashMap::new();

    for cell_name in topo_order.iter().rev() {
      if *cell_name == "$scopeinfo" {
        // Ignore for now
        continue;
      }

      let synth_cell = synth_module.cells.get(*cell_name).unwrap();
      let cell = create_cell(synth_cell)?;

      cells.push(cell);
      cell_info.insert(cell_name.to_string(), synth_cell.cell_type.to_string());
    }

    let mut signal_map = SignalMap::new();
    let mut max_signal = 0;
    for (name, netname) in &synth_module.netnames {
      let mut nets = vec![];
      for bit in &netname.bits {
        let net = match bit {
          yosys::BitVal::N(net) => *net,
          yosys::BitVal::S(constant) => match constant {
            yosys::SpecialBit::_0 => 0, // Global 0
            yosys::SpecialBit::_1 => 1, // Global 1
            yosys::SpecialBit::X => todo!("X bit not supported."),
            yosys::SpecialBit::Z => todo!("Z bit not supported."),
          },
        };
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
    for (name, netname) in &synth_module.netnames {
      if ports.contains_key(name) {
        continue;
      }

      let port = yosys::Port {
        direction: yosys::PortDirection::Output,
        bits: netname.bits.clone(),
        offset: Default::default(),
        upto: Default::default(),
        signed: Default::default(),
      };

      ports.insert(name.clone(), Port::new(&port)?);
    }

    let mut signals = vec![Signal::default(); max_signal + 1];
    signals[0].set_constant(Bit::ZERO);
    signals[1].set_constant(Bit::ONE);

    Ok(Self {
      name: top_module.to_string(),
      ports,
      signal_map,
      signals: signals.into_boxed_slice(),
      cell_info,
      cells: cells.into_boxed_slice(),
      clock_net: None,
      reset_net: None,
    })
  }

  pub fn get_signal_nets(&self, name: &str) -> Result<Box<[usize]>> {
    match self.signal_map.get(name) {
      Some(net) => Ok(net.clone()),
      None => Err(ModuleError::MissingSignal(name.to_string()))?,
    }
  }

  pub fn stick_signal(&mut self, net: usize, value: Bit) -> Result<()> {
    let Some(signal) = self.signals.get_mut(net) else {
      return Err(ModuleError::MissingNet(net))?;
    };
    signal.set_constant(value);
    Ok(())
  }

  pub fn unstick_signal(&mut self, net: usize) -> Result<()> {
    let Some(signal) = self.signals.get_mut(net) else {
      return Err(ModuleError::MissingNet(net))?;
    };
    signal.unset_constant();
    Ok(())
  }

  pub fn set_clock(&mut self, net: usize, polarity: Bit) -> Result<()> {
    match self.signals.get(net) {
      Some(_) => {
        self.clock_net = Some((net, polarity));
        Ok(())
      }
      None => Err(ModuleError::MissingNet(net))?,
    }
  }

  pub fn set_reset(&mut self, net: usize, polarity: Bit) -> Result<()> {
    match self.signals.get(net) {
      Some(_) => {
        self.reset_net = Some((net, polarity));
        Ok(())
      }
      None => Err(ModuleError::MissingNet(net))?,
    }
  }

  // Eval until all signals have settled
  // TODO: Make this more efficient
  pub fn eval(&mut self) {
    loop {
      let before = self
        .signals
        .iter()
        .map(|s| s.get_value())
        .collect::<Vec<Bit>>();

      self
        .cells
        .iter_mut()
        .for_each(|cell| cell.eval(&mut self.signals));

      let after = self
        .signals
        .iter()
        .map(|s| s.get_value())
        .collect::<Vec<Bit>>();

      if before == after {
        break;
      }
    }
  }

  pub fn eval_clocked(&mut self, cycles: Option<u32>) -> Result<()> {
    let Some((clock_net, polarity)) = self.clock_net else {
      return Err(ModuleError::MissingSignal("clock".to_string()))?;
    };

    let cycles = cycles.unwrap_or(1);

    for _ in 0..cycles {
      self.eval();
      self.signals[clock_net].set_value(polarity);
      self.eval();
      self.signals[clock_net].set_value(!polarity);
      self.eval();
    }

    Ok(())
  }

  pub fn eval_reset_clocked(&mut self, cycles: Option<u32>) -> Result<()> {
    let Some((reset_net, polarity)) = self.reset_net else {
      return Err(ModuleError::MissingSignal("reset".to_string()))?;
    };

    self.signals[reset_net].set_value(polarity);
    self.eval_clocked(cycles)?;
    self.signals[reset_net].set_value(!polarity);
    self.eval();

    Ok(())
  }

  pub fn reset(&mut self) {
    self.cells.iter_mut().for_each(|c| c.reset());
    self.signals.iter_mut().for_each(|s| s.reset());
    self.signals[0].set_constant(Bit::ZERO);
    self.signals[1].set_constant(Bit::ONE);
  }

  pub fn set_port_shape(&mut self, name: &str, shape: &[usize; 2]) -> Result<()> {
    match self.ports.get_mut(name) {
      Some(port) => port.set_shape(shape),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_shape(&self, name: &str) -> Result<[usize; 2]> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_shape()),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_direction(&self, name: &str) -> Result<PortDirection> {
    match self.ports.get(name) {
      Some(port) => Ok(port.direction.clone()),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_bits(&self, name: &str) -> Result<BitVec> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_bits(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn set_port_bits(&mut self, name: &str, vals: &BitVec) -> Result<()> {
    match self.ports.get_mut(name) {
      Some(port) => port.set_bits(vals, &mut self.signals),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_int<T: PrimInt + std::ops::BitXorAssign>(&self, name: &str) -> Result<T> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_int(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn set_port_int<T: PrimInt + std::fmt::Display>(&mut self, name: &str, val: T) -> Result<()> {
    match self.ports.get_mut(name) {
      Some(port) => port.set_int(val, &mut self.signals),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_int_vec<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    name: &str,
  ) -> Result<Vec<T>, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_int_vec(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_int_vec<T: PrimInt>(&mut self, name: &str, vals: &[T]) -> Result<()> {
    match self.ports.get_mut(name) {
      Some(port) => port.set_int_vec(vals, &mut self.signals),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_ndarray<T: PrimInt + std::ops::BitXorAssign>(
    &self,
    name: &str,
  ) -> Result<Array1<T>, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_ndarray(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn set_port_ndarray<T: PrimInt + std::ops::BitXorAssign>(
    &mut self,
    name: &str,
    vals: ArrayView1<T>,
  ) -> Result<()> {
    match self.ports.get(name) {
      Some(port) => port.set_ndarray(vals, &mut self.signals),
      None => Err(ModuleError::MissingPort(name.to_string()))?,
    }
  }

  pub fn get_port_string(&self, name: &str) -> Result<String, ModuleError> {
    match self.ports.get(name) {
      Some(port) => Ok(port.get_string(&self.signals)),
      None => Err(ModuleError::MissingPort(name.to_string())),
    }
  }

  pub fn get_cell_breakdown(&self) -> HashMap<String, usize> {
    todo!()
  }

  #[allow(unused_variables)]
  pub fn search_module_cell_breakdown(
    &self,
    name: &str,
  ) -> Result<HashMap<String, usize>, ModuleError> {
    todo!()
  }

  // TODO: Add tests for these

  pub fn get_total_toggle_count(&self) -> usize {
    todo!()
  }

  #[allow(unused_variables)]
  pub fn search_module_total_toggle_count(&self, name: &str) -> Result<usize, ModuleError> {
    todo!()
  }

  #[allow(unused_variables)]
  pub fn get_module_bit_flips(&self, name: &str) -> usize {
    todo!()
  }
}
