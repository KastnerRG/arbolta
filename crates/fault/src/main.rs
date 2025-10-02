use anyhow::{Result, anyhow, ensure};
use arbolta::{
  bit::Bit,
  hardware_module::HardwareModule,
  yosys::{Netlist, parse_torder},
};
use clap::Parser;
use fault::utils::matmul;
// use indicatif::ParallelProgressIterator;
use ndarray::prelude::*; //parallel::prelude::*
use ndarray_npy::read_npy;
use std::path::PathBuf;
// use std::sync::Mutex;

#[derive(Parser, Debug)]
struct Args {
  /// Name of top module in netlist.
  #[clap(long)]
  top_module: String,
  /// Path to topological cell order file.
  #[clap(long)]
  torder_path: String,
  /// Path to Yosys netlist JSON file.
  #[clap(long)]
  netlist_path: String,
  /// Path to NumPy file for matrix A.
  #[clap(long)]
  a_path: PathBuf,
  /// Path to NumPy file for matrix B.
  #[clap(long)]
  b_path: PathBuf,
  /// Path to NumPy targets file.
  // #[clap(long)]
  // targets: PathBuf,
  /// Path to list of nets for fault injection campaign.
  // #[clap(long)]
  // nets: PathBuf,
  #[clap(long)]
  rows: usize,
  #[clap(long)]
  cols: usize,
  // #[clap(long)]
  // iterations: usize,
  #[clap(long)]
  x_width: usize,
  #[clap(long)]
  k_width: usize,
  #[clap(long)]
  m_width: usize,
}

// TODO: Move setup to generic function and do dynamic dispatch here...
#[allow(non_snake_case)]
fn main() -> Result<()> {
  let flags = Args::parse();

  // Load and parse netlist and topological cell order
  let raw_netlist = std::fs::read(flags.netlist_path)?;
  let raw_torder = std::fs::read_to_string(flags.torder_path)?;
  let netlist = Netlist::from_slice(&raw_netlist)?;
  let torder = parse_torder(&raw_torder);

  // Setup design
  let (ROWS, COLS) = (flags.rows, flags.cols); // Copy for easier use

  let mut design = HardwareModule::new(&netlist, &torder, &flags.top_module)?;
  design.set_clock(2, Bit::ONE)?;
  design.set_reset(3, Bit::ZERO)?;
  design.set_port_shape("sx_data_i", &[ROWS, flags.x_width])?;
  design.set_port_shape("sk_data_i", &[COLS, flags.k_width])?;
  design.set_port_shape("m_data_o", &[ROWS, flags.m_width])?;

  // Load and process inputs
  let a: Array2<i8> = read_npy(flags.a_path)?;
  let b: Array2<i8> = read_npy(flags.b_path)?;

  // a: (X, K), b: (K, Z)
  let (X, K, Z) = (a.nrows(), a.ncols(), b.ncols());
  ensure!(K == b.nrows(), anyhow!("{} != {}", K, b.nrows()));

  // Pad `a` and `b` for alignment with systolic array
  let (out_x, out_z) = (X.div_ceil(COLS) * COLS, (Z.div_ceil(ROWS) * ROWS));

  // TODO: May have to adjust for Array3
  let mut a_pad = Array2::<i8>::zeros((out_x, K));
  let mut b_pad = Array2::<i8>::zeros((K, out_z));
  a_pad.slice_mut(s![0..X, 0..K]).assign(&a.view());
  b_pad.slice_mut(s![0..K, 0..Z]).assign(&b.view());

  // Setup padded output array (must crop later)
  let mut out = Array2::<i32>::zeros((out_x, out_z));
  matmul(
    &mut design,
    COLS,
    ROWS,
    a_pad.view(),
    b_pad.view(),
    out.view_mut(),
  )?;

  let temp = out.slice(s![0..X, 0..Z]).to_owned();
  println!("{temp}");

  /*
  // Load everything
  let mut design = HardwareModule::new_from_path(&flags.netlist, &flags.top_module)?;
  let inputs: Array3<u8> = read_npy(&flags.inputs)
    .with_context(|| format!("Failed to read NumPy array from {:?}", flags.inputs))?; // (N, 28, 28)
  let weights: Array2<u8> = read_npy(&flags.weights)
    .with_context(|| format!("Failed to read NumPy array from {:?}", flags.weights))?; // (10, 784)
  // let targets: Array2<u8> = read_npy(&flags.targets)
  //   .with_context(|| format!("Failed to read NumPy array from {:?}", flags.targets))?;

  let nets: Vec<usize> = std::fs::read_to_string(&flags.nets)?
    .split_whitespace()
    .map(|x| x.parse().unwrap())
    .collect();



  let designs: Vec<Mutex<HardwareModule>> = (0..rayon::current_num_threads())
    .map(|_| Mutex::new(design.clone()))
    .collect();

  // Start simulation
  nets.into_par_iter().progress().for_each(|n| {
    for stuck_val in [Bit::ZERO, Bit::ONE] {
      let thread_idx = rayon::current_thread_index().unwrap();
      let design = &mut designs[thread_idx].lock().unwrap();
      worker(design, &inputs.view(), &weights.view(), n, stuck_val).unwrap();
    }
  });

  */
  Ok(())
}
