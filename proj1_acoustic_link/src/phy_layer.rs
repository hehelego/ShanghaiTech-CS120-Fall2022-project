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
mod atomic;
pub use atomic::{AtomicPHY, PacketError};

/// PHY layer with OFDM+PSK modulation for higher bit rate
#[cfg(not(feature = "nofloat"))]
mod ofdm;
#[cfg(not(feature = "nofloat"))]
pub use ofdm::HighBpsPHY;

/// the default PHY layer implementation is plain PHY
pub use PlainPHY as PhyLayer;
