use super::{wrapping_integers::WrappingInt32, ASockProtocol};
use crate::IpAccessor;
use crossbeam_channel::{Receiver, Sender};
use pnet::packet::tcp::{Tcp, TcpFlags};
use std::{
  net::SocketAddrV4,
  thread::{self, JoinHandle},
  time::Duration,
  usize,
};

enum TcpState {
  SynSent,
  SynReceived,
  Established,
  FinWait1,
  FinWait2,
  Closing,
  CloseWait,
  LastAck,
  Closed,
  Terminate,
}

enum StateControlSignal {
  Sync(SocketAddrV4),
  Shutdown,
  Terminate,
}

pub(crate) struct TcpStateMachine {
  join_handler: Option<JoinHandle<()>>,
  control_signal: Sender<StateControlSignal>,
}

impl TcpStateMachine {
  // Create a new state machine with state: Closed
  pub fn new(bytes_assembled: Sender<u8>, bytes_to_send: Receiver<u8>, addr: SocketAddrV4) -> Self {
    let (control_signal_tx, control_signal_rx) = crossbeam_channel::unbounded();
    let thread = thread::spawn(move || {
      let mut worker = TcpStateMachinWroker::new(bytes_assembled, bytes_to_send, control_signal_rx, addr);
      worker.run();
    });
    Self {
      join_handler: Some(thread),
      control_signal: control_signal_tx,
    }
  }
  pub fn connect(&self, dest: SocketAddrV4) -> Result<(), ()> {
    self.control_signal.send(StateControlSignal::Sync(dest)).map_err(|_| ())
  }
  pub fn shutdown(&self) -> Result<(), ()> {
    self.control_signal.send(StateControlSignal::Shutdown).map_err(|_| ())
  }
}

impl Drop for TcpStateMachine {
  fn drop(&mut self) {
    if let Some(thread) = self.join_handler.take() {
      thread.join().unwrap();
    }
  }
}

struct Seq {
  pub initial_state_number: WrappingInt32,
  pub absolute_seqence_number: u64,
}

impl Seq {
  pub fn new() -> Self {
    Self {
      initial_state_number: WrappingInt32::new(rand::random()),
      absolute_seqence_number: 0,
    }
  }
  pub fn with_u32(isn: u32) -> Self {
    Self {
      initial_state_number: WrappingInt32::new(isn),
      absolute_seqence_number: 0,
    }
  }
  pub fn sequence_number_u32(&self) -> u32 {
    WrappingInt32::wrap(self.absolute_seqence_number, self.initial_state_number).raw_value()
  }
  pub fn next(&self) -> u32 {
    WrappingInt32::wrap(self.absolute_seqence_number + 1, self.initial_state_number).raw_value()
  }
  pub fn step(&mut self) {
    self.absolute_seqence_number += 1;
  }
}

struct Reassembler {
  buffer: Vec<Option<u8>>,
  buffer_header: usize,
  capacity: usize,
  output: Sender<u8>,
}

impl Reassembler {
  pub fn with_capacity(output: Sender<u8>, capacity: usize) -> Self {
    Self {
      buffer: vec![],
      buffer_header: 0,
      capacity,
      output,
    }
  }

  pub fn size(&self) -> usize {
    self.capacity - self.buffer.len()
  }
}

struct TcpStateMachinWroker {
  src_addr: SocketAddrV4,
  dest_addr: Option<SocketAddrV4>,
  send_seq: Seq,
  recv_seq: Option<Seq>,
  state: TcpState,
  peer_window_size: u16,
  reassembler: Reassembler,
  // Channels for communication
  packet_to_send: Sender<(Tcp, SocketAddrV4)>,
  packet_received: Receiver<(Tcp, SocketAddrV4)>,
  bytes_to_send: Receiver<u8>,
  control_signal: Receiver<StateControlSignal>,
  access_termination_signal: Sender<()>,
}

