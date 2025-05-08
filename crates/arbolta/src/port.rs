// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::{Bit, BitVec};
use crate::signal::Signal;
use anyhow::Result;
use bincode::{Decode, Encode};
use ndarray::{Array1, ArrayView1};
use num_traits::PrimInt;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::Debug;
use thiserror::Error;
use yosys_netlist_json as yosys;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Encode, Decode)]
pub enum PortDirection {
  Input,
  Output,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Encode, Decode)]
pub struct Port {
  pub direction: PortDirection,
  pub nets: SmallVec<[usize; 512]>,
  pub shape: [usize; 2],
}

#[derive(Debug, Error)]
pub enum PortError {
  #[error("{0}")]
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
  pub fn new(port: &yosys::Port) -> Result<Self> {
    let direction = match port.direction {
      yosys::PortDirection::InOut => Err(PortError::Direction("Inout not supported".to_string()))?,
      yosys::PortDirection::Input => PortDirection::Input,
      yosys::PortDirection::Output => PortDirection::Output,
    };

    let nets: Vec<usize> = port
      .bits
      .iter()
      .map(|bit| match bit {
        yosys::BitVal::N(net) => Ok(*net),
        yosys::BitVal::S(constant) => match constant {
          yosys::SpecialBit::_0 => Ok(0), // Global 0
          yosys::SpecialBit::_1 => Ok(1), // Global 1
          yosys::SpecialBit::X => Err(PortError::Direction("X not supported.".to_string())),
          yosys::SpecialBit::Z => Err(PortError::Direction("Z not supported.".to_string())),
        },
      })
      .collect::<Result<_, _>>()?;

    let shape = [1, nets.len()];

    Ok(Self {
      direction,
      nets: nets.into(),
      shape,
    })
  }

  pub fn set_shape(&mut self, shape: &[usize; 2]) -> Result<()> {
    if shape[0] * shape[1] != self.nets.len() {
      Err(PortError::Shape {
        requested: *shape,
        actual: self.shape,
      })?;
    }

    (self.shape[0], self.shape[1]) = (shape[0], shape[1]);

    Ok(())
  }

  pub fn get_shape(&self) -> [usize; 2] {
    self.shape
  }

  pub fn get_bits(&self, signals: &[Signal]) -> BitVec {
    BitVec::from(
      self
        .nets
        .iter()
        .map(|idx| signals[*idx].get_value())
        .collect::<Vec<Bit>>(),
    )
  }

  pub fn set_bits(&self, vals: &BitVec, signals: &mut [Signal]) -> Result<()> {
    // Should we check direction?
    for (dst, val) in self.nets.iter().zip(vals.bits.iter()) {
      signals[*dst].set_value(*val);
    }

    Ok(())
  }

  pub fn get_int<T: PrimInt + std::ops::BitXorAssign>(&self, signals: &[Signal]) -> T {
    self.get_bits(signals).to_int()
  }

  pub fn set_int<T: PrimInt + std::fmt::Display>(
    &self,
    val: T,
    signals: &mut [Signal],
  ) -> Result<()> {
    // Should we check direction?
    let bits = BitVec::from_int(val)?;

    self.set_bits(&bits, signals)
  }

  pub fn get_int_vec<T: PrimInt + std::ops::BitXorAssign>(&self, signals: &[Signal]) -> Vec<T> {
    let elem_size = self.shape[1];
    self.get_bits(signals).to_ints_sized(elem_size)
  }

  pub fn set_int_vec<T: PrimInt>(&self, vals: &[T], signals: &mut [Signal]) -> Result<()> {
    if vals.len() != self.shape[0] {
      Err(PortError::Shape {
        requested: [vals.len(), std::mem::size_of::<T>() * 8],
        actual: self.shape,
      })?;
    }

    let elem_size = self.shape[1];
    let bits = BitVec::from_ints_sized(vals, elem_size)?;
    self.set_bits(&bits, signals)
  }

  pub fn get_ndarray<T: PrimInt + std::ops::BitXorAssign>(&self, signals: &[Signal]) -> Array1<T> {
    let elem_size = self.shape[1];
    self.get_bits(signals).to_int_ndarray_sized(elem_size)
  }

  pub fn set_ndarray<T: PrimInt>(&self, vals: ArrayView1<T>, signals: &mut [Signal]) -> Result<()> {
    if vals.len() != self.shape[0] {
      Err(PortError::Shape {
        requested: [vals.len(), std::mem::size_of::<T>() * 8],
        actual: self.shape,
      })?;
    }

    let elem_size = self.shape[1];

    let bits = BitVec::from_int_ndarray_sized(vals, elem_size)?;
    self.set_bits(&bits, signals)
  }

  pub fn get_string(&self, signals: &[Signal]) -> String {
    self.get_bits(signals).to_string()
  }
}
