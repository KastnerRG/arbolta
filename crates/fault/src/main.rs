use anyhow::{Context, Ok, Result};
use arbolta::bit::Bit;
use arbolta::hardware_module::HardwareModule;
use clap::Parser;
use fault::utils::run_sa;
use indicatif::{ParallelProgressIterator, ProgressIterator};
use ndarray::parallel::prelude::*;
use ndarray::{Array1, Array2, Array3, Axis};
use ndarray_npy::read_npy;
use ndarray_stats::QuantileExt;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Parser, Debug)]
struct Args {
  /// Name of top module in netlist.
  #[clap(long)]
  top_module: String,
  /// Path to Yosys netlist JSON file.
  #[clap(long)]
  netlist: String,
  /// Path to NumPy inputs file.
  #[clap(long)]
  inputs: PathBuf,
  /// Path to NumPy weights file.
  #[clap(long)]
  weights: PathBuf,
  /// Path to NumPy targets file.
  #[clap(long)]
  targets: PathBuf,
  #[clap(long)]
  rows: usize,
  #[clap(long)]
  cols: usize,
  #[clap(long)]
  iterations: usize,
  #[clap(long)]
  sx_size: usize,
  #[clap(long)]
  sk_size: usize,
  #[clap(long)]
  m_size: usize,
}

fn main() -> Result<()> {
  let flags = Args::parse();

  // Load everything
  let mut design = HardwareModule::new_from_path(&flags.netlist, &flags.top_module)?;
  let inputs: Array3<u8> = read_npy(&flags.inputs)
    .with_context(|| format!("Failed to read NumPy array from {:?}", flags.inputs))?; // (N, 28, 28)
  let weights: Array2<u8> = read_npy(&flags.weights)
    .with_context(|| format!("Failed to read NumPy array from {:?}", flags.weights))?; // (10, 784)
  let targets: Array2<u8> = read_npy(&flags.targets)
    .with_context(|| format!("Failed to read NumPy array from {:?}", flags.targets))?;

  // Setup designs
  design.set_clock(2, Bit::ONE)?;
  design.set_reset(3, Bit::ZERO)?;
  design.set_port_shape("sx_data_i", &[flags.rows, flags.sx_size])?;
  design.set_port_shape("sk_data_i", &[flags.cols, flags.sk_size])?;
  design.set_port_shape("m_data_o", &[1, flags.m_size])?;

  // let mut sa = SystolicArray::new(design);

  // let preds: Vec<u8> = inputs
  //   .axis_iter(Axis(0))
  //   // .progress()
  //   .map(|image| {
  //     let x = image.to_shape((1, flags.iterations)).unwrap();
  //     let mut logits = Array2::<i32>::zeros((10, 1));
  //     sa.run_matmul(&x.t().view(), &weights.t().view(), &mut logits)
  //       .unwrap();

  //     logits.flatten().argmax().unwrap() as u8
  //   })
  //   .collect();

  // let image = inputs.index_axis(Axis(0), 0);
  // let image = image.to_shape((1, flags.iterations)).unwrap();
  // let mut logits = Array2::<i32>::zeros((10, 1));
  // sa.run_matmul(&image.t().view(), &weights.t().view(), &mut logits)?;
  // println!("{logits}");

  let designs: Vec<Mutex<HardwareModule>> = (0..rayon::current_num_threads())
    .map(|_| Mutex::new(design.clone()))
    .collect();

  // Start simulation
  let preds: Vec<u8> = inputs
    .axis_iter(Axis(0))
    .into_par_iter()
    // .progress()
    .map(|image| {
      let x = image.to_shape((1, flags.iterations)).unwrap();
      let mut logits = Array2::<i32>::zeros((10, 1));

      let thread_idx = rayon::current_thread_index().unwrap();
      // let thread_idx = 0;
      let design = &mut designs[thread_idx].lock().unwrap();
      run_sa(design, &x.t().view(), &weights.t().view(), &mut logits).unwrap();

      logits.flatten().argmax().unwrap() as u8
    })
    .collect();

  let preds: Array1<u8> = preds.into();
  let temp: Array1<u8> = targets.into_flat();
  assert_eq!(preds, temp);

  Ok(())
}
