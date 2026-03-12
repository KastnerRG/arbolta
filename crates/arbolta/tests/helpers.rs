use arbolta::yosys::*;
use indexmap::*;

fn allocate_nets(offset: Option<usize>, widths: &[usize]) -> Vec<Vec<BitVal>> {
  let mut all_nets: Vec<Vec<usize>> = vec![];
  let offset = offset.unwrap_or(0);

  for width in widths {
    let start: usize = match all_nets.last() {
      Some(last) => last[last.len() - 1] + 1,
      None => offset,
    };

    all_nets.push((start..start + width).collect());
  }

  let mut bits = vec![];
  for nets in all_nets {
    let new_bits: Vec<BitVal> = nets.iter().map(|&n| BitVal::N(n)).collect();
    bits.push(new_bits)
  }

  bits
}

pub fn int_to_attr(x: u32) -> AttributeVal {
  AttributeVal::S(format!("{:032b}", x))
}

pub fn build_netlist(
  cell_type: &str,
  module_name: &'static str,
  parameters: IndexMap<&str, AttributeVal>,
  ports: IndexMap<&str, (PortDirection, usize)>, // width
) -> (Netlist, TopoOrder<'static>) {
  let mut netlist = Netlist::new("test");

  let widths: Vec<usize> = ports.values().map(|(_, w)| *w).collect();
  let nets = allocate_nets(Some(2), &widths);

  let mut actual_parameters = IndexMap::new();
  for (param_name, val) in parameters {
    actual_parameters.insert(param_name.to_string(), val);
  }

  let mut port_directions = IndexMap::new();
  for (port_name, port_info) in &ports {
    port_directions.insert(port_name.to_string(), port_info.0);
  }

  let mut connections = IndexMap::new();
  for (i, port_name) in ports.keys().enumerate() {
    connections.insert(port_name.to_string(), nets[i].clone());
  }

  let cell = Cell {
    cell_type: cell_type.into(),
    parameters: actual_parameters,
    port_directions,
    connections,
    ..Default::default()
  };

  let mut actual_ports = IndexMap::new();
  let mut netnames = IndexMap::new();
  for (i, (port_name, port_info)) in ports.iter().enumerate() {
    let new_port = Port {
      direction: port_info.0,
      bits: nets[i].clone(),
      offset: 0,
      upto: 0,
      signed: 0,
    };
    let netname = Netname {
      bits: nets[i].clone(),
      ..Default::default()
    };
    actual_ports.insert(port_name.to_string(), new_port);
    netnames.insert(port_name.to_string(), netname);
  }

  let module = Module {
    attributes: indexmap! {
      "top".into() => int_to_attr(1)
    },
    ports: actual_ports,
    netnames,
    cells: indexmap! {"$1".into() => cell},
    ..Default::default()
  };

  netlist.modules.insert(module_name.to_string(), module);

  let torder = TopoOrder::from([(module_name, vec!["$1"])]);

  (netlist, torder)
}
