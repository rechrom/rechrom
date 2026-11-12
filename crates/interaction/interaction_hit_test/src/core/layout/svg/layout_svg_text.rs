//! Source: core/layout/svg/layout_svg_text.cc:355-369, forwarding into the
//! existing LayoutSVGBlock physical fragment NodeAtPoint production path.
use super::transformed_hit_test_location::TransformedHitTestLocation;
use crate::core::{
    layout::{
        hit_test_location::HitTestLocation,
        hit_test_phase::HitTestPhase,
        hit_test_request::HitTestRequest,
        hit_test_result::{HitTestResult, HitTestResultRequest},
    },
    paint::box_fragment_painter::{BoxFragmentPainter, PhysicalFragment},
};
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;

pub trait SVGClipText<L: ?Sized> {
    fn LocalToSVGParentTransform(&self) -> AffineTransform;
    fn HasClipPath(&self) -> bool;
    fn ClipPathHitTest(&self, local: &HitTestLocation) -> bool;
    fn PhysicalFragments(&self) -> Box<dyn Iterator<Item = &dyn PhysicalFragment<L>> + '_>;
}
pub fn NodeAtPointForSvgClipContent<L: ?Sized>(
    text: &dyn SVGClipText<L>,
    location: &HitTestLocation,
) -> bool {
    let local = TransformedHitTestLocation::new(location, &text.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if text.HasClipPath() && !text.ClipPathHitTest(local) {
        return false;
    }
    let request: HitTestResultRequest<'_, L> =
        HitTestRequest::new(HitTestResultRequest::<L>::kSVGClipContent, None, None);
    let mut result = HitTestResult::<L>::new(&request, local);
    for fragment in text.PhysicalFragments() {
        if BoxFragmentPainter::new(fragment).NodeAtPoint(
            &mut result,
            local,
            Default::default(),
            HitTestPhase::kForeground,
        ) {
            return true;
        }
    }
    false
}

/// Preserve the caller's request and result for the general SVGText branch.
pub fn NodeAtPoint<'r>(
    text: &dyn SVGClipText<()>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    phase: HitTestPhase,
) -> bool {
    let local = TransformedHitTestLocation::new(location, &text.LocalToSVGParentTransform());
    if !local.IsValid() {
        return false;
    }
    let local = local.Location();
    if text.HasClipPath() && !text.ClipPathHitTest(local) {
        return false;
    }
    text.PhysicalFragments().any(|fragment| {
        BoxFragmentPainter::new(fragment).NodeAtPoint(result, local, Default::default(), phase)
    })
}
