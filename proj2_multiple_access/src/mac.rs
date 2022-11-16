use std::{
  marker::PhantomData,
  thread::{spawn, JoinHandle},
  time::{Duration, Instant},
};

use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use proj1_acoustic_link::phy_layer::PhyLayer;

use crate::{MacAddr, MacPacket, MacSeq};

pub trait MacStateMachine<PHY: PhyLayer> {
  /// Prepare a MAC state machine:
  /// - `phy`: the PHY layer object upon which the MAC layer is built.
  /// - `self_addr`: the MAC address of this node.
  /// - `packets_to_send`: packets that we want to send through MAC layer.
  /// - `packets_to_send`: packets that we received from MAC layer.
  /// - `terminate_signal`: receive a `()` when MAC layer about to drop.
  fn new(
    phy: PHY,
    self_addr: MacAddr,
    packets_to_send: Receiver<MacPacket<PHY>>,
    packets_received: Sender<MacPacket<PHY>>,
    terminate_signal: Receiver<()>,
  ) -> Self;

  /// Run the MAC state machine.
  /// This function should not return until a `()` is send into `terminal_signal`.
  fn run(&mut self);
}

#[derive(Clone, Copy, Debug)]
pub struct PingTimeout;

/// MAC layer object which is built on a PHY layer object.
/// - `tx_seq`: the number of total packets sent.
/// - `rx_seq`: the nubmer of total packets received.
pub struct MacLayer<PHY, MAC>
where
  PHY: PhyLayer + Send + 'static,
  MAC: MacStateMachine<PHY>,
{
  _phantom: PhantomData<(PHY, MAC)>,
  addr: MacAddr,
  tx_seq: MacSeq,
  rx_seq: MacSeq,
  worker_handler: Option<JoinHandle<()>>,
  pack_send: Sender<MacPacket<PHY>>,
  pack_recv: Receiver<MacPacket<PHY>>,
  terminate_signal: Sender<()>,
}
impl<PHY, MAC> MacLayer<PHY, MAC>
where
  PHY: PhyLayer + Send + 'static,
  MAC: MacStateMachine<PHY>,
{
  /// maximum transmission unit in bytes
  pub const MTU: usize = PHY::PACKET_BYTES;
  pub fn new(addr: MacAddr, phy: PHY) -> Self {
    let (pack_send, packets_to_send) = channel();
    let (packets_received, pack_recv) = channel();
    let (terminate_signal, exit_recv) = channel();

    let worker_handler = Some(spawn(move || {
      MAC::new(phy, addr, packets_to_send, packets_received, exit_recv).run();
    }));

    Self {
      _phantom: Default::default(),
      addr,
      tx_seq: MacSeq(0),
      rx_seq: MacSeq(0),
      worker_handler,
      pack_send,
      pack_recv,
      terminate_signal,
    }
  }

  /// Send a data packet to peer,
  /// `bytes` must be no more than `MacPacket<PHY>::PAYLOAD_BYTES` bytes.
  /// Return immediately
  pub fn send_to(&mut self, dest: MacAddr, bytes: Vec<u8>) {
    let packet = MacPacket::new_data(self.addr, dest, self.tx_seq, &bytes);
    self.tx_seq.step();
    self.pack_send.send(packet).unwrap();
  }

  /// Try to receive a data packet from peers without waiting for incomming ones.
  /// Return the payload in `Option::Some` if a packet can be fetched,
  /// otherwise `Option::None` is returned.
  pub fn try_recv(&mut self) -> Option<Vec<u8>> {
    self
      .pack_recv
      .try_iter()
      .find(|packet| packet.flags.data && packet.seq == self.rx_seq)
      .map(|packet| {
        self.rx_seq.step();
        packet.data
      })
  }
  /// Try to receive a data packet from peer with waiting time.
  /// Return the payload in `Option::Some` if a packet can be fetched,
  /// otherwise `Option::None` is returned.
  pub fn recv_timeout(&mut self, timeout: Duration) -> Option<Vec<u8>> {
    let ddl = Instant::now() + timeout;
    while let Ok(packet) = self.pack_recv.recv_deadline(ddl) {
      if packet.flags.data && packet.seq == self.rx_seq {
        self.rx_seq.step();
        return Some(packet.data);
      }
    }
    None
  }

  /// Send a ping-request and wait for a ping-response
  pub fn ping(&mut self, dest: MacAddr, timeout: Duration) -> Result<Duration, PingTimeout> {
    // send ping
    let seq = self.tx_seq;
    let packet = MacPacket::new_ping(self.addr, dest, seq);
    self.tx_seq.step();
    self.pack_send.send(packet).unwrap();
    // wait pong
    let now = Instant::now();
    let ddl = now + timeout;
    while let Ok(packet) = self.pack_recv.recv_deadline(ddl) {
      if packet.flags.ping_reply && packet.seq == seq {
        return Ok(now.elapsed());
      }
    }
    Err(PingTimeout)
  }
}

impl<PHY, MAC> MacLayer<PHY, MAC>
where
  PHY: PhyLayer + Default + Send + 'static,
  MAC: MacStateMachine<PHY>,
{
  pub fn new_with_default_phy(addr: MacAddr) -> Self {
    Self::new(addr, PHY::default())
  }
}

impl<PHY, MAC> Drop for MacLayer<PHY, MAC>
where
  PHY: PhyLayer + Send + 'static,
  MAC: MacStateMachine<PHY>,
{
  fn drop(&mut self) {
    self.terminate_signal.send(()).unwrap();
    if let Some(worker) = self.worker_handler.take() {
      worker.join().unwrap();
    }
  }
}
