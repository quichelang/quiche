//! Inline (zero-overhead) types for small, frequently-copied values.
//!
//! Inspired by Lobster's inline structs, this provides a marker trait
//! for types that should be copied rather than reference-counted.

/// Marker trait for types that should be allocated inline.
///
/// Types implementing `Inline` are always copied (never reference-counted),
/// making them zero-overhead for small, frequently-used types like vectors
/// and colors.
///
/// # Example
///
/// ```
/// use perceus_mem::Inline;
///
/// #[derive(Clone, Copy)]
/// struct Vec2 { x: f32, y: f32 }
///
/// impl Inline for Vec2 {}
///
/// // Vec2 is always copied, never heap-allocated
/// let a = Vec2 { x: 1.0, y: 2.0 };
/// let b = a; // Copy
/// ```
pub trait Inline: Copy + Sized {}

// Implement for primitive types
impl Inline for i8 {}
impl Inline for i16 {}
impl Inline for i32 {}
impl Inline for i64 {}
impl Inline for i128 {}
impl Inline for isize {}
impl Inline for u8 {}
impl Inline for u16 {}
impl Inline for u32 {}
impl Inline for u64 {}
impl Inline for u128 {}
impl Inline for usize {}
impl Inline for f32 {}
impl Inline for f64 {}
impl Inline for bool {}
impl Inline for char {}
impl Inline for () {}

// Implement for small tuples of inline types
impl<A: Inline> Inline for (A,) {}
impl<A: Inline, B: Inline> Inline for (A, B) {}
impl<A: Inline, B: Inline, C: Inline> Inline for (A, B, C) {}
impl<A: Inline, B: Inline, C: Inline, D: Inline> Inline for (A, B, C, D) {}

// Implement for small arrays of inline types
impl<T: Inline, const N: usize> Inline for [T; N] {}

// Common inline types for math/graphics

/// 2D vector (inline, zero-overhead).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
}

impl Inline for Vec2 {}

/// 3D vector (inline, zero-overhead).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
}

impl Inline for Vec3 {}

/// 4D vector (inline, zero-overhead).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 1.0,
    };
}

impl Inline for Vec4 {}

/// RGBA color (inline, zero-overhead).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}

impl Inline for Color {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_vec2_is_small() {
        assert_eq!(size_of::<Vec2>(), 8);
    }

    #[test]
    fn test_vec3_is_small() {
        assert_eq!(size_of::<Vec3>(), 12);
    }

    #[test]
    fn test_color_is_small() {
        assert_eq!(size_of::<Color>(), 4);
    }

    #[test]
    fn test_inline_copy() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = v1; // Copy
        assert_eq!(v1, v2);
    }
}
