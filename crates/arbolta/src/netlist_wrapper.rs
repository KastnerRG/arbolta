use crate::{
  cell::{Cell, CellMapping, create_cell},
  hardware_module::ModuleError,
  port::{Port, PortDirection, parse_bit},
  yosys::{self, Netlist, RTLID, TopoOrder},
};
use indexmap::{IndexSet, indexmap};
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct NetlistWrapper {
  pub top_module: String,
  netlist: Netlist,
  pub cells: Vec<RTLID>, // Should be in topological order
  pub modules: HashSet<Vec<String>>,
  pub nets: HashMap<RTLID, Box<[usize]>>,
  pub names_to_nets: HashMap<String, Box<[usize]>>,
}

impl NetlistWrapper {
  pub fn new(
    netlist: Netlist,
    top_module: Option<&str>,
    torder: TopoOrder,
    use_slash_hierarchy: bool,
  ) -> Result<Self, ModuleError> {
    let mut netlist = netlist;

    let top_module = top_module
      .or(find_top_module_name(&netlist))
      .ok_or(ModuleError::TopModule)?
      .to_string();
    let module = netlist.modules.get_mut(&top_module).unwrap();

    let torder_cells = IndexSet::<&str>::from_iter(
      torder
        .get(top_module.as_str())
        .ok_or(ModuleError::TopModule)?
        .clone(),
    );
    // Rearrange cells in topological order, put $scopeinfo at end
    module.cells.sort_by_key(|cell_name, _| {
      torder_cells
        .get_index_of(cell_name.as_str())
        .unwrap_or(usize::MAX)
    });

    let (cells, cell_modules) = parse_cells(module, use_slash_hierarchy)?;
    let nets = parse_nets(module, use_slash_hierarchy)?;
    let names_to_nets = HashMap::from_iter(nets.iter().map(|(id, n)| (id.to_string(), n.clone())));

    // Add modules only seen in nets (no synthesized cells)
    let modules = HashSet::from_iter(
      cell_modules
        .into_iter()
        .chain(nets.keys().map(|id| id.parents.clone())),
    );

    Ok(Self {
      top_module,
      netlist,
      cells,
      modules,
      nets,
      names_to_nets,
    })
  }

  pub fn find_module_ports<S: AsRef<str>>(
    &self,
    parents: Option<&[S]>,
  ) -> Result<HashMap<String, Port>, ModuleError> {
    let parents: Vec<&str> = match parents {
      Some(p) => p.iter().map(|s| s.as_ref()).collect(),
      None => vec![self.top_module.as_ref()],
    };

    // TODO: better errors
    let module = self
      .netlist
      .modules
      .get(&parents.join("."))
      .ok_or(ModuleError::MissingModule)?;

    let mut ports = HashMap::new();
    for (port_name, port_info) in &module.ports {
      ports.insert(port_name.clone(), Port::try_from(port_info)?);
    }

    Ok(ports)
  }

  // Only looks in TOP module
  pub fn find_cell(&self, id: &RTLID) -> Option<&yosys::Cell> {
    let module = &self.netlist.modules[&self.top_module];
    // TODO: make this more efficient
    module.cells.get(&id.to_string())
  }

  fn build_cell(&self, cell: &RTLID, mapping: Option<&CellMapping>) -> Result<Cell, ModuleError> {
    println!("{cell:?}");
    let synth_cell = self
      .find_cell(cell)
      .ok_or(ModuleError::MissingCell(cell.to_string()))?;

    let mut connections = BTreeMap::new();
    for (port_name, bits) in &synth_cell.connections {
      let nets: Vec<usize> = bits.iter().map(parse_bit).collect::<Result<Vec<_>, _>>()?;
      connections.insert(port_name.as_str(), nets.into_boxed_slice());
    }

    let mut parameters = BTreeMap::new();
    for (param_name, param) in &synth_cell.parameters {
      // TODO: Error handling
      if let Some(param) = param.to_number() {
        parameters.insert(param_name.as_str(), param);
      } else {
        // TODO: Handle later
        println!("Ignoring parameter `{param_name}={param:?}`");
      }
    }

    Ok(create_cell(
      &synth_cell.cell_type,
      &connections,
      &parameters,
      mapping,
    )?)
  }

  pub fn build_cells(&self, mapping: Option<&CellMapping>) -> Result<Vec<Cell>, ModuleError> {
    let cells = self
      .cells
      .iter()
      .rev()
      .map(|c| Self::build_cell(self, c, mapping))
      .collect::<Result<Vec<Cell>, _>>()?;
    Ok(cells)
  }

