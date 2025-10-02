use anyhow::Result;
use arbolta::{
  bit::{Bit, BitVec},
  hardware_module::HardwareModule,
};
use ndarray::prelude::*;
use ndarray_npy::write_npy;
use ndarray_stats::QuantileExt;
use num_traits::{PrimInt, WrappingAdd, WrappingShl, WrappingSub};

// pub fn worker<A: PrimInt + std::ops::BitXorAssign, B: PrimInt + std::ops::BitXorAssign>(
//   design: &mut HardwareModule,
//   inputs: &ArrayView3<A>,
//   weights: &ArrayView2<B>,
//   net: usize,
//   stuck_val: Bit,
// ) -> Result<()> {
//   design.reset();
//   design.stick_signal(net, stuck_val)?;

//   let preds: Array1<u8> = inputs
//     .axis_iter(Axis(0))
//     .map(|image| {
//       // TODO: Fix hardcoded
//       let x = image.to_shape((1, 28 * 28)).unwrap();
//       let mut logits = Array2::<i32>::zeros((10, 1));

//       run_sa(design, &x.t().view(), &weights.t().view(), &mut logits).unwrap();

//       logits.flatten().argmax().unwrap() as u8
//     })
//     .collect();

//   design.unstick_signal(net)?;

//   write_npy(format!("net_{net}_val_{stuck_val}.npy",), &preds)?;

//   Ok(())
// }

/// Does a: (X_pad, K) @ b: (K, Z_pad) -> (X_pad, Z_pad)
pub fn matmul<A: PrimInt, B: PrimInt, C: PrimInt + WrappingAdd + WrappingShl + WrappingSub>(
  design: &mut HardwareModule,
  cols: usize,
  rows: usize,
  a_pad: ArrayView2<A>,      // (X_pad, K)
  b_pad: ArrayView2<B>,      // (K, Z_pad)
  mut out: ArrayViewMut2<C>, // (X_pad, Z_pad)
) -> Result<()> {
  // Number of tiles in each dimension
  let tiles_x = out.nrows() / cols; // along rows of a/out
  let tiles_z = out.ncols() / rows; // along cols of b/out

  for i in 0..tiles_z {
    let x_block = b_pad.slice(s![.., i * rows..(i + 1) * rows]);

    for j in 0..tiles_x {
      let k_block = a_pad.slice(s![j * cols..(j + 1) * cols, ..]);
      let mut out_block = out.slice_mut(s![j * cols..(j + 1) * cols, i * rows..(i + 1) * rows]);
      run_sa(
        design,
        x_block.view(),
        k_block.t().view(),
        out_block.view_mut(),
      )?;
    }
  }

  Ok(())
}

/// Run inputs through systolic array.
/// Expects x: (K,R), k: (K,C) -> y: (C,R)
#[inline]
pub fn run_sa<A: PrimInt, B: PrimInt, C: PrimInt + WrappingAdd + WrappingShl + WrappingSub>(
  design: &mut HardwareModule,
  x: ArrayView2<A>,        // (K, R)
  k: ArrayView2<B>,        // (K, C)
  mut y: ArrayViewMut2<C>, // (C, R)
) -> Result<()> {
  let iterations: usize = x.shape()[0]; // K
  // TODO: CHECK
  let sx_size = design.get_port_shape("sx_data_i")?[1];
  let sz_size = design.get_port_shape("sk_data_i")?[1];

  design.eval_reset_clocked(Some(1))?;

  // TODO: Check iterations, report failed...
  for i in 0..iterations {
    // SA not ready for inputs
    while design.get_port("s_ready_o")?.bits[0] == Bit::ZERO {
      // TODO: Check some cycle limit
      design.eval_clocked(Some(1))?;
    }

    design.set_port("s_valid_i", [Bit::ONE])?;
    design.set_port(
      "sx_data_i",
      BitVec::from_ints(x.row(i).iter().copied(), Some(sx_size)),
    )?;
    design.set_port(
      "sk_data_i",
      BitVec::from_ints(k.row(i).iter().copied(), Some(sz_size)),
    )?;

    if i == iterations - 1 {
      design.set_port("s_last_i", [Bit::ONE])?;
    }

    design.eval_clocked(Some(1))?;
  }

  design.set_port("m_ready_i", [Bit::ONE])?;
  design.set_port("s_valid_i", [Bit::ZERO])?;
  design.set_port("s_last_i", [Bit::ZERO])?;

  let mut idx = 0;
  loop {
    if design.get_port("m_valid_o")?.bits[0] == Bit::ONE {
      let m_data = design.get_port("m_data_o")?;
      let m_data = m_data.to_ints::<C>(None); // Module should set size
      y.row_mut(idx).assign(&Array1::from_iter(m_data));

      idx += 1;
    }

    if design.get_port("m_last_o")?.bits[0] == Bit::ONE {
      break;
    }

    design.eval_clocked(Some(1))?;
  }

  Ok(())
}
