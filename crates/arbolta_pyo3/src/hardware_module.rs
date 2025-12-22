// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::conversion::{
  bits_to_bool_numpy, bits_to_int_numpy, bool_numpy_to_bits, int_numpy_to_bits,
};
use crate::ports::{PortConfig, Ports};
use arbolta::bit::Bit;
use arbolta::{
  hardware_module::HardwareModule,
  port::PortDirection,
  yosys::{Netlist, parse_torder},
};
use pyo3::{
  exceptions::{PyAttributeError, PyValueError},
  prelude::*,
  types::PyDict,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[pyclass(weakref, dict)]
#[derive(Deserialize, Serialize)]
pub struct HardwareDesign {
  pub module: HardwareModule,
}

impl HardwareDesign {
  fn new_base(
    netlist_path: PathBuf,
    top_module: Option<&str>,
    torder_path: PathBuf,
  ) -> anyhow::Result<Self> {
    // Read raw JSON netlist
    let raw_netlist = std::fs::read(netlist_path)?;
    let netlist = Netlist::from_slice(&raw_netlist)?;

    // Read raw torder
    let raw_torder = std::fs::read_to_string(torder_path)?;
    let torder = parse_torder(&raw_torder);

    let module = HardwareModule::new(netlist, top_module, torder)?;

    Ok(Self { module })
  }

  pub fn get_port_shape(&self, name: &str) -> anyhow::Result<[usize; 2]> {
    Ok(self.module.get_port_shape(name)?)
  }

  pub fn get_port_numpy(&self, name: &str, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
    let item_type = numpy_array.getattr("dtype")?.getattr("str")?.to_string();
    let elem_size = self.get_port_shape(name)?[1];

    let bits = self
      .module
      .get_port(name)
      .map_err(|e| PyAttributeError::new_err(format!("{e}")))?;

    match item_type.as_str() {
      "|b1" => bits_to_bool_numpy(&bits, numpy_array),
      "|u1" | "<V1" => bits_to_int_numpy::<u8>(&bits, elem_size, numpy_array),
      "<u2" => bits_to_int_numpy::<u16>(&bits, elem_size, numpy_array),
      "<u4" => bits_to_int_numpy::<u32>(&bits, elem_size, numpy_array),
      "<u8" => bits_to_int_numpy::<u64>(&bits, elem_size, numpy_array),
      "|i1" => bits_to_int_numpy::<i8>(&bits, elem_size, numpy_array),
      "<i2" => bits_to_int_numpy::<i16>(&bits, elem_size, numpy_array),
      "<i4" => bits_to_int_numpy::<i32>(&bits, elem_size, numpy_array),
      "<i8" => bits_to_int_numpy::<i64>(&bits, elem_size, numpy_array),
      // Cast f16 to u16
      "<f2" => bits_to_int_numpy::<u16>(
        &bits,
        elem_size,
        &numpy_array.call_method1("view", ("uint16",))?,
      ),
      // Cast f32 to u32
      "<f4" => bits_to_int_numpy::<u32>(
        &bits,
        elem_size,
        &numpy_array.call_method1("view", ("uint32",))?,
      ),
      _ => Err(PyValueError::new_err(format!(
        "Unsupported item type: {item_type}"
      ))),
    }
  }

  pub fn set_port_numpy(&mut self, name: &str, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
    let item_type: String = numpy_array.getattr("dtype")?.getattr("str")?.to_string();
    let shape = self.get_port_shape(name)?;
    let elem_size = shape[1];

    let bits = match item_type.as_str() {
      "|b1" => bool_numpy_to_bits(numpy_array)?,
      "|u1" => int_numpy_to_bits::<u8>(numpy_array, elem_size)?,
      "<u2" => int_numpy_to_bits::<u16>(numpy_array, elem_size)?,
      "<u4" => int_numpy_to_bits::<u32>(numpy_array, elem_size)?,
      "<u8" => int_numpy_to_bits::<u64>(numpy_array, elem_size)?,
      "|i1" => int_numpy_to_bits::<i8>(numpy_array, elem_size)?,
      "<i2" => int_numpy_to_bits::<i16>(numpy_array, elem_size)?,
      "<i4" => int_numpy_to_bits::<i32>(numpy_array, elem_size)?,
      "<i8" => int_numpy_to_bits::<i64>(numpy_array, elem_size)?,
      // Cast to raw uint8
      "<V1" => int_numpy_to_bits::<u8>(&numpy_array.call_method1("view", ("uint8",))?, elem_size)?,
      // Cast f16 to u16
      "<f2" => {
        int_numpy_to_bits::<u16>(&numpy_array.call_method1("view", ("uint16",))?, elem_size)?
      }
      // Cast f32 to u32
      "<f4" => {
        int_numpy_to_bits::<u32>(&numpy_array.call_method1("view", ("uint32",))?, elem_size)?
      }
      _ => {
        return Err(PyValueError::new_err(format!(
          "Unsupported item type: {item_type}"
        )));
      }
    };

    self
      .module
      .set_port(name, bits)
      .map_err(|e| PyAttributeError::new_err(format!("{e}")))
  }
}

#[pymethods]
impl HardwareDesign {
  #[new]
  #[pyo3(signature = (netlist_path, torder_path, config, top_module=None))]
  pub fn new(
    py: Python<'_>,
    netlist_path: PathBuf,
    torder_path: PathBuf,
    config: HashMap<String, PyRef<PortConfig>>,
    top_module: Option<&str>,
  ) -> anyhow::Result<Py<Self>> {
    let new_module = Py::new(py, Self::new_base(netlist_path, top_module, torder_path)?)?;

    // Add custom members (can't make them static for serialization)
    let temp_binding = new_module.getattr(py, "__dict__")?;
    let temp_dict = temp_binding.cast_bound::<PyDict>(py).unwrap();

    // Add ports field
    let ports = Py::new(py, Ports::new(py, config, new_module.clone_ref(py))?)?;
    temp_dict.set_item("ports", ports)?;

    Ok(new_module)
  }

  pub fn reset(&mut self) {
    self.module.reset()
  }

  pub fn eval(&mut self) {
    self.module.eval()
  }

  #[pyo3(signature = (cycles=None))]
  pub fn eval_clocked(&mut self, cycles: Option<u32>) -> anyhow::Result<()> {
    Ok(self.module.eval_clocked(cycles)?)
  }

  #[pyo3(signature = (cycles=None))]
  pub fn eval_reset_clocked(&mut self, cycles: Option<u32>) -> anyhow::Result<()> {
    Ok(self.module.eval_reset_clocked(cycles)?)
  }

  pub fn stick_signal(&mut self, net: usize, val: u8) -> anyhow::Result<()> {
    Ok(self.module.stick_signal(net, Bit::from_int(val)?)?)
  }

  pub fn unstick_signal(&mut self, net: usize) -> anyhow::Result<()> {
    Ok(self.module.unstick_signal(net)?)
  }
  // TODO

  pub fn get_module_names(&self) -> Vec<String> {
    // TODO: How to handle top_module?
    self
      .module
      .netlist
      .modules
      .iter()
      .map(|p| p.join("."))
      .collect()
  }

  pub fn is_port_input(&self, name: &str) -> anyhow::Result<bool> {
    let direction = self.module.get_port_direction(name)?;
    Ok(direction == PortDirection::Input)
  }
}
