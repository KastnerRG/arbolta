// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

// Re-export
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

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
    let mut name_clean = remove_whitespace(name);
    let parents_clean: Vec<String> = parents.iter().map(|p| remove_whitespace(p)).collect();

    if !parents_clean.is_empty() {
      let name_stripped = name_clean.strip_prefix("$flatten\\").unwrap_or(&name_clean);
      let parents_prefix = format!("{}.", parents_clean.join("."));

      if let Some(actual_name) = name_stripped.strip_prefix(&parents_prefix) {
        name_clean = actual_name.to_string();
      } else {
        name_clean = name_stripped.to_string();
      }
    }

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

pub fn parse_torder<'a>(raw: &'a str) -> TopoOrder<'a> {
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

  torder
}

// Re-export
pub use yosys_netlist_json::{
  AttributeVal, BitVal, Cell, Module, Netlist, Netname, Port, PortDirection, SpecialBit,
};

// TODO: Maybe re-add Yosys bindings...
