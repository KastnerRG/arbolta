// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::{signal::Signals, yosys};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Deserialize, Serialize)]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Port {
  pub direction: PortDirection,
  pub shape: [usize; 2], // TODO: Change this to option?
}

/// Parse global net from `BitVal`.
/// Errors if bit direction is not supported.
pub fn parse_bit(bit: &yosys::BitVal) -> Result<usize, PortError> {
  match bit {
    yosys::BitVal::N(net) => Ok(*net),
    yosys::BitVal::S(constant) => match constant {
      yosys::SpecialBit::_0 => Ok(Signals::NET_CONST0), // Global 0
      yosys::SpecialBit::_1 => Ok(Signals::NET_CONST1), // Global 1
      yosys::SpecialBit::X => Err(PortError::Direction("X".to_string())),
      yosys::SpecialBit::Z => Err(PortError::Direction("Z".to_string())),
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

impl TryFrom<&yosys::PortDirection> for PortDirection {
  type Error = PortError;
  fn try_from(direction: &yosys::PortDirection) -> Result<Self, Self::Error> {
    match direction {
      yosys::PortDirection::InOut => Err(PortError::Direction("inout".to_string())),
      yosys::PortDirection::Input => Ok(PortDirection::Input),
      yosys::PortDirection::Output => Ok(PortDirection::Output),
    }
  }
}

impl TryFrom<&yosys::Port> for Port {
  type Error = PortError;
  fn try_from(port: &yosys::Port) -> Result<Self, Self::Error> {
    let direction = PortDirection::try_from(&port.direction)?;
    let shape = [1, port.bits.len()];

    Ok(Self { direction, shape })
  }
}

impl Port {
  pub fn set_shape(&mut self, shape: &[usize; 2]) -> Result<(), PortError> {
    if shape[0] * shape[1] != self.shape[0] * self.shape[1] {
      Err(PortError::Shape {
        requested: *shape,
        actual: self.shape,
      })
    } else {
      (self.shape[0], self.shape[1]) = (shape[0], shape[1]);

      Ok(())
    }
  }

  pub fn get_shape(&self) -> [usize; 2] {
    self.shape
  }
}
