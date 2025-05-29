use anyhow::{Ok, Result};
use arbolta::{bit::Bit, hardware_module::HardwareModule};
use ndarray::{Array1, Array2, ArrayView2, ArrayView3, Axis};
use ndarray_npy::write_npy;
use ndarray_stats::QuantileExt;
use num_traits::PrimInt;
use std::fmt::Debug;

pub fn worker<A: PrimInt + std::ops::BitXorAssign, B: PrimInt + std::ops::BitXorAssign>(
  design: &mut HardwareModule,
  inputs: &ArrayView3<A>,
  weights: &ArrayView2<B>,
  net: usize,
  stuck_val: Bit,
) -> Result<()> {
  design.reset();
  design.stick_signal(net, stuck_val)?;

  let preds: Array1<u8> = inputs
    .axis_iter(Axis(0))
    .map(|image| {
      // TODO: Fix hardcoded
      let x = image.to_shape((1, 28 * 28)).unwrap();
      let mut logits = Array2::<i32>::zeros((10, 1));

      run_sa(design, &x.t().view(), &weights.t().view(), &mut logits).unwrap();

      logits.flatten().argmax().unwrap() as u8
    })
    .collect();

  design.unstick_signal(net)?;

  write_npy(format!("net_{net}_val_{stuck_val}.npy",), &preds)?;

  Ok(())
}

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
