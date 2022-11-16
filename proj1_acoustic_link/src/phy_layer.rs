/// PHY layer type traits: send+recv+probe
mod traits;
pub use traits::PhyLayer as PhyTrait;

/// the plain physics layer
mod plain;
pub use plain::PlainPHY;

/// the plain physics layer
mod with_crc;
pub use with_crc::{CrcPhy, CrcPhyRecvErr};

/// the atomic physics layer: detect packet lost/corrupt,
/// no partial failure.
mod atomic;
pub use atomic::{AtomicPHY, AtomicPhyRecvErr};

/// PHY layer with OFDM+PSK modulation for higher bit rate
#[cfg(not(feature = "nofloat"))]
mod ofdm;
#[cfg(not(feature = "nofloat"))]
pub use ofdm::HighBpsPHY;

/// PHY layer implementation for mocking
mod mocking;
pub use mocking::MockingPhy;

/// the default PHY layer implementation is CRC PHY
pub type PhyLayer = CrcPhy;
/// the default PHY layer implementation receiver error type
pub type PhyRecvErr = CrcPhyRecvErr;
