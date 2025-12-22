// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::BitVec;
use num_traits::{PrimInt, WrappingAdd, WrappingShl, WrappingSub};
use numpy::{PyArrayMethods, PyReadonlyArray1, PyReadwriteArray1};
use pyo3::prelude::*;

pub fn bits_to_bool_numpy(bits: &BitVec, numpy_array: &Bound<'_, PyAny>) -> PyResult<()> {
  let mut buffer = numpy_array.extract::<PyReadwriteArray1<bool>>()?;

  bits
    .bits
    .iter()
    .zip(buffer.as_array_mut().iter_mut())
    .for_each(|(&bit, buf)| *buf = bit.0);

  Ok(())
}

pub fn bool_numpy_to_bits(numpy_array: &Bound<'_, PyAny>) -> PyResult<BitVec> {
  let buffer = numpy_array.extract::<PyReadonlyArray1<bool>>()?;
  Ok(BitVec::from_iter(buffer.to_owned_array()))
}

pub fn bits_to_int_numpy<T: PrimInt + WrappingAdd + WrappingShl + WrappingSub + numpy::Element>(
  bits: &BitVec,
  elem_size: usize,
  numpy_array: &Bound<'_, PyAny>,
) -> PyResult<()> {
  let mut buffer = numpy_array.extract::<PyReadwriteArray1<T>>()?;

  bits
    .to_ints(Some(elem_size))
    .zip(buffer.as_array_mut().iter_mut())
    .for_each(|(src, dst)| *dst = src);

  Ok(())
}

pub fn int_numpy_to_bits<T: PrimInt + numpy::Element>(
  numpy_array: &Bound<'_, PyAny>,
  elem_size: usize,
) -> PyResult<BitVec> {
  let buffer = numpy_array.extract::<PyReadonlyArray1<T>>()?;
  Ok(BitVec::from_ints(buffer.to_owned_array(), Some(elem_size)))
}
