//! Source native LayoutObject virtual dispatch over a borrowed resident view.
//! Source methods are dispatched to their source-named owning modules below.
//! This module supplies the typed native subclass selection and ports.
use super::{
    hit_test_location::HitTestLocation,
    hit_test_phase::HitTestPhase,
    hit_test_result::HitTestResult,
    layout_inline::{self, LayoutInlineHitTestSource},
};
use crate::core::paint::box_fragment_painter::PhysicalFragment;
use foundation::{layout_hit_test::LayoutHitTestKind as Kind, PhysicalOffset};

pub use super::layout_block::NodeAtPoint as BlockNodeAtPoint;
pub use super::layout_replaced::{NodeAtPoint as ReplacedNodeAtPoint, ReplacedHitTestSource};
/// A native subclass requires its own owner; absence is never a miss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeHitTestRequiredPort {
    Inline,
    TextControl,
    EmbeddedContent,
    SvgForeignObjectLayer {
        layout_identity: usize,
    },
    /// A real local child exists, but its prepared same-local-root query
    /// owner has not been supplied. This is never the remote/no-frame case.
    LocalChildFrame {
        frame_identity: u64,
    },
    Special(Kind),
    MissingLayoutObject,
    MissingPaintFragment {
        layout_identity: usize,
    },
}
pub type NativeHitTestResult = Result<bool, NativeHitTestRequiredPort>;

pub trait NativeLayoutHitTestSource<'r> {
    fn Kind(&self) -> Kind;
    fn Fragment(&self) -> Option<&dyn PhysicalFragment<()>>;
    fn MayIntersect(
        &self,
        result: &HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn Inline(&self) -> Result<&dyn LayoutInlineHitTestSource<'r, ()>, NativeHitTestRequiredPort> {
        Err(NativeHitTestRequiredPort::Inline)
    }
    fn TextControl(
        &self,
    ) -> Result<
        &dyn super::layout_text_control_hit_test::TextControlHitTestSource<'r>,
        NativeHitTestRequiredPort,
    > {
        Err(NativeHitTestRequiredPort::TextControl)
    }
    fn Replaced(&self) -> &dyn ReplacedHitTestSource<'r>;
    fn Embedded(
        &self,
    ) -> Result<
        &dyn super::layout_embedded_content_hit_test::EmbeddedContentHitTestSource<'r>,
        NativeHitTestRequiredPort,
    > {
        Err(NativeHitTestRequiredPort::EmbeddedContent)
    }
    /// The corresponding native SVG/form/frame override owns its extra
    /// source state. It receives the unchanged canonical request/result.
    fn SpecialNodeAtPoint(
        &self,
        kind: Kind,
        result: &mut HitTestResult<'r, ()>,
        location: &HitTestLocation,
        offset: PhysicalOffset,
        phase: HitTestPhase,
    ) -> NativeHitTestResult {
        let _ = (result, location, offset, phase);
        Err(NativeHitTestRequiredPort::Special(kind))
    }
}
pub fn NodeAtPoint<'r>(
    source: &dyn NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    Ok(match source.Kind() {
        Kind::Base | Kind::Hidden => false, // Actual LayoutObject/hidden source bodies.
        Kind::Block => BlockNodeAtPoint(source, result, location, offset, phase),
        Kind::TextControlSingleLine => super::layout_text_control_hit_test::SingleLineNodeAtPoint(
            source.TextControl()?,
            result,
            location,
            offset,
            phase,
        ),
        Kind::TextControlMultiLine => super::layout_text_control_hit_test::MultiLineNodeAtPoint(
            source.TextControl()?,
            result,
            location,
            offset,
            phase,
        ),
        Kind::Inline => {
            layout_inline::NodeAtPoint(source.Inline()?, result, location, offset, phase)
        }
        Kind::Replaced => ReplacedNodeAtPoint(source.Replaced(), result, location, offset, phase)?,
        Kind::EmbeddedContent => super::layout_embedded_content_hit_test::NodeAtPoint(
            source.Embedded()?,
            result,
            location,
            offset,
            phase,
        )?,
        Kind::Image => super::layout_image::NodeAtPoint(source, result, location, offset, phase)?,
        kind => source.SpecialNodeAtPoint(kind, result, location, offset, phase)?,
    })
}
pub fn HitTestAllPhases<'r>(
    source: &dyn NativeLayoutHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
) -> NativeHitTestResult {
    if matches!(
        source.Kind(),
        Kind::Block
            | Kind::Replaced
            | Kind::Image
            | Kind::EmbeddedContent
            | Kind::SvgRoot
            | Kind::TextControlSingleLine
            | Kind::TextControlMultiLine
    ) {
        super::layout_box_hit_test::HitTestAllPhases(source, result, location, offset)
    } else {
        super::layout_object::HitTestAllPhases(source, result, location, offset)
    }
}