impl TcpStateMachinWroker {
  pub fn new(
    bytes_assembled: Sender<u8>,
    bytes_to_send: Receiver<u8>,
    control_signal: Receiver<StateControlSignal>,
    src_addr: SocketAddrV4,
  ) -> TcpStateMachinWroker {
    const WINDOW_SIZE: usize = 50;
    let (control_signal_tx, control_signal_rx) = crossbeam_channel::unbounded();
    let (packet_received_tx, packet_received_rx) = crossbeam_channel::unbounded();
    let (packet_to_send_tx, packet_to_send_rx) = crossbeam_channel::unbounded();

    thread::spawn(move || {
      let path = format!("/tmp/tcp_clinet_{}", src_addr);
      let accessor = IpAccessor::new(&path).unwrap();
      accessor.bind(ASockProtocol::TCP, src_addr).unwrap();
      while control_signal_rx.is_empty() {
        if let Ok((packet, addr)) = accessor.recv_tcp() {
          packet_received_tx.send((packet, addr)).unwrap();
        }
        if let Ok((packet, addr)) = packet_to_send_rx.recv() {
          accessor.send_tcp(packet, addr).unwrap();
        }
      }
    });

    let reassembler = Reassembler::with_capacity(bytes_assembled, WINDOW_SIZE);
    Self {
      send_seq: Seq::new(),
      state: TcpState::Closed,
      bytes_to_send,
      control_signal,
      src_addr,
      dest_addr: None,
      recv_seq: None,
      peer_window_size: 0,
      packet_to_send: packet_to_send_tx,
      packet_received: packet_received_rx,
      access_termination_signal: control_signal_tx,
      reassembler,
    }
  }
  /// The state transition function
  pub(crate) fn run(&mut self) {
    loop {
      self.state = match self.state {
        TcpState::SynSent => self.handle_syn_sent(),
        TcpState::SynReceived => todo!(),
        TcpState::Established => todo!(),
        TcpState::FinWait1 => todo!(),
        TcpState::FinWait2 => todo!(),
        TcpState::Closing => todo!(),
        TcpState::CloseWait => todo!(),
        TcpState::LastAck => todo!(),
        TcpState::Closed => self.handle_closed(),
        TcpState::Terminate => break,
      };
    }
  }
  // The handle function for TcpState::Closed
  fn handle_closed(&mut self) -> TcpState {
    match self.control_signal.recv().unwrap() {
      StateControlSignal::Sync(dest_addr) => {
        self.dest_addr = Some(dest_addr);
        self.packet_to_send.send((self.pack_sync(), dest_addr)).unwrap();
        TcpState::SynSent
      }
      StateControlSignal::Terminate => TcpState::Terminate,
      _ => TcpState::Closed,
    }
  }

  // The handle function for TcpState::Syn
  fn handle_syn_sent(&mut self) -> TcpState {
    const MAX_RETRY_COUNT: usize = 5;
    const MAX_WAIT_TIME: Duration = Duration::from_secs(1);
    let mut retry_count = 0;
    while retry_count < MAX_RETRY_COUNT {
      if let Ok((packet, addr)) = self.packet_received.recv_timeout(MAX_WAIT_TIME) {
        // Check if the packet is sync-ack
        if addr == self.dest_addr.unwrap()
          && packet.flags & TcpFlags::SYN == 1
          && packet.flags & TcpFlags::ACK == 1
          && packet.acknowledgement == self.send_seq.next()
        {
          // Increase send sequnce number
          self.send_seq.step();
          // Get recv sequence number
          let mut seq = Seq::with_u32(packet.sequence);
          seq.step();
          self.recv_seq = Some(seq);
          // Get peer window size
          self.peer_window_size = packet.window;
          // Send ack and enter TcpState::Established
          self
            .packet_to_send
            .send((self.pack_ack(), self.dest_addr.unwrap()))
            .unwrap();
          return TcpState::Established;
        }
      }
      //TODO - Receive shutdown signal
      self
        .packet_to_send
        .send((self.pack_sync(), self.dest_addr.unwrap()))
        .unwrap();
      retry_count += 1
    }
    TcpState::Closed
  }

  fn handle_established(&self) {}

  // Pack a sync packet
  fn pack_sync(&self) -> Tcp {
    let mut packet = self.pack_vanilla();
    packet.flags = TcpFlags::SYN;
    packet
  }

  // Pack a ack apcket
  fn pack_ack(&self) -> Tcp {
    let mut packet = self.pack_vanilla();
    packet.flags = TcpFlags::ACK;
    packet
  }

  // Pack a packet without any flag and payload
  fn pack_vanilla(&self) -> Tcp {
    Tcp {
      source: self.src_addr.port(),
      destination: self.dest_addr.unwrap().port(),
      sequence: self.send_seq.sequence_number_u32(),
      acknowledgement: self.recv_seq.as_ref().map_or(0, |s| s.sequence_number_u32()),
      data_offset: 5,
      reserved: 0,
      flags: 0,
      window: self.reassembler.size() as u16,
      checksum: 0,
      urgent_ptr: 0,
      options: vec![],
      payload: vec![],
    }
  }
}
