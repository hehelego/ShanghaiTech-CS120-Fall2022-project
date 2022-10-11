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
