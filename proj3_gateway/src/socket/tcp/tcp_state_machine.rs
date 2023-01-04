use crossbeam_channel::{Receiver, Sender};
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
  pub fn connect(&self, dest: SocketAddrV4) -> Result<(), ()> {
    self.control_signal.send(StateControlSignal::Sync(dest)).map_err(|_| ())
  }
  pub fn shutdown(&self) -> Result<(), ()> {
    self.control_signal.send(StateControlSignal::Shutdown).map_err(|_| ())
  }
}

impl Drop for TcpStateMachine {
  // Gracefully shutdown
  fn drop(&mut self) {
    if let Some(thread) = self.join_handler.take() {
      thread.join().unwrap();
    }
  }
}

mod tcp_state_machine_worker;
mod wrapping_integers;
