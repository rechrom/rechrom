//! resize_observer_utilities.cc:24-120 and resize_observer_entry.cc:79-105.
//! Borrow the resident LayoutObject; no fragment or style copy is retained.
use super::*;
use resize_observer::{ObservationSize, ResizeObserverGeometry, ResizeObserverSize};
fn raw_size(inline: foundation::LayoutUnit, block: foundation::LayoutUnit) -> ObservationSize {
    ObservationSize {
        // ComputeTargetSize constructs LogicalSize from ComputeZoomAdjustedBox's
        // gfx::SizeF, so retain that float roundtrip even at large dimensions.
        inline_size: foundation::LayoutUnit::from_f32(inline.ToFloat()).RawValue(),
        block_size: foundation::LayoutUnit::from_f32(block.ToFloat()).RawValue(),
    }
}
fn size(inline: foundation::LayoutUnit, block: foundation::LayoutUnit) -> ResizeObserverSize {
    // Source gfx::SizeF is float even though ResizeObserverSize exposes double.
    ResizeObserverSize {
        inline_size: f64::from(inline.ToFloat()),
        block_size: f64::from(block.ToFloat()),
    }
}
struct ResizeObserverLayoutGeometry<'a> {
    object: &'a crate::internal::layout_object::LayoutObject,
    engine: &'a LayoutEngine,
}
impl ResizeObserverLayoutGeometry<'_> {
    pub fn ResizeObserverIsNonAtomicInline(&self) -> bool {
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        self.object.IsNonAtomicInline()
    }
    pub fn ResizeObserverLayoutBoxGeometry(&self) -> Option<ResizeObserverGeometry> {
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        let box_ = unsafe { DynamicTo::<LayoutBox>(self.object as *const _).as_ref() }?;
        let style = self.object.StyleRef();
        let zoom = style.EffectiveZoom();
        let content = box_.PhysicalContentBoxRect().size;
        let (inline, block) = if style.IsHorizontalWritingMode() {
            (content.width, content.height)
        } else {
            (content.height, content.width)
        };
        let adjust = |value: foundation::LayoutUnit| foundation::LayoutUnit::from_f32(value / zoom);
        let content_inline = adjust(inline);
        let content_block = adjust(block);
        let border_inline = adjust(box_.LogicalWidth());
        let border_block = adjust(box_.LogicalHeight());
        let offset = self.object.FirstFragment().PaintOffset();
        let (inline_offset, block_offset) = if style.IsHorizontalWritingMode() {
            (offset.left, offset.top)
        } else {
            (offset.top, offset.left)
        };
        let device_inline = foundation::SnapSizeToPixel(inline, inline_offset);
        let device_block = foundation::SnapSizeToPixel(block, block_offset);
        let padding = box_.PaddingOutsets();
        Some(ResizeObserverGeometry {
            content_rect: [
                f64::from(adjust(padding.left).ToFloat()),
                f64::from(adjust(padding.top).ToFloat()),
                f64::from(adjust(content.width).ToFloat()),
                f64::from(adjust(content.height).ToFloat()),
            ],
            content_box: size(content_inline, content_block),
            border_box: size(border_inline, border_block),
            device_pixel_content_box: ResizeObserverSize {
                inline_size: f64::from(device_inline),
                block_size: f64::from(device_block),
            },
            observation_sizes: [
                raw_size(content_inline, content_block),
                raw_size(border_inline, border_block),
                raw_size(
                    foundation::LayoutUnit::from_signed(device_inline),
                    foundation::LayoutUnit::from_signed(device_block),
                ),
            ],
        })
    }
}
impl ResizeObserverLayoutGeometry<'_> {
    /// ResizeObservation.cc:29-49,123-145; ResizeObserverEntry.cc:107-135.
    /// SVG GraphicsElement identity is supplied by canonical DOM allocation.
    /// Counter facts are source outputs; the caller records them on Document.
    pub fn ResizeObserverGeometry(
        &self,
        is_svg_graphics: bool,
    ) -> (ResizeObserverGeometry, bool, bool) {
        let _scope = LayoutPassScope::new(&self.engine.algorithms_, &self.engine.objects_);
        if !self.object.IsSVGChild() {
            return (
                self.ResizeObserverLayoutBoxGeometry().unwrap_or_default(),
                false,
                false,
            );
        }
        if !is_svg_graphics {
            return (ResizeObserverGeometry::default(), false, false);
        }
        let style = self.object.StyleRef();
        let zoom = style.EffectiveZoom();
        let mut bounds = self.object.ObjectBoundingBox();
        let diagnostics = self
            .engine
            .algorithms_
            .svg_support
            .resize_observer_bbox_diagnostics
            .expect("native SVG GetBBox diagnostic owner")(self.object);
        // SVGGraphicsElement::GetBBox -> NoopWillBeInvScaleRect.
        if foundation::RuntimeEnabledFeatures::SvgNewZoomEnabled() {
            assert!(
                zoom.is_finite() && zoom != 0.0,
                "source SVG inverse scale requires nonzero finite zoom"
            );
            bounds.Scale(1.0 / zoom, 1.0 / zoom);
        }
        let width = bounds.width();
        let height = bounds.height();
        let content = ResizeObserverSize {
            inline_size: f64::from(width),
            block_size: f64::from(height),
        };
        let observation = raw_size(
            foundation::LayoutUnit::from_f32(width),
            foundation::LayoutUnit::from_f32(height),
        );
        let offset = self.object.FirstFragment().PaintOffset();
        let (inline_offset, block_offset) = if style.IsHorizontalWritingMode() {
            (offset.left, offset.top)
        } else {
            (offset.top, offset.left)
        };
        let device_inline = foundation::SnapSizeToPixel(
            foundation::LayoutUnit::from_f32(width * zoom),
            inline_offset,
        );
        let device_block = foundation::SnapSizeToPixel(
            foundation::LayoutUnit::from_f32(height * zoom),
            block_offset,
        );
        (
            ResizeObserverGeometry {
                content_rect: [0.0, 0.0, f64::from(width), f64::from(height)],
                content_box: content,
                border_box: content,
                device_pixel_content_box: ResizeObserverSize {
                    inline_size: f64::from(device_inline),
                    block_size: f64::from(device_block),
                },
                observation_sizes: [
                    observation,
                    observation,
                    raw_size(
                        foundation::LayoutUnit::from_signed(device_inline),
                        foundation::LayoutUnit::from_signed(device_block),
                    ),
                ],
            },
            diagnostics.0,
            diagnostics.1,
        )
    }
}

