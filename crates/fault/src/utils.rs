use anyhow::{Ok, Result};
use arbolta::hardware_module::HardwareModule;
use ndarray::{Array2, ArrayView2};
use num_traits::PrimInt;
use std::fmt::Debug;

/// Run inputs through systolic array.
/// Expects x: (K,R), k: (K,C) -> y: (C,R)
pub fn run_sa<
  A: PrimInt + std::ops::BitXorAssign,
  B: PrimInt + std::ops::BitXorAssign,
  C: PrimInt + std::ops::BitXorAssign + Debug,
>(
  design: &mut HardwareModule,
  x: &ArrayView2<A>,
  k: &ArrayView2<B>,
  y: &mut Array2<C>,
) -> Result<()> {
  let iterations: usize = x.shape()[0]; // K

  design.eval_reset_clocked(Some(1))?;
  for i in 0..iterations {
    while design.get_port_int::<u32>("s_ready_o")? == 0 {
      design.eval_clocked(Some(1))?;
    }
    design.set_port_int("s_valid_i", 1_u32)?;
    design.set_port_ndarray("sx_data_i", x.row(i))?;
    design.set_port_ndarray("sk_data_i", k.row(i))?;

    if i == iterations - 1 {
      design.set_port_int("s_last_i", 1_u32)?;
    }

    design.eval_clocked(Some(1))?;
  }

  design.set_port_int("m_ready_i", 1_u32)?;
  design.set_port_int("s_valid_i", 0_u32)?;
  design.set_port_int("s_last_i", 0_u32)?;

  let mut idx = 0;
  loop {
    if design.get_port_int::<u32>("m_valid_o")? == 1 {
      let m_data = design.get_port_ndarray::<C>("m_data_o")?;
      m_data.assign_to(y.row_mut(idx));
      idx += 1;
    }

    if design.get_port_int::<u32>("m_last_o")? == 1 {
      break;
    }
    design.eval_clocked(Some(1))?;
  }

  Ok(())
}
