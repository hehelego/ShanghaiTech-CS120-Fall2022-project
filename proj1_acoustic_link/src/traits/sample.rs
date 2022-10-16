use std::cmp::{PartialEq, PartialOrd};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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

/// create a number type wrapper,
/// implement arithmetic operator traits for it.
macro_rules! num_type {
  ($name:ident,$inner:ty) => {
    use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
    #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
    pub struct $name($inner);
    impl Neg for $name {
      type Output = $name;
      fn neg(self) -> $name {
        $name(-self.0)
      }
    }
    impl Add for $name {
      type Output = $name;
      fn add(self, rhs: $name) -> $name {
        $name(self.0 + rhs.0)
      }
    }
    impl Sub for $name {
      type Output = $name;
      fn sub(self, rhs: $name) -> $name {
        $name(self.0 - rhs.0)
      }
    }
    impl Mul for $name {
      type Output = $name;
      fn mul(self, rhs: $name) -> $name {
        $name(self.0 * rhs.0)
      }
    }
    impl Div for $name {
      type Output = $name;
      fn div(self, rhs: $name) -> $name {
        $name(self.0 / rhs.0)
      }
    }
    impl AddAssign for $name {
      fn add_assign(&mut self, rhs: $name) {
        self.0 += rhs.0;
      }
    }
    impl SubAssign for $name {
      fn sub_assign(&mut self, rhs: $name) {
        self.0 -= rhs.0;
      }
    }
    impl MulAssign for $name {
      fn mul_assign(&mut self, rhs: $name) {
        self.0 *= rhs.0;
      }
    }
    impl DivAssign for $name {
      fn div_assign(&mut self, rhs: $name) {
        self.0 /= rhs.0;
      }
    }
  };
}

#[cfg(feature = "nofloat")]
pub use fixed_point_sample::FP;
#[cfg(not(feature = "nofloat"))]
pub use float_point_sample::FP;

#[cfg(feature = "nofloat")]
mod fixed_point_sample {
  type FixedPoint = fixed::types::I32F32;
  num_type! {FP, FixedPoint}
  impl super::Sample for FP {
    fn from_f32(x: f32) -> Self {
      Self(az::CastFrom::cast_from(x))
    }
    fn into_f32(self) -> f32 {
      az::Cast::cast(self.0)
    }

    const PI: Self = Self(FixedPoint::PI);
    const TAU: Self = Self(FixedPoint::TAU);
    const ONE: Self = Self(FixedPoint::ONE);
    const ZERO: Self = Self(FixedPoint::ZERO);

    fn sqrt(self) -> Self {
      Self(cordic::sqrt(self.0))
    }
    fn sin(self) -> Self {
      Self(cordic::sin(self.0))
    }
  }
}
#[cfg(not(feature = "nofloat"))]
mod float_point_sample {
  num_type! {FP, f32}
  impl super::Sample for FP {
    fn from_f32(x: f32) -> Self {
      Self(x)
    }
    fn into_f32(self) -> f32 {
      self.0
    }

    const PI: Self = Self(std::f32::consts::PI);
    const TAU: Self = Self(std::f32::consts::TAU);
    const ONE: Self = Self(1.0);
    const ZERO: Self = Self(0.0);

    fn sqrt(self) -> Self {
      Self(self.0.sqrt())
    }
    fn sin(self) -> Self {
      Self(self.0.sin())
    }
  }
}
