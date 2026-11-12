//! Chromium fragment-local geometry projection.
//!
//! SVG text transforms, stitched decoration geometry, box paint geometry,
//! and content size are fragment output concerns and keep package-qualified
//! sampling symbols here.

#![allow(non_snake_case)]

use super::*;
use crate::internal::outline_info::LayoutOutlineInfo;
use crate::internal::outline_rect_collector::VectorOutlineRectCollector;
use crate::internal::scrollbar_orientation::ScrollbarOrientation;

fn FragmentLocalAffineTransform(
    transform: &AffineTransform,
    fragment_offset: PhysicalOffset,
) -> TransformMatrix {
    let x = Number(fragment_offset.left);
    let y = Number(fragment_offset.top);
    let mut output = TransformMatrix::default();
    output.values[0] = transform.A();
    output.values[1] = transform.B();
    output.values[4] = transform.C();
    output.values[5] = transform.D();
    // T(-fragment_offset) * transform * T(fragment_offset).
    output.values[12] = transform.A() * x + transform.C() * y + transform.E() - x;
    output.values[13] = transform.B() * x + transform.D() * y + transform.F() - y;
    output
}

#[inline(never)]
pub(super) fn ExportSvgTextPaintData(
    item: &FragmentItem,
    fragment_rect: PhysicalRect,
    paint: &mut PaintProperties,
) {
    let svg = item.GetSvgFragmentData();
    if svg.is_null() {
        return;
    }
    let svg = unsafe { &*svg };
    let scaling_factor = item.SvgScalingFactor();
    assert!(scaling_factor.is_finite() && scaling_factor > 0.0);
    assert!(svg.length_adjust_scale.is_finite() && svg.length_adjust_scale > 0.0);
    // TextFragmentPainter uses the parent SVG visual rect, not this inline
    // fragment's pre-textPath box (core/paint/text_fragment_painter.cc:422-431).
    let parent = unsafe { &*item.GetLayoutObject() }.Parent();
    assert!(!parent.is_null());
    let algorithms = LayoutPassScope::Algorithms();
    let visual_rect =
        unsafe { algorithms.as_ref() }
            .and_then(|algorithms| algorithms.svg_support.text_visual_rect)
            .expect("SVG text visual bounds query is not installed")(unsafe { &*parent });
    let mut output = SvgTextPaintData {
        visual_rect: FragmentPaintRect {
            offset: Offset {
                x: f64::from(visual_rect.x()) - Number(fragment_rect.offset.left),
                y: f64::from(visual_rect.y()) - Number(fragment_rect.offset.top),
            },
            size: Size {
                width: f64::from(visual_rect.width()),
                height: f64::from(visual_rect.height()),
            },
        },
        glyph_origin: Offset {
            x: svg.rect.x() as f64 - Number(fragment_rect.offset.left),
            y: svg.rect.y() as f64 - Number(fragment_rect.offset.top),
        },
        untransformed_size: Size {
            width: (svg.rect.width() / svg.length_adjust_scale) as f64,
            height: svg.rect.height() as f64,
        },
        scaling_factor: scaling_factor as f64,
        ..SvgTextPaintData::default()
    };
    let transform = item.BuildSvgTransformForPaint();
    output.has_transform = scaling_factor != 1.0 || !transform.IsIdentity();
    output.local_transform = FragmentLocalAffineTransform(&transform, fragment_rect.offset);
    output.hidden = item.IsHiddenForPaint();
    paint.svg_text = Some(Arc::new(output));
}

