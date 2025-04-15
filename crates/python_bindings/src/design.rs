// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::conversion::{
  bits_to_bool_numpy, bits_to_int_numpy, bool_numpy_to_bits, int_numpy_to_bits,
};

use arbol::hardware_module::HardwareModule;
use arbol::port::PortDirection;
use bincode::{Decode, Encode};
use pyo3::exceptions::{PyAttributeError, PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[pyclass(dict, module = "arbolta", name = "Design")]
#[derive(Deserialize, Serialize, Decode, Encode)]
pub struct PyDesign {
  #[pyo3(get)]
  pub top_module: String,
  pub netlist_path: String,
  module: HardwareModule,
}

#[pymethods]
impl PyDesign {
  #[new]
  fn __new__(top_module: &str, netlist_path: &str) -> PyResult<Self> {
    let module = match HardwareModule::new_from_path(netlist_path, top_module) {
      Ok(module) => module,
      Err(err) => return Err(PyException::new_err(format!("{err}"))),
    };

    Ok(Self {
      top_module: top_module.to_string(),
      netlist_path: netlist_path.to_string(),
      module,
    })
  }

  fn __setstate__(&mut self, state: &Bound<'_, PyBytes>) {
    let config = bincode::config::standard();
    (*self, _) = bincode::decode_from_slice(state.as_bytes(), config).unwrap();
  }

  fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyBytes>> {
    let config = bincode::config::standard();
    match bincode::encode_to_vec(self, config) {
      Ok(bytes) => Ok(PyBytes::new(py, &bytes)),
      Err(err) => Err(PyValueError::new_err(format!("{err}"))),
    }
  }

  fn __getnewargs__(&self) -> (String, String) {
    (self.top_module.clone(), self.netlist_path.clone())
  }

  fn save(&self, path: &str) -> PyResult<()> {
    match self.module.save(path) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyValueError::new_err(format!("{err}"))),
    }
  }

  #[staticmethod]
  fn load(path: &str) -> PyResult<Self> {
    let module = match HardwareModule::load(path) {
      Ok(module) => module,
      Err(err) => return Err(PyValueError::new_err(format!("{err}"))),
    };

    let top_module = module.name.clone();

    Ok(Self {
      top_module,
      netlist_path: String::new(), // TODO: Fix
      module,
    })
  }

  fn get_port_shape(&self, name: &str) -> PyResult<[usize; 2]> {
    match self.module.get_port_shape(name) {
      Ok(shape) => Ok(shape),
      Err(err) => Err(PyAttributeError::new_err(format!("{err}"))),
    }
  }

  fn set_port_shape(&mut self, name: &str, shape: [usize; 2]) -> PyResult<()> {
    if shape[0] != 1 {
      return Err(PyValueError::new_err(format!(
        "Only 1D shapes supported: {shape:?}"
      )));
    }

    let internal_shape = self.get_port_shape(name)?;
    let (num_elems, elem_size) = (shape[1], internal_shape[1] / shape[1]);
    match self.module.set_port_shape(name, &[num_elems, elem_size]) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyAttributeError::new_err(format!("{err}"))),
    }
  }

  fn get_module_names(&self) -> Vec<String> {
    todo!()
  }

  fn get_signal_map(&self) -> HashMap<String, Vec<usize>> {
    self
      .module
      .signal_map
      .iter()
      .map(|(name, nets)| (name.to_string(), nets.to_vec()))
      .collect()
  }

  fn get_cell_info(&self, py: Python<'_>) -> PyResult<PyObject> {
    let all_cell_info = PyDict::new(py);
    for (name, cell_type) in self.module.cell_info.iter() {
      let cell_info = PyDict::new(py);
      cell_info.set_item("type", cell_type)?;

      // TODO: Add other fields
      all_cell_info.set_item(name.clone(), cell_info)?;
    }

    Ok(all_cell_info.into())
  }

  pub fn stick_signal(&mut self, net: usize, val: bool) -> PyResult<()> {
    match self.module.stick_signal(net, val.into()) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyException::new_err(format!("{err}"))),
    }
  }

  pub fn unstick_signal(&mut self, net: usize) -> PyResult<()> {
    match self.module.unstick_signal(net) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyException::new_err(format!("{err}"))),
    }
  }

  fn set_clock(&mut self, name: &str, polarity: bool) -> PyResult<()> {
    let Some(nets) = self.module.signal_map.get(name) else {
      return Err(PyException::new_err(format!("No signal `{name}`")));
    };

    if nets.len() != 1 {
      return Err(PyException::new_err("Clock net ambiguous".to_string()));
    }

    self.module.set_clock(nets[0], polarity.into()).unwrap();

    Ok(())
  }

  fn set_reset(&mut self, name: &str, polarity: bool) -> PyResult<()> {
    let Some(nets) = self.module.signal_map.get(name) else {
      return Err(PyException::new_err(format!("No signal `{name}`")));
    };

    if nets.len() != 1 {
      return Err(PyException::new_err("Reset net ambiguous".to_string()));
    }

    self.module.set_reset(nets[0], polarity.into()).unwrap();

    Ok(())
  }

  fn reset(&mut self) {
    self.module.reset();
  }

  fn eval(&mut self) {
    self.module.eval();
  }

  // TODO: Fix this
  fn eval_clocked(&mut self, cycles: u32) -> PyResult<()> {
    match self.module.eval_clocked(Some(cycles)) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyException::new_err(format!("{err}"))),
    }
  }

  fn eval_reset_clocked(&mut self, cycles: u32) -> PyResult<()> {
    match self.module.eval_reset_clocked(Some(cycles)) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyException::new_err(format!("{err}"))),
    }
  }

  #[allow(unused_variables)]
  fn get_module_breakdown(&self, name: &str) -> PyResult<HashMap<String, usize>> {
    todo!()
  }

  #[allow(unused_variables)]
  fn get_module_area(&self, name: &str) -> PyResult<f64> {
    todo!()
  }

  #[allow(unused_variables)]
  fn get_module_total_toggle_count(&self, name: &str) -> PyResult<usize> {
    todo!()
  }

  fn get_port_string(&self, name: &str) -> PyResult<String> {
    match self.module.get_port_string(name) {
      Ok(bit_string) => Ok(bit_string),
      Err(err) => Err(PyAttributeError::new_err(format!("{err}"))),
    }
  }

  fn is_port_input(&self, name: &str) -> PyResult<bool> {
    let direction = match self.module.get_port_direction(name) {
      Ok(direction) => direction,
      Err(err) => return Err(PyAttributeError::new_err(format!("{err}"))),
    };

    Ok(direction == PortDirection::Input)
  }

  fn get_port_numpy(&self, name: &str, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
    let item_type = numpy_array.getattr("dtype")?.getattr("str")?.to_string();
    let shape = self.get_port_shape(name)?;
    let elem_size = shape[1];
    let bits = match self.module.get_port_bits(name) {
      Ok(bits) => bits,
      Err(err) => return Err(PyAttributeError::new_err(format!("{err}"))),
    };
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

  fn set_port_numpy(&mut self, name: &str, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
    let item_type = numpy_array.getattr("dtype")?.getattr("str")?.to_string();
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
        )))
      }
    };
    match self.module.set_port_bits(name, &bits) {
      Ok(()) => Ok(()),
      Err(err) => Err(PyAttributeError::new_err(format!("{err}"))),
    }
  }
}
