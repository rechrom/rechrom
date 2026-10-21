//! Canonical unpremultiplied floating-point color carried by Paint records.
//!
//! Source: include/core/SkColor.h SkColor4f and src/core/SkColor.cpp. Concrete
//! legacy/translated Skia color values are constructed only by raster adapters.
#![allow(non_snake_case)]

use std::ops::{Index, IndexMut};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color4f {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color4f {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    // Source: SkColor4f::FromColor.
    pub fn FromColor(argb: u32) -> Self {
        let scale = 1. / 255.;
        Self {
            red: ((argb >> 16) & 255) as f32 * scale,
            green: ((argb >> 8) & 255) as f32 * scale,
            blue: (argb & 255) as f32 * scale,
            alpha: (argb >> 24) as f32 * scale,
        }
    }

    // Source: SkColor4f::toSkColor / SkColor4f_toBytes_RGBA.
    pub fn toSkColor(self) -> u32 {
        let byte = |value: f32| (value * 255. + 0.5).clamp(0., 255.) as u32;
        (byte(self.alpha) << 24)
            | (byte(self.red) << 16)
            | (byte(self.green) << 8)
            | byte(self.blue)
    }
}

impl Index<usize> for Color4f {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.red,
            1 => &self.green,
            2 => &self.blue,
            3 => &self.alpha,
            _ => panic!("Color4f component index {index} is out of range"),
        }
    }
}

impl IndexMut<usize> for Color4f {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.red,
            1 => &mut self.green,
            2 => &mut self.blue,
            3 => &mut self.alpha,
            _ => panic!("Color4f component index {index} is out of range"),
        }
    }
}