impl LayoutNode<'_> {
    pub fn ResizeObserverGeometry(
        &self,
        is_svg_graphics: bool,
    ) -> (ResizeObserverGeometry, bool, bool) {
        ResizeObserverLayoutGeometry {
            object: self.object,
            engine: self.output.engine,
        }
        .ResizeObserverGeometry(is_svg_graphics)
    }
    pub fn ResizeObserverIsNonAtomicInline(&self) -> bool {
        ResizeObserverLayoutGeometry {
            object: self.object,
            engine: self.output.engine,
        }
        .ResizeObserverIsNonAtomicInline()
    }
}
impl LayoutEngine {
    // LayoutObject storage remains resident when Paint temporarily owns the
    // LayoutResult. Observer callbacks do not need a second result/tree owner.
    pub fn ResizeObserverGeometryForNode(
        &self,
        node: u64,
        is_svg_graphics: bool,
    ) -> Option<(ResizeObserverGeometry, bool, bool)> {
        let object = self.GetLayoutTree()?.GetLayoutObjectForNode(node)?;
        Some(
            ResizeObserverLayoutGeometry {
                object,
                engine: self,
            }
            .ResizeObserverGeometry(is_svg_graphics),
        )
    }
    pub fn ResizeObserverNodeIsNonAtomicInline(&self, node: u64) -> bool {
        self.GetLayoutTree()
            .and_then(|tree| tree.GetLayoutObjectForNode(node))
            .is_some_and(|object| {
                ResizeObserverLayoutGeometry {
                    object,
                    engine: self,
                }
                .ResizeObserverIsNonAtomicInline()
            })
    }
}
