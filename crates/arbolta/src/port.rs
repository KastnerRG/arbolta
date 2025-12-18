// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::yosys;
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
      yosys::SpecialBit::_0 => Ok(0), // Global 0
      yosys::SpecialBit::_1 => Ok(1), // Global 1
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
  // TODO: Remove and use try_from
  // pub fn new(port: &yosys_json::Port) -> Result<Self, PortError> {
  //   let direction = PortDirection::try_from(&port.direction)?;
  //   let nets: Vec<usize> = port.bits.iter().map(parse_bit).collect::<Result<_, _>>()?;
  //   let shape = [1, nets.len()];

  //   Ok(Self {
  //     direction,
  //     nets: nets.into(),
  //     shape,
  //   })
  // }

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
