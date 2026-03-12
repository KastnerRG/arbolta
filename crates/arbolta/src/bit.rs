// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use anyhow::Result;
use derive_more::{BitAnd, BitOr, BitXor, Debug, IntoIterator, Not};
use num_traits::{PrimInt, WrappingAdd, WrappingShl, WrappingSub};
use serde::{Deserialize, Serialize};
use std::{
  convert::{From, Into},
  fmt,
  str::FromStr,
};
use thiserror::Error;

/// Primitive signal value
#[repr(transparent)]
#[derive(
  Debug,
  Clone,
  Eq,
  Copy,
  PartialEq,
  Deserialize,
  derive_more::From,
  derive_more::Into,
  Serialize,
  Default,
  BitAnd,
  BitOr,
  BitXor,
  Not,
)]
pub struct Bit(#[debug("{}", if *_0 {"1"} else {"0"})] pub bool);

#[derive(Debug, PartialEq, Eq, Error)]
#[error("Couldn't convert `{0}` to a Bit, must be 0 or 1")]
pub enum ParseBitError {
  Char(char),
  Int(String), // Easier than using generics
}

impl Bit {
  pub const ZERO: Bit = Bit(false);
  pub const ONE: Bit = Bit(true);

  pub fn to_int<T: PrimInt>(self) -> T {
    match self {
      Self(false) => T::zero(),
      Self(true) => T::one(),
    }
  }

  pub fn from_int<T: PrimInt + fmt::Display>(val: T) -> Result<Self, ParseBitError> {
    if val == T::zero() {
      Ok(Self::ZERO)
    } else if val == T::one() {
      Ok(Self::ONE)
    } else {
      Err(ParseBitError::Int(format!("{val}")))
    }
  }
}

impl TryFrom<char> for Bit {
  type Error = ParseBitError;
  fn try_from(val: char) -> Result<Self, Self::Error> {
    match val {
      '0' => Ok(Bit::ZERO),
      '1' => Ok(Bit::ONE),
      _ => Err(ParseBitError::Char(val)),
    }
  }
}

impl From<&Bit> for Bit {
  fn from(val: &Bit) -> Self {
    *val
  }
}

impl From<&bool> for Bit {
  fn from(val: &bool) -> Self {
    (*val).into()
  }
}

impl From<Bit> for char {
  fn from(bit: Bit) -> Self {
    match bit {
      Bit::ZERO => '0',
      Bit::ONE => '1',
    }
  }
}

impl fmt::Display for Bit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", <Self as Into<char>>::into(*self))
  }
}

/// Structure for storing+manipulating a vector of `Bit`s
#[derive(Debug, PartialEq, Eq, Default, IntoIterator)]
pub struct BitVec {
  #[into_iterator(owned, ref, ref_mut)]
  pub bits: Vec<Bit>,
  pub shape: [usize; 2],
}

impl From<Vec<Bit>> for BitVec {
  fn from(bits: Vec<Bit>) -> Self {
    let shape = [1, bits.len()];
    Self { bits, shape }
  }
}

impl From<BitVec> for Vec<bool> {
  fn from(value: BitVec) -> Self {
    value.bits.iter().map(|&b| b.into()).collect()
  }
}

impl fmt::Display for BitVec {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let bit_string: String = self
      .into_iter()
      .rev()
      .map(|b| <Bit as Into<char>>::into(*b))
      .collect();
    write!(f, "{bit_string}")
  }
}

impl FromStr for BitVec {
  type Err = ParseBitError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut bits: Vec<Bit> = vec![];
    for c in s.chars().rev() {
      bits.push(Bit::try_from(c)?);
    }
    Ok(bits.into())
  }
}

impl TryFrom<&str> for BitVec {
  type Error = ParseBitError;
  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let mut bits: Vec<Bit> = vec![];
    for c in value.chars().rev() {
      bits.push(Bit::try_from(c)?);
    }
    Ok(bits.into())
  }
}

