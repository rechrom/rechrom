// Translation status: complete
// Copyright 2013 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/tiles/tile_draw_info.h:1-18,120-122;
//         cc/tiles/tile_draw_info.cc:1-11,50.
// Rust modules replace include guards, includes, namespaces and export macros.
use compositor_draw::Color4f;
use compositor_tree::{geometry::Size, property_types::PropertyTrace};

/// Minimal move-only lease contract consumed by tile lifecycle state.
/// The resource package supplies the concrete id, format, allocation and pool.
pub trait TileDrawResource: Default {
    type ResourceId: Copy;
    type Format;

    fn EstimatedSizeInBytes(&self, format: &Self::Format, size: &Size) -> usize;
    fn memory_usage(&self) -> usize;
    fn IsValid(&self) -> bool;
    fn resource_id_for_export(&self) -> Self::ResourceId;
    fn size(&self) -> &Size;
    fn format(&self) -> &Self::Format;
}

// Source: cc/tiles/tile_draw_info.h:19.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Mode {
    RESOURCE_MODE = 0,
    SOLID_COLOR_MODE = 1,
    OOM_MODE = 2,
}

// Source: cc/tiles/tile_draw_info.h:16-17,110-118.
// The resource lease is owned and is never cloned. Its external pool owns the
// resource implementation; the move/reset state machine belongs to this type.
pub struct TileDrawInfo<R: TileDrawResource> {
    mode_: Mode,
    solid_color_: Color4f,
    resource_: R,
    is_resource_ready_to_draw_: bool,
    resource_is_checker_imaged_: bool,
}
impl<R: TileDrawResource> Default for TileDrawInfo<R> {
    // Source: cc/tiles/tile_draw_info.h:110-117;
    //         cc/tiles/tile_draw_info.cc:13.
    fn default() -> Self {
        Self {
            mode_: Mode::RESOURCE_MODE,
            solid_color_: Color4f {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            },
            resource_: R::default(),
            is_resource_ready_to_draw_: false,
            resource_is_checker_imaged_: false,
        }
    }
}
impl<R: TileDrawResource> Drop for TileDrawInfo<R> {
    // Source: cc/tiles/tile_draw_info.cc:14-16.
    fn drop(&mut self) {
        debug_assert!(!self.resource_.IsValid());
    }
}
impl<R: TileDrawResource> TileDrawInfo<R> {
    // Source: cc/tiles/tile_draw_info.h:21-22; cc/tiles/tile_draw_info.cc:13.
    pub fn new() -> Self {
        Self::default()
    }
    // Source: cc/tiles/tile_draw_info.h:24.
    pub fn mode(&self) -> Mode {
        self.mode_
    }
    // Source: cc/tiles/tile_draw_info.h:26-35.
    // The exhaustive source enum makes the final NOTREACHED unrepresentable.
    pub fn IsReadyToDraw(&self) -> bool {
        match self.mode_ {
            Mode::RESOURCE_MODE => self.is_resource_ready_to_draw_,
            Mode::SOLID_COLOR_MODE | Mode::OOM_MODE => true,
        }
    }
    // Source: cc/tiles/tile_draw_info.h:36-46.
    pub fn NeedsRaster(&self) -> bool {
        match self.mode_ {
            Mode::RESOURCE_MODE => !self.resource_.IsValid(),
            Mode::SOLID_COLOR_MODE => false,
            Mode::OOM_MODE => true,
        }
    }
    // Source: cc/tiles/tile_draw_info.h:48-52.
    pub fn resource_id_for_export(&self) -> R::ResourceId {
        debug_assert_eq!(self.mode_, Mode::RESOURCE_MODE);
        debug_assert!(self.resource_.IsValid());
        self.resource_.resource_id_for_export()
    }
    // Source: cc/tiles/tile_draw_info.h:54-58.
    pub fn resource_size(&self) -> &Size {
        debug_assert_eq!(self.mode_, Mode::RESOURCE_MODE);
        debug_assert!(self.resource_.IsValid());
        self.resource_.size()
    }
    // Source: cc/tiles/tile_draw_info.h:60-64.
    pub fn resource_shared_image_format(&self) -> &R::Format {
        debug_assert_eq!(self.mode_, Mode::RESOURCE_MODE);
        debug_assert!(self.resource_.IsValid());
        self.resource_.format()
    }
    // Source: cc/tiles/tile_draw_info.h:66-69.
    pub fn solid_color(&self) -> Color4f {
        debug_assert_eq!(self.mode_, Mode::SOLID_COLOR_MODE);
        self.solid_color_
    }
    // Source: cc/tiles/tile_draw_info.h:71-73.
    pub fn requires_resource(&self) -> bool {
        self.mode_ == Mode::RESOURCE_MODE || self.mode_ == Mode::OOM_MODE
    }
    // Source: cc/tiles/tile_draw_info.h:75-76.
    pub fn has_resource(&self) -> bool {
        self.resource_.IsValid()
    }
    pub fn is_resource_ready_to_draw(&self) -> bool {
        self.is_resource_ready_to_draw_
    }
    // Source: cc/tiles/tile_draw_info.h:78;
    //         cc/tiles/tile_draw_info.cc:36-40.
    pub fn GetResource(&self) -> &R {
        debug_assert_eq!(self.mode_, Mode::RESOURCE_MODE);
        debug_assert!(self.resource_.IsValid());
        &self.resource_
    }
    // Source: cc/tiles/tile_draw_info.h:80-83.
    pub fn is_checker_imaged(&self) -> bool {
        debug_assert!(!self.resource_is_checker_imaged_ || self.resource_.IsValid());
        self.resource_is_checker_imaged_
    }
    // Source: cc/tiles/tile_draw_info.h:85.
    pub fn SetSolidColorForTesting(&mut self, color: Color4f) {
        self.set_solid_color(&color)
    }
    // Source: cc/tiles/tile_draw_info.h:87;
    //         cc/tiles/tile_draw_info.cc:18-22.
    pub fn AsValueInto(&self, state: &mut (impl PropertyTrace + ?Sized)) {
        state.SetBoolean("is_solid_color", self.mode_ == Mode::SOLID_COLOR_MODE);
        // Source dependency: third_party/skia/include/core/SkColor.h:345-347.
        let is_transparent = self.mode_ == Mode::SOLID_COLOR_MODE && {
            debug_assert!(self.solid_color_.alpha <= 1.0 && self.solid_color_.alpha >= 0.0);
            self.solid_color_.alpha != 1.0
        };
        state.SetBoolean("is_transparent", is_transparent);
    }
    // Source: cc/tiles/tile_draw_info.h:89-96;
    //         cc/tiles/tile_draw_info.cc:24-34.
    // crate visibility represents friendship to Tile/TileManager/fake layer.
    pub fn SetResource(&mut self, resource: R, resource_is_checker_imaged: bool) {
        debug_assert!(!self.resource_.IsValid());
        debug_assert!(resource.IsValid());
        self.mode_ = Mode::RESOURCE_MODE;
        self.is_resource_ready_to_draw_ = false;
        self.resource_is_checker_imaged_ = resource_is_checker_imaged;
        self.resource_ = resource;
    }
    // Source: cc/tiles/tile_draw_info.h:96;
    //         cc/tiles/tile_draw_info.cc:42-48.
    pub fn TakeResource(&mut self) -> R {
        debug_assert_eq!(self.mode_, Mode::RESOURCE_MODE);
        debug_assert!(self.resource_.IsValid());
        self.is_resource_ready_to_draw_ = false;
        self.resource_is_checker_imaged_ = false;
        // InUsePoolResource's source move leaves the existing slot empty.
        std::mem::take(&mut self.resource_)
    }
    // Source: cc/tiles/tile_draw_info.h:98-100.
    pub fn set_resource_ready_for_draw(&mut self) {
        self.is_resource_ready_to_draw_ = true
    }
    // Source: cc/tiles/tile_draw_info.h:102-106.
    pub fn set_solid_color(&mut self, color: &Color4f) {
        debug_assert!(!self.resource_.IsValid());
        self.mode_ = Mode::SOLID_COLOR_MODE;
        self.solid_color_ = *color;
    }
    // Source: cc/tiles/tile_draw_info.h:108.
    pub fn set_oom(&mut self) {
        self.mode_ = Mode::OOM_MODE
    }
}
