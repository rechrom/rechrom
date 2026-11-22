//! LayoutReplaced, LayoutImage, LayoutMedia and LayoutVideo: Chromium
//! core/layout/layout_replaced.{h,cc}, layout_image.{h,cc}, layout_media.cc and
//! layout_video.cc. Object-model classes owned by base (R7a); the replaced
//! layout algorithm stays in `layout_algo_replaced`.

#![allow(non_snake_case)]

#[path = "../replaced/layout_replaced_interface.rs"]
mod layout_algo_replaced_layout_replaced_interface;

pub use layout_algo_replaced_layout_replaced_interface::*;

#[path = "../replaced/layout_replaced_core.rs"]
pub mod layout_replaced_core;
#[path = "../replaced/layout_video.rs"]
pub mod layout_video;
#[path = "../replaced/layout_media.rs"]
pub mod layout_media;