fn ExportStitchedDecorationData(
    box_fragment: &PhysicalBoxFragment,
) -> Option<StitchedDecorationData> {
    let object = box_fragment.GetLayoutObject();
    if box_fragment.IsInlineBox()
        || box_fragment.IsOnlyForNode()
        || object.is_null()
        || !unsafe { &*object }.IsBox()
        || unsafe { &*object }.IsFieldset()
        || box_fragment.Style().BoxDecorationBreak() != EBoxDecorationBreak::kSlice
    {
        return None;
    }

    if box_fragment.IsTable() {
        let writing_direction = box_fragment.Style().GetWritingDirection();
        let mut stitched_grid = layoutng_geometry::geometry::logical_rect::LogicalRect::default();
        let mut current_grid = layoutng_geometry::geometry::logical_rect::LogicalRect::default();
        let mut stitched_block_size = LayoutUnit::default();
        let mut has_grid = false;
        let mut found_current = false;
        let layout_box = To::<LayoutBox>(object);
        for fragment in unsafe { &*layout_box }.PhysicalFragments() {
            let mut grid = fragment.TableGridRect();
            grid.offset.block_offset += stitched_block_size;
            if !has_grid {
                stitched_grid = grid;
                has_grid = true;
            } else {
                stitched_grid.Unite(&grid);
            }
            if std::ptr::eq(fragment, box_fragment) {
                current_grid = grid;
                found_current = true;
            }
            stitched_block_size += LogicalFragment::new(writing_direction, fragment).BlockSize();
        }
        if !has_grid || !found_current {
            return None;
        }
        current_grid.offset -= stitched_grid.offset;
        let stitched_converter =
            WritingModeConverter::from_logical_size(writing_direction, stitched_grid.size);
        let current_in_stitched = stitched_converter.ToPhysicalRect(current_grid);
        let fragment_converter = WritingModeConverter::new(writing_direction, box_fragment.Size());
        let local_grid = fragment_converter.ToPhysicalRect(box_fragment.TableGridRect());
        let stitched_size = stitched_converter.ToPhysicalSize(stitched_grid.size);
        return Some(StitchedDecorationData {
            fragment_origin: Offset {
                x: Number(local_grid.offset.left),
                y: Number(local_grid.offset.top),
            },
            fragment_offset: Offset {
                x: Number(current_in_stitched.offset.left),
                y: Number(current_in_stitched.offset.top),
            },
            size: Size {
                width: Number(stitched_size.width),
                height: Number(stitched_size.height),
            },
        });
    }

    let mut stitched_size = PhysicalSize::default();
    let stitched_offset = OffsetInStitchedFragments(box_fragment, &mut stitched_size);
    Some(StitchedDecorationData {
        fragment_offset: Offset {
            x: Number(stitched_offset.left),
            y: Number(stitched_offset.top),
        },
        size: Size {
            width: Number(stitched_size.width),
            height: Number(stitched_size.height),
        },
        ..StitchedDecorationData::default()
    })
}

/// Export FragmentPainter::PaintOutline's source geometry while the canonical
/// PhysicalBoxFragment and the exact style selected for painting are both
/// available. Paint cannot reconstruct descendant/anchor outline rects from
/// the flattened fragment tree.
#[inline(never)]
pub(super) fn ExportSelfOutlineGeometry(
    box_fragment: &PhysicalBoxFragment,
    style_to_use: &StyleComputedStyle,
    paint: &mut PaintProperties,
) {
    if !style_to_use.HasOutline() {
        paint.self_outline = None;
        return;
    }

    let mut collector = VectorOutlineRectCollector::default();
    let mut info = LayoutOutlineInfo::default();
    box_fragment.AddSelfOutlineRects(
        PhysicalOffset::default(),
        style_to_use.OutlineRectsShouldIncludeBlockInkOverflow(),
        &mut collector,
        Some(&mut info),
    );
    paint.self_outline = Some(SelfOutlinePaintData {
        rects: collector
            .TakeRects()
            .into_iter()
            .map(|rect| FragmentPaintRect {
                offset: Offset {
                    x: Number(rect.offset.left),
                    y: Number(rect.offset.top),
                },
                size: Size {
                    width: Number(rect.size.width),
                    height: Number(rect.size.height),
                },
            })
            .collect(),
        width: f64::from(info.width),
        offset: f64::from(info.offset),
    });
}

