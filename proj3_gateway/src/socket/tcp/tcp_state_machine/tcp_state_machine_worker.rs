use super::super::ASockProtocol;
use super::wrapping_integers::WrappingInt32;
use crate::IpAccessor;
use crossbeam_channel::{Receiver, Sender};
use pnet::packet::tcp::{Tcp, TcpFlags};
use std::{
  net::SocketAddrV4,
  thread::{self, JoinHandle},
  time::Duration,
  usize,
};

use super::StateControlSignal;

/// States for the TcpStateMachine
#[derive(Debug)]
enum TcpState {
  SynSent,
  #[allow(unused)]
  SynReceived,
  Established,
  FinWait1,
  FinWait2,
  Closing,
  CloseWait,
  LastAck,
  TimeWait,
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

  pub fn update(&mut self, new_seq: WrappingInt32) -> u64 {
    let new_seq = WrappingInt32::unwrap(new_seq, self.initial_state_number, self.absolute_seqence_number);
    let diff = if new_seq > self.absolute_seqence_number {
      new_seq - self.absolute_seqence_number
    } else {
      0
    };
    self.absolute_seqence_number += diff;
    log::trace!(
      "[Seq update]\n
      new_seq: {}, isn: {}, diff: {}, asn: {}",
      new_seq,
      self.initial_state_number.raw_value(),
      diff,
      self.absolute_seqence_number,
    );
    diff
  }

  pub fn add(&mut self, delta: u32) {
    self.absolute_seqence_number += delta as u64;
  }
}

struct Reassembler {
  buffer: Vec<Option<u8>>,
  buffer_header: usize,
  capacity: usize,
  output: Option<Sender<u8>>,
  fin_byte: Option<usize>,
  is_fin: bool,
}

impl Reassembler {
  pub fn with_capacity(output: Sender<u8>, capacity: usize) -> Self {
    Self {
      buffer: vec![None; capacity],
      buffer_header: 0,
      capacity,
      output: Some(output),
      fin_byte: None,
      is_fin: false,
    }
  }

  pub fn size(&self) -> usize {
    self.capacity
  }

  /// Push new data into the reassmebler and send to the client
  /// Return the bytes send to the client
  pub fn update(&mut self, data: &[u8], pos: u64, fin: bool) -> u32 {
    if self.is_fin {
      return 0;
    }
    if fin {
      self.fin_byte = Some(pos as usize + data.len() - 1);
    }
    let mut bytes_sent = 0;
    let (data_start, buffer_start) = if (pos as usize) < self.buffer_header {
      ((self.buffer_header - pos as usize), 0)
    } else {
      (0, (pos as usize - self.buffer_header))
    };
    log::trace!(
      "[Reassmbler]: pos: {}, buffer_header: {}, data len: {}, data start:{}, buffer_start: {}",
      pos,
      self.buffer_header,
      data.len(),
      data_start,
      buffer_start
    );
    if data_start < data.len() {
      // Buffer the data
      for (bi, d) in (buffer_start..self.capacity)
        .chain(0..buffer_start)
        .zip(data[data_start..].iter().cloned())
      {
        self.buffer[(self.buffer_header + bi) % self.capacity] = Some(d)
      }
      for i in (self.buffer_header % self.capacity..self.capacity).chain(0..self.buffer_header) {
        if let Some(byte) = self.buffer[i].take() {
          self.buffer_header += 1;
          bytes_sent += 1;
          if let Some(output) = self.output.as_ref() {
            let _ = output.send(byte);
          }
        } else {
          break;
        }
      }
    }
    if self.fin_byte.map_or(false, |v| self.buffer_header > v) {
      self.is_fin = true;
      drop(self.output.take());
    }
    log::trace!(
      "[Reassmbler]: Update \n
      data len: {}, byte_reassembled: {}, header_pos: {}, is_fin: {}",
      data.len(),
      bytes_sent,
      self.buffer_header,
      self.output.is_none()
    );
    bytes_sent
  }

  pub fn sync(&mut self) {
    self.buffer_header += 1;
    if let Some(output) = self.output.as_ref() {
      output.send(0).unwrap();
    }
  }

