/// bytes to bits. the bits order does not matter.
pub fn bytes_to_bits(bytes: &[u8]) -> Vec<u8> {
  let mut bits = Vec::with_capacity(bytes.len() * 8);
  bytes
    .iter()
    .for_each(|byte| (0..8).for_each(|i| bits.push((byte >> i) & 1)));
  bits
}
/// the reverse process of [`bytes_to_bits`].
pub fn bits_to_bytes(bits: &[u8]) -> Vec<u8> {
  assert_eq!(bits.len() % 8, 0);
  let mut bytes = Vec::with_capacity(bits.len() / 8);
  bits
    .chunks_exact(8)
    .for_each(|bits| bytes.push(bits.iter().rev().fold(0, |s, bit| (s << 1) | bit)));
  bytes
}
