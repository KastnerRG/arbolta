// Copyright (c) 2024 Advanced Micro Devices, Inc.
// Copyright (c) 2026 Alexander Redding
// SPDX-License-Identifier: MIT

use arbolta::{bit::Bit, cell::*, signal::Signals};
use rstest::rstest;

const NET_OFFSET: usize = Signals::NET_CONST1 + 1;

fn convert_bits<const T: usize>(x: [u8; T]) -> [Bit; T] {
  let mut converted = [Bit::ZERO; T];
  for i in 0..T {
    converted[i] = Bit::from_int(x[i]).unwrap();
  }
  converted
}

macro_rules! make_cell {
  ($ty:ident, 2) => {
    Box::new($ty::new(NET_OFFSET, NET_OFFSET + 1))
  };
  ($ty:ident, 3) => {
    Box::new($ty::new(NET_OFFSET, NET_OFFSET + 1, NET_OFFSET + 2))
  };
  ($ty:ident, 4) => {
    Box::new($ty::new(
      NET_OFFSET,
      NET_OFFSET + 1,
      NET_OFFSET + 2,
      NET_OFFSET + 3,
    ))
  };
  ($ty:ident, 5) => {
    Box::new($ty::new(
      NET_OFFSET,
      NET_OFFSET + 1,
      NET_OFFSET + 2,
      NET_OFFSET + 3,
      NET_OFFSET + 4,
    ))
  };
}

#[rstest]
#[case::buffer(make_cell!(Buffer, 2), [ // (A, Y)
    (0, 0),
    (1, 1),
])]
#[case::inverter(make_cell!(Inverter, 2), [ // (A, Y)
    (0, 1),
    (1, 0),
])]
fn test_cell_unary(#[case] mut cell: Box<dyn CellFn>, #[case] cases: [(u8, u8); 2]) {
  let mut signals = Signals::new(2);
  for (a, expected) in cases {
    let [a, expected] = convert_bits([a, expected]);

    signals.set_net(NET_OFFSET, a);
    cell.eval(&mut signals);
    assert_eq!(signals.get_net(NET_OFFSET + 1), expected, "input `{a}`")
  }
}

#[rstest]
#[case::and(make_cell!(And2, 3), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case::andnot(make_cell!(AndNot2, 3), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 0),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case::nand(make_cell!(Nand2, 3), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
#[case::nor(make_cell!(Nor2, 3), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 0),
])]
#[case::or(make_cell!(Or2, 3), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case::ornot(make_cell!(OrNot2, 3), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 1),
    (1, 1, 1),
])]
#[case::xnor(make_cell!(Xnor2, 3), [ // (A, B, Y)
    (0, 0, 1),
    (0, 1, 0),
    (1, 0, 0),
    (1, 1, 1),
])]
#[case::xor(make_cell!(Xor2, 3), [ // (A, B, Y)
    (0, 0, 0),
    (0, 1, 1),
    (1, 0, 1),
    (1, 1, 0),
])]
fn test_cell_binary_one_output(
  #[case] mut cell: Box<dyn CellFn>,
  #[case] cases: [(u8, u8, u8); 4],
) {
  let mut signals = Signals::new(3);
  for (a, b, expected) in cases {
    let [a, b, expected] = convert_bits([a, b, expected]);

    signals.set_net(NET_OFFSET, a);
    signals.set_net(NET_OFFSET + 1, b);

    cell.eval(&mut signals);
    assert_eq!(
      signals.get_net(NET_OFFSET + 2),
      expected,
      "inputs `{a}`, `{b}`"
    )
  }
}

#[rstest]
#[case::andorinvert(make_cell!(AndOrInvert3, 4), [ // (A, B, C, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 0),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
#[case::mux2(make_cell!(Mux2, 4), [ // (A, B, S, Y)
    (0, 0, 0, 0),
    (0, 0, 1, 0),
    (0, 1, 0, 0),
    (0, 1, 1, 1),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 1),
])]
#[case::nmux2(make_cell!(NMux2, 4), [ // (A, B, S, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 0),
    (1, 0, 1, 1),
    (1, 1, 0, 0),
    (1, 1, 1, 0),
])]
#[case::orandinvert(make_cell!(OrAndInvert3, 4), [ // (A, B, C, Y)
    (0, 0, 0, 1),
    (0, 0, 1, 1),
    (0, 1, 0, 1),
    (0, 1, 1, 0),
    (1, 0, 0, 1),
    (1, 0, 1, 0),
    (1, 1, 0, 1),
    (1, 1, 1, 0),
])]
fn test_cell_ternary_one_output(
  #[case] mut cell: Box<dyn CellFn>,
  #[case] cases: [(u8, u8, u8, u8); 8],
) {
  let mut signals = Signals::new(4);
  for (a, b, c, expected) in cases {
    let [a, b, c, expected] = convert_bits([a, b, c, expected]);

    signals.set_net(NET_OFFSET, a);
    signals.set_net(NET_OFFSET + 1, b);
    signals.set_net(NET_OFFSET + 2, c);
    cell.eval(&mut signals);
    assert_eq!(
      signals.get_net(NET_OFFSET + 3),
      expected,
      "inputs `{a}`, `{b}`, `{c}`"
    )
  }
}

#[rstest]
#[case::half_adder_inv(make_cell!(Asap7HalfAdderInv, 4), [ // A, B, SO, CO
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

    signals.set_net(NET_OFFSET, a);
    signals.set_net(NET_OFFSET + 1, b);
    cell.eval(&mut signals);
    assert_eq!(
      signals.get_net(NET_OFFSET + 2),
      exp_x,
      "inputs `{a}`, `{b}`"
    );
    assert_eq!(
      signals.get_net(NET_OFFSET + 3),
      exp_y,
      "inputs `{a}`, `{b}`"
    );
  }
}

#[rstest]
#[case::full_adder_inv(make_cell!(Asap7FullAdderInv, 5), [ // A, B, CI, SO, CO
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

    signals.set_net(NET_OFFSET, a);
    signals.set_net(NET_OFFSET + 1, b);
    signals.set_net(NET_OFFSET + 2, c);
    cell.eval(&mut signals);
    assert_eq!(
      signals.get_net(NET_OFFSET + 3),
      exp_x,
      "inputs `{a}, {b}, {c}`"
    );
    assert_eq!(
      signals.get_net(NET_OFFSET + 4),
      exp_y,
      "inputs `{a}, {b}, {c}`"
    );
  }
}

#[rstest]
fn test_cell_dff_posedge() {
  let (clock, data_in, data_out) = (NET_OFFSET, NET_OFFSET + 1, NET_OFFSET + 2);
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
  let (clock, reset, data_in, data_out) =
    (NET_OFFSET, NET_OFFSET + 1, NET_OFFSET + 2, NET_OFFSET + 3);
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
