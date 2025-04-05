// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use crate::bit::Bit;
use serde::{Deserialize, Serialize};

/// Connection between cells/modules.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Signal {
  /// Value of net
  pub value: Bit,
  // Is signal constant
  pub constant: bool,
  /// Number of times net has transitioned from 0 -> 1
  pub toggle_count_rising: usize,
  /// Number of times net has transitioned from 1 -> 0
  pub toggle_count_falling: usize,
}

impl Signal {
  /// Create a new constant.
  ///
  /// # Arguments
  /// * `value` - Constant `Bit` value.
  pub fn new_constant(value: Bit) -> Self {
    Self {
      value,
      constant: true,
      ..Default::default()
    }
  }

  pub fn set_constant(&mut self, value: Bit) {
    self.constant = true;
    self.value = value;
    self.toggle_count_falling = 0;
    self.toggle_count_rising = 0;
  }

  /// Reset signal value to zero.
  /// Clear all signal statistics.
  pub fn reset(&mut self) {
    // Ignore constants
    // if !self.constant {
    // self.value = Bit::Zero
    // }
    self.constant = false;
    self.value = Bit::Zero;
    self.toggle_count_falling = 0;
    self.toggle_count_rising = 0;
  }

  /// Get value of signal.
  pub fn get_value(&self) -> Bit {
    self.value
  }

  /// Set value of signal. Updates toggle statistics.
  pub fn set_value(&mut self, val: Bit) {
    if !self.constant {
      match &[self.value, val] {
        [Bit::Zero, Bit::One] => self.toggle_count_rising += 1,
        [Bit::One, Bit::Zero] => self.toggle_count_falling += 1,
        [Bit::Zero, Bit::Zero] | [Bit::One, Bit::One] => return,
      }
      self.value = val;
    }
  }

  /// Get total signal toggle count (rising + falling).
  pub fn get_total_toggle_count(&self) -> usize {
    self.toggle_count_falling + self.toggle_count_rising
  }

  /// Get total rising toggle count of signal.
  pub fn get_toggle_count_rising(&self) -> usize {
    self.toggle_count_rising
  }

  /// Get total falling toggle count of signal.
  pub fn get_toggle_count_falling(&self) -> usize {
    self.toggle_count_falling
  }
}
