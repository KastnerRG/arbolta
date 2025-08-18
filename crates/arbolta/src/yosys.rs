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
