/// bytes to bits.
/// each byte is converted to 8 bits, which are then stored in 8 bytes.
/// the bits order does not matter.
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
  let mut bits = Vec::with_capacity(bytes.len() * 8);
  bytes
    .iter()
    .for_each(|byte| (0..8).for_each(|i| bits.push((byte >> i) & 1)));
  bits
}
/// the reverse process of [`bytes_to_bits`].
/// every 8 element in the input are combined into a byte.
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
  assert_eq!(bits.len() % 8, 0);
  let mut bytes = Vec::with_capacity(bits.len() / 8);
  bits
    .chunks_exact(8)
    .for_each(|bits| bytes.push(bits.iter().rev().fold(0, |s, bit| (s << 1) | bit)));
  bytes
}

/// convert each bit to a 0/1 character.
/// the result vector length is equal to the input bits slice length
pub fn bits_to_chars(bits: &[u8]) -> String {
  bits
    .iter()
    .map(|x| match x {
      0 => '0',
      1 => '1',
      _ => panic!("non 0/1 element is found in bits"),
    })
    .collect()
}

/// convert a 0/1 string to a vector holding 0/1 intergers.
/// the result vector length is equal to the input bits slice length
pub fn chars_to_bits(chars: &str) -> Vec<u8> {
  chars
    .chars()
    .map(|x| match x {
      '0' => 0,
      '1' => 1,
      _ => panic!("non 0/1 element is found in bits"),
    })
    .collect()
}

/// 4b5b MSB mapping table
const TBL: [u8; 16] = [
  0b_11110, 0b_01001, 0b_10100, 0b_10101, 0b_01010, 0b_01011, 0b_01110, 0b_01111, 0b_10010, 0b_10011, 0b_10110,
  0b_10111, 0b_11010, 0b_11011, 0b_11100, 0b_11101,
];

use bitvec::prelude::*;
type ORDER = Msb0;
type STORE = u8;
type BV = BitVec<STORE, ORDER>;
type BS = BitSlice<STORE, ORDER>;

fn eval_bits(bits: &BS) -> usize {
  bits.iter().map(|x| *x as usize).fold(0, |s, b| (s << 1) | b)
}
fn read_bits(val: u8, len: usize) -> BV {
  let mut bits = BV::with_capacity(len);
  for i in (0..len).rev() {
    let bit = (val >> i) & 1;
    bits.push(bit != 0);
  }
  bits
}

/// input: little-endian bits
/// output: bit-endian bits
pub fn encode_4b5b(bits: BV) -> BV {
  assert!(bits.len() % 4 == 0);
  let mut out = BV::with_capacity(bits.len() / 4 * 5);
  bits.chunks_exact(4).for_each(|bits| {
    let val = eval_bits(bits);
    out.extend(read_bits(TBL[val], 5));
  });
  out
}
pub fn decode_4b5b(bits: BV) -> BV {
  assert!(bits.len() % 5 == 0);
  let mut out = BV::with_capacity(bits.len() / 5 * 4);
  for bits in bits.chunks_exact(5) {
    let val_5b = eval_bits(bits) as u8;
    let val_4b = TBL.iter().position(|&map_5b| map_5b == val_5b).unwrap_or(0) as u8;
    out.extend(read_bits(val_4b, 4));
  }
  out
}

pub fn encode_nrzi(bits: BV) -> BV {
  let mut out = BV::with_capacity(bits.len());
  let mut cur = false;
  for bit in bits {
    if bit {
      cur = !cur;
    }
    out.push(cur);
  }
  out
}
pub fn decode_nrzi(bits: BV) -> BV {
  let mut out = BV::with_capacity(bits.len());
  let mut cur = false;
  for bit in bits {
    out.push(cur != bit);
    cur = bit;
  }
  out
}