#[inline(never)]
pub(super) fn ExportBoxPaintGeometry(
    box_fragment: &PhysicalBoxFragment,
    paint: &mut PaintProperties,
) {
    let layout_box = DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject());
    if !layout_box.is_null() {
        let layout_box = unsafe { &*layout_box };
        paint.scroll_size = Size {
            width: Number(layout_box.ScrollWidth()),
            height: Number(layout_box.ScrollHeight()),
        };
        // CSSOM View client geometry is not FragmentPaintData's paint content size.
        // Element::clientWidth/Height read the layout viewport for the root,
        // otherwise the source padding box (which excludes classic bars).
        let client_size = if layout_box.IsLayoutView() {
            let view = unsafe { &*To::<LayoutView>(box_fragment.GetLayoutObject()) };
            let size = view.GetLayoutSize(
                crate::internal::scroll_types::IncludeScrollbarsInRect::kExcludeScrollbars,
            );
            Size {
                width: size.width() as f64,
                height: size.height() as f64,
            }
        } else {
            let size = if layout_box.IsTable() {
                layout_box.StitchedSize()
            } else {
                layout_box.PhysicalPaddingBoxRect().size
            };
            Size {
                width: Number(size.width),
                height: Number(size.height),
            }
        };
        paint.client_size = Some(client_size);
        paint.scrolls_overflow = layout_box.ScrollsOverflow();
        // Node::GetLayoutBox uses the source class hierarchy. Extracted SVG
        // shapes/containers share LayoutBox storage but are not CSS boxes;
        // SVGRoot and SVGBlock subclasses (text/foreignObject) really are.
        let object = unsafe { &*box_fragment.GetLayoutObject() };
        if object.IsSVG()
            && !object.IsSVGRoot()
            && !object.IsSVGText()
            && !object.IsSVGForeignObject()
        {
            paint.client_size = Some(Size::default());
            paint.scroll_size = Size::default();
        }

        // The element input is the source tree's current, already-clamped
        // offset and is also used by the retained scroll-property refresh.
        // Generated scroll owners without element input read their local area.
        let area = layout_box.GetScrollableArea();
        if !paint.has_source {
            paint.scroll_offset = if area.is_null() {
                Offset::default()
            } else {
                let current = unsafe { &*area }.GetScrollOffset();
                Offset {
                    x: current.x() as f64,
                    y: current.y() as f64,
                }
            };
        }
        // Source: core/paint/paint_property_tree_builder.cc:499-530.
        // Export the source decision itself by materializing ScrollContainer
        // data whenever Blink needs the Scroll/ScrollTranslation pair. Paint
        // must not reconstruct it from CSS overflow or current geometry.
        let needs_local_scroll = if object.IsOverscrollContainer() {
            true
        } else if !layout_box.IsScrollContainer() {
            false
        } else if object.IsGlobalRootScroller() {
            // Rechrom always has an accelerated compositor for a live page.
            true
        } else {
            // Source: core/paint/paint_property_tree_builder.cc:516-530.
            // A LayoutBox that needs Scroll/ScrollTranslation must already
            // own its PaintLayerScrollableArea.  Chromium CHECKs this owner;
            // silently dropping the scroll node here creates a second,
            // geometry-derived lifecycle.
            assert!(!area.is_null());
            let area = unsafe { &*area };
            let position = area.ScrollPosition();
            let offset = area.GetScrollOffset();
            layout_box.ScrollsOverflow()
                || position.x() != 0.0
                || position.y() != 0.0
                || offset.x() != 0.0
                || offset.y() != 0.0
                || !area.CanPropagateScroll()
        };
        if layout_box.IsLayoutView() || needs_local_scroll {
            // Source: core/paint/paint_property_tree_builder.cc:3574-3614.
            // UpdateScrollNode consumes the retained ScrollableArea directly;
            // it never manufactures scroll state from the exported clip or
            // fragment geometry.
            assert!(!area.is_null());
            let area = unsafe { &*area };
            let clip = layout_box.OverflowClipRectForScrollNode();
            let minimum = area.MinimumScrollOffsetInt();
            let maximum = area.MaximumScrollOffsetInt();
            let minimum_scroll_offset = Offset {
                x: minimum.x() as f64,
                y: minimum.y() as f64,
            };
            let maximum_scroll_offset = Offset {
                x: maximum.x() as f64,
                y: maximum.y() as f64,
            };
            let horizontal = area.UserInputScrollable(ScrollbarOrientation::kHorizontalScrollbar);
            let vertical = area.UserInputScrollable(ScrollbarOrientation::kVerticalScrollbar);
            let scrolling_background_client = area.GetScrollingBackgroundDisplayItemClient();
            // Source: core/paint/paint_layer_scrollable_area.cc:3407-3418 and
            // core/paint/box_fragment_painter.cc:1408-1411. Preserve the
            // source pixel-snapped visual rect separately from the enlarged
            // scrolling background paint rect.
            let paint_offset = layout_box.FirstFragment().PaintOffset();
            let rect = area.ScrollingBackgroundVisualRect(&paint_offset);
            let scrolling_background_visual_rect = FragmentPaintRect {
                offset: Offset {
                    x: rect.x() as f64 - Number(paint_offset.left),
                    y: rect.y() as f64 - Number(paint_offset.top),
                },
                size: Size {
                    width: rect.width() as f64,
                    height: rect.height() as f64,
                },
            };
            let origin = area.ScrollOrigin();
            let position = area.ScrollPosition();
            let scroll_origin = Offset {
                x: origin.x() as f64,
                y: origin.y() as f64,
            };
            let scroll_position = Offset {
                x: position.x() as f64,
                y: position.y() as f64,
            };
            let scroll_snap_type = layout_box.StyleRef().GetScrollSnapType();
            let scroll_snap_axis = match scroll_snap_type.axis {
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kBoth => {
                    ScrollSnapAxisPaintData::Both
                }
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kX => {
                    ScrollSnapAxisPaintData::X
                }
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kY => {
                    ScrollSnapAxisPaintData::Y
                }
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kBlock => {
                    ScrollSnapAxisPaintData::Block
                }
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kInline => {
                    ScrollSnapAxisPaintData::Inline
                }
                layoutng_style::style::scroll_snap_data::cc::SnapAxis::kPair => {
                    ScrollSnapAxisPaintData::Pair
                }
            };
            let scroll_snap_strictness = match scroll_snap_type.strictness {
                layoutng_style::style::scroll_snap_data::cc::SnapStrictness::kProximity => {
                    ScrollSnapStrictnessPaintData::Proximity
                }
                layoutng_style::style::scroll_snap_data::cc::SnapStrictness::kMandatory => {
                    ScrollSnapStrictnessPaintData::Mandatory
                }
            };
            paint.scroll_container = Some(ScrollContainerPaintData {
                prefers_non_composited_scrolling: Some(area.PrefersNonCompositedScrolling()),
                scroll_snap_type: Some(ScrollSnapTypePaintData {
                    is_none: scroll_snap_type.is_none,
                    axis: scroll_snap_axis,
                    strictness: scroll_snap_strictness,
                }),
                background_needs_repaint_on_scroll: Some(area.BackgroundNeedsRepaintOnScroll()),
                // Source: core/paint/paint_layer_scrollable_area.cc:2515-2538.
                scrollable_axes: if !layout_box.IsScrollContainer() {
                    [false, false]
                } else if layout_box.IsLayoutView() {
                    [true, true]
                } else {
                    [
                        layout_box.StyleRef().IsOverflowValueScrollableX(),
                        layout_box.StyleRef().IsOverflowValueScrollableY(),
                    ]
                },
                compositor_element_id: area.GetScrollElementId(),
                scroll_origin,
                scroll_position,
                container_rect: FragmentPaintRect {
                    offset: Offset {
                        x: Number(clip.offset.left),
                        y: Number(clip.offset.top),
                    },
                    size: Size {
                        width: Number(clip.size.width),
                        height: Number(clip.size.height),
                    },
                },
                scrolling_background_visual_rect,
                minimum_scroll_offset,
                maximum_scroll_offset,
                user_scrollable_horizontal: horizontal,
                user_scrollable_vertical: vertical,
                scrolling_background_client_id: scrolling_background_client.Id() as u64,
                scrolling_background_client_invalidation_reason: Some(
                    scrolling_background_client.GetPaintInvalidationReason(),
                ),
                scrolling_background_client_is_cacheable: scrolling_background_client.IsCacheable(),
                scrolling_background_client_is_just_created: std::cell::Cell::new(
                    scrolling_background_client.IsJustCreated(),
                ),
            });
        }
        let replaced = DynamicTo::<LayoutReplaced>(box_fragment.GetLayoutObject());
        if !replaced.is_null() {
            paint.replaced_content = Some(ExportReplacedContentPaintData(replaced));
        }
    }
    let borders = box_fragment.Borders();
    paint.border = Edges {
        top: Number(borders.top),
        right: Number(borders.right),
        bottom: Number(borders.bottom),
        left: Number(borders.left),
    };
    let sides = box_fragment.SidesToInclude();
    paint.border_sides = FragmentBoxSides {
        top: sides.top,
        right: sides.right,
        bottom: sides.bottom,
        left: sides.left,
    };
    let padding = box_fragment.Padding();
    paint.padding = Edges {
        top: Number(padding.top),
        right: Number(padding.right),
        bottom: Number(padding.bottom),
        left: Number(padding.left),
    };
    paint.box_decoration_break =
        if box_fragment.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone {
            BoxDecorationBreak::kClone
        } else {
            BoxDecorationBreak::kSlice
        };
    paint.stitched_decoration = ExportStitchedDecorationData(box_fragment);
}

