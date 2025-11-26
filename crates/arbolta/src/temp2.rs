use crate::{
  hardware_module::ModuleError,
  port::{PortDirection, parse_bit},
};
use petgraph::visit::IntoNodeReferences;
use petgraph::{prelude::*, visit::DfsPostOrder}; // dot::Dot
use std::collections::{BTreeMap, HashMap, HashSet};
// use std::io::Write;
use yosys_netlist_json as yosys_json;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, derive_more::Constructor)]
pub struct TopoCellParent {
  pub name: String,
  pub cell_type: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, derive_more::Display)]
#[display("name: {name}")]
pub struct TopoCell {
  pub parents: Vec<TopoCellParent>,
  pub name: String,
  pub cell_type: String,
  // Only after getting global nets...
  // BTreeMap is hashable! :)
  pub connections: Option<BTreeMap<String, Box<[usize]>>>,
  pub port_directions: Option<BTreeMap<String, PortDirection>>,
  pub parameters: Option<BTreeMap<String, usize>>,
}

pub fn collect_cells(
  top_module: &str,
  parents: &[TopoCellParent],
  netlist: &yosys_json::Netlist,
) -> Result<Vec<TopoCell>, ModuleError> {
  let synth_module = netlist
    .modules
    .get(top_module)
    .ok_or(ModuleError::TopModule(top_module.to_string()))?;

  let mut flattened_cells = vec![];
  for (cell_name, cell_info) in &synth_module.cells {
    let cell_type = &cell_info.cell_type;

    // Submodule
    if netlist.modules.contains_key(cell_type) {
      let mut parents = parents.to_vec();
      parents.push(TopoCellParent {
        name: cell_name.to_owned(),
        cell_type: cell_type.to_owned(),
      });

      flattened_cells.append(&mut collect_cells(cell_type, &parents, netlist)?);
    // Primitive cell
    } else {
      flattened_cells.push(TopoCell {
        parents: parents.to_vec(),
        name: cell_name.to_owned(),
        cell_type: cell_type.to_owned(),
        ..Default::default()
      });
    }
  }

  Ok(flattened_cells)
}

pub type TopoHierarchy = HashMap<TopoCellParent, HashSet<TopoCellParent>>;

pub fn get_cell_hierarchy(cells: &[TopoCell]) -> TopoHierarchy {
  let mut hierarchy = TopoHierarchy::new();
  cells.iter().map(|c| &c.parents).for_each(|parents| {
    // For each topo cell's parents
    (0..parents.len() - 1).for_each(|i| {
      hierarchy
        .entry(parents[i].clone()) // Previous parent
        .or_default()
        .insert(parents[i + 1].clone()); // Current parent
    });
  });

  hierarchy
}

pub type TopoNetMap = HashMap<TopoCellParent, HashMap<usize, usize>>;

pub fn collect_nets(
  parent: &TopoCellParent,
  hierarchy: &TopoHierarchy,
  netlist: &yosys_json::Netlist,
  global_nets: &mut TopoNetMap,
  global_net_max: &mut usize,
) -> Result<(), ModuleError> {
  let synth_module = netlist
    .modules
    .get(&parent.cell_type)
    .ok_or(ModuleError::TopModule(parent.cell_type.to_string()))?;

  let nets = synth_module
    .netnames
    .values()
    .flat_map(|net_info| &net_info.bits)
    .map(parse_bit)
    .collect::<Result<Vec<_>, _>>()?;

  if !global_nets.contains_key(parent) {
    global_nets.insert(parent.clone(), HashMap::from([(0, 0), (1, 1)]));

    nets.iter().for_each(|&n| {
      global_nets.get_mut(parent).unwrap().insert(n, n);
      *global_net_max = std::cmp::max(*global_net_max, n);
    });
  }

  // println!("GLOBAL NETS");
  // for (key, val) in global_nets.iter() {
  // println!("{key:?}");
  // let mut vals: Vec<(&usize, &usize)> = val.iter().collect();
  // vals.sort();
  // for v in vals {
  // println!("{v:?}");
  // }
  // }

  // Add rest of nets
  for n in &nets {
    if !global_nets[parent].contains_key(n) {
      *global_net_max += 1;
      global_nets
        .get_mut(parent)
        .unwrap()
        .insert(*n, *global_net_max);
    }
  }

  // Add children
  if let Some(children) = hierarchy.get(parent) {
    for child in children {
      global_nets.insert(child.clone(), HashMap::from([(0, 0), (1, 1)]));
      let synth_cell = &synth_module.cells[&child.name.to_string()];
      let child_module = &netlist.modules[&child.cell_type.to_string()];

      for (port_name, port_info) in &child_module.ports {
        let conn_bits = &synth_cell.connections[port_name];
        for (net_bit, conn_bit) in port_info.bits.iter().zip(conn_bits) {
          let net = parse_bit(net_bit)?;
          // Translate connection
          let conn = global_nets[parent][&parse_bit(conn_bit)?];

          global_nets.get_mut(child).unwrap().insert(net, conn);
        }
      }

      collect_nets(child, hierarchy, netlist, global_nets, global_net_max)?;
    }
  }

  Ok(())
}

