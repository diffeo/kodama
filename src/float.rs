use std::ops::{Add, Div, Mul, Sub};

mod private {
    /// The `Sealed` trait stops crates other than kodama from implementing any
    /// traits that use it.
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}
}

/// A trait for writing generic code over floating point numbers.
///
/// We used to use the corresponding trait from the `num-traits` crate, but for
/// operational simplicity, we provide our own trait to avoid the dependency.
///
/// This trait is sealed. Callers therefore can not implement it. It is only
/// implemented for the `f32` and `f64` types.
pub trait Float:
    self::private::Sealed
    + Copy
    + Clone
    + PartialEq
    + PartialOrd
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<Self, Output = Self>
{
    /// Converts a `usize` to a float.
    fn from_usize(v: usize) -> Self;
    /// Converts any floating type to this one.
    fn from_float<F: Float>(v: F) -> Self;
    /// Converts this floating type to a `f64`.
    fn to_f64(self) -> f64;

    /// Returns the representation of "infinity" for this float type.
    fn infinity() -> Self;
    /// Returns the maximum value for this float type.
    fn max_value() -> Self;
    /// Returns the square root.
    fn sqrt(self) -> Self;
    /// Returns the absolute value.
    fn abs(self) -> Self;
}

impl Float for f32 {
    fn from_usize(v: usize) -> f32 {
        v as f32
    }

    fn from_float<F: Float>(v: F) -> f32 {
        v.to_f64() as f32
    }

    fn to_f64(self) -> f64 {
        self as f64
    }

    fn infinity() -> f32 {
        f32::INFINITY
    }

    fn max_value() -> f32 {
        f32::MAX
    }

    fn sqrt(self) -> f32 {
        f32::sqrt(self)
    }

    fn abs(self) -> f32 {
        f32::abs(self)
    }
}

impl Float for f64 {
    fn from_usize(v: usize) -> f64 {
        v as f64
    }

    fn from_float<F: Float>(v: F) -> f64 {
        v.to_f64()
    }

    fn to_f64(self) -> f64 {
        self
    }

    fn infinity() -> f64 {
        f64::INFINITY
    }

    fn max_value() -> f64 {
        f64::MAX
    }

    fn sqrt(self) -> f64 {
        f64::sqrt(self)
    }

    fn abs(self) -> f64 {
        f64::abs(self)
    }
}
