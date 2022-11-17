use std::{
  collections::VecDeque,
  time::{Duration, Instant},
};

use crate::{mac::MacStateMachine, MacAddr, MacPacket};
use crossbeam_channel::{Receiver, Sender};
use proj1_acoustic_link::{
  phy_layer::{DefaultPhy, PhyLayer},
  traits::{PacketReceiver, PacketSender},
};

struct PendingPacket {
  packet: MacPacket<DefaultPhy>,
  send_time: Instant,
  retry_count: usize,
}

impl PendingPacket {
  fn with_packet(packet: MacPacket<DefaultPhy>) -> Self {
    Self {
      packet,
      send_time: Instant::now(),
      retry_count: 0,
    }
  }
  fn resend(&mut self, phy: &mut DefaultPhy) {
    phy.send(self.packet.into_phy()).unwrap();
    self.send_time = Instant::now();
    self.retry_count += 1;
  }
}

/// Simple MAC implementaion for peer to peer full duplex connection:
/// stop-and-wait or sliding window.
pub struct Simple {
  phy: DefaultPhy,
  addr: MacAddr,
  packets_to_send: Receiver<MacPacket<DefaultPhy>>,
  packets_received: Sender<MacPacket<DefaultPhy>>,
  terminate_signal: Receiver<()>,
  pending_packets: VecDeque<PendingPacket>,
}

impl Simple {
  const WINDOW_SIZE: usize = 3;
  fn resent_interval() -> Duration {
    DefaultPhy::ESTIMATED_RTT * 3 / 2
  }

  fn can_send_packet(&self) -> bool {
    return self.pending_packets.len() < Self::WINDOW_SIZE && !self.packets_to_send.is_empty();
  }

  fn check_and_send_packet(&mut self) {
    while self.can_send_packet() {
      let packet = self.packets_to_send.recv().unwrap();
      self.phy.send(packet.into_phy()).unwrap();
      self.pending_packets.push_back(PendingPacket::with_packet(packet));
    }
  }
  fn check_and_resend_packet(&mut self) {
    for pending_pacekt in &mut self.pending_packets {
      if pending_pacekt.send_time.elapsed() > Self::resent_interval() {
        pending_pacekt.resend(&mut self.phy);
      }
    }
  }
  fn receive_packet(&mut self) {
    let mut pending_ack: VecDeque<MacPacket<DefaultPhy>> = VecDeque::new();
    while let Ok(packet) = self.phy.recv() {
      let packet: MacPacket<DefaultPhy> = MacPacket::from_phy(&packet);
      if packet.dest != self.addr {
        continue;
      }
      if let Some(ack) = packet.reply_packet() {
        pending_ack.push_back(ack);
      }
      self.packets_received.send(packet).unwrap();
    }
    for packet in pending_ack {
      self.phy.send(packet.into_phy()).unwrap();
    }
  }
}
impl MacStateMachine<DefaultPhy> for Simple {
  fn new(
    phy: DefaultPhy,
    addr: MacAddr,
    packets_to_send: Receiver<MacPacket<DefaultPhy>>,
    packets_received: Sender<MacPacket<DefaultPhy>>,
    terminate_signal: Receiver<()>,
  ) -> Self {
    Self {
      phy,
      addr,
      packets_to_send,
      packets_received,
      terminate_signal,
      pending_packets: VecDeque::new(),
    }
  }

  fn run(&mut self) {
    while self.terminate_signal.try_recv().is_err() {
      self.check_and_resend_packet();
      self.check_and_send_packet();
      self.receive_packet();
    }
  }
}
