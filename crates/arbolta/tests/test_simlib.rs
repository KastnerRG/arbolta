// use arbolta::bit::{Bit, BitVec};
// use arbolta::cell::*;
// use arbolta::signal::Signals;
// use rstest::rstest;
// use std::str::FromStr;

// #[rstest]
// #[case::add_unsigned(|a, b, c| Box::new(Add::new(false, a, b, c)) as Box<dyn CellFn>, vec![
//     ("00000111", "00000111", "00001110"), // 7 + 7 = 49
//     ("00000111", "111", "01110"), // 7 + 7 = 49
//     ("111", "111", "1110"), // 7 + 7 = 14
//     ("111", "111", "10"), // 7 + 7 = 4 (overflow)
// ])]
// #[case::add_signed(|a, b, c| Box::new(Add::new(true, a, b, c)) as Box<dyn CellFn>, vec![
//     ("00000111", "00000111", "00001110"), // 7 + 7 = 49
//     ("00000111", "1001", "00000"), // 7 + -7 = 0
//     ("1001", "11001", "11110010"), // -7 + -7 = -14
//     ("1001", "1001", "10"), // -7 + -7 = -4 (overflow)
//     ("111", "111", "10"), // 7 + 7 = 4 (overflow)
//     ("1", "0", "111111111111"), // sign extend
//     ("0", "1", "111111111111"), // sign extend
//     ("1", "1", "111111111110"), // -1 + -1 = -2
//     ("01", "1", "000000000000"), // 1 + -1 = 0
//     ("11111111111111111111111111111111111111111111111111111111111111100010011111010100010011110001101110011011000110100110111110001011",
//      "00000000000000000000000000000011000001000110100000010001100111010011111100101100011100101011100000110101000110100011101011011110",
//      "00000000000000000000000000000011000001000110100000010001100110110110011100000000110000011101001111010000001101001010101001101001")
// ])]
// #[case::and_unsigned(|a, b, c| Box::new(And::new(false, a, b, c)) as Box<dyn CellFn>, vec![
//     ("00000111", "00000111", "00001110"), // 7 + 7 = 49
//     ("00000111", "111", "01110"), // 7 + 7 = 49
//     ("111", "111", "1110"), // 7 + 7 = 14
//     ("111", "111", "10"), // 7 + 7 = 4 (overflow)
// ])]
// fn test_cell_input2(
//   #[case] cell_new: impl Fn(Box<[usize]>, Box<[usize]>, Box<[usize]>) -> Box<dyn CellFn>,
//   #[case] cases: Vec<(&str, &str, &str)>,
// ) {
//   for (a, b, expected) in cases {
//     // Rstest won't automatically convert
//     let (a, b, expected) = (
//       BitVec::from_str(a).unwrap(),
//       BitVec::from_str(b).unwrap(),
//       BitVec::from_str(expected).unwrap(),
//     );

//     let (a_nets, b_nets, y_nets) = (
//       (0..a.shape[1]).collect::<Vec<usize>>(),
//       (a.shape[1]..a.shape[1] + b.shape[1]).collect::<Vec<usize>>(),
//       (a.shape[1] + b.shape[1]..a.shape[1] + b.shape[1] + expected.shape[1])
//         .collect::<Vec<usize>>(),
//     );

//     let mut signals = Signals::new(a_nets.len() + b_nets.len() + y_nets.len());

//     // Set inputs
//     a_nets
//       .iter()
//       .enumerate()
//       .for_each(|(i, n)| signals.set_net(*n, a.bits[i]));

//     b_nets
//       .iter()
//       .enumerate()
//       .for_each(|(i, n)| signals.set_net(*n, b.bits[i]));

//     let mut cell = cell_new(a_nets.into(), b_nets.into(), y_nets.clone().into());
//     cell.eval(&mut signals);

//     // Get outputs
//     let actual: BitVec = y_nets
//       .iter()
//       .map(|i| signals.get_net(*i))
//       .collect::<Vec<Bit>>()
//       .into();
//     assert_eq!(actual, expected, "inputs `{a}`, `{b}`");
//   }
// }
