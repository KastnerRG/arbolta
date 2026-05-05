// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Connection between cells/modules and related statistics.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Signals {
  // Total number of nets
  pub size: usize,
  /// Value of nets
  nets: Box<[Bit]>,
  /// Signals have been modified
  dirty: bool,
  // Is net constant
  constant: Box<[bool]>,
  /// Number of times net has transitioned from 0 -> 1
  toggles_rising: Box<[u64]>,
  /// Number of times net has transitioned from 1 -> 0
  toggles_falling: Box<[u64]>,
}

#[derive(Debug, Error)]
pub enum SignalError {
  #[error("Cannot set constant for net `{0}`")]
  SetConstant(usize),
  #[error("Cannot unset constant for net `{0}`")]
  UnsetConstant(usize),
}

// pub type Net = usize;
impl Signals {
  pub const NET_CONST0: usize = 0;
  pub const NET_CONST1: usize = 1;

  pub fn new(size: usize) -> Self {
    let actual_size = size + 2; // +2 for constant 0 and 1
    let mut nets = vec![Bit::ZERO; actual_size];
    let mut constant = vec![false; actual_size];

    // Add 0, 1 constants
    (nets[Self::NET_CONST0], constant[Self::NET_CONST0]) = (Bit::ZERO, true);
    (nets[Self::NET_CONST1], constant[Self::NET_CONST1]) = (Bit::ONE, true);

    Self {
      size: actual_size,
      dirty: false,
      nets: nets.into(),
      constant: constant.into(),
      toggles_rising: vec![0; actual_size].into(),
      toggles_falling: vec![0; actual_size].into(),
    }
  }

  /// Set value of net. Updates toggle statistics.
  /// # Arguments
  /// * `net` - Selected signal net.
  /// * `val` - New `Bit` value to change `net` to.
  #[inline]
  pub fn set_net(&mut self, net: usize, val: Bit) {
    // Constant or unchanged, do nothing
    if self.constant[net] || self.nets[net] == val {
      return;
    }

    match val {
      Bit::ONE => self.toggles_rising[net] += 1,
      Bit::ZERO => self.toggles_falling[net] += 1,
    }

    self.dirty = true;
    self.nets[net] = val;
  }

  /// Get value of net.
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn get_net(&self, net: usize) -> Bit {
    self.nets[net]
  }

  /// Make net constant.
  /// Net cannot be updated until calling `unset_constant`.
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  /// * `val` - Constant `Bit` value.
  #[inline]
  pub fn set_constant(&mut self, net: usize, val: Bit) -> Result<(), SignalError> {
    if net == Self::NET_CONST0 || net == Self::NET_CONST1 {
      Err(SignalError::SetConstant(net))
    } else {
      self.constant[net] = true;
      self.nets[net] = val;

      Ok(())
    }
  }

  pub fn is_constant(&self, net: usize) -> bool {
    self.constant[net]
  }

  /// Make net modifiable.
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn unset_constant(&mut self, net: usize) -> Result<(), SignalError> {
    if net == Self::NET_CONST0 || net == Self::NET_CONST1 {
      Err(SignalError::UnsetConstant(net))
    } else {
      self.constant[net] = false;

      Ok(())
    }
  }

  #[inline]
  pub fn is_dirty(&self) -> bool {
    self.dirty
  }

  #[inline]
  pub fn clear_dirty(&mut self) {
    self.dirty = false
  }

  /// Reset all nets to `Bit::ZERO` and clear statistics.
  pub fn reset(&mut self) {
    self.dirty = false;
    self.nets.iter_mut().for_each(|n| *n = Bit::ZERO);
    self.constant.iter_mut().for_each(|c| *c = false);
    self.toggles_rising.iter_mut().for_each(|t| *t = 0);
    self.toggles_falling.iter_mut().for_each(|t| *t = 0);

    // Add 0, 1 constants
    (self.nets[Self::NET_CONST0], self.constant[Self::NET_CONST0]) = (Bit::ZERO, true);
    (self.nets[Self::NET_CONST1], self.constant[Self::NET_CONST1]) = (Bit::ONE, true);
  }

  /// Total number times `net` has been toggled since last reset.
  /// Includes both rising (0->1) and falling (1->0).
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn get_toggles_total(&self, net: usize) -> u64 {
    self.toggles_falling[net] + self.toggles_rising[net]
  }

  /// Total number times `net` has been toggled from 0 -> 1.
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn get_toggles_rising(&self, net: usize) -> u64 {
    self.toggles_rising[net]
  }

  /// Total number times `net` has been toggled from 1 -> 0.
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn get_toggles_falling(&self, net: usize) -> u64 {
    self.toggles_falling[net]
  }
}
