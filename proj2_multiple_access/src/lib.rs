use proj1_acoustic_link::phy_layer::DefaultPhy;
/// Define the MAC layer packet:
///
/// - MAC address
/// - MAC frame type
/// - MAC packet
mod packet;
pub use packet::{MacAddr, MacFlags, MacPacket, MacSeq};

/// Define MAC state machine trait and MAC layer object
mod mac;

/// CSMA implementation.
mod csma;
/// Simple MAC protocol for peer-to-peer full duplex connection.
mod p2p_full_duplex;

/// export the default MAC layer implementation.
pub type MacLayer = mac::MacLayer<DefaultPhy, p2p_full_duplex::Simple>;
