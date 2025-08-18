// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::*;
use arbolta::signal::Signals;
use rstest::rstest;

#[rstest]
#[case::buffer(Box::new(Buffer::new(0,1)), [ // (A, Y)
    (Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE),
])]
#[case::inverter(Box::new(Inverter::new(0,1)), [ // (A, Y)
    (Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ZERO),
])]
fn test_cell_unary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(Bit, Bit); 2]) {
  let mut signals = Signals::new(2);
  for (a, expected) in cases {
    signals.set_net(0, a);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(1), expected, "input `{a}`")
  }
}

#[rstest]
#[case::and(Box::new(And::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ONE),
])]
#[case::nand(Box::new(Nand::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::or(Box::new(Or::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ONE),
])]
#[case::nor(Box::new(Nor::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::xor(Box::new(Xor::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::xnor(Box::new(Xnor::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ONE),
])]
#[case::andnot(Box::new(AndNot::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::ornot(Box::new(OrNot::new(0, 1, 2)), [ // (A, B, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ONE),
])]
fn test_cell_binary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(Bit, Bit, Bit); 4]) {
  let mut signals = Signals::new(3);
  for (a, b, expected) in cases {
    signals.set_net(0, a);
    signals.set_net(1, b);

    cell.eval(&mut signals);
    assert_eq!(signals.get_net(2), expected, "inputs `{a}`, `{b}`")
  }
}

#[rstest]
#[case::mux2(Box::new(Mux2::new(0, 1, 2, 3)), [ // (A, B, S, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ONE, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ONE, Bit::ONE),
])]
#[case::nmux2(Box::new(NMux2::new(0, 1, 2, 3)), [ // (A, B, S, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::andorinvert(Box::new(AndOrInvert::new(0, 1, 2, 3)), [ // (A, B, C, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ZERO, Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ZERO, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ONE, Bit::ZERO),
])]
#[case::orandinvert(Box::new(OrAndInvert::new(0, 1, 2, 3)), [ // (A, B, C, Y)
    (Bit::ZERO, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ZERO, Bit::ONE, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ZERO, Bit::ONE, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ZERO, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ZERO),
    (Bit::ONE, Bit::ONE, Bit::ZERO, Bit::ONE),
    (Bit::ONE, Bit::ONE, Bit::ONE, Bit::ZERO),
])]
fn test_cell_ternary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(Bit, Bit, Bit, Bit); 8]) {
  let mut signals = Signals::new(4);
  for (a, b, c, expected) in cases {
    signals.set_net(0, a);
    signals.set_net(1, b);
    signals.set_net(2, c);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(3), expected, "inputs `{a}`, `{b}`, `{c}`")
  }
}

#[rstest]
fn test_cell_dff_posedge() {
  // D, C, Q
  let (data_in, clock, data_out) = (0, 1, 2);
  let mut cell = Dff::new(Bit::ONE, clock, data_in, data_out);
  let mut signals = Signals::new(3);

  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ZERO);

  signals.set_net(data_in, Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ZERO);

  signals.set_net(clock, Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ONE);

  signals.set_net(clock, Bit::ZERO); // Falling edge
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ONE);

  signals.set_net(data_in, Bit::ZERO);
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ONE);

  signals.set_net(clock, Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ZERO);
}

/*
#[rstest]
fn test_cell_sdff_pp() {
  // D, C, R, Q
  let (data_in, clock, reset, data_out) = (0, 1, 2, 3);
  let mut cell = DffReset::new(Bit::ONE, data_in, clock, reset, data_out);
  let mut signals = vec![Signal::default(); 4].into_boxed_slice();

  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[data_in].set_value(Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ZERO); // Falling edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[reset].set_value(Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ONE);

  signals[clock].set_value(Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals[data_out].get_value(), Bit::ZERO);
}
*/
