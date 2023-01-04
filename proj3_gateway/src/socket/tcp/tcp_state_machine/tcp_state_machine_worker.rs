use super::super::ASockProtocol;
use super::wrapping_integers::WrappingInt32;
use crate::{packet, IpAccessor};
use crossbeam_channel::{select, Receiver, Sender};
use pnet::packet::tcp::{Tcp, TcpFlags};
use std::{net::SocketAddrV4, thread, time::Duration, usize};

use super::StateControlSignal;

/// States for the TcpStateMachine
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

#[derive(Clone, Copy)]
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

  pub fn update(&mut self, new_seq: WrappingInt32) {
    let diff = new_seq - self.initial_state_number;
    self.absolute_seqence_number += diff.max(0) as u64;
  }

  pub fn add(&mut self, delta: u32) {
    self.absolute_seqence_number += delta as u64;
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
      buffer: vec![None; capacity],
      buffer_header: 1,
      capacity,
      output,
    }
  }

  pub fn size(&self) -> usize {
    self.capacity
  }

  /// Push new data into the reassmebler and send to the client
  /// Return the bytes send to the client
  pub fn update(&mut self, data: Vec<u8>, pos: u64) -> u32 {
    let mut bytes_sent = 0;
    let (data_start, buffer_start) = if (pos as usize) < self.buffer_header {
      ((self.buffer_header - pos as usize), 0)
    } else {
      (0, (pos as usize - self.buffer_header))
    };
    // Buffer the data
    for (bi, d) in (buffer_start..self.capacity)
      .chain(0..buffer_start)
      .zip(data[data_start..].iter().cloned())
    {
      self.buffer[bi] = Some(d)
    }
    for i in (self.buffer_header..self.capacity).chain(0..self.buffer_header) {
      if let Some(byte) = self.buffer[i].take() {
        self.output.send(byte).unwrap();
        self.buffer_header = (self.buffer_header + 1) % self.capacity;
        bytes_sent += 1;
      } else {
        break;
      }
    }
    bytes_sent
  }
}

pub(super) struct TcpStateMachineWorker {
  src_addr: SocketAddrV4,
  dest_addr: Option<SocketAddrV4>,
  send_seq: Seq,
  recv_seq: Option<Seq>,
  state: TcpState,
  peer_window_size: u16,
  reassembler: Reassembler,
  // Channels for communication
  packet_to_send: Sender<(Tcp, SocketAddrV4)>, // packets send to the ip accessor
  packet_received: Receiver<(Tcp, SocketAddrV4)>, // packets received from the ip accessor
  bytes_to_send: Receiver<u8>,                 // data from the client
  control_signal: Receiver<StateControlSignal>, // signal from TcpStateMachine
  access_termination_signal: Sender<()>,       // terminate signale for the accessor
  // terminate statemachine
  terminating: bool,
}

impl TcpStateMachineWorker {
  const ESTIMATE_RTT: Duration = Duration::from_secs(1);
  const MAX_DATA_LENGTH: usize = 1024;
  /// Create a new TcpStateMachine with State closed.
  pub fn new(
    bytes_assembled: Sender<u8>,
    bytes_to_send: Receiver<u8>,
    control_signal: Receiver<StateControlSignal>,
    src_addr: SocketAddrV4,
  ) -> TcpStateMachineWorker {
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
      terminating: false,
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
    if self.terminating {
      return TcpState::Terminate;
    }
    match self.control_signal.recv().unwrap() {
      // Active open
      StateControlSignal::Sync(dest_addr) => {
        self.dest_addr = Some(dest_addr);
        self.packet_to_send.send((self.pack_sync(), dest_addr)).unwrap();
        TcpState::SynSent
      }
      StateControlSignal::Terminate => TcpState::Terminate,
      _ => TcpState::Closed,
    }
  }

  // The handle function for TcpState::SynSent
  fn handle_syn_sent(&mut self) -> TcpState {
    const MAX_RETRY_COUNT: usize = 5;
    let mut retry_count = 0;
    while retry_count < MAX_RETRY_COUNT {
      if let Ok((packet, addr)) = self.packet_received.recv_timeout(Self::ESTIMATE_RTT) {
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

      // Receive shutdown signal. We have not select to use.
      if let Ok(signal) = self.control_signal.try_recv() {
        match signal {
          // receive closed
          StateControlSignal::Shutdown => return TcpState::Closed,
          // terminate
          StateControlSignal::Terminate => return TcpState::Terminate,
          _ => (),
        }
      }

      self
        .packet_to_send
        .send((self.pack_sync(), self.dest_addr.unwrap()))
        .unwrap();
      retry_count += 1
    }
    // Connection establish failed
    TcpState::Closed
  }

  fn handle_established(&mut self) -> TcpState {
    loop {
      // Check close/terminate
      if let Ok(signal) = self.control_signal.try_recv() {
        match signal {
          StateControlSignal::Shutdown => {
            return TcpState::FinWait1;
          }
          StateControlSignal::Terminate => {
            self.terminating = true;
            return TcpState::FinWait1;
          }
          _ => (),
        }
      }

      if self.receive_data() {
        return TcpState::CloseWait;
      }
      self.send_data(false);
    }
  }

  // Pack a sync packet
  fn pack_sync(&self) -> Tcp {
    let mut packet = self.pack_vanilla();
    packet.flags = TcpFlags::SYN;
    packet
  }

  // Pack a ack packet
  fn pack_ack(&self) -> Tcp {
    let mut packet = self.pack_vanilla();
    packet.flags = TcpFlags::ACK;
    packet
  }

  // Pack a data packet
  fn pack_data(&self, data: Vec<u8>, fin: bool) -> Tcp {
    let mut packet = self.pack_ack();
    if fin {
      packet.flags |= TcpFlags::FIN;
    }
    packet.payload = data;
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

  // Return true if fin
  fn receive_data(&mut self) -> bool {
    // Receive the data
    while let Ok((packet, addr)) = self.packet_received.try_recv() {
      // Wrong packet send here
      if addr != self.src_addr {
        continue;
      }
      // Update the send sequence
      if packet.flags & TcpFlags::ACK != 0 {
        self.send_seq.update(WrappingInt32::new(packet.acknowledgement));
      }
      // Get the data
      let ack_delta = self.reassembler.update(
        packet.payload,
        WrappingInt32::unwrap(
          WrappingInt32::new(packet.sequence),
          self.recv_seq.unwrap().initial_state_number,
          self.recv_seq.unwrap().absolute_seqence_number,
        ),
      );
      self.recv_seq.unwrap().add(ack_delta);
      if packet.flags & TcpFlags::FIN != 0 {
        return true;
      }
    }
    false
  }

  fn send_data(&mut self, send_fin: bool) {
    let mut send_buffer: Vec<u8> = Vec::new();
    // Prepare the data
  }
}
