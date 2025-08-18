use anyhow::Result;
// use arbolta::bit::Bit;
// use arbolta::hardware_module::HardwareModule;
// use clap::Parser;
// use fault::utils::worker;
// use indicatif::ParallelProgressIterator;
// use ndarray::parallel::prelude::*;
// use ndarray::{Array2, Array3};
// use ndarray_npy::read_npy;
// use std::path::PathBuf;
// use std::sync::Mutex;

// #[derive(Parser, Debug)]
// struct Args {
//   /// Name of top module in netlist.
//   #[clap(long)]
//   top_module: String,
//   /// Path to Yosys netlist JSON file.
//   #[clap(long)]
//   netlist: String,
//   /// Path to NumPy inputs file.
//   #[clap(long)]
//   inputs: PathBuf,
//   /// Path to NumPy weights file.
//   #[clap(long)]
//   weights: PathBuf,
//   /// Path to NumPy targets file.
//   // #[clap(long)]
//   // targets: PathBuf,
//   /// Path to list of nets for fault injection campaign.
//   #[clap(long)]
//   nets: PathBuf,
//   #[clap(long)]
//   rows: usize,
//   #[clap(long)]
//   cols: usize,
//   #[clap(long)]
//   iterations: usize,
//   #[clap(long)]
//   sx_size: usize,
//   #[clap(long)]
//   sk_size: usize,
//   #[clap(long)]
//   m_size: usize,
// }

fn main() -> Result<()> {
  // let flags = Args::parse();

  // // Load everything
  // let mut design = HardwareModule::new_from_path(&flags.netlist, &flags.top_module)?;
  // let inputs: Array3<u8> = read_npy(&flags.inputs)
  //   .with_context(|| format!("Failed to read NumPy array from {:?}", flags.inputs))?; // (N, 28, 28)
  // let weights: Array2<u8> = read_npy(&flags.weights)
  //   .with_context(|| format!("Failed to read NumPy array from {:?}", flags.weights))?; // (10, 784)
  // // let targets: Array2<u8> = read_npy(&flags.targets)
  // //   .with_context(|| format!("Failed to read NumPy array from {:?}", flags.targets))?;

  // let nets: Vec<usize> = std::fs::read_to_string(&flags.nets)?
  //   .split_whitespace()
  //   .map(|x| x.parse().unwrap())
  //   .collect();

  // // Setup designs
  // design.set_clock(2, Bit::ONE)?;
  // design.set_reset(3, Bit::ZERO)?;
  // design.set_port_shape("sx_data_i", &[flags.rows, flags.sx_size])?;
  // design.set_port_shape("sk_data_i", &[flags.cols, flags.sk_size])?;
  // design.set_port_shape("m_data_o", &[1, flags.m_size])?;

  // let designs: Vec<Mutex<HardwareModule>> = (0..rayon::current_num_threads())
  //   .map(|_| Mutex::new(design.clone()))
  //   .collect();

  // // Start simulation
  // nets.into_par_iter().progress().for_each(|n| {
  //   for stuck_val in [Bit::ZERO, Bit::ONE] {
  //     let thread_idx = rayon::current_thread_index().unwrap();
  //     let design = &mut designs[thread_idx].lock().unwrap();
  //     worker(design, &inputs.view(), &weights.view(), n, stuck_val).unwrap();
  //   }
  // });

  Ok(())
}
