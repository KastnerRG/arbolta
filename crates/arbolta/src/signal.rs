// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Connection between cells/modules and related statistics.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default, Encode, Decode)]
pub struct Signals {
  // Total number of nets
  pub size: usize,
  /// Value of nets
  pub nets: Box<[Bit]>, // TODO: Make this private
  // Is net constant
  constant: Box<[bool]>,
  /// Number of times net has transitioned from 0 -> 1
  toggles_rising: Box<[u64]>,
  /// Number of times net has transitioned from 1 -> 0
  toggles_falling: Box<[u64]>,
}

impl Signals {
  pub fn new(size: usize) -> Self {
    Self {
      size,
      nets: vec![Bit::ZERO; size].into(),
      constant: vec![false; size].into(),
      toggles_rising: vec![0; size].into(),
      toggles_falling: vec![0; size].into(),
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
  pub fn set_constant(&mut self, net: usize, val: Bit) {
    self.constant[net] = true;
    self.nets[net] = val;
  }

  /// Make net modifiable.
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn unset_constant(&mut self, net: usize) {
    self.constant[net] = false;
  }

  /// Reset all nets to `Bit::ZERO` and clear statistics.
  pub fn reset(&mut self) {
    self.nets.iter_mut().for_each(|n| *n = Bit::ZERO);
    self.constant.iter_mut().for_each(|c| *c = false);
    self.toggles_rising.iter_mut().for_each(|t| *t = 0);
    self.toggles_falling.iter_mut().for_each(|t| *t = 0);
  }

  /// Total number times `net` has been toggled since last reset.
  /// Includes both rising (0->1) and falling (1->0).
  ///
  /// # Arguments
  /// * `net` - Selected signal net.
  #[inline]
  pub fn get_total_toggles(&self, net: usize) -> u64 {
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