pub fn update_cells(
  cells: &mut [TopoCell],
  global_nets: &TopoNetMap,
  netlist: &yosys_json::Netlist,
) -> Result<(), ModuleError> {
  for cell in cells.iter_mut() {
    // TODO: error handling
    let parent = cell.parents.last().unwrap();
    let synth_cell = &netlist.modules[&parent.cell_type.to_string()].cells[&cell.name.to_string()];

    let mut connections = BTreeMap::new();
    let mut port_directions = BTreeMap::new();
    for (port_name, bits) in &synth_cell.connections {
      let direction = PortDirection::try_from(&synth_cell.port_directions[port_name])?;
      let mut nets: Vec<usize> = bits.iter().map(parse_bit).collect::<Result<Vec<_>, _>>()?;
      // Actual mapping
      nets = nets.iter().map(|n| global_nets[parent][n]).collect();

      port_directions.insert(port_name.to_string(), direction);
      connections.insert(port_name.to_string(), nets.into_boxed_slice());
    }

    // Add parameters
    let mut parameters = BTreeMap::new();
    for (param_name, param) in &synth_cell.parameters {
      // TODO: Error handling
      if let Some(param) = param.to_number() {
        parameters.insert(param_name.to_string(), param);
      } else {
        println!("Couldn't convert parameter `{param_name}={param:?}`");
      }
    }

    cell.port_directions = Some(port_directions);
    cell.connections = Some(connections);
    cell.parameters = Some(parameters);
  }

  Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, derive_more::Display)]
pub enum TopoNode {
  Cell(TopoCell),
  Net(usize),
}
// TODO: Change net node or graph edge to some enum/struct that has netname...
// enum for multi bit port
// enum::single net(name, b)
// enum::multi net(name, index, b) merge later...
pub type NetlistGraph = DiGraph<TopoNode, usize>;

pub fn build_graph(
  top_module: &TopoCellParent,
  cells: &[TopoCell],
  global_nets: &TopoNetMap,
  netlist: &yosys_json::Netlist,
) -> Result<NetlistGraph, ModuleError> {
  let synth_module = netlist
    .modules
    .get(&top_module.cell_type.to_string())
    .ok_or(ModuleError::TopModule(top_module.cell_type.to_string()))?;

  let mut cell_nodes = HashMap::<TopoCell, NodeIndex>::new();
  let mut net_nodes = HashMap::<usize, NodeIndex>::new();
  let mut graph = NetlistGraph::new();

  // All top module port inputs driven by "input" cell for ordering
  let input_cell = TopoCell {
    parents: vec![top_module.clone()],
    name: "input".to_string(),
    cell_type: "input".to_string(),
    ..Default::default()
  };

  // Create all cell nodes (ensure input cell is first)
  std::slice::from_ref(&input_cell)
    .iter()
    .chain(cells)
    .for_each(|c| {
      cell_nodes.insert(c.clone(), graph.add_node(TopoNode::Cell(c.clone())));
    });

  // Create all net nodes
  for net in global_nets.values().flat_map(|c| c.values()) {
    // Avoid duplicates
    if !net_nodes.contains_key(net) {
      net_nodes.insert(*net, graph.add_node(TopoNode::Net(*net)));
    }
  }

  // Connect input cell to top module input nets
  for port_info in synth_module.ports.values() {
    if port_info.direction == yosys_json::PortDirection::Input {
      for net in port_info
        .bits
        .iter()
        .map(parse_bit)
        .collect::<Result<Vec<_>, _>>()?
      {
        // Avoid duplicates
        if !graph.contains_edge(cell_nodes[&input_cell], net_nodes[&net]) {
          graph.add_edge(cell_nodes[&input_cell], net_nodes[&net], net);
        }
      }
    }
  }

  // Connect all other cells/nets
  for cell in cells {
    let cell_node = &cell_nodes[cell];

    // TODO: Error handling
    for (port_name, nets) in cell.connections.as_ref().unwrap() {
      // TODO: Error handling
      let direction = &cell.port_directions.as_ref().unwrap()[port_name];

      for net in nets {
        let net_node = &net_nodes[net];

        match direction {
          PortDirection::Input if !graph.contains_edge(*net_node, *cell_node) => {
            graph.add_edge(*net_node, *cell_node, *net);
          }
          PortDirection::Output if !graph.contains_edge(*cell_node, *net_node) => {
            graph.add_edge(*cell_node, *net_node, *net);
          }
          _ => continue, // Edge exists, do nothing
        }
      }
    }
  }

  Ok(graph)
}

