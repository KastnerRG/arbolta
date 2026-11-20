// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::hardware_design::HardwareDesign;
use arbolta::bit::Bit;
use pyo3::{
  exceptions::PyIndexError,
  prelude::*,
  types::{PyDict, PyList},
};

#[pyclass]
pub struct Signal {
  design: Py<HardwareDesign>,
  #[pyo3(get)]
  pub net: usize,
}

impl Signal {
  pub fn new(py: Python<'_>, design: Py<HardwareDesign>, net: usize) -> PyResult<Py<Self>> {
    Py::new(py, Self { design, net })
  }
}

#[pymethods]
impl Signal {
  #[getter]
  fn get_value(&self, py: Python<'_>) -> anyhow::Result<Bit> {
    Ok(self.design.borrow(py).inner.get_signal(self.net)?)
  }
  #[setter]
  fn set_value(&mut self, py: Python<'_>, val: Bit) -> anyhow::Result<()> {
    let design = &mut self.design.borrow_mut(py).inner;
    Ok(design.set_signal(self.net, val)?)
  }

  #[getter]
  fn get_constant(&self, py: Python<'_>) -> bool {
    self.design.borrow(py).inner.signals.is_constant(self.net)
  }

  #[setter]
  fn set_constant(&mut self, py: Python<'_>, constant: bool) -> anyhow::Result<()> {
    let design = &mut self.design.borrow_mut(py).inner;

    // TODO: Fix original `set_constant` function so we don't have to do this
    if constant {
      // Get original signal value/bit and make that constant
      design.stick_signal(self.net, self.get_value(py)?)?;
    } else {
      design.unstick_signal(self.net)?;
    }

    Ok(())
  }

  #[getter]
  fn get_toggles_rising(&self, py: Python<'_>) -> u64 {
    self
      .design
      .borrow(py)
      .inner
      .signals
      .get_toggles_rising(self.net)
  }

  #[getter]
  fn get_toggles_falling(&self, py: Python<'_>) -> u64 {
    self
      .design
      .borrow(py)
      .inner
      .signals
      .get_toggles_falling(self.net)
  }

  #[getter]
  fn __dict__(self_: Bound<'_, Self>) -> PyResult<Bound<'_, PyDict>> {
    let binding = &mut self_.borrow();
    let py = self_.py();

    let dict = PyDict::new(py);
    dict.set_item("net", binding.net)?;
    dict.set_item("value", binding.get_value(py)?)?;
    dict.set_item("constant", binding.get_constant(py))?;
    dict.set_item("toggles_rising", binding.get_toggles_rising(py))?;
    dict.set_item("toggles_falling", binding.get_toggles_falling(py))?;

    Ok(dict)
  }

  fn __repr__(self_: Bound<'_, Self>) -> PyResult<Bound<'_, PyAny>> {
    self_.getattr("__dict__")?.call_method0("__repr__")
  }
}

#[pyclass]
pub struct Signals {
  design: Py<HardwareDesign>,
}

impl Signals {
  pub fn new(py: Python<'_>, design: Py<HardwareDesign>) -> PyResult<Py<Self>> {
    Py::new(py, Self { design })
  }
}

#[pymethods]
impl Signals {
  fn __getitem__(self_: Bound<'_, Self>, key: &Bound<'_, PyAny>) -> anyhow::Result<Py<PyAny>> {
    let binding = &mut self_.borrow();
    let py = self_.py();

    // Single net, return `Net`
    if let Ok(net) = key.extract::<usize>() {
      Ok((Signal::new(py, binding.design.clone_ref(py), net)?).into())

    // TODO: Check slice
    // Potentially multiple nets, return list['net']
    } else if let Ok(name) = key.extract::<&str>() {
      let design = binding.design.borrow(py);
      let nets = design
        .inner
        .get_net(name)
        .ok_or(PyIndexError::new_err(format!(
          "Net name `{name}` doesn't exist"
        )))?;

      let new_nets: Result<Vec<Py<Signal>>, _> = nets
        .iter()
        .map(|&n| Signal::new(py, binding.design.clone_ref(py), n))
        .collect();

      Ok((PyList::new(py, new_nets?)?).into())
    } else {
      Err(PyIndexError::new_err(format!("Unsupported signal index: {key}")).into())
    }
  }

  fn __len__(&self, py: Python<'_>) -> usize {
    self.design.borrow(py).inner.signals.size
  }
}
