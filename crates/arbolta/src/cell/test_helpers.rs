// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use crate::bit::BitVec;

#[macro_export]
macro_rules! make_unary_wires {
  ($a:expr, $y:expr) => {{
    let a_size = $a.bits.len();
    let y_size = $y.bits.len();
    let a_nets: Vec<usize> = (0..a_size).collect();
    let y_nets: Vec<usize> = (a_size..a_size + y_size).collect();
    (a_nets, y_nets)
  }};
}

#[allow(unused)]
pub(crate) use make_unary_wires;

#[macro_export]
macro_rules! make_binary_wires {
  ($a:expr, $b:expr, $y:expr) => {{
    let a_size = $a.bits.len();
    let b_size = $b.bits.len();
    let y_size = $y.bits.len();
    let a_nets: Vec<usize> = (0..a_size).collect();
    let b_nets: Vec<usize> = (a_size..a_size + b_size).collect();
    let y_nets: Vec<usize> = (a_size + b_size..a_size + b_size + y_size).collect();
    (a_nets, b_nets, y_nets)
  }};
}

#[allow(unused)]
pub(crate) use make_binary_wires;

#[macro_export]
macro_rules! run_unary_cell_case_signed {
  ($cell:ty, $signed:expr, $a:expr, $expected:expr) => {{
    let (a_nets, y_nets) = make_unary_wires!($a, $expected);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);

    // Set inputs
    a_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $a.bits[i]));

    // Create cell and evaluate
    let mut cell: $cell = <$cell>::new($signed, a_nets.into(), y_nets.clone().into());
    cell.eval(&mut signals);

    // Get outputs
    let actual: $crate::bit::BitVec = y_nets
      .iter()
      .map(|&i| signals.get_net(i))
      .collect::<Vec<$crate::bit::Bit>>()
      .into();

    assert_eq!(actual, $expected);
  }};
}

#[allow(unused)]
pub(crate) use run_unary_cell_case_signed;

#[macro_export]
macro_rules! run_unary_cell_case {
  ($cell:ty, $a:expr, $expected:expr) => {{
    let (a_nets, y_nets) = make_unary_wires!($a, $expected);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);

    // Set inputs
    a_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $a.bits[i]));

    // Create cell and evaluate
    let mut cell: $cell = <$cell>::new(a_nets.into(), y_nets.clone().into());
    cell.eval(&mut signals);

    // Get outputs
    let actual: $crate::bit::BitVec = y_nets
      .iter()
      .map(|&i| signals.get_net(i))
      .collect::<Vec<$crate::bit::Bit>>()
      .into();

    assert_eq!(actual, $expected);
  }};
}

#[allow(unused)]
pub(crate) use run_unary_cell_case;

#[macro_export]
macro_rules! run_binary_cell_case {
  ($cell:ty, $a:expr, $b:expr, $expected:expr) => {{
    let (a_nets, b_nets, y_nets) = make_binary_wires!($a, $b, $expected);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);

    // Set inputs
    a_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $a.bits[i]));

    b_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $b.bits[i]));

    // Create cell and evaluate
    let mut cell: $cell = <$cell>::new(a_nets.into(), b_nets.into(), y_nets.clone().into());
    cell.eval(&mut signals);

    // Get outputs
    let actual: $crate::bit::BitVec = y_nets
      .iter()
      .map(|&i| signals.get_net(i))
      .collect::<Vec<$crate::bit::Bit>>()
      .into();

    assert_eq!(actual, $expected);
  }};
}

#[allow(unused)]
pub(crate) use run_binary_cell_case;

#[macro_export]
macro_rules! run_binary_cell_case_signed {
  ($cell:ty, $signed:expr, $a:expr, $b:expr, $expected:expr) => {{
    let (a_nets, b_nets, y_nets) = make_binary_wires!($a, $b, $expected);
    let mut signals = Signals::new(y_nets.last().unwrap() + 1);

    // Set inputs
    a_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $a.bits[i]));

    b_nets
      .iter()
      .enumerate()
      .for_each(|(i, n)| signals.set_net(*n, $b.bits[i]));

    // Create cell and evaluate
    let mut cell: $cell =
      <$cell>::new($signed, a_nets.into(), b_nets.into(), y_nets.clone().into());
    cell.eval(&mut signals);

    // Get outputs
    let actual: $crate::bit::BitVec = y_nets
      .iter()
      .map(|&i| signals.get_net(i))
      .collect::<Vec<$crate::bit::Bit>>()
      .into();

    assert_eq!(actual, $expected);
  }};
}

#[allow(unused)]
pub(crate) use run_binary_cell_case_signed;

#[allow(unused)]
pub fn allocate_nets(offset: Option<usize>, data: &[&BitVec]) -> Vec<Box<[usize]>> {
  let mut nets: Vec<Box<[usize]>> = vec![];
  let offset = offset.unwrap_or(0);

  for bits in data {
    let start: usize = match nets.last() {
      Some(last) => last[last.len() - 1] + 1,
      None => offset,
    };

    nets.push((start..start + bits.len()).collect());
  }

  nets
}
