// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::{
  conversion::{bits_to_bool_numpy, bits_to_int_numpy, bool_numpy_to_bits, int_numpy_to_bits},
  ports::{PortConfig, Ports},
};
use arbolta::{
  bit::Bit,
  cell::CellMapping,
  hardware_module::{HardwareModule, ToggleCount},
  netlist_wrapper::NetlistWrapper,
  port::{PortDirection, parse_bit},
  yosys::{self, Netlist},
};
use petgraph::visit::EdgeRef;
use pyo3::{
  exceptions::{PyAttributeError, PyTypeError, PyValueError},
  prelude::*,
  types::{PyBytes, PyDict, PyList, PyString, PyTuple},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[pyclass(weakref, dict, module = "arbolta")]
#[derive(Deserialize, Serialize)]
pub struct HardwareDesign {
  pub inner: HardwareModule,
}

impl HardwareDesign {
  pub fn get_port_shape(&self, name: &str) -> anyhow::Result<[usize; 2]> {
    Ok(self.inner.get_port_shape(name)?)
  }

  pub fn get_port_numpy(&self, name: &str, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
    let item_type = numpy_array.getattr("dtype")?.getattr("str")?.to_string();
    let elem_size = self.get_port_shape(name)?[1];

    let bits = self
      .inner
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
      .inner
      .set_port(name, bits)
      .map_err(|e| PyAttributeError::new_err(format!("{e}")))
  }
}

fn parse_bytes_or_read_path(py: Python<'_>, data: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
  // Check bytes
  if let Ok(bytes) = data.cast::<PyBytes>() {
    return Ok(bytes.as_bytes().to_vec());
  }

  // Check pathlike
  let os = PyModule::import(py, "os")?;

  if data.cast::<PyString>().is_ok() || data.is_instance(&os.getattr("PathLike")?)? {
    // Read
    let path: String = os.getattr("fspath")?.call1((data,))?.extract()?;
    let bytes = std::fs::read(path)?;
    return Ok(bytes);
  }

  Err(PyTypeError::new_err("expected bytes or str/os.PathLike"))
}

#[pymethods]
impl HardwareDesign {
  #[new]
  #[pyo3(signature = (netlist, config, torder=None, hierarchy_separator=None, top_module=None, cell_mapping=None))]
  pub fn new(
    py: Python<'_>,
    netlist: &Bound<'_, PyAny>,
    config: Py<PyAny>,
    torder: Option<&Bound<'_, PyAny>>,
    hierarchy_separator: Option<&str>,
    top_module: Option<&str>,
    cell_mapping: Option<CellMapping>,
  ) -> anyhow::Result<Py<Self>> {
    let raw_netlist = parse_bytes_or_read_path(py, netlist)?;
    let netlist = Netlist::from_slice(&raw_netlist)?;

    let raw_torder = match torder {
      Some(torder) => parse_bytes_or_read_path(py, torder)?,
      None => yosys::run_torder(&netlist)?.into_bytes(),
    };
    let torder = yosys::parse_torder(&raw_torder)?;

    let netlist_wrapper = NetlistWrapper::new(top_module, netlist, torder, hierarchy_separator)?;

    let module = Self {
      inner: HardwareModule::new(netlist_wrapper, cell_mapping.as_ref())?,
    };

    // Get submodules before binding to Python
    let submodules = module
      .inner
      .netlist
      .modules
      .iter()
      .map(|p| p.join("."))
      .collect::<Vec<String>>();

    let py_module = Py::new(py, module)?;

    // Add custom members (can't make them static for serialization)
    let dict_binding = py_module.getattr(py, "__dict__")?;
    let self_dict = dict_binding.cast_bound::<PyDict>(py).unwrap();

    // Add ports field
    let temp_config: HashMap<String, PortConfig> = config.extract(py)?;
    let ports = Py::new(py, Ports::new(py, &temp_config, py_module.clone_ref(py))?)?;
    self_dict.set_item("ports", ports)?;

    // Add modules/submodules field
    self_dict.set_item("modules", PyList::new(py, submodules)?)?;

    // Add config
    self_dict.set_item("config", config)?;

    Ok(py_module)
  }

  pub fn __getnewargs__<'p>(
    self_: Bound<'p, Self>,
    py: Python<'p>,
  ) -> anyhow::Result<Bound<'p, PyTuple>> {
    let dict_binding = self_.getattr("__dict__")?.cast_into::<PyDict>().unwrap();
    let config = dict_binding
      .get_item("config")?
      .ok_or_else(|| anyhow::anyhow!("Missing self.__dict__"))?;

    let self_binding = &mut self_.borrow();
    let netlist = self_binding.inner.netlist.netlist.to_string()?.into_bytes();

    Ok((netlist, config).into_pyobject(py)?)
  }

  pub fn __getstate__(&self, py: Python) -> anyhow::Result<Py<PyAny>> {
    let mut serializer = flexbuffers::FlexbufferSerializer::new();
    self.inner.serialize(&mut serializer)?;

    let data = serializer.view();
    Ok(PyBytes::new(py, data).into())
  }

  pub fn __setstate__(&mut self, py: Python, state: Py<PyAny>) -> anyhow::Result<()> {
    let data = state
      .extract::<&[u8]>(py)
      .map_err(|e| anyhow::anyhow!(format!("{e}")))?;
    let reader = flexbuffers::Reader::get_root(data)?;
    self.inner = HardwareModule::deserialize(reader)?;

    Ok(())
  }

  pub fn reset(&mut self) {
    self.inner.reset()
  }

  pub fn eval(&mut self) {
    self.inner.eval()
  }

  #[pyo3(signature = (cycles=None))]
  pub fn eval_clocked(&mut self, cycles: Option<u32>) -> anyhow::Result<()> {
    Ok(self.inner.eval_clocked(cycles)?)
  }

  #[pyo3(signature = (cycles=None))]
  pub fn eval_reset_clocked(&mut self, cycles: Option<u32>) -> anyhow::Result<()> {
    Ok(self.inner.eval_reset_clocked(cycles)?)
  }

  pub fn stick_signal(&mut self, net: usize, val: u8) -> anyhow::Result<()> {
    Ok(self.inner.stick_signal(net, Bit::from_int(val)?)?)
  }

  pub fn unstick_signal(&mut self, net: usize) -> anyhow::Result<()> {
    Ok(self.inner.unstick_signal(net)?)
  }

  pub fn is_port_input(&self, name: &str) -> anyhow::Result<bool> {
    let direction = self.inner.get_port_direction(name)?;
    Ok(direction == PortDirection::Input)
  }

  #[pyo3(signature = (category="total", by_net=true))]
  pub fn toggle_count(&self, py: Python<'_>, category: &str, by_net: bool) -> PyResult<Py<PyDict>> {
    let category = match category {
      "falling" => ToggleCount::Falling,
      "rising" => ToggleCount::Rising,
      "total" => ToggleCount::Total,
      _ => {
        return Err(PyValueError::new_err(format!(
          "Invalid toggle category `{category}`"
        )));
      }
    };

    let toggles = PyDict::new(py);
    if by_net {
      for (submodule_names, nets_ref) in self.inner.get_submodule_toggles_by_net(category) {
        let name = submodule_names.join(".");
        let nets: HashMap<String, u64> = nets_ref
          .into_iter()
          .map(|(n, c)| (n.to_string(), c))
          .collect();

        toggles.set_item(name, nets)?;
      }
    } else {
      for (submodule_names, count) in self.inner.get_submodule_toggles_total(category) {
        let name = submodule_names.join(".");

        toggles.set_item(name, count)?;
      }
    }

    Ok(toggles.into())
  }

  pub fn torder(&self) -> PyResult<Vec<String>> {
    let cells: Vec<String> = self
      .inner
      .netlist
      .cells
      .iter()
      .map(|c| c.to_string())
      .collect();

    Ok(cells)
  }

  pub fn netlist(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
    let json = py.import("json")?;
    let netlist = self
      .inner
      .netlist
      .netlist
      .to_string()
      .map_err(|e| PyAttributeError::new_err(format!("{e}")))?;

    let netlist_dict: Bound<'_, PyDict> = json.getattr("loads")?.call1((netlist,))?.cast_into()?;
    Ok(netlist_dict.unbind())
  }

  pub fn netlist_graph(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let networkx = py.import("networkx")?;

    let graph = self
      .inner
      .netlist
      .build_graph()
      .map_err(|e| PyAttributeError::new_err(format!("{e}")))?;

    let nx_graph = networkx.getattr("DiGraph")?.call0()?;

    // Add nodes
    let mut nodes: Vec<(usize, Py<PyDict>)> = vec![];
    for i in graph.node_indices() {
      let rtlid = graph.node_weight(i).unwrap();

      let entry = PyDict::new(py);
      entry.set_item("rtlid", rtlid.to_string())?;

      // $input or $output cell, early continue
      let Some(cell) = self.inner.netlist.find_cell(rtlid) else {
        nodes.push((i.index(), entry.unbind()));
        continue;
      };

      // Extract other fields
      entry.set_item("cell_type", cell.cell_type.clone())?;

      if let Some(src_attr) = cell.attributes.get("src")
        && let Some(src) = src_attr.to_string_if_string()
      {
        entry.set_item("src", src.to_string())?;
      }

      nodes.push((i.index(), entry.unbind()));
    }
    nx_graph.call_method1("add_nodes_from", (nodes,))?;

    // Add edges
    let mut edges: Vec<(usize, usize, Py<PyDict>)> = vec![];
    for edge in graph.edge_references() {
      let net_driver_node = edge.source();
      let net_user_node = edge.target();

      let net_driver = graph.node_weight(net_driver_node).unwrap();
      let net_user = graph.node_weight(net_user_node).unwrap();
      let net = *edge.weight();

      let entry = PyDict::new(py);
      entry.set_item("net", net)?;

      if let Some(driver_cell) = self.inner.netlist.find_cell(net_driver) {
        for (port_name, bits) in &driver_cell.connections {
          // TODO: pre-process this somehow
          let nets = HashSet::<usize>::from_iter(
            bits
              .iter()
              .map(parse_bit)
              .collect::<Result<Vec<_>, _>>()
              .map_err(|e| PyAttributeError::new_err(format!("{e}")))?,
          );

          if nets.contains(&net) {
            entry.set_item("driver", port_name.clone())?;
          }
        }
      }

      if let Some(user_cell) = self.inner.netlist.find_cell(net_user) {
        for (port_name, bits) in &user_cell.connections {
          // TODO: pre-process this somehow
          let nets = HashSet::<usize>::from_iter(
            bits
              .iter()
              .map(parse_bit)
              .collect::<Result<Vec<_>, _>>()
              .map_err(|e| PyAttributeError::new_err(format!("{e}")))?,
          );

          if nets.contains(&net) {
            entry.set_item("user", port_name.clone())?;
          }
        }
      }

      edges.push((
        net_driver_node.index(),
        net_user_node.index(),
        entry.unbind(),
      ));
    }
    nx_graph.call_method1("add_edges_from", (edges,))?;

    Ok(nx_graph.into())
  }
}
