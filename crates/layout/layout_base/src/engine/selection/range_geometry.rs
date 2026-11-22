//! Range::ComputeTextQuads/GetBorderAndTextQuads, range.cc:1705-1714,
//! 1745-1753,1764-1805; Document.cc:8928-8940; adjust_for_absolute_zoom.h:39-46,115-123.
//! Queries borrow the resident object/fragment owners and never run lifecycle.
use super::*;
use crate::internal::{layout_object::AssociatedLayoutObjectOfDefault, layout_text::LayoutText};
fn AdjustQuads(
    quads: &mut [gfx::QuadF],
    object: &LayoutObject,
    engine: &LayoutEngine,
    has_frame_view: bool,
    standardized_browser_zoom: bool,
) {
    if !has_frame_view {
        return;
    }
    let zoom = if standardized_browser_zoom {
        engine.constraints_.layout_zoom_factor as f32
    } else {
        object.StyleRef().EffectiveZoom()
    };
    assert!(
        zoom.is_finite() && zoom != 0.0,
        "source absolute zoom must be finite and nonzero"
    );
    if zoom != 1.0 {
        for quad in quads {
            quad.Scale(1.0 / zoom, 1.0 / zoom);
        }
    }
}
fn ComputeTextQuads(
    text: &LayoutText,
    start: u32,
    end: u32,
    engine: &LayoutEngine,
    has_frame_view: bool,
    standardized_browser_zoom: bool,
) -> Vec<gfx::QuadF> {
    let mut quads = Vec::new();
    text.AbsoluteQuadsForRange(&mut quads, start, end);
    AdjustQuads(
        &mut quads,
        text,
        engine,
        has_frame_view,
        standardized_browser_zoom,
    );
    quads
}
impl LayoutNode<'_> {
    pub fn RangeElementQuads(
        &self,
        has_frame_view: bool,
        standardized_browser_zoom: bool,
    ) -> Vec<gfx::QuadF> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let mut quads = Vec::new();
        self.object.AbsoluteQuads(
            &mut quads,
            crate::internal::map_coordinates_flags::MapCoordinatesFlags::default(),
            crate::internal::layout_object::BoxQuadType::kBorder,
        );
        AdjustQuads(
            &mut quads,
            self.object,
            self.output.engine,
            has_frame_view,
            standardized_browser_zoom,
        );
        quads
    }
    pub fn RangeTextQuads(
        &self,
        start: u32,
        end: u32,
        has_frame_view: bool,
        standardized_browser_zoom: bool,
    ) -> Vec<gfx::QuadF> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let Some(text) =
            (unsafe { foundation::DynamicTo::<LayoutText>(self.object as *const _).as_ref() })
        else {
            return Vec::new();
        };
        if !text.IsTextFragment() {
            return ComputeTextQuads(
                text,
                start,
                end,
                self.output.engine,
                has_frame_view,
                standardized_browser_zoom,
            );
        }
        // Text::GetLayoutObject is the remaining fragment. Resolve offset 0
        // through AssociatedLayoutObjectOf, including the source boundary rule.
        #[cfg(feature = "translation_in_progress")]
        {
            let node = unsafe { self.object.GetNode().as_ref() }
                .expect("Range Text has its canonical Node");
            let first = unsafe { AssociatedLayoutObjectOfDefault(node, 0).as_ref() }
                .expect("first-letter associated object");
            assert!(first.IsText());
            let first_text = unsafe { &*(first as *const LayoutObject).cast::<LayoutText>() };
            assert!(first_text.IsTextFragment());
            let first = unsafe {
                &*(first as *const LayoutObject)
                    .cast::<crate::layout_text_fragment::LayoutTextFragment>()
            };
            let mut quads = Vec::new();
            if start < first.FragmentLength() || (start == first.FragmentLength() && end == start) {
                quads.extend(ComputeTextQuads(
                    first,
                    start,
                    end.min(first.FragmentLength()),
                    self.output.engine,
                    has_frame_view,
                    standardized_browser_zoom,
                ));
            }
            let remaining = unsafe {
                &*(text as *const LayoutText)
                    .cast::<crate::layout_text_fragment::LayoutTextFragment>()
            };
            if end > remaining.Start() {
                let start = start.max(remaining.Start()) - remaining.Start();
                let end = if end == u32::MAX {
                    end
                } else {
                    end - remaining.Start()
                };
                quads.extend(ComputeTextQuads(
                    remaining,
                    start,
                    end,
                    self.output.engine,
                    has_frame_view,
                    standardized_browser_zoom,
                ));
            }
            quads
        }
        #[cfg(not(feature = "translation_in_progress"))]
        unreachable!("block-only assembly cannot allocate first-letter inline fragments")
    }
}
