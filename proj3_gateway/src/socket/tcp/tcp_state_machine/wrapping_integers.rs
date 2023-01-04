//! A 32-bit integer, expressed relative to an arbitrary initia&l sequence number (ISN)
//! This is used to express TCP sequence numbers (seqno) and acknowledgment numbers (ackno)
use std::fmt;
use std::ops;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct WrappingInt32 {
  raw_value: u32,
}

impl WrappingInt32 {
  pub fn new(raw_value: u32) -> WrappingInt32 {
    WrappingInt32 { raw_value }
  }

  pub fn raw_value(&self) -> u32 {
    self.raw_value
  }

  /// Transform a 64-bit absolute sequence number (zero-indexed) into a 32-bit relative sequence number
  /// n: the absolute sequence number
  /// isn: the initial sequence number
  /// returns the relative sequence number
  pub fn wrap(n: u64, isn: WrappingInt32) -> WrappingInt32 {
    isn + (n as u32)
  }

  /// Transform a 32-bit relative sequence number into a 64-bit absolute sequence
  /// number (zero-indexed)
  ///
  /// `n` - The relative sequence number  
  ///
  /// `isn` - The initial sequence number
  ///
  /// `checkpoint` - A recent absolute sequence number
  ///
  /// returns the absolute sequence number that wraps to `n` and is closest to
  /// `checkpoint`
  ///
  /// Each of the two streams of the TCP connection has its own ISN. One
  /// stream runs from the local TCPSender to the remote TCPReceiver and has one
  /// ISN, and the other stream runs from the remote TCPSender to the local
  /// TCPReceiver and has a different ISN.
  pub fn unwrap(n: WrappingInt32, isn: WrappingInt32, checkpoint: u64) -> u64 {
    // We shoud first wrap i32 around u32 and then extend to u64.
    // If we write `(n - isn) as u64` directly, we are wrapping i32 around u64,
    // which will cause an error.
    let offset = ((n - isn) as u32) as u64;
    if checkpoint < offset {
      offset
    } else {
      let offset = offset | (((checkpoint - offset) >> 32) << 32);
      if checkpoint - offset <= (1u64 << 31) {
        offset
      } else {
        offset.wrapping_add(1u64 << 32)
      }
    }
  }
}

/// # Helper Functions

/// The offset of `a` relative to `b`
/// returns the number of increments needed to get from `b` to `a`,
/// negative if the number of decrements needed is less than or equal to
/// the number of increments
impl ops::Sub<WrappingInt32> for WrappingInt32 {
  type Output = i32;

  fn sub(self, rhs: WrappingInt32) -> Self::Output {
    ((self.raw_value() as i64) - (rhs.raw_value() as i64)) as i32
  }
}

impl fmt::Display for WrappingInt32 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.raw_value())
  }
}

/// The point `b` steps past `a`.
impl ops::Add<u32> for WrappingInt32 {
  type Output = WrappingInt32;
  fn add(self, rhs: u32) -> Self::Output {
    WrappingInt32 {
      raw_value: self.raw_value().wrapping_add(rhs),
    }
  }
}

/// The point `b` steps before `a`.
impl ops::Sub<u32> for WrappingInt32 {
  type Output = WrappingInt32;

  fn sub(self, rhs: u32) -> Self::Output {
    WrappingInt32 {
      raw_value: self.raw_value().wrapping_sub(rhs),
    }
  }
}