  pub fn ack(&mut self) {
    self.buffer_header += 1;
  }
  pub fn set_down(&mut self) {
    if let Some(o) = self.output.take() {
      drop(o)
    }
  }
  pub fn is_fin(&self) -> bool {
    self.is_fin
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
  send_buffer: Vec<u8>,
  // Channels for communication
  packet_to_send: Sender<(Tcp, SocketAddrV4)>, // packets send to the ip accessor
  packet_received: Receiver<(Tcp, SocketAddrV4)>, // packets received from the ip accessor
  bytes_to_send: Receiver<u8>,                 // data from the client
  control_signal: Receiver<StateControlSignal>, // signal from TcpStateMachine
  access_termination_signal: Sender<()>,       // terminate signale for the accessor
  // terminate statemachine
  read_down: bool,
  write_down: bool,
  terminating: bool,
  accessor_handler: Option<JoinHandle<()>>,
}

impl TcpStateMachineWorker {
  const ESTIMATE_RTT: Duration = Duration::from_secs(2);
  const MAX_DATA_LENGTH: usize = 1024;
  const MAX_RETRY_COUNT: usize = 5;
  /// Create a new TcpStateMachine with State closed.
  pub fn new(
    bytes_assembled: Sender<u8>,
    bytes_to_send: Receiver<u8>,
    control_signal: Receiver<StateControlSignal>,
    src_addr: SocketAddrV4,
  ) -> TcpStateMachineWorker {
    const WINDOW_SIZE: usize = TcpStateMachineWorker::MAX_DATA_LENGTH;
    let (control_signal_tx, control_signal_rx) = crossbeam_channel::unbounded();
    let (packet_received_tx, packet_received_rx) = crossbeam_channel::unbounded();
    let (packet_to_send_tx, packet_to_send_rx) = crossbeam_channel::unbounded();

    let accessor_handler = thread::spawn(move || {
      let path = format!("/tmp/tcp_client_{}", src_addr);
      let accessor = IpAccessor::new(&path).unwrap();
      accessor.bind(ASockProtocol::TCP, src_addr).unwrap();
      while control_signal_rx.try_recv().is_err() {
        if let Ok((packet, addr)) = accessor.recv_tcp() {
          log::debug!("[Tcp Accessor] receive tcp from {}", addr);
          packet_received_tx.send((packet, addr)).unwrap();
          while accessor.recv_tcp().is_ok() {
            // Consume all remaining tcp packet
          }
        }
        if let Ok((packet, addr)) = packet_to_send_rx.try_recv() {
          accessor.send_tcp(packet, addr).unwrap();
        }
      }
      log::debug!("[Tcp Accessor] exit");
    });

    let reassembler = Reassembler::with_capacity(bytes_assembled, WINDOW_SIZE);
    log::debug!("[Tcp Worker] created. src_addr: {}", src_addr);
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
      send_buffer: Vec::new(),
      accessor_handler: Some(accessor_handler),
      read_down: false,
      write_down: false,
    }
  }

