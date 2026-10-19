// Source: components/viz/common/surfaces/surface_info.h:1-63
// Source: components/viz/common/surfaces/surface_info.cc:1-23
#![allow(non_snake_case)]

use super::surface_id::SurfaceId;
use compositor_core::adapter::geometry::Size;

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceInfo {
    id_: SurfaceId,
    device_scale_factor_: f32,
    size_in_pixels_: Size,
}
impl SurfaceInfo {
    pub fn new(id: SurfaceId, scale: f32, size: Size) -> Self {
        Self {
            id_: id,
            device_scale_factor_: scale,
            size_in_pixels_: size,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.id_.is_valid() && self.device_scale_factor_ != 0.0 && !self.size_in_pixels_.IsEmpty()
    }
    pub fn id(&self) -> SurfaceId {
        self.id_
    }
    pub fn device_scale_factor(&self) -> f32 {
        self.device_scale_factor_
    }
    pub fn size_in_pixels(&self) -> Size {
        self.size_in_pixels_
    }
    pub fn ToString(&self) -> String {
        format!(
            "SurfaceInfo({}, scale {}, {:?})",
            self.id_, self.device_scale_factor_, self.size_in_pixels_
        )
    }
}
