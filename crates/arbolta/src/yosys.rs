//   cell::CellError,
// use crate::{
// cell::CellError,
// hardware_module::ModuleError,
// port::{PortDirection, parse_bit},
// temp2::{TopoCell, TopoNetMap},
// };
// use std::collections::HashMap;
// use yosys_netlist_json as yosys_json;

// #[derive(Debug, Clone, Eq, PartialEq, Default)]
// pub struct SynthCell<'a> {
//   cell_type: &'a str,
//   parameters: HashMap<&'a str, usize>,
//   // attributes: HashMap<&'a str, &'a str>,
//   port_directions: HashMap<&'a str, PortDirection>,
//   connections: HashMap<&'a str, Box<[usize]>>,
// }

// impl<'a> SynthCell<'a> {
//   pub fn from_yosys(
//     topo_cell: &TopoCell<'a>,
//     global_nets: &TopoNetMap<'a>,
//     netlist: &'a yosys_json::Netlist,
//   ) -> Result<Self, CellError> {
//     let mut cell = Self {
//       cell_type: &topo_cell.cell_type,
//       ..Default::default()
//     };

//     // TODO: error handling
//     let mut synth_module = &netlist.modules[topo_cell.parents.last().unwrap().cell_type];
//     let mut synth_cell = synth_module.cells[topo_cell.name];

//     todo!()
//   }
// }

// impl TryFrom<&yosys_json::Cell> for SynthCell {
//   type Error = ModuleError;
//   fn try_from(cell: &yosys_json::Cell) -> Result<Self, Self::Error> {
//     let mut synth_cell = SynthCell {
//       cell_type: cell.cell_type.to_string(),
//       ..Default::default()
//     };

//     // Add connections and directions
//     for (port_name, conn_bits) in cell.connections.iter() {
//       let direction = PortDirection::try_from(&cell.port_directions[port_name])?;
//       let conn_nets: Vec<usize> = conn_bits
//         .iter()
//         .map(parse_bit)
//         .collect::<Result<Vec<_>, _>>()?;

//       synth_cell
//         .port_directions
//         .insert(port_name.clone(), direction);
//       synth_cell.connections.insert(port_name.clone(), conn_nets);
//     }

//     // Add parameters
//     for (param_name, param) in cell.parameters.iter() {
//       let Some(param) = param.to_number() else {
//         // TODO: Clean this up
//         return Err(CellError::Parameter(param_name.to_string(), param.clone()).into());
//       };
//       synth_cell.parameters.insert(param_name.to_string(), param);
//     }

//     // Add attributes
//     for (attr_name, attr) in cell.attributes.iter() {
//       let Some(attr) = attr.to_string_if_string() else {
//         // TODO: Clean this up
//         return Err(CellError::Attribute(attr_name.to_string(), attr.clone()).into());
//       };
//       synth_cell
//         .attributes
//         .insert(attr_name.to_string(), attr.to_string());
//     }

//     Ok(synth_cell)
//   }
// }

use std::{
  collections::HashMap,
  net::{IpAddr, Ipv6Addr},
  time::Duration,
  {path::PathBuf, process::Command},
};
use tarpc::{client, context, tokio_serde::formats::Json};
use thiserror::Error;
use tokio::time;
use yosys_netlist_json as yosys_json;
pub use yosys_service::*;

#[derive(Debug)]
pub struct YosysClient {
  pub port: u16,
  pub yosys_server_path: PathBuf,
  pub yosys_path: PathBuf,
}

impl Default for YosysClient {
  fn default() -> Self {
    Self {
      port: 8080,
      yosys_server_path: "yosys_server".into(),
      yosys_path: "yosys".into(),
    }
  }
}

