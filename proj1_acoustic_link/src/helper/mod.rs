mod bytes_bits;
pub use bytes_bits::{bits_to_bytes, bytes_to_bits};

mod paddings;
pub use paddings::{add_padding, remove_padding};

mod crc_seq;
pub use crc_seq::CrcSeq;

mod signal;
pub use signal::{chirp, copy, dot_product};

#[cfg(test)]
mod tests;
