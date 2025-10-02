// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use yosys_netlist_json as yosys_json;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Encode, Decode)]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Encode, Decode)]
pub struct Port {
  pub direction: PortDirection,
  pub nets: Box<[usize]>,
  pub shape: [usize; 2],
}

/// Parse global net from `BitVal`.
/// Errors if bit direction is not supported.
pub fn parse_bit(bit: &yosys_json::BitVal) -> Result<usize, PortError> {
  match bit {
    yosys_json::BitVal::N(net) => Ok(*net),
    yosys_json::BitVal::S(constant) => match constant {
      yosys_json::SpecialBit::_0 => Ok(0), // Global 0
      yosys_json::SpecialBit::_1 => Ok(1), // Global 1
      yosys_json::SpecialBit::X => Err(PortError::Direction("X".to_string())),
      yosys_json::SpecialBit::Z => Err(PortError::Direction("Z".to_string())),
    },
  }
}

#[derive(Debug, Error)]
pub enum PortError {
  #[error("Direction `{0}` not supported")]
  Direction(String),
  #[error("couldn't convert port to type")]
  Conversion,
  #[error("incompatible shapes: requested={requested:?}, actual={actual:?}")]
  Shape {
    requested: [usize; 2],
    actual: [usize; 2],
  },
}

impl Port {
  pub fn new(port: &yosys_json::Port) -> Result<Self, PortError> {
    let direction = match port.direction {
      yosys_json::PortDirection::InOut => Err(PortError::Direction("inout".to_string()))?,
      yosys_json::PortDirection::Input => PortDirection::Input,
      yosys_json::PortDirection::Output => PortDirection::Output,
    };

    let nets: Vec<usize> = port.bits.iter().map(parse_bit).collect::<Result<_, _>>()?;
    let shape = [1, nets.len()];

    Ok(Self {
      direction,
      nets: nets.into(),
      shape,
    })
  }

  pub fn set_shape(&mut self, shape: &[usize; 2]) -> Result<(), PortError> {
    if shape[0] * shape[1] != self.nets.len() {
      return Err(PortError::Shape {
        requested: *shape,
        actual: self.shape,
      });
    }

    (self.shape[0], self.shape[1]) = (shape[0], shape[1]);

    Ok(())
  }

  pub fn get_shape(&self) -> [usize; 2] {
    self.shape
  }
}
