// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

// Re-export
use serde::{Deserialize, Serialize};
use std::{
  fs::File,
  io::BufWriter,
  process::Command,
  {collections::HashMap, fmt, str},
};

fn remove_whitespace<S: AsRef<str>>(s: &S) -> String {
  s.as_ref().chars().filter(|c| !c.is_whitespace()).collect()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct RTLID {
  pub parents: Vec<String>,
  pub name: String,
}

impl RTLID {
  pub fn new<S: AsRef<str>>(parents: &[S], name: &S) -> Self {
    let name_clean = remove_whitespace(name);

    let parents_clean: Vec<String> = parents
      .iter()
      .map(|p| {
        let clean_p = remove_whitespace(p).replace("\\", "");
        clean_p
          .strip_prefix("$flatten")
          .unwrap_or(&clean_p)
          .to_string()
      })
      .collect();

    Self {
      parents: parents_clean,
      name: name_clean,
    }
  }
}

impl fmt::Display for RTLID {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.parents.is_empty() {
      write!(f, "{}", self.name)
    } else {
      write!(f, "{}.{}", self.parents.join("."), self.name)
    }
  }
}

pub type TopoOrder<'a> = HashMap<&'a str, Vec<&'a str>>;

pub fn parse_torder<'a, T>(raw: &'a T) -> Result<TopoOrder<'a>, str::Utf8Error>
where
  T: AsRef<[u8]> + ?Sized,
{
  let raw = str::from_utf8(raw.as_ref())?;
  let mut torder = TopoOrder::new();
  let mut current_module: Option<&'a str> = None;

  for line in raw.lines() {
    let line = line.trim();

    // Start new module
    if let Some(module_name) = line.strip_prefix("module ") {
      let module_name = if let Some((_cell_type, module_name)) = module_name.split_once("$") {
        module_name
      } else {
        module_name
      };

      current_module = Some(module_name);
      torder.insert(module_name, vec![]);
    } else if let Some(module_path) = current_module
      && let Some(cell_name) = line.strip_prefix("cell ")
    {
      torder.entry(module_path).or_default().push(cell_name);
    }
  }

  Ok(torder)
}

pub fn run_torder(netlist: &Netlist) -> anyhow::Result<String> {
  // Create temporary directory
  let temp_dir = tempfile::tempdir()?;

  // Write netlist to file in temp dir
  let netlist_path = temp_dir.path().join("netlist.json");
  let netlist_file = File::create(&netlist_path)?;
  netlist.to_writer(BufWriter::new(netlist_file))?;

  let torder_path = temp_dir.path().join("torder.txt");

  let passes = [
    format!("read_json {}", netlist_path.display()),
    format!("tee -o {} torder", torder_path.display()),
  ]
  .join("; ");

  // Run Yosys
  let command = Command::new("yosys").arg("-p").arg(passes).output()?;

  // Check if Yosys failed
  if !command.stderr.is_empty() {
    let error_log = String::from_utf8(command.stderr)?;
    anyhow::bail!(error_log)
  }

  // Load raw topological cell order
  let torder = std::fs::read_to_string(torder_path)?;

  Ok(torder)
}

// Re-export
pub use yosys_netlist_json::{
  AttributeVal, BitVal, Cell, Module, Netlist, Netname, Port, PortDirection, SpecialBit,
};