impl From<BitVec> for String {
  fn from(val: BitVec) -> Self {
    val
      .into_iter()
      .rev()
      .map(<Bit as Into<char>>::into)
      .collect()
  }
}

impl FromIterator<bool> for BitVec {
  fn from_iter<I: IntoIterator<Item = bool>>(iter: I) -> Self {
    iter
      .into_iter()
      .map(|b| b.into())
      .collect::<Vec<Bit>>()
      .into()
  }
}

impl BitVec {
  pub fn len(&self) -> usize {
    self.bits.len()
  }

  pub fn is_empty(&self) -> bool {
    self.bits.is_empty()
  }

  /// Create from int.
  ///
  /// # Arguments
  /// * `val` - Int to convert.
  /// * `size` - Number of bits to use. Defaults to `sizeof(T)` * 8.
  pub fn from_int<T: PrimInt>(val: T, size: Option<usize>) -> Self {
    let bit_width = std::mem::size_of::<T>() * 8;
    let size = size.unwrap_or(bit_width);
    // Have to do this since no support for right shift without panic
    let mut bits = (0..size.min(bit_width))
      .map(|n| Bit::from((val >> n) & T::one() == T::one()))
      .collect::<Vec<Bit>>();

    // Have to pad
    if bits.len() < size {
      let is_signed = T::min_value() != T::zero();
      let sign_bit_set: bool = bits.last().is_some_and(|&b| b.into());
      let pad_bit: Bit = (is_signed && sign_bit_set).into();

      let pad_size = size - bits.len();
      bits.extend(std::iter::repeat_n(pad_bit, pad_size));
    }

    bits.into()
  }

  /// Create from iterator of ints.
  ///
  /// # Arguments
  /// * `vals` - Ints to convert.
  /// * `elem_size` - Number of bits per int. Defaults to `sizeof(T)` * 8.
  pub fn from_ints<T, I>(vals: I, elem_size: Option<usize>) -> Self
  where
    T: PrimInt,
    I: IntoIterator<Item = T>,
  {
    let elem_size = elem_size.unwrap_or(std::mem::size_of::<T>() * 8); // bytes to bits
    let bits = vals
      .into_iter()
      .flat_map(|v| Self::from_int(v, Some(elem_size)))
      .collect::<Vec<Bit>>();
    let size = bits.len() / elem_size;
    Self {
      bits,
      shape: [size, elem_size],
    }
  }

  /// Convert to int.
  /// Automatically extends sign if target int type is signed.
  pub fn to_int<T: PrimInt + WrappingAdd + WrappingShl + WrappingSub>(&self) -> T {
    let mut val = T::zero();
    self
      .bits
      .iter()
      .enumerate()
      .for_each(|(i, bit)| val = val.wrapping_add(&(*bit).to_int::<T>().wrapping_shl(i as u32)));

    // Signed int and negative value, need to sign extend
    if T::min_value() != T::zero() && self.bits.last().is_some_and(|&b| b.into()) {
      let mask = !T::one()
        .wrapping_shl(self.bits.len() as u32 - 1)
        .wrapping_sub(&T::one());
      val = val | mask;
    }

    val
  }

  /// Convert to iterator of ints.
  /// Automatically extends sign if target int type is signed.
  ///
  /// # Arguments
  /// * `elem_size` - Number of bits per int. Defaults to `sizeof(T)` * 8.
  pub fn to_ints<T: PrimInt + WrappingAdd + WrappingShl + WrappingSub>(
    &self,
    elem_size: Option<usize>,
  ) -> impl Iterator<Item = T> + '_ {
    // let elem_size = elem_size.unwrap_or(std::mem::size_of::<T>() * 8);
    let elem_size = elem_size.unwrap_or(self.shape[1]);
    self
      .bits
      .chunks(elem_size)
      .map(|chunk| Self::from(chunk.to_vec()).to_int())
  }
}
