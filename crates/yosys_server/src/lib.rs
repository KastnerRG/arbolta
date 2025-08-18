pub mod server;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yosys_netlist_json::Netlist;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FlattenRequest {
  pub top_module: String,
  pub netlist: Netlist,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SynthConfig {
  pub run_synth: bool,                             // if false, just run proc
  pub icells: bool,                                // -icells
  pub lib: bool,                                   // -lib
  pub defer: bool,                                 // -defer
  pub defines: Option<Vec<String>>,                // -Dname[=definition]
  pub parameters: Option<HashMap<String, String>>, // -chparam .. (if top module given)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SimpleSynthRequest {
  pub verilog_source: String,
  pub top_module: Option<String>, // Hierarchy top if set
  pub config: SynthConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetlistResponse {
  pub netlist: Option<Netlist>,
  pub topo_order: Option<HashMap<String, Vec<String>>>,
  pub error_log: String,
}

#[tarpc::service]
pub trait Synthesis {
  async fn flatten(request: FlattenRequest) -> Result<NetlistResponse, serde_error::Error>;
  async fn simple_synth(request: SimpleSynthRequest)
  -> Result<NetlistResponse, serde_error::Error>;
}
