use std::ops::{Add, Div, Mul, Sub};

mod private {
    /// The `Sealed` trait stops crates other than kodama from implementing any
    /// traits that use it.
    pub trait Sealed {}
    impl Sealed for f32 {}
    impl Sealed for f64 {}

    /// The `SealedCast` trait stops crates other than kodama from implementing 
    /// any traits that use it.
    pub trait SealedCast {}
    impl SealedCast for f32 {}
    impl SealedCast for f64 {}
    impl SealedCast for usize {}
}

/// A trait for numbers that can be converted to a float primitive.
/// 
/// This is a simplified copy of the homonymous trait from the `num-traits`,
/// copied here to offer just the features required by `kodama`.
pub trait ToPrimitive:
    self::private::SealedCast
{
    fn to_f32(&self) -> Option<f32>;
    fn to_f64(&self) -> Option<f64>;
}

impl ToPrimitive for f32 {
    fn to_f32(&self) -> Option<f32> {
        Some(*self)
    }

    fn to_f64(&self) -> Option<f64> {
        Some(*self as f64)
    }
}

impl ToPrimitive for f64 {
    fn to_f32(&self) -> Option<f32> {
        Some(*self as f32)
    }

    fn to_f64(&self) -> Option<f64> {
        Some(*self)
    }
}

impl ToPrimitive for usize {
    fn to_f32(&self) -> Option<f32> {
        Some(*self as f32)
    }

    fn to_f64(&self) -> Option<f64> {
        Some(*self as f64)
    }
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
    /// Converts any floating type to this one.
    fn from<T: ToPrimitive>(v: T) -> Option<Self>;
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
    fn from<T: ToPrimitive>(v: T) -> Option<f32> {
        v.to_f32()
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
    fn from<T: ToPrimitive>(v: T) -> Option<f64> {
        v.to_f64()
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
