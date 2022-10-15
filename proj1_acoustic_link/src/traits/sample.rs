use std::cmp::{PartialEq, PartialOrd};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// pub type FixedPoint = fixed::types::I20F12;
pub type FixedPoint = fixed::types::I32F32;

#[cfg(feature = "nofloat")]
pub type FP = FixedPoint;
#[cfg(not(feature = "nofloat"))]
pub type FP = f32;

pub trait Sample:
  'static
  + Copy
  + Clone
  + PartialOrd
  + PartialEq
  + Neg
  + Add
  + Sub
  + Mul
  + Div
  + AddAssign
  + SubAssign
  + MulAssign
  + DivAssign
{
  /// type cast: f32 -> Sample
  fn from_f32(x: f32) -> Self;
  /// type cast: Sample -> f32
  fn into_f32(self) -> f32;

  /// constant: PI
  const PI: Self;
  /// constant: TAU = 2 * PI
  const TAU: Self;
  /// constant: 1 One
  const ONE: Self;
  /// constant: 0 Zero
  const ZERO: Self;

  /// elementrary function square root
  fn sqrt(self) -> Self;
  /// trigeometric function sine
  fn sin(self) -> Self;
}

impl Sample for f32 {
  fn from_f32(x: f32) -> Self {
    x
  }
  fn into_f32(self) -> f32 {
    self
  }

  const PI: Self = std::f32::consts::PI;
  const TAU: Self = std::f32::consts::TAU;
  const ONE: Self = 1.0;
  const ZERO: Self = 0.0;

  fn sqrt(self) -> Self {
    f32::sqrt(self)
  }
  fn sin(self) -> Self {
    f32::sin(self)
  }
}

impl Sample for FixedPoint {
  fn from_f32(x: f32) -> Self {
    az::CastFrom::cast_from(x)
  }
  fn into_f32(self) -> f32 {
    az::Cast::cast(self)
  }

  const PI: Self = Self::PI;
  const TAU: Self = Self::TAU;
  const ONE: Self = Self::ONE;
  const ZERO: Self = Self::ZERO;

  fn sqrt(self) -> Self {
    cordic::sqrt(self)
  }
  fn sin(self) -> Self {
    cordic::sin(self)
  }
}