pub fn get_topo_cell_order(graph: &NetlistGraph) -> Vec<&TopoCell> {
  let mut cell_graph: DiGraph<&TopoCell, usize> = DiGraph::new();
  let mut orig_cell_nodes = HashMap::<&TopoCell, NodeIndex>::new();
  let mut cell_nodes = HashMap::<&TopoCell, NodeIndex>::new();

  // Create new graph with only cell nodes
  // Get node indices for cells in original graph
  for (node_index, node) in graph.node_references() {
    if let TopoNode::Cell(cell) = node {
      orig_cell_nodes.insert(cell, node_index);
      cell_nodes.insert(cell, cell_graph.add_node(cell));
    }
  }

  // Create connections
  for (cell, orig_node_index) in orig_cell_nodes {
    // All neighbors should be net nodes
    for net_index in graph.neighbors(orig_node_index) {
      let TopoNode::Net(n) = &graph[net_index] else {
        todo!()
      };

      // All neighbors should be cell nodes
      for driven_node in graph.neighbors(net_index) {
        let driver_node = &cell_nodes[cell];
        let TopoNode::Cell(driven_cell) = graph.node_weight(driven_node).unwrap() else {
          todo!()
        };
        let driven_node = &cell_nodes[driven_cell];

        cell_graph.add_edge(*driver_node, *driven_node, *n);
      }
    }
  }

  // TODO: check weight that node is == input
  let mut topo_search = DfsPostOrder::new(&cell_graph, NodeIndex::from(0));
  let mut found = vec![];

  while let Some(visited) = topo_search.next(&cell_graph) {
    let cell = cell_graph.node_weight(visited).unwrap();
    if cell.cell_type != "input" {
      // Don't include input cell/node
      found.push(*cell);
    }
  }

  found.reverse();
  found
}

pub fn parse_module(
  top_module: &str,
  netlist: &yosys_json::Netlist,
) -> Result<(NetlistGraph, TopoNetMap), ModuleError> {
  let top_cell_parent = TopoCellParent {
    name: top_module.to_string(),
    cell_type: top_module.to_string(),
  };

  let flattened_cells = collect_cells(top_module, std::slice::from_ref(&top_cell_parent), netlist)?;
  let hierarchy = get_cell_hierarchy(&flattened_cells);

  let mut global_nets = TopoNetMap::new();
  collect_nets(
    &top_cell_parent,
    &hierarchy,
    netlist,
    &mut global_nets,
    &mut 0,
  )?;

  let graph = build_graph(&top_cell_parent, &flattened_cells, &global_nets, netlist)?;

  // let dot_output = format!("{}", Dot::new(&graph));
  // let mut file = std::fs::File::create("graph.dot").expect("Could not create file");
  // file
  //   .write_all(dot_output.as_bytes())
  //   .expect("Could not write to file");

  Ok((graph, global_nets))
}
