// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::hardware_module::HardwareDesign;
use arbolta::{bit::Bit, port::PortDirection};
use pyo3::{
  exceptions::{PyAttributeError, PyValueError},
  prelude::*,
  types::PyDict,
};
use std::collections::{HashMap, HashSet};

// Treat as a dataclass
#[pyclass]
pub struct PortConfig {
  #[pyo3(get, set)]
  pub shape: (usize, usize), // Defaults (1,1)
  #[pyo3(get, set)]
  pub dtype: Py<PyAny>, // Numpy datatype, defaults to np.uint
  #[pyo3(get, set)]
  pub clock: bool,
  #[pyo3(get, set)]
  pub reset: bool,
  #[pyo3(get, set)]
  pub polarity: Option<u8>, // Literal [0, 1]
}

#[pymethods]
impl PortConfig {
  #[new]
  #[pyo3(signature = (shape=(1,1), dtype=None, clock=false, reset=false, polarity=None))]
  fn new(
    py: Python<'_>,
    shape: (usize, usize),
    dtype: Option<Py<PyAny>>,
    clock: bool,
    reset: bool,
    polarity: Option<u8>,
  ) -> PyResult<Self> {
    let dtype = match dtype {
      Some(d) => d,
      None => py.import("numpy")?.getattr("uint")?.into(),
    };

    Ok(Self {
      shape,
      dtype,
      clock,
      reset,
      polarity,
    })
  }
}

#[pyclass(dict)]
pub struct Ports {
  module: Py<HardwareDesign>,
  input_ports: HashSet<String>,
  output_ports: HashSet<String>,
}

#[pymethods]
impl Ports {
  fn __setattr__(self_: Bound<'_, Self>, name: &str, value: &Bound<'_, PyAny>) -> PyResult<()> {
    let binding = &mut self_.borrow();

    if binding.output_ports.contains(name) {
      return Err(PyAttributeError::new_err(format!(
        "Cannot set output port `{name}`"
      )));
    }

    if !binding.input_ports.contains(name) {
      return Err(PyAttributeError::new_err(format!(
        "Port `{name}` doesn't exist"
      )));
    }

    let py = self_.py();
    let object_type = py
      .get_type::<PyAny>()
      .py()
      .import("builtins")?
      .getattr("object")?;

    let buffer_ref = object_type.call_method1("__getattribute__", (&self_, name))?;

    let np = py.import("numpy")?;
    np.getattr("copyto")?.call1((&buffer_ref, value))?;

    binding
      .module
      .borrow_mut(py)
      .set_port_numpy(name, &buffer_ref)?;

    Ok(())
  }

  fn __getattribute__(self_: Bound<'_, Self>, name: &str) -> PyResult<Py<PyAny>> {
    let binding = &mut self_.borrow();

    let py = self_.py();
    let object_type = py
      .get_type::<PyAny>()
      .py()
      .import("builtins")?
      .getattr("object")?;
    let buffer_ref = object_type.call_method1("__getattribute__", (&self_, name))?;

    // Update
    if binding.output_ports.contains(name) {
      binding
        .module
        .borrow_mut(py)
        .get_port_numpy(name, &buffer_ref)?;
    }

    Ok(buffer_ref.into())
  }
}

impl Ports {
  fn new_base(module: Py<HardwareDesign>) -> Self {
    Self {
      module,
      input_ports: Default::default(),
      output_ports: Default::default(),
    }
  }

  pub fn new(
    py: Python<'_>,
    config: HashMap<String, PyRef<PortConfig>>,
    module: Py<HardwareDesign>,
  ) -> anyhow::Result<Py<Self>> {
    let np = py.import("numpy")?;
    let module_ref = &mut module.bind(py).borrow_mut().module;

    let new_self = Py::new(py, Self::new_base(module))?;
    let temp_binding = new_self.getattr(py, "__dict__")?;
    let ports = temp_binding.cast_bound::<PyDict>(py).unwrap();

    let binding = &mut new_self.bind(py).borrow_mut();

    let port_names: Vec<String> = module_ref.ports.keys().cloned().collect();
    for port_name in port_names {
      let direction = module_ref.get_port_direction(&port_name)?;
      let kwargs = PyDict::new(py);
      let buffer_len: usize;

      if let Some(port_config) = config.get(&port_name) {
        if port_config.reset || port_config.clock {
          if let Some(polarity) = port_config.polarity {
            let polarity = Bit::from_int(polarity)?;
            let nets = module_ref
              .get_net(&port_name)
              .ok_or(PyAttributeError::new_err(format!("No net `{port_name}`")))?;

            if nets.len() != 1 {
              return Err(PyAttributeError::new_err(">1 clock or reset bits".to_string()).into());
            }

            let net = *nets.first().unwrap();
            if port_config.reset {
              module_ref.set_reset(net, polarity)?;
            }

            if port_config.clock {
              module_ref.set_clock(net, polarity)?;
            }
          } else {
            return Err(PyValueError::new_err("No polarity given".to_string()).into());
          }
        }

        let shape: [usize; 2] = [port_config.shape.0, port_config.shape.1];

        if shape[0] != 1 {
          return Err(PyValueError::new_err(format!("Only 1D shapes supported: {shape:?}")).into());
        }

        let internal_shape = module_ref.get_port_shape(&port_name)?;
        let (num_elems, elem_size) = (shape[1], internal_shape[1] / shape[1]);
        module_ref.set_port_shape(&port_name, &[num_elems, elem_size])?;
        kwargs.set_item("dtype", port_config.dtype.bind(py))?;
        buffer_len = num_elems;
      // No config given
      } else {
        kwargs.set_item("dtype", np.getattr("uint")?)?;
        buffer_len = module_ref.get_port_shape(&port_name)?[0];
      }

      let buffer = np.getattr("zeros")?.call((buffer_len,), Some(&kwargs))?;
      ports.set_item(port_name.clone(), buffer)?;

      match direction {
        PortDirection::Input => &mut binding.input_ports,
        PortDirection::Output => &mut binding.output_ports,
      }
      .insert(port_name);
    }

    Ok(new_self)
  }
}
