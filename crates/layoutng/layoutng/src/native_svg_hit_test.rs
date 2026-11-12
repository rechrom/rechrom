//! Resident SVG geometry queries for translated Interaction NodeAtPoint.
//! Chromium layout_svg_shape.h:69-99,126-141; layout_svg_shape.cc:300-330,
//! 530-602; layout_svg_container.cc:227-277; layout_svg_root.cc:505-525;
//! layout_svg_viewport_container.cc:94-113. No SVG tree or Path is projected.
use super::NativeHitTestCursor;
use crate::internal::{layout_algorithm_set::SvgLayoutSupport, layout_pass_scope::LayoutPassScope};
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform,
    gfx::RectF,
    native_path::{NativePath, ShapeFillGeometry, ShapeStrokeGeometry},
};

impl<'a> NativeHitTestCursor<'a> {
    pub fn SvgReferenceBox(
        self,
        geometry: layoutng_style::style::computed_style_constants::GeometryBox,
    ) -> RectF {
        self.SvgHitSupport()
            .hit_test_reference_box
            .expect("native SVG reference box")(self.object, geometry)
    }
    pub fn SvgResolveClip(
        self,
        reference: &layoutng_style::style::reference_clip_path_operation::ReferenceClipPathOperation,
    ) -> Option<Self> {
        let query = self
            .SvgHitSupport()
            .hit_test_resolve_clip
            .expect("native SVG resource resolver");
        let pointer =
            LayoutPassScope::WithSVGResourceTree(|tree| query(tree, self.object, reference))
                .expect("resident SVG clip query retains resource-tree scope");
        // ResourceQueryScope borrows the same retained Layout tree as this
        // cursor; the native resolver returns an owner from that tree.
        unsafe { pointer.as_ref() }.map(Self::ForObject)
    }
    pub fn SvgClipUnits(self) -> bool {
        self.SvgHitSupport()
            .hit_test_clip_units
            .expect("native clip units")(self.object)
    }
    pub fn SvgClipTransform(self) -> AffineTransform {
        self.SvgHitSupport()
            .hit_test_clip_transform
            .expect("native clip transform")(self.object)
    }
    pub fn IsSvgChild(self) -> bool {
        self.object.IsSVGChild()
    }
    pub fn FirstChild(self) -> Option<Self> {
        unsafe { self.object.SlowFirstChild().as_ref() }.map(Self::ForObject)
    }
    pub fn NextSibling(self) -> Option<Self> {
        unsafe { self.object.NextSibling().as_ref() }.map(Self::ForObject)
    }
    pub fn LockedAncestorPreventingLayout(self) -> bool {
        let mut parent = self.object.Parent();
        while let Some(object) = unsafe { parent.as_ref() } {
            if object.ChildLayoutBlockedByDisplayLock() {
                return true;
            }
            parent = object.Parent();
        }
        false
    }
    /// SVGUseElement::VisibleTargetGraphicsElementForClipping, cc:545-560.
    /// The already built native instance root owns the direct shape/text.
    pub fn SvgIsUse(self) -> bool {
        use crate::internal::layout_node_metadata::Element;
        unsafe { foundation::DynamicTo::<Element>(self.object.GetNode()).as_ref() }
            .and_then(|node| node.InputElementData().as_ref())
            .is_some_and(|data| data.svg_use.is_some())
    }
    pub fn SvgUseClipTarget(self) -> Option<Self> {
        let target = self.FirstChild()?;
        matches!(
            target.Kind(),
            foundation::layout_hit_test::LayoutHitTestKind::SvgShape
                | foundation::layout_hit_test::LayoutHitTestKind::SvgText
        )
        .then_some(target)
    }
    fn SvgHitSupport(self) -> &'a SvgLayoutSupport {
        // The read-query scope owns this algorithm table for the whole query,
        // just as it owns the native object and resource-tree leases.
        &unsafe { LayoutPassScope::Algorithms().as_ref() }
            .expect("resident SVG query retains its algorithm scope")
            .svg_support
    }
    pub fn SvgPath(self) -> Option<&'a dyn NativePath> {
        self.SvgHitSupport()
            .hit_test_path
            .expect("native SVG Path query")(self.object)
    }
    pub fn SvgShapeEmpty(self) -> bool {
        self.SvgHitSupport()
            .hit_test_shape_empty
            .expect("native SVG shape emptiness")(self.object)
    }
    pub fn SvgFillGeometry(self) -> ShapeFillGeometry {
        self.SvgHitSupport()
            .hit_test_fill_geometry
            .expect("native SVG fill geometry")(self.object)
    }
    pub fn SvgStrokeGeometry(self) -> ShapeStrokeGeometry {
        self.SvgHitSupport()
            .hit_test_stroke_geometry
            .expect("native SVG stroke geometry")(self.object)
    }
    pub fn SvgHitTestStrokeBounds(self) -> RectF {
        self.SvgHitSupport()
            .hit_test_stroke_bounds
            .expect("native SVG stroke bounds")(self.object)
    }
    pub fn SvgStrokeWidth(self) -> f32 {
        self.SvgHitSupport()
            .hit_test_stroke_width
            .expect("native SVG stroke width")(self.object)
    }
    pub fn SvgObjectBoundsValid(self) -> bool {
        self.SvgHitSupport()
            .hit_test_object_bounds_valid
            .expect("native SVG bounds validity")(self.object)
    }
    pub fn SvgObjectBoundingBox(self) -> RectF {
        self.SvgHitSupport()
            .object_bounding_box
            .expect("native SVG object bounds")(self.object)
    }
    pub fn SvgDecoratedBoundingBox(self) -> RectF {
        self.SvgHitSupport()
            .decorated_bounding_box
            .expect("native SVG decorated bounds")(self.object)
    }
    pub fn SvgFilteredDecoratedBoundingBox(self) -> RectF {
        let query = self
            .SvgHitSupport()
            .hit_test_filtered_bounds
            .expect("native SVG filter bounds");
        LayoutPassScope::WithSVGResourceTree(|tree| query(tree, self.object))
            .expect("resident SVG filter query retains resource-tree scope")
    }
    pub fn SvgHasPaintServer(self, stroke: bool) -> bool {
        let query = self
            .SvgHitSupport()
            .hit_test_has_paint_server
            .expect("native SVG paint server");
        LayoutPassScope::WithSVGResourceTree(|tree| query(tree, self.object, stroke))
            .expect("resident SVG paint-server query retains resource-tree scope")
    }
    pub fn SvgLocalToParentTransform(self) -> AffineTransform {
        self.SvgHitSupport()
            .local_to_parent_transform
            .expect("native SVG local transform")(self.object)
    }
    pub fn SvgRootContentTransform(self) -> AffineTransform {
        self.SvgHitSupport()
            .hit_test_root_transform
            .expect("native SVG root transform")(self.object)
    }
    pub fn SvgViewport(self) -> (RectF, AffineTransform) {
        self.SvgHitSupport()
            .hit_test_viewport
            .expect("native SVG viewport")(self.object)
    }
    /// SVGLayoutSupport::IsOverflowHidden (cc:231-235) intentionally reads X.
    pub fn SvgOverflowHidden(self) -> bool {
        matches!(
            self.Style().OverflowX(),
            foundation::EOverflow::kHidden
                | foundation::EOverflow::kClip
                | foundation::EOverflow::kScroll
        )
    }
    pub fn HasClipPath(self) -> bool {
        self.object.HasClipPath()
    }
    pub fn IsSvgForeignObject(self) -> bool {
        self.object.IsSVGForeignObject()
    }
}
