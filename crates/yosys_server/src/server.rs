use super::Synthesis;
use crate::*;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tarpc::context::Context;
use tempfile::TempDir;

#[derive(Clone)]
pub struct SynthesisServer {
  pub yosys_path: PathBuf,
}

fn parse_torder(raw: &str) -> HashMap<String, Vec<String>> {
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

fn generate_synth_command(
  request: &SimpleSynthRequest,
  rtl_path: &Path,
  export_path: &Path,
) -> String {
  // Default to System Verilog
  let mut read_pass = vec![format!(
    "read_verilog -sv {} {} {}",
    if request.config.icells { "-icells" } else { "" },
    if request.config.lib { "-lib" } else { "" },
    if request.config.defer { "-defer" } else { "" },
  )];

  if let Some(defines) = &request.config.defines {
    defines
      .iter()
      .for_each(|d| read_pass.push(format!("-D{d}")));
  }

  read_pass.push(format!("{};", rtl_path.display()));
  let read_pass = read_pass.join(" ");

  let mut hierarchy_pass = vec![];
  if let Some(top_module) = &request.top_module {
    hierarchy_pass.push(format!("hierarchy -top {top_module}"));

    if let Some(parameters) = &request.config.parameters {
      parameters
        .iter()
        .for_each(|(k, v)| hierarchy_pass.push(format!("-chparam {k} {v}")));
    }

    hierarchy_pass.push(";".to_string());
  }
  let hierarchy_pass = hierarchy_pass.join(" ");

  let proc_pass = "proc; clean; autoname; setundef -zero; flatten -scopename;".to_string();
  let synth_pass = if request.config.run_synth {
    format!("synth; {proc_pass}")
  } else {
    "".to_string()
  };
  let export_pass = format!("write_json {}", export_path.display());

  [
    read_pass,
    hierarchy_pass,
    proc_pass,
    synth_pass,
    export_pass,
  ]
  .join(" ")
}

impl Synthesis for SynthesisServer {
  async fn flatten(
    self,
    _context: Context,
    request: FlattenRequest,
  ) -> Result<NetlistResponse, serde_error::Error> {
    let temp_dir = TempDir::new().map_err(|e| serde_error::Error::new(&e))?;

    let original_netlist_path = temp_dir.path().join("original.json");
    let original_netlist_file =
      File::create(&original_netlist_path).map_err(|e| serde_error::Error::new(&e))?;

    let flattened_netlist_path = temp_dir.path().join("flattened.json");
    let flattened_netlist_file =
      File::create_new(&flattened_netlist_path).map_err(|e| serde_error::Error::new(&e))?;

    let topo_order_path = temp_dir.path().join("topo.txt");
    let mut topo_order_file =
      File::create_new(&topo_order_path).map_err(|e| serde_error::Error::new(&e))?;

    // Write old netlist to file
    request
      .netlist
      .to_writer(original_netlist_file)
      .map_err(|e| serde_error::Error::new(&e))?;

    // Run Yosys
    let command = Command::new(self.yosys_path)
      .arg("-f")
      .arg("json")
      .arg(original_netlist_path)
      .arg("-p")
      .arg(format!(
        "flatten -scopename; write_json {}; tee -o {} torder",
        flattened_netlist_path.display(),
        topo_order_path.display()
      ))
      .output()
      .map_err(|e| serde_error::Error::new(&e))?;

    // Yosys failed
    let error_log = String::from_utf8(command.stderr).unwrap();
    if !error_log.is_empty() {
      return Ok(NetlistResponse {
        netlist: None,
        topo_order: None,
        error_log,
      });
    }

    // Load in flattened netlist
    let flattened_netlist = yosys_netlist_json::Netlist::from_reader(flattened_netlist_file)
      .map_err(|e| serde_error::Error::new(&e))?;

    // Load topological cell order
    let mut topo_order = String::new();
    _ = topo_order_file
      .read_to_string(&mut topo_order)
      .map_err(|e| serde_error::Error::new(&e))?;

    let topo_order = parse_torder(&topo_order);

    Ok(NetlistResponse {
      netlist: Some(flattened_netlist),
      topo_order: Some(topo_order),
      error_log,
    })
  }

  // Can't have include unless full path...
  async fn simple_synth(
    self,
    _context: Context,
    request: SimpleSynthRequest,
  ) -> Result<NetlistResponse, serde_error::Error> {
    let temp_dir = TempDir::new().map_err(|e| serde_error::Error::new(&e))?;

    let rtl_path = temp_dir.path().join("design.v");
    let mut rtl_file = File::create(&rtl_path).map_err(|e| serde_error::Error::new(&e))?;

    // Write source to file
    rtl_file
      .write(request.verilog_source.as_bytes())
      .map_err(|e| serde_error::Error::new(&e))?;

    let netlist_path = temp_dir.path().join("flattened.json");
    let netlist_file = File::create_new(&netlist_path).map_err(|e| serde_error::Error::new(&e))?;

    let torder_path = temp_dir.path().join("topo.txt");
    let mut torder_file =
      File::create_new(&torder_path).map_err(|e| serde_error::Error::new(&e))?;

    let passes = generate_synth_command(&request, &rtl_path, &netlist_path);
    // TODO: Hacky, fix this
    let passes = [passes, format!("; tee -o {} torder", torder_path.display())].concat();
    println!("{passes}");

    // Run Yosys
    let command = Command::new(self.yosys_path)
      .arg("-p")
      .arg(passes)
      .output()
      .map_err(|e| serde_error::Error::new(&e))?;

    // Yosys failed
    let error_log = String::from_utf8(command.stderr).unwrap();
    if !error_log.is_empty() {
      return Ok(NetlistResponse {
        netlist: None,
        topo_order: None,
        error_log,
      });
    }

    // Load in netlist
    let netlist = yosys_netlist_json::Netlist::from_reader(netlist_file)
      .map_err(|e| serde_error::Error::new(&e))?;

    // Load topological cell order
    let mut torder = String::new();
    _ = torder_file
      .read_to_string(&mut torder)
      .map_err(|e| serde_error::Error::new(&e))?;

    let torder = parse_torder(&torder);

    Ok(NetlistResponse {
      netlist: Some(netlist),
      topo_order: Some(torder),
      error_log,
    })
  }
}

#[test]
fn test_parse_torder() {
  let raw = r#"

    8. Executing TORDER pass (print cells in topological order).
    module alu
      cell $flatten\i_adder.$add$
      cell $flatten\i_subtracter.$sub$
      cell $procmux$7

    module alu2
      cell $flatten\i_adder.$add$
      cell $flatten\i_subtracter.$sub$
      cell $procmux$7


  "#;
  let torder = parse_torder(raw);
  println!("{torder:#?}");
}
