/// Define common things for the MAC layer:
///
/// constants
/// - estimated RTT
/// - sliding window
///
/// helpers
/// - bit masks
/// - const function div floor
/// - const function div ceil
pub mod common;

/// Define the MAC layer packet:
///
/// - MAC address
/// - MAC frame type
/// - MAC packet
pub mod packet;
