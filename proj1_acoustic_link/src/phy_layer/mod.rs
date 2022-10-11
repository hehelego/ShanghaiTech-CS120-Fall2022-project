/// define the defaults
/// - audio stream
/// - modulation scheme
/// - preamble generation
/// - preamble detection
mod common;

/// the plain physics layer
mod plain;
pub use plain::PlainPHY;

/// the atomic physics layer: detect packet lost/corrupt,
/// no partial failure.
mod crc_seq;
pub use crc_seq::{AtomicPHY, PacketError};

/// the default PHY layer implementation is plain PHY
pub use PlainPHY as PhyLayer;