// Chromium's PhysicalBoxFragment::ContentRect() DCHECKs that border and
// padding fit inside the fragment. That precondition intentionally does not
// hold for every table-part fragment: CSS padding may remain represented in
// the fragment metadata even though the table algorithm resolves a zero row
// block-size. The public FragmentPaintData boundary still needs a total content-box
// size for every fragment, so use the same non-negative subtraction semantics
// as Blink's ShrinkLogicalSize rather than calling ContentRect() outside its
// valid domain.
pub(super) fn ExportContentSize(box_fragment: &PhysicalBoxFragment) -> Size {
    let insets = box_fragment.Borders() + box_fragment.Padding();
    let size = box_fragment.Size();
    Size {
        width: Number((size.width - insets.HorizontalSum()).ClampNegativeToZero()),
        height: Number((size.height - insets.VerticalSum()).ClampNegativeToZero()),
    }
}

/// LayoutVideo display mode can change without a new physical fragment.
/// Re-read the same native replaced geometry at the paint-only commit too.
pub(super) fn ExportReplacedContentPaintData(
    replaced: *const LayoutReplaced,
) -> ReplacedContentPaintData {
    let rect = unsafe { &*replaced }.ReplacedContentRect();
    ReplacedContentPaintData {
        offset: Offset {
            x: Number(rect.offset.left),
            y: Number(rect.offset.top),
        },
        size: Size {
            width: Number(rect.size.width),
            height: Number(rect.size.height),
        },
        respects_css_overflow: unsafe { &*replaced }.RespectsCSSOverflow(),
        clips_to_content_box: unsafe { &*replaced }.ClipsToContentBox(),
        is_svg_root: unsafe { &*replaced }.IsSVGRoot(),
        is_image: unsafe { &*replaced }.IsImage(),
        is_canvas: unsafe { &*replaced }.IsCanvas(),
        draws_background_onto_content_layer: unsafe { &*replaced }
            .DrawsBackgroundOntoContentLayer(),
        is_media: unsafe { &*replaced }.IsMedia(),
        is_video: unsafe { &*replaced }.IsVideo(),
        video_uses_poster: unsafe { &*replaced }.IsVideo()
            && unsafe { &*replaced.cast::<crate::internal::layout_replaced::LayoutVideo>() }
                .GetDisplayMode()
                == crate::internal::layout_replaced::VideoDisplayMode::Poster,
        video_would_taint_origin: unsafe { &*replaced }.IsVideo()
            && unsafe { &*replaced.cast::<crate::internal::layout_replaced::LayoutVideo>() }
                .ElementState()
                .would_taint_origin,
        is_layout_embedded_content: unsafe { &*replaced }.IsLayoutEmbeddedContent(),
        is_embedded_object: unsafe { &*replaced }.IsEmbeddedObject(),
    }
}