#[derive(Debug, Error)]
pub enum YosysError {
  #[error("Server error: {0}")]
  Server(#[from] serde_error::Error),

  #[error("RPC error: {0}")]
  Rpc(#[from] tarpc::client::RpcError),

  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),

  #[error("Yosys error: {0}")]
  Yosys(String),
}

const MAX_RETRIES: u32 = 4;
const RETRY_DELAY_MS: u64 = 1;

impl YosysClient {
  pub async fn flatten_netlist(
    &self,
    top_module: &str,
    netlist: yosys_json::Netlist,
  ) -> Result<(yosys_json::Netlist, HashMap<String, Vec<String>>), YosysError> {
    // Start Yosys server
    let mut yosys = Command::new(&self.yosys_server_path)
      .arg("--port")
      .arg(format!("{}", self.port))
      .arg("--yosys-path")
      .arg(&self.yosys_path)
      .spawn()?;

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), self.port);

    // Keep trying to connect to server
    let mut retries = 0;
    let transport = loop {
      let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
      transport.config_mut().max_frame_length(usize::MAX);

      match transport.await {
        Ok(t) => break t,
        Err(e) => {
          retries += 1;
          eprintln!("Yosys client failed to connect (attempt {retries}): {e}");

          if retries >= MAX_RETRIES {
            return Err(e.into());
          }

          time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
      }
    };

    let request = FlattenRequest {
      top_module: top_module.into(),
      netlist,
    };
    let client = SynthesisClient::new(client::Config::default(), transport).spawn();

    let response = async move {
      tokio::select! {
        response1 = client.flatten(context::current(), request) => {response1}
      }
    }
    .await??;

    // Check Yosys error log
    if !response.error_log.is_empty() {
      return Err(YosysError::Yosys(response.error_log));
    }

    yosys.kill()?; // Kill Yosys server

    Ok((response.netlist.unwrap(), response.topo_order.unwrap()))
  }

  // TODO: De-duplicate code
  pub async fn simple_synth(
    &self,
    verilog_path: &PathBuf,
    top_module: Option<String>,
    config: SynthConfig,
  ) -> Result<(yosys_json::Netlist, HashMap<String, Vec<String>>), YosysError> {
    let verilog_source = std::fs::read_to_string(verilog_path)?;

    // Try to start Yosys server, should fail if already started
    // TODO: Clean this up
    let mut _yosys = Command::new(&self.yosys_server_path)
      .arg("--port")
      .arg(format!("{}", self.port))
      .arg("--yosys-path")
      .arg(&self.yosys_path)
      .spawn();

    let server_addr = (IpAddr::V6(Ipv6Addr::LOCALHOST), self.port);

    // Keep trying to connect to server
    let mut retries = 0;
    let transport = loop {
      let mut transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
      transport.config_mut().max_frame_length(usize::MAX);

      match transport.await {
        Ok(t) => break t,
        Err(e) => {
          retries += 1;
          eprintln!("Yosys client failed to connect (attempt {retries}): {e}",);

          if retries >= MAX_RETRIES {
            return Err(e.into());
          }

          time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
      }
    };

    let request = SimpleSynthRequest {
      verilog_source,
      top_module,
      config,
    };

    let client = SynthesisClient::new(client::Config::default(), transport).spawn();

    let response = async move {
      tokio::select! {
        response1 = client.simple_synth(context::current(), request) => {response1}
      }
    }
    .await??;

    // Check Yosys error log
    if !response.error_log.is_empty() {
      return Err(YosysError::Yosys(response.error_log));
    }

    // yosys.kill()?; // Kill Yosys server

    Ok((response.netlist.unwrap(), response.topo_order.unwrap()))
  }
}

// TODO: De-duplicate with yosys_server
/// Parse raw Yosys topological order output
pub fn parse_torder(raw: &str) -> HashMap<String, Vec<String>> {
  let mut torder: HashMap<String, Vec<String>> = HashMap::new();
  let mut current_module: Option<&str> = None; // If Some, save cells

  for line in raw.lines() {
    let line = line.trim();

    // Start new module
    if let Some(module_name) = line.strip_prefix("module ") {
      current_module = Some(module_name)
    } else if let Some(module_name) = current_module
      && let Some(cell_name) = line.strip_prefix("cell ")
    {
      torder
        .entry(module_name.to_string())
        .or_default()
        .push(cell_name.to_string());
    }
  }

  torder
}

// Re-export
pub use yosys_json::Netlist;
