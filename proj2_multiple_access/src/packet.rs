use std::marker::PhantomData;

use proj1_acoustic_link::phy_layer::PhyLayer;

/// MAC address to identify a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr(pub u8);

/// MAC packet sequence number
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacSeq(pub(crate) u8);

impl MacSeq {
  /// The next sequence number.
  /// Sequence nubmer will wrap around to 0 when it exceeds `1<<8`
  pub fn next(&self) -> Self {
    Self(self.0.wrapping_add(1))
  }
  /// Increase the sequence
  pub fn step(&mut self) {
    *self = self.next();
  }
}

/// MAC packet flags field:
/// - `ack`: does this packet have an valid ACK field.
/// - `data`: does this packet contain a valid payload section.
/// - `ping_request`: is this packet a request for ping.
/// - `ping_reply`: is this packet a reply to previous ping request.
#[derive(Debug, Clone, Copy)]
pub struct MacFlags {
  pub(crate) ack: bool,
  pub(crate) data: bool,
  pub(crate) ping_request: bool,
  pub(crate) ping_reply: bool,
}

impl MacFlags {
  /// for DATA packet
  const DATA: Self = Self {
    ack: false,
    data: true,
    ping_request: false,
    ping_reply: false,
  };
  /// for ping packet
  const PING: Self = Self {
    ack: false,
    data: false,
    ping_request: true,
    ping_reply: false,
  };
}

/// MAC packet data structure:
/// - `src`: source node MAC address
/// - `dest`: destination node MAC address
/// - `flags: packet metadata flags (have an ack field, contain any data, is ping packet)
/// - `seq`: MAC packet sequence number.
/// - `data`: the payload data section
pub struct MacPacket<PHY: PhyLayer> {
  _phantom: PhantomData<PHY>,
  pub(crate) src: MacAddr,
  pub(crate) dest: MacAddr,
  pub(crate) seq: MacSeq,
  pub(crate) flags: MacFlags,
  pub(crate) data: Vec<u8>,
}

impl<PHY: PhyLayer> MacPacket<PHY> {
  /// The number of payload bytes in one MAC packet.
  pub const PAYLOAD_SIZE: usize = PHY::PACKET_BYTES - 4;

  /// Create a MAC packet from the specified arguments.
  /// `data` should contain no more than `MacPacket::PAYLOAD_SIZE` bytes.
  fn new(src: MacAddr, dest: MacAddr, seq: MacSeq, flags: MacFlags, data: &[u8]) -> Self {
    assert!(data.len() <= Self::PAYLOAD_SIZE);
    let mut data = Vec::from(data);
    data.resize(Self::PAYLOAD_SIZE, 0);

    Self {
      _phantom: Default::default(),
      src,
      dest,
      seq,
      flags,
      data,
    }
  }

  /// Create a data packet
  pub fn new_data(src: MacAddr, dest: MacAddr, seq: MacSeq, data: &[u8]) -> Self {
    Self::new(src, dest, seq, MacFlags::DATA, data)
  }
  pub fn new_ping(src: MacAddr, dest: MacAddr, seq: MacSeq) -> Self {
    Self::new(src, dest, seq, MacFlags::PING, &[])
  }

  /// Only packets with Ping-Request or Data flag set need replay.
  pub fn need_reply(&self) -> bool {
    self.flags.data || self.flags.ping_request
  }

  /// Construct the replying packet, wrap it in `Option::Some`.
  /// If the packet does not need a reply, return `Option::None`.
  pub fn reply_packet(&self) -> Option<Self> {
    if self.need_reply() {
      let flags = MacFlags {
        ack: true,
        data: false,
        ping_request: false,
        ping_reply: self.flags.ping_request,
      };
      Some(MacPacket {
        _phantom: Default::default(),
        flags,
        src: self.dest,
        dest: self.src,
        seq: self.seq,
        data: self.data.clone(),
      })
    } else {
      None
    }
  }

  /// Parse a MAC packet from a PHY packet.
  pub fn from_phy(phy_packet: &[u8]) -> Self {
    assert_eq!(phy_packet.len(), PHY::PACKET_BYTES);
    let src = MacAddr(phy_packet[0]);
    let dest = MacAddr(phy_packet[1]);
    let seq = MacSeq(phy_packet[2]);
    let flags = phy_packet[3];
    let data = &phy_packet[4..];

    let flags = MacFlags {
      ack: (flags & 0b0001) != 0,
      data: (flags & 0b0010) != 0,
      ping_request: (flags & 0b0100) != 0,
      ping_reply: (flags & 0b1000) != 0,
    };
    Self::new(src, dest, seq, flags, data)
  }

  /// Dump a MAC packet into a PHY packet.
  pub fn into_phy(&self) -> Vec<u8> {
    let mut pack = vec![0; PHY::PACKET_BYTES];

    pack[0] = self.src.0;
    pack[1] = self.dest.0;
    pack[2] = self.seq.0;

    let ack = self.flags.ack as u8;
    let data = self.flags.data as u8;
    let ping_req = self.flags.ping_request as u8;
    let ping_rpl = self.flags.ping_reply as u8;
    pack[3] = ack | (data << 1) | (ping_req << 2) | (ping_rpl << 3);

    pack[4..].copy_from_slice(&self.data);

    pack
  }
}
