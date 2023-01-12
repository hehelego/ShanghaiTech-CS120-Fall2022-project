use crossbeam_channel::{Receiver, Sender};
use pnet::packet::tcp::Tcp;
use std::{
  net::SocketAddrV4,
  thread::{self, JoinHandle},
};
use tcp_state_machine_worker::TcpStateMachineWorker;

/// Tcp control signal
enum StateControlSignal {
  Sync(SocketAddrV4),
  Shutdown,
  Terminate,
}

/// The TcpStateMachine.
/// TcpStateMachine manage the TcpStateMachineWorker.
pub(super) struct TcpStateMachine {
  join_handler: Option<JoinHandle<()>>,
  control_signal: Sender<StateControlSignal>,
}

impl TcpStateMachine {
  // Create a new state machine with state: Closed
  pub fn new(bytes_assembled: Sender<u8>, bytes_to_send: Receiver<u8>, addr: SocketAddrV4) -> Self {
    let (control_signal_tx, control_signal_rx) = crossbeam_channel::unbounded();
    let thread = thread::spawn(move || {
      let mut worker = TcpStateMachineWorker::new(bytes_assembled, bytes_to_send, control_signal_rx, addr);
      worker.run();
    });
    Self {
      join_handler: Some(thread),
      control_signal: control_signal_tx,
    }
  }
  /// Create a new state machine with state: SynReceived
  pub fn syn_received(
    src_addr: SocketAddrV4,
    dest_addr: SocketAddrV4,
    sync_pack: Tcp,
    bytes_assembled: Sender<u8>,
    packet_to_send: Sender<(Tcp, SocketAddrV4)>,
    packet_received: Receiver<(Tcp, SocketAddrV4)>,
    bytes_to_send: Receiver<u8>,
    access_termination_signal: Sender<()>,
  ) -> Self {
    let (control_signal_tx, control_signal_rx) = crossbeam_channel::unbounded();
    let handle = thread::spawn(move || {
      let mut worker = TcpStateMachineWorker::with_sync(
        src_addr,
        dest_addr,
        sync_pack,
        bytes_assembled,
        packet_to_send,
        packet_received,
        bytes_to_send,
        control_signal_rx,
        access_termination_signal,
      );
      worker.run();
    });

    Self {
      join_handler: Some(handle),
      control_signal: control_signal_tx,
    }
  }

  pub fn connect(&self, dest: SocketAddrV4) -> Result<(), ()> {
    log::debug!("[Tcp Machine] connect");
    self.control_signal.send(StateControlSignal::Sync(dest)).map_err(|_| ())
  }
  pub fn shutdown(&self) -> Result<(), ()> {
    self.control_signal.send(StateControlSignal::Shutdown).map_err(|_| ())
  }
}

impl Drop for TcpStateMachine {
  // Gracefully shutdown
  fn drop(&mut self) {
    self.control_signal.send(StateControlSignal::Terminate).unwrap();
    if let Some(thread) = self.join_handler.take() {
      thread.join().unwrap();
    }
  }
}

mod tcp_state_machine_worker;
mod wrapping_integers;
