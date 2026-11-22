//! LayoutReplaced, LayoutImage, LayoutMedia and LayoutVideo: Chromium
//! core/layout/layout_replaced.{h,cc}, layout_image.{h,cc}, layout_media.cc and
//! layout_video.cc. Object-model classes owned by base (R7a); the replaced
//! layout algorithm stays in `layout_algo_replaced`.

#![allow(non_snake_case)]

mod layout_algo_replaced_layout_replaced_interface;

pub use layout_algo_replaced_layout_replaced_interface::*;

pub mod layout_replaced_core;
pub mod layout_video;
pub mod layout_media;
