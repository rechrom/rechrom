//! Backend-owned canonical Skia path values for Paint geometry services.
//! Paint's immutable input contract never exposes a Skia implementation type.
#![allow(non_snake_case)]

pub mod path;

pub mod path_hit_test;

mod basic_shape_path;

pub mod svg_shape_path;

mod contoured_rect;

pub mod shape_coverage;
