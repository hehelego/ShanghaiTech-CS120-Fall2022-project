mod bytes_bits;
pub use bytes_bits::{
  bits_to_bytes, bits_to_chars, bytes_to_bits, chars_to_bits, decode_4b5b, decode_nrzi, encode_4b5b, encode_nrzi,
};

mod paddings;
pub use paddings::{add_padding, remove_padding};

mod crc_seq;
pub use crc_seq::{CrcSeq, SEQ_MOD};

mod signal;
pub use signal::{chirp, copy, dot_product};

#[cfg(test)]
mod tests;
