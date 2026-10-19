//! Non-owning identity used in place of C++ PropertyTrees backpointers.
//! Data and algorithms remain in cc/trees/property_tree.rs. Real-time context
//! borrows resolve this identity; it never stores/copies tree or node snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyTreesOwnerId(u64);
impl PropertyTreesOwnerId {
    pub const fn FromOwnerValue(value: u64) -> Self {
        Self(value)
    }
}

use super::geometry::{PointF, Vector2dF};
use super::property_types::PropertyTransform;
/// Primitive gfx::Transform services required by property-tree algorithms.
/// Matrix storage/arithmetic stays in the owning geometry implementation.
pub trait PropertyTreeTransform: PropertyTransform {
    fn PostConcat(&mut self, transform: &Self);
    fn PreConcat(&mut self, transform: &Self);
    fn MultiplyAssign(&mut self, transform: &Self);
    fn Flatten(&mut self);
    fn GetInverse(&self, inverse: &mut Self) -> bool;
    fn Translate(&mut self, x: f32, y: f32);
    fn Translate3d(&mut self, x: f32, y: f32, z: f32);
    fn Scale(&mut self, x: f32, y: f32);
    fn PostTranslate(&mut self, offset: Vector2dF);
    fn MapPoint(&self, point: PointF) -> PointF;
    fn Round2dTranslationComponents(&mut self);
    fn IsApproximatelyIdentityOrTranslation(&self, tolerance: f32) -> bool;
    fn IsApproximatelyIdentityOrIntegerTranslation(&self, tolerance: f32) -> bool;
    fn RoundToIdentityOrIntegerTranslation(&mut self);
    fn IsBackFaceVisible(&self) -> bool;
    fn ComputeTransform2dScaleComponents(&self, fallback: f32) -> Vector2dF;
    fn IsPositiveScaleOrTranslation(&self) -> bool;
    fn MapRect(&self, rect: super::geometry::RectF) -> super::geometry::RectF;
    fn HasPerspective(&self) -> bool;
    fn PostScale(&mut self, x: f32, y: f32);
}

impl Vector2dF {
    pub fn IsZero(self) -> bool {
        self.x() == 0.0 && self.y() == 0.0
    }
}
pub fn RoundedVector2dF(value: Vector2dF) -> Vector2dF {
    Vector2dF::new(
        value.x().round() as i32 as f32,
        value.y().round() as i32 as f32,
    )
}

#[derive(Clone, Copy, Debug, Default)]
pub struct OutsetsF {
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
}
impl super::geometry::RectF {
    pub fn Outset(&mut self, outsets: OutsetsF) {
        *self = Self::new(
            self.x() - outsets.left,
            self.y() - outsets.top,
            self.width() + outsets.left + outsets.right,
            self.height() + outsets.top + outsets.bottom,
        );
    }
}

/// External paint operations; implementation belongs to the paint backend.
pub trait PropertyTreeFilters: super::property_types::FilterOperations {
    fn AllowsLCDText(&self) -> bool;
    fn HasZoomFilter(&self) -> bool;
}
// Canonical native value owner; no duplicate flags representation.
pub use crate::cc::layers::render_surface_impl::RenderSurfacePropertyChangedFlags;
pub trait PropertyTreeRenderSurface {
    fn id(&self) -> super::input_types::ElementId;
    fn set_effect_tree_index(&mut self, id: i32);
    fn GetPropertyChangeFlags(&self) -> RenderSurfacePropertyChangedFlags;
    fn ApplyPropertyChangeFlags(&mut self, flags: RenderSurfacePropertyChangedFlags);
    fn ResetPropertyChangedFlags(&mut self);
}
pub trait PropertyTreeRenderSurfaceFactory<R: PropertyTreeRenderSurface> {
    fn CreateRenderSurface(&mut self, id: super::input_types::ElementId) -> Box<R>;
}
/// Viz requests retain owned lifetime and abort-on-drop behavior in the backend.
pub trait PropertyTreeCopyOutputRequest {
    fn has_area(&self) -> bool;
    fn area(&self) -> super::geometry::Rect;
    fn set_area(&mut self, area: super::geometry::Rect);
    fn is_scaled(&self) -> bool;
    fn has_result_selection(&self) -> bool;
    fn scale_from(&self) -> super::geometry::Vector2d;
    fn scale_to(&self) -> super::geometry::Vector2d;
    fn SetScaleRatio(&mut self, from: super::geometry::Vector2d, to: super::geometry::Vector2d);
}
pub trait PropertyTreeFeatures {
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool;
}

pub trait PropertyTreeScrollLayerTree<T: PropertyTreeTransform> {
    fn DidUpdateScrollOffset(
        &mut self,
        id: super::input_types::ElementId,
        pushed_from_main_or_pending_tree: bool,
        scroll_tree: &crate::cc::trees::property_tree::ScrollTree,
        context: &mut crate::cc::trees::property_tree::ScrollTreeMutationContext<'_, T>,
    );
    fn DidAnimateScrollOffset(&mut self);
}
pub trait PropertyTreeMutatorHost {
    fn ScrollOffsetAnimationWasInterrupted(&self, id: super::input_types::ElementId) -> bool;
}

/// ui/gfx/animation/keyframe/target_property.h:12-15 std::bitset<32>.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TargetProperties(pub u32);
impl TargetProperties {
    pub fn test(self, index: usize) -> bool {
        assert!(index < 32);
        self.0 & (1u32 << index) != 0
    }
    pub fn set(&mut self, index: usize, value: bool) {
        assert!(index < 32);
        if value {
            self.0 |= 1u32 << index;
        } else {
            self.0 &= !(1u32 << index);
        }
    }
    pub fn reset(&mut self) {
        self.0 = 0;
    }
    pub fn all(self) -> bool {
        self.0 == u32::MAX
    }
}
impl std::ops::BitOr for TargetProperties {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
impl std::ops::BitXor for TargetProperties {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}
impl std::ops::BitAnd for TargetProperties {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}
impl std::ops::Not for TargetProperties {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl super::geometry::Size {
    pub fn IsEmpty(self) -> bool {
        self.width() == 0 || self.height() == 0
    }
    pub fn Enlarge(&mut self, width: i32, height: i32) {
        *self = Self::new(
            self.width().saturating_add(width),
            self.height().saturating_add(height),
        );
    }
}
impl super::geometry::SizeF {
    pub fn Enlarge(&mut self, width: f32, height: f32) {
        *self = Self::new(self.width() + width, self.height() + height);
    }
}
impl Vector2dF {
    pub fn Scale(&mut self, scale: f32) {
        self.set_x(self.x() * scale);
        self.set_y(self.y() * scale);
    }
}