  pub fn with_sync(
    src_addr: SocketAddrV4,
    dest_addr: SocketAddrV4,
    sync_pack: Tcp,
    bytes_assembled: Sender<u8>,
    packet_to_send: Sender<(Tcp, SocketAddrV4)>,
    packet_received: Receiver<(Tcp, SocketAddrV4)>,
    bytes_to_send: Receiver<u8>,
    control_signal: Receiver<StateControlSignal>,
    access_termination_signal: Sender<()>,
  ) -> Self {
    let mut recv_seq = Seq::with_u32(sync_pack.sequence);
    recv_seq.step();
    let mut reassembler = Reassembler::with_capacity(bytes_assembled, TcpStateMachineWorker::MAX_DATA_LENGTH);
    reassembler.ack();
    Self {
      src_addr,
      dest_addr: Some(dest_addr),
      send_seq: Seq::new(),
      recv_seq: Some(recv_seq),
      state: TcpState::SynReceived,
      peer_window_size: sync_pack.window,
      reassembler,
      send_buffer: Vec::new(),
      packet_to_send,
      packet_received,
      bytes_to_send,
      control_signal,
      access_termination_signal,
      terminating: false,
      accessor_handler: None,
      read_down: false,
      write_down: false,
    }
  }
  /// The state transition function
  pub(crate) fn run(&mut self) {
    log::info!("[Tcp Worker] start run. state: {:?}", self.state);
    loop {
      self.state = match self.state {
        TcpState::SynSent => self.handle_syn_sent(),
        TcpState::SynReceived => self.handle_syn_rcvd(),
        TcpState::Established => self.handle_established(),
        TcpState::FinWait1 => self.handle_fin_wait1(),
        TcpState::FinWait2 => self.handle_fin_wait2(),
        TcpState::Closing => self.handle_closing(),
        TcpState::CloseWait => self.handle_close_wait(),
        TcpState::LastAck => self.handle_last_ack(),
        TcpState::Closed => self.handle_closed(),
        TcpState::TimeWait => self.handle_time_wait(),
        TcpState::Terminate => {
          self.access_termination_signal.send(()).unwrap();
          if let Some(handle) = self.accessor_handler.take() {
            handle.join().unwrap();
          }
          break;
        }
      };
    }
    log::info!("[Tcp Worker] worker terminate");
  }
  // The handle function for TcpState::Closed
  fn handle_closed(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker] \n
     CLOSED"
    );
    if self.terminating {
      return TcpState::Terminate;
    }
    match self.control_signal.recv().unwrap() {
      // Active open
      StateControlSignal::Sync(dest_addr) => {
        log::debug!("[Tcp Worker] sync {}", dest_addr);
        self.dest_addr = Some(dest_addr);
        self.packet_to_send.send((self.pack_sync(), dest_addr)).unwrap();
        TcpState::SynSent
      }
      StateControlSignal::Terminate => TcpState::Terminate,
      StateControlSignal::ShutdownRead => {
        self.read_down = true;
        TcpState::Closed
      }
      StateControlSignal::ShutdownWrite => {
        self.write_down = true;
        TcpState::Closed
      }
    }
  }

  // The handle function for TcpState::SynSent
  fn handle_syn_sent(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker] \n
    SynSent"
    );
    let mut retry_count = 0;
    while retry_count < Self::MAX_RETRY_COUNT {
      if let Ok((packet, addr)) = self.packet_received.recv_timeout(Self::ESTIMATE_RTT) {
        // Check if the packet is sync-ack
        log::debug!(
          "self addr: {}, packet addr: {}\n
        is SYN-ACK: {},\n
        self seq: {}, packet ack:{}\n
        packet seq: {}
        ",
          self.src_addr,
          addr,
          packet.flags & TcpFlags::SYN != 0 && packet.flags & TcpFlags::ACK != 0,
          self.send_seq.next(),
          packet.acknowledgement,
          packet.sequence
        );
        if packet.flags & TcpFlags::SYN != 0
          && packet.flags & TcpFlags::ACK != 0
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
          self.send_ack();
          self.reassembler.sync();
          log::debug!("[TCP] worker enter established");
          self.send_data(true);
          return TcpState::Established;
        }
      }
      // If no data to send, check if the host tell us to shutdown.
      self.handle_control_signal();
      if self.read_down && self.write_down {
        return TcpState::Closed;
      }

      self
        .packet_to_send
        .send((self.pack_sync(), self.dest_addr.unwrap()))
        .unwrap();
      retry_count += 1
    }
    // Connection establish failed
    log::info!("[Tcp Worker] Sync failed.");
    TcpState::Terminate
  }

  // The handle function for TcpState::SynRcvd
  fn handle_syn_rcvd(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker] \n
    SynReceived"
    );
    let mut retry_count = 0;
    while retry_count < Self::MAX_RETRY_COUNT {
      // send sync-ack
      self
        .packet_to_send
        .send((self.pack_sync_ack(), self.dest_addr.unwrap()))
        .unwrap();
      if let Ok((packet, _)) = self.packet_received.recv_timeout(Self::ESTIMATE_RTT) {
        // check if it is ack
        if packet.flags & TcpFlags::ACK != 0 && packet.acknowledgement == self.send_seq.next() {
          // increase seq
          self.send_seq.step();
          self.peer_window_size = packet.window;
          // Get data
          self.update_data(&packet.payload, packet.sequence, packet.flags);
          self.send_data(true);
          return TcpState::Established;
        }
        retry_count += 1;
      }
    }
    TcpState::Closed
  }

  /// The handle function for TcpState::ESTABLISHED
  fn handle_established(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    Established"
    );
    let mut retry_times = 0;
    loop {
      // Check close/terminate
      self.handle_control_signal();

      let data_receive_result = self.receive_data();
      match data_receive_result {
        Ok(true) => {
          self.recv_seq.as_mut().unwrap().step();
          self.send_data(true);

          return TcpState::CloseWait;
        }
        Ok(false) => {
          self.send_data(true);

          retry_times = 0;
        }
        Err(_) => {
          if !self.send_buffer.is_empty() {
            retry_times += 1;
          }
          self.send_data(false);
        }
      }
      if self.write_down && self.bytes_to_send.is_empty() {
        return TcpState::FinWait1;
      }
      if retry_times > Self::MAX_RETRY_COUNT {
        return TcpState::Terminate;
      }
    }
  }

  /// The handle function for TcpState::FinWait1
  /// In this state, we have received close signal from the client.
  /// We are trying to send all our data.
  /// If we receive FIN from the peer, but haven't sent all of the data, we enter the CLOSING state.
  /// If we send all the data, and the same time we reive FIN from the peer, we enter TIME_WAIT.
  /// If we send all the data, and the peer still have data to send, we enter FIN_WAIT2.
  fn handle_fin_wait1(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    FinWait1"
    );
    let mut fin_received = false;
    let mut retry_count = 0;
    while !fin_received && !self.send_buffer.is_empty() {
      self.send_data(true);
      match self.receive_data() {
        Ok(true) => {
          retry_count = 0;
          if !fin_received {
            self.recv_seq.as_mut().unwrap().step()
          }
          fin_received = true;
        }
        Ok(false) => {
          retry_count = 0;
        }
        Err(_) => {
          log::debug!("[Tcp Worker] FinWait1 timeout");
          if !self.send_buffer.is_empty() {
            retry_count += 1;
          }
        }
      }
      if retry_count > Self::MAX_RETRY_COUNT {
        return TcpState::Terminate;
      }
    }
    if fin_received && self.send_buffer.is_empty() {
      TcpState::TimeWait
    } else if fin_received {
      TcpState::Closing
    } else {
      TcpState::FinWait2
    }
  }

  /// Function for TcpState::FinWait2.
  /// In this state, we have sent all of our data, and we wait for the peer to finish its transmission.
  /// After receive FIN from the peer, we enter TIME_WAIT.
  fn handle_fin_wait2(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    FinWait2"
    );
    loop {
      self.send_ack();
      match self.receive_data() {
        Ok(true) => {
          self.recv_seq.as_mut().unwrap().step();
          return TcpState::TimeWait;
        }
        _ => (),
      }
    }
  }

  /// Function for TcpState::Closing
  fn handle_closing(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    Closing"
    );
    let mut retry_times = 0;
    while !self.send_buffer.is_empty() {
      self.send_data(true);
      match self.receive_ack() {
        Ok(_) => retry_times = 0,
        Err(_) => retry_times += 1,
      }
      if retry_times > Self::MAX_RETRY_COUNT {
        return TcpState::Terminate;
      }
    }
    TcpState::TimeWait
  }

  /// Function for TcpState::CloseWait
  fn handle_close_wait(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    CloseWait"
    );
    let mut retry_times = 0;
    while retry_times < Self::MAX_RETRY_COUNT {
      self.handle_control_signal();
      if (self.write_down && self.bytes_to_send.is_empty()) || self.terminating {
        return TcpState::LastAck;
      }
      self.send_data(true);
      match self.receive_ack() {
        Ok(_) => retry_times = 0,
        Err(_) => {
          if !self.send_buffer.is_empty() {
            retry_times += 1;
          }
        }
      }
    }
    TcpState::Terminate
  }

  /// Function for TcpState::LastAck
  fn handle_last_ack(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    LastAck"
    );
    let mut retry_times = 0;
    while retry_times < Self::MAX_RETRY_COUNT {
      self.send_data(true);
      match self.receive_ack() {
        Ok(_) => {
          if self.send_buffer.is_empty() {
            return TcpState::Closed;
          }
          retry_times = 0;
        }
        Err(_) => {
          retry_times += 1;
        }
      }
    }
    TcpState::Terminate
  }

  /// Function for TcpState::TimeWait
  fn handle_time_wait(&mut self) -> TcpState {
    log::info!(
      "[Tcp Worker]: \n
    TimeWait"
    );
    loop {
      self.send_ack();
      if self.receive_data().is_err() {
        break TcpState::Closed;
      }
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
  // Pack a sync_ack packet
  fn pack_sync_ack(&self) -> Tcp {
    let mut packet = self.pack_vanilla();
    packet.flags = TcpFlags::ACK | TcpFlags::SYN;
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

  /// Pack a packet without any flag and payload
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

  /// Return Ok(true) if fin, Ok(false) if not fin. Return Err(()) if timeout
  fn receive_data(&mut self) -> Result<bool, ()> {
    // Receive the data
    if let Ok((packet, _)) = self.packet_received.recv_timeout(Self::ESTIMATE_RTT) {
      // Wrong packet send here
      // Update the send sequence
      log::debug!(
        "[Tcp Worker] Receive packet,\n
      seq = {}, ack = {},
      self ack = {}, seq = {}
      ACK: {}, SYN: {}, RST: {}, FIN: {}
      data offset: {}, data len: {},\n
      payload: {:?}
      ",
        packet.sequence,
        packet.acknowledgement,
        self.recv_seq.unwrap().sequence_number_u32(),
        self.send_seq.sequence_number_u32(),
        packet.flags & TcpFlags::ACK,
        packet.flags & TcpFlags::SYN,
        packet.flags & TcpFlags::RST,
        packet.flags & TcpFlags::FIN,
        packet.data_offset,
        packet.payload.len(),
        String::from_utf8_lossy(&packet.payload)
      );
      if packet.flags & TcpFlags::ACK != 0 {
        // Update the buffer
        for _ in 0..self.send_seq.update(WrappingInt32::new(packet.acknowledgement)) {
          self.send_buffer.pop();
        }
      }
      // Get the data
      self.update_data(&packet.payload, packet.sequence, packet.flags);
      if self.reassembler.is_fin() {
        return Ok(true);
      }
      // Update peer window
      self.peer_window_size = packet.window;
      return Ok(false);
    }
    Err(())
  }

  fn receive_ack(&mut self) -> Result<(), ()> {
    if let Ok((packet, _)) = self.packet_received.recv_timeout(Self::ESTIMATE_RTT) {
      log::debug!(
        "[Tcp Worker] Receive packet,\n
      seq = {}, ack = {},
      self ack = {}, seq = {}
      ",
        packet.sequence,
        packet.acknowledgement,
        self.recv_seq.unwrap().sequence_number_u32(),
        self.send_seq.sequence_number_u32()
      );
      // Wrong packet send here
      // Update the send sequence
      if packet.flags & TcpFlags::ACK != 0 {
        // Update the buffer
        for _ in 0..self.send_seq.update(WrappingInt32::new(packet.acknowledgement)) {
          self.send_buffer.pop();
        }
      }
      // Update peer window
      self.peer_window_size = packet.window;
      Ok(())
    } else {
      Err(())
    }
  }

  fn send_data(&mut self, ack: bool) {
    // Prepare the data
    while self.send_buffer.len() < self.peer_window_size as usize && self.send_buffer.len() < Self::MAX_DATA_LENGTH {
      if let Ok(byte) = self.bytes_to_send.try_recv() {
        self.send_buffer.push(byte);
      } else {
        break;
      }
    }
    if !self.write_down && self.send_buffer.is_empty() {
      if ack {
        self.send_ack()
      }
      return;
    }
    let packet = self.pack_data(
      self.send_buffer.clone(),
      self.write_down && self.bytes_to_send.is_empty(),
    );
    log::debug!(
      "[Tcp Worker] Send packet to {},\n
      seq = {}, ack = {}, len = {}\n
      FIN: {}
      ",
      self.dest_addr.unwrap(),
      packet.sequence,
      packet.acknowledgement,
      packet.payload.len(),
      packet.flags & TcpFlags::FIN
    );
    self.packet_to_send.send((packet, self.dest_addr.unwrap())).unwrap();
    log::debug!("[Tcp Worker] Packet sent successfully",);
  }

  fn send_ack(&mut self) {
    let packet = self.pack_ack();
    log::debug!(
      "[Tcp Worker] Send Ack to {},\n
      ack = {}
      ",
      self.dest_addr.unwrap(),
      packet.acknowledgement,
    );
    self.packet_to_send.send((packet, self.dest_addr.unwrap())).unwrap();
  }

  fn update_data(&mut self, data: &[u8], sequence: u32, flags: u16) {
    // Get the data
    let fin_flag = flags & TcpFlags::FIN != 0;
    // If client shutdown the read
    if self.read_down {
      self.reassembler.set_down();
    }
    let ack_delta = self.reassembler.update(
      data,
      WrappingInt32::unwrap(
        WrappingInt32::new(sequence),
        self.recv_seq.unwrap().initial_state_number,
        self.recv_seq.unwrap().absolute_seqence_number,
      ),
      fin_flag,
    );
    // Update recv seq
    self.recv_seq.as_mut().unwrap().add(ack_delta);
    log::trace!(
      "Peer seq update: \n
        delat:{}, seq: {}",
      ack_delta,
      self.recv_seq.unwrap().absolute_seqence_number,
    )
  }

  fn handle_control_signal(&mut self) {
    if let Ok(signal) = self.control_signal.try_recv() {
      match signal {
        StateControlSignal::ShutdownRead => self.read_down = true,
        StateControlSignal::ShutdownWrite => self.write_down = true,
        StateControlSignal::Terminate => {
          self.read_down = true;
          self.write_down = true;
          self.terminating = true;
        }
        _ => (),
      }
    }
  }
}
