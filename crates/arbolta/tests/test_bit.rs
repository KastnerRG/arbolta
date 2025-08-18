// Copyright (c) 2024 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: MIT

use arbolta::bit::Bit;
use rstest::rstest;

#[rstest]
#[case('0', Bit::ZERO)]
#[case('1', Bit::ONE)]
fn test_bit_from_char(#[case] val: char, #[case] expected: Bit) {
  assert_eq!(Bit::try_from(val).unwrap(), expected);
}

#[rstest]
#[case(Bit::ZERO, '0')]
#[case(Bit::ONE, '1')]
fn test_bit_to_char(#[case] bit: Bit, #[case] expected: char) {
  assert_eq!(<Bit as Into<char>>::into(bit), expected);
}

#[rstest]
#[case(false, Bit::ZERO)]
#[case(true, Bit::ONE)]
fn test_bit_from_bool(#[case] val: bool, #[case] expected: Bit) {
  assert_eq!(Bit(val), expected);
}

#[rstest]
#[case(Bit::ZERO, false)]
#[case(Bit::ONE, true)]
fn test_bit_to_bool(#[case] bit: Bit, #[case] expected: bool) {
  assert_eq!(bit.0, expected);
}

#[test]
fn test_bit_not() {
  assert_eq!(!Bit::ZERO, Bit::ONE);
  assert_eq!(!Bit::ONE, Bit::ZERO);
}

#[test]
fn test_bit_and() {
  assert_eq!(Bit::ZERO & Bit::ZERO, Bit::ZERO);
  assert_eq!(Bit::ZERO & Bit::ONE, Bit::ZERO);
  assert_eq!(Bit::ONE & Bit::ZERO, Bit::ZERO);
  assert_eq!(Bit::ONE & Bit::ONE, Bit::ONE);
}

#[test]
fn test_bit_or() {
  assert_eq!(Bit::ZERO | Bit::ZERO, Bit::ZERO);
  assert_eq!(Bit::ZERO | Bit::ONE, Bit::ONE);
  assert_eq!(Bit::ONE | Bit::ZERO, Bit::ONE);
  assert_eq!(Bit::ONE | Bit::ONE, Bit::ONE);
}

#[test]
fn test_bit_xor() {
  assert_eq!(Bit::ZERO ^ Bit::ZERO, Bit::ZERO);
  assert_eq!(Bit::ZERO ^ Bit::ONE, Bit::ONE);
  assert_eq!(Bit::ONE ^ Bit::ZERO, Bit::ONE);
  assert_eq!(Bit::ONE ^ Bit::ONE, Bit::ZERO);
}
