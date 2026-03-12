// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use arbolta::cell::*;
use arbolta::signal::Signals;
use rstest::rstest;

fn convert_bits<const T: usize>(x: [u8; T]) -> [Bit; T] {
  let mut converted = [Bit::ZERO; T];
  for i in 0..T {
    converted[i] = Bit::from_int(x[i]).unwrap();
  }
  converted
}

#[rstest]
#[case::buffer(Box::new(Buffer::new(0,1)), [ // (A, Y)
    (0, 0),
    (1, 1),
])]
#[case::inverter(Box::new(Inverter::new(0,1)), [ // (A, Y)
    (0, 1),
    (1, 0),
])]
fn test_cell_unary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(u8, u8); 2]) {
  let mut signals = Signals::new(2);
  for (a, expected) in cases {
    let [a, expected] = convert_bits([a, expected]);

    signals.set_net(0, a);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(1), expected, "input `{a}`")
  }
}

#[rstest]
#[case::and(Box::new(And2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case::andnot(Box::new(AndNot2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case::nand(Box::new(Nand2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case::nor(Box::new(Nor2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 0),
])]
#[case::or(Box::new(Or2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case::ornot(Box::new(OrNot2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case::xnor(Box::new(Xnor2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case::xor(Box::new(Xor2::new(0, 1, 2)), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
fn test_cell_binary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(u8, u8, u8); 4]) {
  let mut signals = Signals::new(3);
  for (a, b, expected) in cases {
    let [a, b, expected] = convert_bits([a, b, expected]);

    signals.set_net(0, a);
    signals.set_net(1, b);

    cell.eval(&mut signals);
    assert_eq!(signals.get_net(2), expected, "inputs `{a}`, `{b}`")
  }
}

#[rstest]
#[case::andorinvert(Box::new(AndOrInvert3::new(0, 1, 2, 3)), [ // (A, B, C, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 0),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
#[case::mux2(Box::new(Mux2::new(0, 1, 2, 3)), [ // (A, B, S, Y)
    (0, 0, 0, 0),
    (0, 0, 1, 0),
    (0, 1, 0, 0),
    (0, 1, 1, 1),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 1),
])]
#[case::nmux2(Box::new(NMux2::new(0, 1, 2, 3)), [ // (A, B, S, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 0),
    (1, 0, 1, 1),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
#[case::orandinvert(Box::new(OrAndInvert3::new(0, 1, 2, 3)), [ // (A, B, C, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 0),
])]
fn test_cell_ternary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(u8, u8, u8, u8); 8]) {
  let mut signals = Signals::new(4);
  for (a, b, c, expected) in cases {
    let [a, b, c, expected] = convert_bits([a, b, c, expected]);

    signals.set_net(0, a);
    signals.set_net(1, b);
    signals.set_net(2, c);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(3), expected, "inputs `{a}`, `{b}`, `{c}`")
  }
}

#[rstest]
#[case::half_adder_inv(Box::new(Asap7HalfAdderInv::new(0, 1, 2, 3)), [ // A, B, SO, CO
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (1, 0, 0, 1),
    (1, 1, 1, 0),
])]
fn test_cell_binary_two_output(
  #[case] mut cell: Box<dyn CellFn>,
  #[case] cases: [(u8, u8, u8, u8); 4],
) {
  let mut signals = Signals::new(4);
  for (a, b, exp_x, exp_y) in cases {
    let [a, b, exp_x, exp_y] = convert_bits([a, b, exp_x, exp_y]);

    signals.set_net(0, a);
    signals.set_net(1, b);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(2), exp_x, "inputs `{a}`, `{b}`");
    assert_eq!(signals.get_net(3), exp_y, "inputs `{a}`, `{b}`");
  }
}

#[rstest]
#[case::full_adder_inv(Box::new(Asap7FullAdderInv::new(0, 1, 2, 3, 4)), [ // A, B, CI, SO, CO
  (0, 0, 0, 1, 1),
  (0, 0, 1, 0, 1),
  (0, 1, 0, 0, 1),
  (0, 1, 1, 1, 0),
  (1, 0, 0, 0, 1),
  (1, 0, 1, 1, 0),
  (1, 1, 0, 1, 0),
  (1, 1, 1, 0, 0),
])]
fn test_cell_ternary_two_output(
  #[case] mut cell: Box<dyn CellFn>,
  #[case] cases: [(u8, u8, u8, u8, u8); 8],
) {
  let mut signals = Signals::new(5);
  for (a, b, c, exp_x, exp_y) in cases {
    let [a, b, c, exp_x, exp_y] = convert_bits([a, b, c, exp_x, exp_y]);

    signals.set_net(0, a);
    signals.set_net(1, b);
    signals.set_net(2, c);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(3), exp_x, "inputs `{a}, {b}, {c}`");
    assert_eq!(signals.get_net(4), exp_y, "inputs `{a}, {b}, {c}`");
  }
}

#[rstest]
fn test_cell_dff_posedge() {
  let (clock, data_in, data_out) = (0, 1, 2);
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

#[rstest]
fn test_cell_sdff_pp() {
  let (clock, reset, data_in, data_out) = (0, 1, 2, 3);
  let mut cell = DffReset::new(
    Bit::ONE,
    Bit::ONE,
    Bit::ZERO,
    clock,
    reset,
    data_in,
    data_out,
  );
  let mut signals = Signals::new(4);

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

  signals.set_net(reset, Bit::ONE);
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ONE);

  signals.set_net(clock, Bit::ONE); // Rising edge
  cell.eval(&mut signals);
  assert_eq!(signals.get_net(data_out), Bit::ZERO);
}
