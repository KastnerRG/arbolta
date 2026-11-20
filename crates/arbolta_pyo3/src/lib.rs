// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use pyo3::prelude::*;
mod conversion;
mod hardware_design;
mod ports;
mod signals;

#[pymodule]
fn arbolta(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<ports::Ports>()?;
  m.add_class::<hardware_design::HardwareDesign>()?;

  Ok(())
}