  // CLEAN
  pub fn build_graph(&self) -> Result<NetlistGraph, ModuleError> {
    let mut graph = NetlistGraph::new();
    let mut cell_nodes = BTreeMap::<&RTLID, NodeIndex>::new();
    let mut bit_drivers = BTreeMap::<usize, HashSet<&RTLID>>::new();
    let mut bit_users = BTreeMap::<usize, HashSet<&RTLID>>::new();

    let input_cell = RTLID::new(&[], &"$input");
    let output_cell = RTLID::new(&[], &"$output");
    let module = &self.netlist.modules[&self.top_module];
    for port_info in module.ports.values() {
      if port_info.direction == yosys::PortDirection::Input {
        let nets: Vec<usize> = port_info
          .bits
          .iter()
          .map(parse_bit)
          .collect::<Result<Vec<_>, _>>()?;
        for n in nets {
          bit_drivers.entry(n).or_default().insert(&input_cell);
        }
      } else if port_info.direction == yosys::PortDirection::Output {
        let nets: Vec<usize> = port_info
          .bits
          .iter()
          .map(parse_bit)
          .collect::<Result<Vec<_>, _>>()?;
        for n in nets {
          bit_users.entry(n).or_default().insert(&output_cell);
        }
      }
    }
    cell_nodes.insert(&input_cell, graph.add_node(input_cell.clone()));
    cell_nodes.insert(&output_cell, graph.add_node(output_cell.clone()));

    for cell_id in &self.cells {
      let cell = self.find_cell(cell_id).unwrap();

      for (port_name, bits) in &cell.connections {
        if let Some(direction) = cell.port_directions.get(port_name) {
          let direction = PortDirection::try_from(direction)?;
          let nets: Vec<usize> = bits.iter().map(parse_bit).collect::<Result<Vec<_>, _>>()?;

          for n in nets {
            match direction {
              PortDirection::Input => {
                bit_users.entry(n).or_default().insert(cell_id);
              }
              PortDirection::Output => {
                bit_drivers.entry(n).or_default().insert(cell_id);
              }
            }
          }
        }
      }
      cell_nodes.insert(cell_id, graph.add_node(cell_id.clone()));
    }

    for (net, user_cells) in &bit_users {
      if let Some(drivers) = bit_drivers.get(net) {
        for driver_cell in drivers {
          let driver_node = &cell_nodes[driver_cell];

          for user_cell in user_cells {
            let user_node = &cell_nodes[user_cell];

            graph.add_edge(*driver_node, *user_node, *net);
          }
        }
      }
    }

    Ok(graph)
  }
}

pub type NetlistGraph = DiGraph<RTLID, usize>;

/// Returns (cells, modules)
fn parse_cells(
  module: &mut yosys::Module,
  use_slash_hierarchy: bool,
) -> Result<(Vec<RTLID>, Vec<Vec<String>>), ModuleError> {
  let mut scope_cells = vec![];
  let mut primitive_cells = vec![];

  // Separate cells into scope and primitive
  module.cells.iter().for_each(|(cell_name, cell_info)| {
    if cell_info.cell_type == "$scopeinfo" {
      scope_cells.push((cell_name, cell_info));
    } else {
      primitive_cells.push((cell_name, cell_info));
    }
  });

  let mut modules = vec![];
  for (scope_name, cell_info) in scope_cells {
    if let Some(hdlname) = cell_info.attributes.get("hdlname")
      && let Some(hdlname) = hdlname.to_string_if_string()
    {
      modules.push(
        hdlname
          .split(" ")
          .map(|s| s.to_string())
          .collect::<Vec<String>>(),
      );
    } else {
      modules.push(vec![scope_name.clone()])
    }
  }

  let mut cells = vec![];
  let mut new_cells = indexmap! {};
  for (cell_name, cell_info) in primitive_cells {
    // TODO: Check if cell is primitive?
    let id = if let Some(scopename) = cell_info.attributes.get("scopename")
      && let Some(scopename) = scopename.to_string_if_string()
    {
      RTLID::new(&[scopename], &cell_name.as_str())
    } else if use_slash_hierarchy {
      let split_name: Vec<&str> = cell_name.split("/").collect();
      if let Some((name, parents)) = split_name.split_last() {
        // Add new module
        modules.push(parents.iter().map(|s| s.to_string()).collect());

        RTLID::new(parents, name)
      } else {
        RTLID::new(&[], cell_name)
      }
    } else {
      RTLID::new(&[], cell_name)
    };

    new_cells.insert(id.to_string(), cell_info.clone());
    cells.push(id);
  }

  module.cells = new_cells;

  Ok((cells, modules))
}

fn parse_nets(
  module: &mut yosys::Module,
  use_slash_hierarchy: bool,
) -> Result<HashMap<RTLID, Box<[usize]>>, ModuleError> {
  let mut all_nets = HashMap::new();

  let mut new_nets = indexmap! {};
  for (net_name, net_info) in &module.netnames {
    let id = if let Some(scopename) = net_info.attributes.get("scopename")
      && let Some(scopename) = scopename.to_string_if_string()
    {
      RTLID::new(&[scopename], &net_name.as_str())
    } else if let Some(hdlname) = net_info.attributes.get("hdlname")
      && let Some(hdlname) = hdlname.to_string_if_string()
    {
      let hdlname_split: Vec<&str> = hdlname.split(" ").collect();
      let (name, parents) = hdlname_split.split_last().unwrap();
      RTLID::new(parents, name)
    } else if use_slash_hierarchy {
      let split_name: Vec<&str> = net_name.split("/").collect();
      if let Some((name, parents)) = split_name.split_last() {
        RTLID::new(parents, name)
      } else {
        RTLID::new(&[], net_name)
      }
    } else {
      RTLID::new(&[], net_name)
    };

    new_nets.insert(id.to_string(), net_info.clone());

    let nets: Vec<usize> = net_info
      .bits
      .iter()
      .map(parse_bit)
      .collect::<Result<_, _>>()?;

    all_nets.insert(id, nets.into_boxed_slice());
  }

  module.netnames = new_nets;

  Ok(all_nets)
}

fn find_top_module_name(netlist: &Netlist) -> Option<&str> {
  for (module_name, module_info) in &netlist.modules {
    if let Some(top_val) = module_info.attributes.get("top")
      && let Some(top) = top_val.to_number()
      && top == 1
    {
      return Some(module_name);
    }
  }

  None
}
