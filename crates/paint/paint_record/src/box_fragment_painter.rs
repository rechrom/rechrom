#![allow(non_snake_case)]

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::box_border_painter::{BoxBorderPainter, DrawBoxSide, RoundedBorderSideClips};
use crate::box_decoration_data::BoxDecorationData;
use crate::contoured_border_geometry::{
    ExpandCornerRadiiAxes, IncludedBorderEdges, ResolveCornerRadii,
};
use crate::display_item_cache_skipper::DisplayItemCacheSkipper;
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_recorder::DrawingRecorder;
use crate::fieldset_painter::FieldsetPainter;
use crate::frame_set_painter::FrameSetPainter;
use crate::geometry_mapper::{MapRectWithTransforms, MultiplyTransforms, TranslationTransform};
use crate::html_canvas_painter::HTMLCanvasPainter;
use crate::inline_box_fragment_painter::{InlineBoxFragmentPainter, LineBoxFragmentPainter};
use crate::line_relative_rect::LineRelativeRect;
use crate::mathml_painter::MathMLPainter;
use crate::nine_piece_image_painter::NinePieceImagePainter;
use crate::paint_auto_dark_mode::{
    ApplyMaskLayerImageAutoDarkMode, ElementRole, GetImageAutoDarkMode, InvertColorIfNeeded,
};
use crate::paint_context::PaintContext;
use crate::paint_engine::{
    DisplayItem, DisplayItemType, PaintHitTestOpaqueness, PaintPhase,
    ResolveDocumentImageContainerKey,
};
use crate::paint_info::PaintInfo;
use crate::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyNodeLifecycle, PropertyTreeState,
};
use crate::paint_shader_resolver::{PrivacyPreservingPaintServer, ResolvePaintShader};
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::scrollable_area_painter::ScrollableAreaPainter;
use crate::selection_bounds_recorder::SelectionBoundsRecorder;
use crate::svg_shape_painter::SVGShapePainter;
use crate::table_painters::TablePainter;
use crate::text_decoration_painter::{Phase as TextDecorationPhase, TextDecorationPainter};
use crate::theme_painter::ThemePainter;
use crate::video_painter::VideoPainter;
use crate::PaintRect;
use foundation::{
    blink_geometry::transforms::affine_transform::AffineTransform, LayoutUnit, PhysicalOffset,
    PhysicalRect, PhysicalSize, WritingMode as StyleWritingMode,
};
use layoutng_assembly::fragment_tree::{
    FragmentKind, FragmentPaintData, FragmentPaintRect, PaintGlyph,
};
use layoutng_assembly::internal::background_bleed_avoidance::{
    BackgroundBleedAvoidance, BleedAvoidanceIsClipping,
};
use layoutng_assembly::internal::caret_rect::CaretShape;
use layoutng_assembly::internal::layout_input::PaintStyle;
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Display, Edges, FloatSide, FocusedImageMapArea, ImageMapAreaShape,
    ListStyleType, NodeKind, Offset, PaintImage, PaintPathCommand, PaintPathVerb, Size,
    TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundAttachment, BackgroundBox, BackgroundImageLayer, PaintBlendMode, PaintCornerRadii,
    SvgPaintComponent,
};
use layoutng_assembly::internal::selection_state::SelectionState;
use paint_model::geometry_mapper::SourceToDestinationRect;

// Chromium entity: core/paint/box_fragment_painter.h:35-344.
pub struct BoxFragmentPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

/// The exported subset of ObjectPainter::GetHitTestOpaqueness() used by both
/// scroll-container metadata and source scrollbar recording.
pub fn GetHitTestOpaqueness(fragment: &FragmentPaintData) -> PaintHitTestOpaqueness {
    let style = &fragment.paint.style;
    if !fragment.paint.visible_to_hit_testing || !style.visible() || style.pointer_events_none() {
        PaintHitTestOpaqueness::Transparent
    } else if style.border_radius > 0.0
        || style.border_radii.is_some()
        || fragment.paint.is_svg_child
    {
        PaintHitTestOpaqueness::Mixed
    } else {
        PaintHitTestOpaqueness::Opaque
    }
}

// Chromium: box_fragment_painter.cc:739-750,817-822. Non-overlay controls
// belong before the scrolling background during a self-background phase;
// every other control path stays in the post-children position.
fn PaintsOverflowControlsBeforeContents(phase: PaintPhase, should_paint_as_overlay: bool) -> bool {
    !should_paint_as_overlay && paint_model::paint_phase::ShouldPaintSelfBlockBackground(phase)
}

// Chromium: box_fragment_painter.cc:713-714,795-798. kOutline traverses
// descendants first; kSelfOutlineOnly is the direct layer-root/self pass.
fn PaintsSelfOutlineBeforeChildren(phase: PaintPhase) -> bool {
    phase == PaintPhase::kSelfOutlineOnly
}

fn PaintsSelfOutlineAfterChildren(phase: PaintPhase) -> bool {
    phase == PaintPhase::kOutline
}

// Chromium: box_fragment_painter.cc:100-130.  `hidden` is the exported
// PhysicalFragment/FragmentItem IsHiddenForPaint result and is deliberately
// distinct from CSS visibility. Hidden table parts still supply their
// background to visible cells.
fn IsVisibleToPaint(fragment: &FragmentPaintData) -> bool {
    if fragment.paint.hidden {
        return false;
    }
    fragment.paint.style.visible()
        || matches!(
            fragment.paint.display,
            Display::kTableSection
                | Display::kTableHeaderGroup
                | Display::kTableFooterGroup
                | Display::kTableRow
                | Display::kTableColumn
                | Display::kTableColumnGroup
        )
}

// Source: text_painter.cc:669-716. Printing without backgrounds in economy
// mode darkens colors that would disappear against the forced white page.
fn TextColorForWhiteBackground(color: Color) -> Color {
    let byte = |component: f32| (component.clamp(0.0, 1.0) * 255.0).round() as i32;
    let red_from_white = byte(color.red) - 255;
    let green_from_white = byte(color.green) - 255;
    let blue_from_white = byte(color.blue) - 255;
    let difference = red_from_white * red_from_white
        + green_from_white * green_from_white
        + blue_from_white * blue_from_white;
    if difference > 65_025 {
        return color;
    }
    let value = color.red.max(color.green).max(color.blue);
    let multiplier = if value == 0.0 {
        0.0
    } else {
        ((value - 0.33) / value).max(0.0)
    };
    let scale = f32::from_bits(256.0_f32.to_bits() - 1);
    let quantize = |component: f32| (component * multiplier * scale) as i32 as f32 / 255.0;
    Color {
        red: quantize(color.red),
        green: quantize(color.green),
        blue: quantize(color.blue),
        alpha: color.alpha,
    }
}

fn ShouldForceWhiteBackgroundForPrintEconomy(fragment: &FragmentPaintData) -> bool {
    ShouldForceWhiteBackgroundForPrintEconomyWithStyle(fragment, &fragment.paint.style)
}

fn BaseBackgroundColor(fragment: &FragmentPaintData) -> Option<Color> {
    let resources = fragment.paint.resources.as_deref()?;
    if !resources.paints_opaque_base_background {
        return None;
    }
    let color = resources.base_background_color;
    if color.alpha <= 0.0 {
        return None;
    }
    // ViewPainter leaves the printing surface transparent only for the
    // ordinary white paper backdrop. Embedder-provided non-white colors remain
    // observable in print output.
    if fragment.paint.IsPrinting()
        && color.red == 1.0
        && color.green == 1.0
        && color.blue == 1.0
        && color.alpha == 1.0
    {
        return None;
    }
    Some(color)
}

fn CompositeColor(background: Color, foreground: Color) -> Color {
    let foreground_alpha = foreground.alpha.clamp(0.0, 1.0);
    let background_alpha = background.alpha.clamp(0.0, 1.0);
    let alpha = foreground_alpha + background_alpha * (1.0 - foreground_alpha);
    if alpha <= 0.0 {
        return Color::default();
    }
    let background_weight = background_alpha * (1.0 - foreground_alpha);
    Color {
        red: (foreground.red * foreground_alpha + background.red * background_weight) / alpha,
        green: (foreground.green * foreground_alpha + background.green * background_weight) / alpha,
        blue: (foreground.blue * foreground_alpha + background.blue * background_weight) / alpha,
        alpha,
    }
}

fn CanvasBackgroundMarksFirstPaint(fragment: &FragmentPaintData, root_color: Color) -> bool {
    let Some(resources) = fragment.paint.resources.as_deref() else {
        return false;
    };
    let painted_base = BaseBackgroundColor(fragment).unwrap_or_default();
    CompositeColor(painted_base, root_color) != resources.base_background_color
}

fn ShouldForceWhiteBackgroundForPrintEconomyWithStyle(
    fragment: &FragmentPaintData,
    style: &PaintStyle,
) -> bool {
    fragment.paint.IsPrinting()
        && style.print_color_adjust_economy()
        && !fragment.paint.ShouldPrintBackgrounds()
}

/// BoxPainterBase::FillLayerInfo rewrites every authored image to no image in
/// print-economy mode. Only the bottom FillLayer may paint the replacement
/// white color, so a non-bottom image over an otherwise transparent bottom
/// layer must leave the page transparent.
pub(crate) fn BackgroundForPainting(
    fragment: &FragmentPaintData,
    style: &PaintStyle,
    background_images: &[BackgroundImageLayer],
) -> (Color, bool) {
    let color = style.background_color();
    let (color, paints_background_images) =
        if ShouldForceWhiteBackgroundForPrintEconomyWithStyle(fragment, style) {
            // BoxBackgroundPaintContext::ShouldSkipBackgroundIfWhite: page
            // border boxes deliberately stay transparent so the paper/PDF
            // viewer white can still be customized by the embedding surface.
            if fragment.paint.is_page_border_box {
                return (Color::default(), false);
            }
            let paints_white = color.alpha > 0.0
                || background_images
                    .iter()
                    .any(|layer| layer.paints_background_color);
            (
                if paints_white {
                    Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 1.0,
                    }
                } else {
                    Color::default()
                },
                false,
            )
        } else if fragment.paint.InForcedColorsMode()
            && fragment.paint.box_fragment_is_inline_box
            && style.forced_color_adjust_auto()
        {
            (color, false)
        } else {
            (color, true)
        };
    (
        InvertColorIfNeeded(style, color, ElementRole::kBackground),
        paints_background_images,
    )
}

fn BackgroundStyleForPainting(fragment: &FragmentPaintData) -> Option<&PaintStyle> {
    if fragment.paint.is_page_border_box {
        fragment.paint.page_document_background_style.as_ref()
    } else {
        Some(&fragment.paint.style)
    }
}

// Chromium: box_fragment_painter.cc:2009-2014.
fn PaintsTextInPhase(phase: PaintPhase) -> bool {
    matches!(
        phase,
        PaintPhase::kForeground
            | PaintPhase::kSelectionDragImage
            | PaintPhase::kTextClip
            | PaintPhase::kMask
    )
}

fn RefineSelectionStateForTextFragment(
    state: SelectionState,
    fragment_start: u32,
    fragment_end: u32,
    selection_start: Option<u32>,
    selection_end: Option<u32>,
) -> SelectionState {
    let contains = |offset: u32| fragment_start <= offset && offset <= fragment_end;
    match state {
        SelectionState::kStart => selection_start
            .filter(|offset| contains(*offset))
            .map_or(SelectionState::kNone, |_| SelectionState::kStart),
        SelectionState::kEnd => selection_end
            .filter(|offset| contains(*offset))
            .map_or(SelectionState::kNone, |_| SelectionState::kEnd),
        SelectionState::kStartAndEnd => {
            let has_start = selection_start.is_some_and(contains);
            let has_end = selection_end.is_some_and(contains);
            match (has_start, has_end) {
                (true, true) => SelectionState::kStartAndEnd,
                (true, false) => SelectionState::kStart,
                (false, true) => SelectionState::kEnd,
                (false, false) => SelectionState::kInside,
            }
        }
        SelectionState::kInside => SelectionState::kInside,
        _ => SelectionState::kNone,
    }
}

// Chromium: box_fragment_painter.cc:260-321. FragmentItems are projected as
// resident line/inline/text PaintTree nodes. Keep the source pre-order walk
// and split only after two consecutive forced line breaks; atomic inline and
// self-painting-layer contents are not part of the owning formatting context.
fn BuildBackplates(root: &PaintTreeNode<'_>) -> Vec<PaintRect> {
    #[derive(Default)]
    struct Backplates {
        paragraphs: Vec<PaintRect>,
        current: Option<PaintRect>,
        consecutive_line_breaks: u32,
    }

    impl Backplates {
        fn AddTextRect(&mut self, rect: PaintRect) {
            if self.consecutive_line_breaks >= 2 {
                if let Some(current) = self.current.take() {
                    self.paragraphs.push(current);
                }
            }
            self.consecutive_line_breaks = 0;
            if let Some(current) = &mut self.current {
                current.union(rect);
            } else {
                self.current = Some(rect);
            }
        }

        fn AddLineBreak(&mut self) {
            self.consecutive_line_breaks += 1;
        }

        fn Finish(mut self) -> Vec<PaintRect> {
            if let Some(current) = self.current.take() {
                self.paragraphs.push(current);
            }
            self.paragraphs
        }
    }

    fn Visit(node: &PaintTreeNode<'_>, backplates: &mut Backplates) {
        let Some(fragment) = node.fragment.as_deref() else {
            return;
        };
        if node.is_paint_layer {
            return;
        }
        if !fragment.paint.hidden
            && matches!(
                fragment.kind,
                FragmentKind::kText | FragmentKind::kGeneratedText
            )
        {
            if fragment.paint.is_line_break {
                backplates.AddLineBreak();
            } else {
                backplates.AddTextRect(PaintRect {
                    x: node.paint_offset.x,
                    y: node.paint_offset.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
                });
            }
            return;
        }
        if fragment.paint.painted_atomically {
            return;
        }
        for child in &node.children {
            let Some(child_fragment) = child.fragment.as_deref() else {
                continue;
            };
            if matches!(
                child_fragment.kind,
                FragmentKind::kLine | FragmentKind::kText | FragmentKind::kGeneratedText
            ) || child_fragment.paint.box_fragment_is_inline_box
            {
                Visit(child, backplates);
            }
        }
    }

    let mut backplates = Backplates::default();
    for child in &root.children {
        Visit(child, &mut backplates);
    }
    backplates.Finish()
}

// cpp: core/paint/box_painter.cc:45-57. Printing and drag-image paint omit
// compositor metadata. Canvas drawElement is the source exception because its
// captured subtree still needs scroll hit testing even without compositing.
fn ShouldRecordScrollHitTestDataForPaint(
    omit_compositing_info: bool,
    canvas_draw_element_enabled: bool,
    is_in_canvas_subtree: bool,
) -> bool {
    !omit_compositing_info || (canvas_draw_element_enabled && is_in_canvas_subtree)
}

// Chromium checks the unsnapped content-box dimensions before pixel snapping,
// then insets the snapped rectangle by half the one-pixel stroke width.
// cpp: core/paint/image_painter.cc:130-135,184-192
fn MissingImagePlaceholderRect(
    is_image: bool,
    content_box: PaintRect,
    mut snapped_content_box: PaintRect,
) -> Option<PaintRect> {
    if !is_image || content_box.width <= 2.0 || content_box.height <= 2.0 {
        return None;
    }
    snapped_content_box.x += 0.5;
    snapped_content_box.y += 0.5;
    snapped_content_box.width -= 1.0;
    snapped_content_box.height -= 1.0;
    (snapped_content_box.width > 0.0 && snapped_content_box.height > 0.0)
        .then_some(snapped_content_box)
}

// Chromium limits sprite-sheet subrect painting to SVG images. Rechrom's
// document-backed PaintImage is the typed SVG image boundary; bitmap images
// deliberately retain the complete visual rect to avoid subrect-decoding
// bleed. FragmentData's cull rect is already in the local paint space.
// cpp: core/paint/image_painter.cc:147-166
fn SvgSpriteVisualRect(
    image: &PaintImage,
    visual_rect: PaintRect,
    local_cull_rect: Option<PaintRect>,
) -> PaintRect {
    if !image.IsDocumentImage() {
        return visual_rect;
    }
    let Some(cull_rect) = local_cull_rect else {
        return visual_rect;
    };
    let enclosing_visual_rect = visual_rect.enclosing();
    let contains_cull_rect = cull_rect.x >= enclosing_visual_rect.x
        && cull_rect.y >= enclosing_visual_rect.y
        && cull_rect.x + cull_rect.width <= enclosing_visual_rect.x + enclosing_visual_rect.width
        && cull_rect.y + cull_rect.height <= enclosing_visual_rect.y + enclosing_visual_rect.height;
    if contains_cull_rect {
        visual_rect.intersection(cull_rect)
    } else {
        visual_rect
    }
}

// LayoutImage::MutableForPainting::UpdatePaintedRect invalidates the image's
// DisplayItemClient when this value changes. The retained fragment payload is
// Rechrom's package-boundary owner for the same LayoutImage paint lifecycle
// value. Returning `true` lets the caller reject the old drawing before
// PaintController can move it into the new artifact.
// cpp: core/layout/layout_image.cc:411-421
// cpp: core/paint/image_painter.cc:167-173
fn UpdateImagePaintedRect(node: &PaintTreeNode<'_>, visual_rect: PaintRect) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let painted_rect = FragmentPaintRect {
        offset: Offset {
            x: visual_rect.x,
            y: visual_rect.y,
        },
        size: Size {
            width: visual_rect.width,
            height: visual_rect.height,
        },
    };
    let changed = fragment.paint.image_last_paint_rect.get() != Some(painted_rect);
    fragment.paint.image_last_paint_rect.set(Some(painted_rect));
    changed
}

fn UseCachedImageDrawingIfPossible(
    context: &RefCell<PaintContext<'_>>,
    node: &PaintTreeNode<'_>,
    phase: PaintPhase,
    painted_rect_changed: bool,
) -> bool {
    !painted_rect_changed
        && DrawingRecorder::UseCachedDrawingIfPossible(
            context,
            node,
            DisplayItemIdType::PaintPhaseToDrawingType(phase),
        )
}

// Source: core/html/canvas/image_element_base.cc:232-248 and
// core/paint/image_painter.cc:255-261.
fn ImageDecodingModeForPainting(
    fragment: &FragmentPaintData,
    image: &PaintImage,
) -> image_resource::ImageDecodingMode {
    let previous = fragment
        .paint
        .image_last_painted_id
        .replace(image.paint_image_id);
    let content_transitioned = previous != image_resource::PaintImageId::INVALID
        && image.paint_image_id != image_resource::PaintImageId::INVALID
        && previous != image.paint_image_id;
    if content_transitioned
        && fragment.paint.image_decoding_mode == image_resource::ImageDecodingMode::kUnspecified
    {
        image_resource::ImageDecodingMode::kSync
    } else {
        fragment.paint.image_decoding_mode
    }
}

// Chromium: box_fragment_painter.cc:851-856. Table rows/sections paint their
// descendants but never contribute an ordinary scroll hit-test display item.
fn ShouldRecordHitTestData(display: Display) -> bool {
    !matches!(
        display,
        Display::kTableRow
            | Display::kTableSection
            | Display::kTableHeaderGroup
            | Display::kTableFooterGroup
    )
}

// Source: box_painter_base.cc:1797-1823. These cases deliberately repaint
// while paint-under-invalidation diagnostics are enabled because their source
// state may change before an ordinary invalidation reaches Paint.
fn ShouldSkipPaintUnderInvalidationChecking(fragment: &FragmentPaintData) -> bool {
    fragment
        .paint
        .pre_paint_invalidation
        .should_delay_full_paint_invalidation
        || fragment.paint.form_control.as_ref().is_some_and(|control| {
            control.appearance == layoutng_style::style::appearance::AppearanceValue::kMediaSlider
                || (control.is_progress && control.indeterminate)
        })
}

fn PaintsCaretBeforeChildren(shape: Option<CaretShape>, unscaled_text_combine_caret: bool) -> bool {
    shape == Some(CaretShape::kBlock) || unscaled_text_combine_caret
}

fn PaintsCaretAfterChildren(shape: Option<CaretShape>, unscaled_text_combine_caret: bool) -> bool {
    shape.is_some() && shape != Some(CaretShape::kBlock) && !unscaled_text_combine_caret
}

impl<'n, 'f, 'c, 'o> BoxFragmentPainter<'n, 'f, 'c, 'o> {
    // Source: box_painter.cc:18-35 and box_fragment_painter.cc:3032-3048.
    pub(crate) fn RecordRegionCaptureAndTrackedElementData(&self) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("element capture metadata requires a fragment");
        if fragment.paint.region_capture_crop_id.is_none()
            && fragment.paint.tracked_element_sub_rects.is_empty()
        {
            return;
        }
        let paint_rect = PixelSnappedBorderBoxRect(self.node, fragment.size);
        self.RecordRegionCaptureAndTrackedElementDataForClient(
            paint_rect,
            fragment.paint.display_item_client_id,
            crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
        );
    }

    fn RecordRegionCaptureAndTrackedElementDataForClient(
        &self,
        paint_rect: PaintRect,
        client_id: u64,
        client: crate::paint_chunker::DisplayItemClientInfo,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("element capture metadata requires a fragment");
        let mut context = self.context.borrow_mut();
        if let Some(crop_id) = fragment.paint.region_capture_crop_id {
            context.RecordRegionCaptureDataWithClient(
                self.node, client_id, client, crop_id, paint_rect,
            );
        }
        if !fragment.paint.tracked_element_sub_rects.is_empty() {
            context.RecordTrackedElementDataWithClient(
                self.node,
                client_id,
                client,
                paint_rect,
                &fragment.paint.tracked_element_sub_rects,
            );
        }
    }

    // Source: box_fragment_painter.cc:1440-1444 and
    // object_painter.cc:137-158. Background hit-test data follows the same
    // ordinary/scrolling display-item client split as the painted background.
    fn RecordHitTestDataForClient(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        paint_rect: PaintRect,
        client_id: u64,
        client: crate::paint_chunker::DisplayItemClientInfo,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("box hit-test metadata requires a fragment");
        if paint_info.ShouldOmitCompositingInfo()
            || !fragment.paint.style.visible()
            || !ShouldRecordHitTestData(fragment.paint.display)
        {
            return;
        }
        self.context.borrow_mut().RecordHitTestDataWithClient(
            self.node,
            client_id,
            client,
            paint_rect,
            GetHitTestOpaqueness(fragment),
        );
    }

    // Chromium constructors: box_fragment_painter.h:40-46,114-119,346-388.
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // Chromium: box_fragment_painter.h:57-60; box_fragment_painter.cc:615-641.
    pub fn ShouldPaintFragment(
        node: &'n PaintTreeNode<'f>,
        paint_info: &PaintInfo<'c, 'o>,
    ) -> bool {
        let paint_state = crate::scoped_paint_state::ScopedPaintState::new(node, paint_info);
        Self::ShouldPaintWithState(node, &paint_state)
    }

    // Chromium: box_fragment_painter.cc:2016-2022,2059-2064,2106-2119.
    // FragmentItems are resident PaintTree children in Rechrom. Their source
    // ink overflow remains fragment-local while `paint_offset` is the source
    // `paint_offset + item.OffsetInContainerFragment()` value.
    pub fn ChildIntersectsCullRect(
        node: &PaintTreeNode<'_>,
        paint_info: &PaintInfo<'_, '_>,
    ) -> bool {
        let Some(fragment) = node.fragment.as_deref() else {
            return true;
        };
        // PaintText performs the source HasSelection exception after resolving
        // the actual selection range. Admit line breaks to that exact check.
        if fragment.paint.is_line_break {
            return true;
        }
        // SVGShapePainter/SVGContainerPainter own SVG geometry culling against
        // their transformed visual bounds. SVG model objects are not
        // FragmentItems, and their anonymous zero-sized NG boxes must not
        // suppress those source culls while the parent walks resident
        // children. Chromium: svg_shape_painter.cc:52-64 and
        // svg_container_painter.cc:36-90.
        if fragment.paint.svg_shape.is_some()
            || matches!(
                fragment.paint.source_kind,
                NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
            )
        {
            return true;
        }
        let Some(rect) = fragment.paint.ink_overflow else {
            return true;
        };
        // Chromium: box_fragment_painter.cc:1923-1936. Soft wraps and <br>
        // may paint outside a line item's inline-direction ink overflow, so a
        // line-box child is culled only in the block direction. Ordinary
        // FragmentItems keep the full-rectangle admission below.
        if fragment.kind == FragmentKind::kLine {
            let cull = paint_info.GetCullRect();
            return match fragment.paint.writing_mode {
                layoutng_assembly::internal::layout_input::WritingMode::kHorizontalTb => {
                    let y = node.paint_offset.y + rect.offset.y;
                    cull.IntersectsVerticalRange(y, y + rect.size.height)
                }
                _ => {
                    let x = node.paint_offset.x + rect.offset.x;
                    cull.IntersectsHorizontalRange(x, x + rect.size.width)
                }
            };
        }
        let local_rect = PaintRect {
            x: rect.offset.x,
            y: rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        };
        paint_info.IntersectsCullRect(local_rect, node.paint_offset)
    }

    // Chromium: box_fragment_painter.cc:1016-1018. MathMLPainter records the
    // stretched operator glyph itself, so the ordinary line-item traversal
    // must not paint the operator's source text a second time. Physical box
    // children remain independently traversable.
    pub fn ShouldPaintChildFragment(parent: &PaintTreeNode<'_>, child: &PaintTreeNode<'_>) -> bool {
        let parent_is_mathml_operator = parent
            .fragment
            .as_deref()
            .and_then(|fragment| fragment.paint.mathml.as_ref())
            .is_some_and(|math| {
                math.kind == layoutng_assembly::fragment_tree::MathMLPaintKind::kOperator
            });
        if !parent_is_mathml_operator {
            return true;
        }
        let child = child
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        !matches!(
            child.kind,
            FragmentKind::kLine | FragmentKind::kText | FragmentKind::kGeneratedText
        ) && !child.paint.box_fragment_is_inline_box
    }

    // Chromium: box_fragment_painter.cc:615-620,643-669,2122-2133.
    pub fn PaintFragment(node: &'n PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        let paint_state = crate::scoped_paint_state::ScopedPaintState::new(node, paint_info);
        if !Self::ShouldPaintWithState(node, &paint_state) {
            return;
        }
        Self::new(node, paint_info.context).Paint(paint_state.GetPaintInfo());
    }

    fn ShouldPaintWithState(
        node: &PaintTreeNode<'_>,
        paint_state: &crate::scoped_paint_state::ScopedPaintState<'_, '_, '_, '_>,
    ) -> bool {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.is_paginated_root {
            return true;
        }
        // SVGShapePainter performs its own transformed visual-bounds cull.
        // The resident NG box for a shape has no CSS box ink overflow.
        if fragment.paint.svg_shape.is_some() {
            return true;
        }
        let Some(rect) = fragment.paint.filtered_ink_overflow else {
            // Standalone synthetic fragments have no source LayoutObject from
            // which to export filtered overflow. Conservatively admit them;
            // source paint inputs always carry the exact rectangle.
            return true;
        };
        let mut local_rect = PaintRect {
            x: rect.offset.x,
            y: rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        };
        // ReplacedPainter::ShouldPaint unions LocalSelectionVisualRect so an
        // inline selection that extends beyond ordinary ink remains admitted.
        // cpp: core/paint/replaced_painter.cc:410-413
        if let Some(selection_rect) = fragment.paint.replaced_local_selection_visual_rect {
            local_rect.union(PaintRect {
                x: selection_rect.offset.x,
                y: selection_rect.offset.y,
                width: selection_rect.size.width,
                height: selection_rect.size.height,
            });
        }
        paint_state.LocalRectIntersectsCullRect(local_rect)
    }

    // Chromium: box_fragment_painter.cc:768-781. A block caret overlaps the
    // following glyph, so it is recorded before descendant text.
    pub fn PaintFragmentBeforeChildren(
        node: &'n PaintTreeNode<'f>,
        paint_info: &PaintInfo<'c, 'o>,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let unscaled_text_combine_caret = node.applies_transform
            && fragment.paint.text_combine.is_some()
            && paint_info.context.borrow().caret.is_some();
        if paint_info.phase == PaintPhase::kForeground
            && fragment.paint.HasSource()
            && IsVisibleToPaint(fragment)
            && PaintsCaretBeforeChildren(
                paint_info.context.borrow().caret_shape,
                unscaled_text_combine_caret,
            )
        {
            crate::caret_display_item_client::PaintCaret(node, paint_info.context);
        }
        if paint_info.phase == PaintPhase::kForeground
            && fragment.paint.HasSource()
            && IsVisibleToPaint(fragment)
            && PaintsCaretBeforeChildren(
                paint_info
                    .context
                    .borrow()
                    .drag_caret
                    .and_then(|state| state.native.map(|native| native.shape)),
                node.applies_transform && fragment.paint.text_combine.is_some(),
            )
        {
            crate::caret_display_item_client::PaintDragCaret(node, paint_info.context);
        }
    }

    // Rechrom's paint_walk owns Chromium's in-method child traversal. This
    // hook owns only the corresponding post-children caret, overflow-control,
    // table and frameset recording work.
    pub fn PaintFragmentAfterChildren(node: &'n PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        let _properties = if node.applies_transform
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.text_combine.is_some())
        {
            crate::paint_context::ScopedPaintChunkProperties::text_combine_parent(
                paint_info.context,
                node,
                paint_info.phase,
            )
        } else {
            crate::paint_context::ScopedPaintChunkProperties::properties_only(
                paint_info.context,
                node,
            )
        };
        Self::new(node, paint_info.context).PaintAfterChildren(paint_info);
    }

    // Recording bridge for SVGModelObjectPainter::PaintOutline
    // (svg_model_object_painter.cc:55-71). The caller has released
    // ScopedSVGPaintState while retaining the SVG transform.
    pub fn PaintSVGOutline(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) {
        Self::new(node, context).PaintOutline(PaintPhase::kSelfOutlineOnly);
    }

    // Chromium: box_fragment_painter.h:62; box_fragment_painter.cc:643-662.
    fn Paint(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.style.border_shape_unsupported() {
            panic!("unsupported CSS border-shape paint path");
        }
        let text_combine = fragment.paint.text_combine.is_some();
        if fragment.paint.svg_hidden_container
            || (!fragment.paint.HasSource() && !text_combine)
            || fragment.paint.hidden
            || !fragment.paint.CanPaintFragmentOccurrence()
        {
            return;
        }
        // cpp: core/layout/layout_image.cc:302-314. LayoutImage performs its
        // animated reuse check after delegating to ImagePainter::Paint, even
        // when that painter emits no image drawing. Retain the exact paint
        // entry fact for the document-owned UseCounter coordinator.
        if fragment.paint.replaced_is_image {
            if let Some(image_id) = fragment.paint.style.image_resource_id {
                self.context.borrow_mut().RecordPaintedLayoutImage(image_id);
            }
        }
        if text_combine {
            // Chromium paints the anonymous wrapper's compressed text during
            // child traversal, then restores its affine transform before
            // TextCombinePainter records the parent's decorations/emphasis.
            // PaintAfterChildren owns that source ordering in Rechrom.
            return;
        }
        self.PaintObject(paint_info);
    }

    // Chromium: box_fragment_painter.h:63-66; box_fragment_painter.cc:858-959.
    fn PaintObject(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let phase = paint_info.phase;
        let is_visible = IsVisibleToPaint(fragment);
        if phase == PaintPhase::kForcedColorsModeBackplate {
            if fragment.paint.is_inline_formatting_context && fragment.paint.InForcedColorsMode() {
                self.PaintBackplate();
            }
            return;
        }
        if phase == PaintPhase::kMask {
            if is_visible
                && matches!(
                    fragment.kind,
                    FragmentKind::kText | FragmentKind::kGeneratedText
                )
            {
                self.PaintText(paint_info);
            } else if is_visible {
                self.PaintMask(DisplayItemIdType::PaintPhaseToDrawingType(
                    PaintPhase::kMask,
                ));
            }
            return;
        }
        if matches!(
            phase,
            PaintPhase::kSelectionDragImage | PaintPhase::kTextClip
        ) && matches!(
            fragment.kind,
            FragmentKind::kText | FragmentKind::kGeneratedText
        ) {
            if is_visible {
                self.PaintText(paint_info);
            }
            return;
        }
        if fragment.paint.source_kind == NodeKind::kFrameSet {
            return;
        }
        if phase == PaintPhase::kForeground
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.box_fragment_is_inline_box
        {
            InlineBoxFragmentPainter::new(self.node, self.context).Paint(paint_info, self);
        }
        if phase == PaintPhase::kForeground && fragment.kind == FragmentKind::kLine {
            LineBoxFragmentPainter::new(self.node, self.context).Paint(paint_info, self);
        }
        // SVGContainerPainter paints content/effects/outline, never CSS box
        // decoration. The LayoutBox fragment storage ABI does not make a
        // nested SVG viewport into a CSS border/background box.
        if paint_model::paint_phase::ShouldPaintSelfBlockBackground(phase)
            && is_visible
            && !matches!(
                fragment.paint.source_kind,
                NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
            )
            && fragment.kind == FragmentKind::kBox
            && !fragment.paint.box_fragment_is_inline_box
            && fragment.paint.establishes_paint_state
        {
            // ViewPainter queries the current chunk before recording the root
            // background. A later query would observe the drawing we are
            // about to append and spuriously turn every root into a special
            // hit-test client.
            let records_background_hit_test = !fragment.paint.IsLayoutView()
                || paint_info.IsPaintingBackgroundInContentsSpace()
                || fragment.paint.effective_allowed_touch_action != foundation::TouchAction::kAuto
                || fragment.paint.inside_blocking_wheel_event_handler
                || fragment.paint.style.pointer_events_none()
                || self
                    .context
                    .borrow()
                    .CurrentChunkIsNonEmptyAndTransparentToHitTest();
            // ViewPainter::PaintBoxDecorationBackground paints no background
            // for the paginated LayoutView itself. Each selected page
            // container owns the `@page` background instead.
            if fragment.paint.is_paginated_root {
                return;
            }
            if fragment.paint.is_page_container {
                self.PaintPageContainerBackground(paint_info);
                return;
            }
            if self.node.is_root && !paint_info.ShouldSkipBackground() {
                if fragment.paint.background_transfers_to_view
                    && fragment.paint.canvas_background.is_some()
                {
                    let background_state = CanvasBackgroundPaintState(self.node);
                    let paints_separate_backdrop =
                        !background_state.same_nodes(&self.node.contents_properties.nodes);
                    let paints_separate_effect = !background_state
                        .effect
                        .lifecycle
                        .same_node(&self.node.contents_properties.nodes.effect.lifecycle);
                    if paints_separate_backdrop {
                        self.PaintRootGroup(phase);
                    }
                    self.PaintCanvasBackgroundWithState(
                        paint_info,
                        background_state,
                        paints_separate_backdrop,
                        paints_separate_effect,
                    );
                } else {
                    self.PaintRootGroup(phase);
                }
            }
            let composites_fixed_background = UsesCompositedFixedBackground(self.node);
            if composites_fixed_background {
                self.PaintCompositedFixedBackground(paint_info);
            }
            // Chromium constructs one BoxDecorationData before consulting the
            // drawing cache. Background, border, shadow and theme decisions
            // must therefore stay coupled to the same PaintInfo flags.
            let mut decoration_data = BoxDecorationData::new(self.node, paint_info);
            if !fragment.paint.background_paint_location.PaintsBorderBox()
                || composites_fixed_background
            {
                decoration_data = decoration_data.WithoutBackground();
            }
            let table_checks_column_backgrounds =
                fragment.paint.table.as_ref().is_some_and(|table| {
                    table.columns.iter().any(|column| {
                        column.style.background_color().alpha > 0.0
                            || column.style.has_background_images()
                    })
                });
            let has_box_decoration =
                decoration_data.ShouldPaint() || table_checks_column_backgrounds;
            let skip_under_invalidation_cache = self
                .context
                .borrow()
                .PaintUnderInvalidationCheckingEnabled()
                && ShouldSkipPaintUnderInvalidationChecking(fragment);
            let cache_skipper =
                skip_under_invalidation_cache.then(|| DisplayItemCacheSkipper::new(self.context));
            let decoration_visual_rect = DecorationVisualRect(self.node);
            let cached_box_decoration = has_box_decoration
                && DrawingRecorder::UseCachedDrawingIfPossible(
                    self.context,
                    self.node,
                    DisplayItemIdType::kBoxDecorationBackground,
                );
            let background_recorder = (has_box_decoration && !cached_box_decoration).then(|| {
                DrawingRecorder::new(
                    self.context,
                    self.node,
                    DisplayItemIdType::kBoxDecorationBackground,
                    decoration_visual_rect,
                )
            });
            // Chromium theme ownership is a property of the control appearance,
            // independent of whether PaintController moved the previous
            // BoxDecorationBackground record into this artifact.
            let theme_owns_decoration = decoration_data.HasAppearance();
            let is_fieldset = fragment.paint.fieldset.is_some();
            if !cached_box_decoration
                && theme_owns_decoration
                && decoration_data.ShouldPaintShadow()
            {
                self.PaintBoxShadows(phase, false);
            }
            if !cached_box_decoration && theme_owns_decoration {
                let painted_by_theme = ThemePainter::new(self.node, self.context).Paint(paint_info);
                debug_assert_eq!(painted_by_theme, theme_owns_decoration);
            }
            if !cached_box_decoration && !theme_owns_decoration {
                if is_fieldset {
                    if decoration_data.ShouldPaint() {
                        FieldsetPainter::new(self.node, self.context).PaintBoxDecorationBackground(
                            self,
                            paint_info,
                            &decoration_data,
                        );
                    }
                } else {
                    self.PaintCssBoxDecoration(paint_info, &decoration_data);
                }
            }
            if !cached_box_decoration
                && theme_owns_decoration
                && decoration_data.ShouldPaintShadow()
            {
                self.PaintBoxShadows(phase, true);
            }
            // Official TablePainter::PaintBoxDecorationBackground (.cc:421)
            // paints all column backgrounds inside the table's surrounding
            // BoxDecorationBackground recorder, not a record per column/cell.
            if !cached_box_decoration && table_checks_column_backgrounds {
                TablePainter::new(self.node, self.context).PaintColumnBackgrounds(paint_info);
            }
            drop(background_recorder);
            drop(cache_skipper);
            if fragment.paint.background_paint_location.PaintsBorderBox() {
                let painting_contents = paint_info.IsPaintingBackgroundInContentsSpace();
                let paint_rect = if fragment.paint.IsLayoutView() {
                    ViewBackgroundRect(self.node, painting_contents)
                        .unwrap_or_else(|| PixelSnappedBorderBoxRect(self.node, fragment.size))
                } else {
                    PixelSnappedBorderBoxRect(self.node, fragment.size)
                };
                let scrolling_client = painting_contents
                    .then(|| {
                        let scroll = fragment.paint.scroll_container.as_ref()?;
                        let client =
                            crate::paint_chunker::DisplayItemClientInfo::from_scrolling_background(
                                fragment,
                            )?;
                        Some((scroll.scrolling_background_client_id, client))
                    })
                    .flatten();
                let (background_client_id, background_client) = scrolling_client.unwrap_or((
                    fragment.paint.display_item_client_id,
                    crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
                ));
                if !(paint_info.IsPaintingBackgroundInContentsSpace()
                    && paint_info.ShouldSkipBackground())
                {
                    self.RecordRegionCaptureAndTrackedElementDataForClient(
                        paint_rect,
                        background_client_id,
                        background_client,
                    );
                }
                if records_background_hit_test {
                    self.RecordHitTestDataForClient(
                        paint_info,
                        paint_rect,
                        background_client_id,
                        background_client,
                    );
                }
            }
            // Chromium paints non-overlay controls before the scrolling
            // background so that background and contents can share their
            // scrolling layer (box_fragment_painter.cc:739-750). The
            // post-children hook derives the same condition and skips the
            // duplicate late pass.
            if fragment
                .paint
                .scrollbars
                .as_ref()
                .is_some_and(|scrollbars| {
                    PaintsOverflowControlsBeforeContents(
                        paint_info.phase,
                        scrollbars.should_paint_as_overlay,
                    )
                })
            {
                ScrollableAreaPainter::new(self.node, self.context)
                    .PaintOverflowControls(paint_info);
            }
            if !theme_owns_decoration {
                let paints_contents = fragment.paint.background_paint_location.PaintsContents();
                if paints_contents || fragment.paint.scrolls_overflow {
                    let _contents_properties =
                        crate::scoped_paint_state::ScopedBoxContentsPaintState::new(
                            paint_info, self.node,
                        );
                    if paints_contents {
                        self.PaintContentsBackground(paint_info);
                    }
                    if decoration_data.ShouldPaintGapDecorations() {
                        self.PaintGapRules(phase);
                    }
                    if fragment.paint.scrolls_overflow {
                        self.PaintColumnRules(phase);
                    }
                } else {
                    if decoration_data.ShouldPaintGapDecorations() {
                        self.PaintGapRules(phase);
                    }
                    self.PaintColumnRules(phase);
                }
            }
        }
        // Source: box_fragment_painter.cc:1446-1450 and box_painter.cc:37-122.
        // Record this after the non-scrolling background so background layer
        // squashing is unchanged. The current scoped properties are the local
        // border box; the payload points at the real contents ScrollTranslation.
        if paint_model::paint_phase::ShouldPaintSelfBlockBackground(phase)
            && is_visible
            && fragment.kind == FragmentKind::kBox
            && ShouldRecordHitTestData(fragment.paint.display)
            && fragment.paint.scroll_container.is_some()
            && ShouldRecordScrollHitTestDataForPaint(
                paint_info.ShouldOmitCompositingInfo(),
                fragment.paint.CanvasDrawElementEnabled(),
                fragment.paint.IsInCanvasSubtree(),
            )
        {
            let translation = self.node.contents_properties.nodes.transform.clone();
            if translation.scroll.is_some() {
                let opaqueness = GetHitTestOpaqueness(fragment);
                self.context.borrow_mut().RecordScrollHitTestData(
                    self.node,
                    DecorationVisualRect(self.node),
                    opaqueness,
                    translation,
                    PaintRect::default(),
                );
                if opaqueness != PaintHitTestOpaqueness::Transparent {
                    ScrollableAreaPainter::new(self.node, self.context)
                        .RecordResizerScrollHitTestData();
                }
            }
        }
        let _contents_properties = (matches!(
            phase,
            PaintPhase::kForeground | PaintPhase::kSelectionDragImage
        ) && fragment.kind == FragmentKind::kBox)
            .then(|| {
                crate::scoped_paint_state::ScopedBoxContentsPaintState::new(paint_info, self.node)
            });
        if phase == PaintPhase::kForeground
            && is_visible
            && matches!(
                fragment.kind,
                FragmentKind::kText | FragmentKind::kGeneratedText
            )
        {
            self.PaintText(paint_info);
        }
        if phase == PaintPhase::kForeground && is_visible && fragment.paint.mathml.is_some() {
            MathMLPainter::new(self.node, self.context).Paint(paint_info);
        }
        if phase == PaintPhase::kForeground && is_visible && fragment.paint.svg_shape.is_some() {
            SVGShapePainter::new(self.node, self.context).Paint(paint_info);
        }
        if matches!(
            phase,
            PaintPhase::kForeground | PaintPhase::kSelectionDragImage
        ) && is_visible
            && fragment.paint.source_kind == NodeKind::kReplaced
            && fragment.paint.establishes_paint_state
            && (phase != PaintPhase::kSelectionDragImage
                || fragment.paint.replaced_selection_state != SelectionState::kNone)
        {
            // cpp: core/layout/layout_replaced.cc:238-241
            // Source: image_painter.cc:59-60. ImagePainter delegates the
            // common box/background/border/outline pass to LayoutReplaced;
            // this point supplies only its replaced foreground contribution.
            if fragment
                .paint
                .replaced_content
                .as_ref()
                .is_some_and(|content| content.is_layout_embedded_content)
            {
                // EmbeddedContentPainter traversal belongs to paint_walk.
            } else if fragment
                .paint
                .replaced_content
                .as_ref()
                .is_some_and(|content| content.is_canvas)
            {
                HTMLCanvasPainter::new(self.node, self.context).PaintReplaced(paint_info);
            } else if fragment
                .paint
                .replaced_content
                .as_ref()
                .is_some_and(|content| content.is_video && !content.video_uses_poster)
                && (paint_info.ShouldOmitCompositingInfo()
                    || (phase == PaintPhase::kForeground
                        && self
                            .context
                            .borrow()
                            .ForeignLayerForNode(fragment.node_id)
                            .is_some()))
            {
                VideoPainter::new(self.node, self.context).PaintReplaced(paint_info);
            } else {
                self.PaintReplaced(paint_info);
            }
        }
        if PaintsSelfOutlineBeforeChildren(phase)
            && is_visible
            && !crate::scoped_svg_paint_state::ScopedSVGPaintState::OwnsPaintScope(self.node)
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.establishes_paint_state
        {
            self.PaintOutline(phase);
        }
        // Source: image_painter.cc:62-64. The specialized image-map focus
        // ring follows LayoutReplaced's ordinary outline contribution.
        if phase == PaintPhase::kOutline
            && is_visible
            && fragment.paint.replaced_is_image
            && fragment.paint.establishes_paint_state
        {
            self.PaintAreaElementFocusRing();
        }
    }

    // Blink box_fragment_painter.cc:1343-1370 and
    // box_painter_base.cc:1777-1795: one source client/phase records the
    // mask FillLayers and optional mask-box-image after contents.
    // DrawingRecorder paints the actual mask-image source after the contents.
    // Chromium: box_fragment_painter.h:193; box_fragment_painter.cc:1343-1369.
    pub(crate) fn PaintMask(&self, item_type: DisplayItemIdType) {
        if !self.node.applies_mask {
            return;
        }
        if self
            .node
            .fragment
            .as_deref()
            .expect("mask fragment")
            .paint
            .fieldset
            .is_some()
        {
            FieldsetPainter::new(self.node, self.context).PaintMask(self, item_type);
            return;
        }
        self.PaintMaskImages(item_type, None);
    }

    pub(crate) fn PaintMaskImages(
        &self,
        item_type: DisplayItemIdType,
        reference_rect: Option<PaintRect>,
    ) {
        let fragment = self.node.fragment.as_deref().expect("mask fragment");
        let mut bounds = self.node.mask_layers.first().map(|layer| layer.clip_rect);
        for layer in self.node.mask_layers.iter().skip(1) {
            bounds.as_mut().unwrap().union(layer.clip_rect);
        }
        if let Some(mask_box) = fragment.paint.style.mask_box_image.as_ref() {
            let mask_box_bounds = reference_rect.map_or_else(
                || NinePieceImagePainter::Bounds(self.node, mask_box),
                |rect| NinePieceImagePainter::BoundsWithin(rect, mask_box),
            );
            bounds = Some(bounds.map_or(mask_box_bounds, |mut bounds| {
                bounds.union(mask_box_bounds);
                bounds
            }));
        }
        let Some(bounds) = bounds else {
            return;
        };
        if !IsVisible(self.node, &bounds) {
            return;
        }
        let _drawing = DrawingRecorder::new(self.context, self.node, item_type, bounds);
        let mut mask_layers = self.node.mask_layers.clone();
        for layer in &mut mask_layers {
            layer.dynamic_range_limit = fragment.paint.style.dynamic_range_limit();
        }
        ApplyMaskLayerImageAutoDarkMode(
            &fragment.paint.style,
            fragment
                .paint
                .resources
                .as_deref()
                .map_or(1.0, |resources| resources.layout_zoom_factor),
            &mut mask_layers,
        );
        // Chromium records only the mask source in the kMask paint phase. Its
        // EffectPaintPropertyNode supplies kDstIn when PaintArtifactCompositor
        // builds the cc mask layer; applying DstIn inside this source record
        // would first intersect the mask with a transparent destination and
        // publish an empty mask layer.
        if !mask_layers.is_empty() {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawMask,
                    phase: PaintPhase::kMask,
                    node_id: fragment.node_id,
                    rect: bounds,
                    mask_layers,
                    ..Default::default()
                },
                self.node,
            );
        }
        if let Some(reference_rect) = reference_rect {
            NinePieceImagePainter::PaintMaskBoxWithin(
                self.node,
                self.context,
                PaintPhase::kMask,
                reference_rect,
            );
        } else {
            NinePieceImagePainter::PaintMaskBox(self.node, self.context, PaintPhase::kMask);
        }
    }

    // See PaintFragmentAfterChildren for the source ranges owned by this split.
    fn PaintAfterChildren(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let has_source = fragment.paint.HasSource();
        let text_combine = fragment.paint.text_combine.is_some();
        if (!has_source && !text_combine) || !IsVisibleToPaint(fragment) {
            return;
        }
        // Chromium: box_fragment_painter.cc:785-793. Bar and underscore
        // carets are recorded after descendant text.
        let caret_was_painted_before_text_combine = self.node.applies_transform
            && fragment.paint.text_combine.is_some()
            && self.context.borrow().caret.is_some();
        if has_source
            && paint_info.phase == PaintPhase::kForeground
            && PaintsCaretAfterChildren(
                self.context.borrow().caret_shape,
                caret_was_painted_before_text_combine,
            )
        {
            crate::caret_display_item_client::PaintCaret(self.node, self.context);
        }
        if has_source
            && paint_info.phase == PaintPhase::kForeground
            && PaintsCaretAfterChildren(
                self.context
                    .borrow()
                    .drag_caret
                    .and_then(|state| state.native.map(|native| native.shape)),
                self.node.applies_transform && fragment.paint.text_combine.is_some(),
            )
        {
            crate::caret_display_item_client::PaintDragCaret(self.node, self.context);
        }
        if has_source
            && PaintsSelfOutlineAfterChildren(paint_info.phase)
            && !crate::scoped_svg_paint_state::ScopedSVGPaintState::OwnsPaintScope(self.node)
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.establishes_paint_state
        {
            self.PaintOutline(PaintPhase::kSelfOutlineOnly);
        }
        if text_combine {
            self.PaintTextCombine(paint_info);
        }
        if !has_source {
            return;
        }
        if fragment.paint.source_kind == NodeKind::kFrameSet {
            FrameSetPainter::new(self.node, self.context).PaintBorders(paint_info);
        }
        if matches!(
            paint_info.phase,
            PaintPhase::kDescendantBlockBackgroundsOnly | PaintPhase::kBlockBackground
        ) && fragment.paint.collapsed_table.is_some()
        {
            TablePainter::new(self.node, self.context).PaintCollapsedBorders(paint_info);
        }
        let overflow_controls_were_painted_before_contents = fragment
            .paint
            .scrollbars
            .as_ref()
            .is_some_and(|scrollbars| {
                PaintsOverflowControlsBeforeContents(
                    paint_info.phase,
                    scrollbars.should_paint_as_overlay,
                )
            });
        if fragment.paint.establishes_paint_state
            && fragment.paint.scrollbars.is_some()
            && !overflow_controls_were_painted_before_contents
        {
            ScrollableAreaPainter::new(self.node, self.context).PaintOverflowControls(paint_info);
        }
        if fragment.paint.source_kind == NodeKind::kReplaced
            && fragment.paint.establishes_paint_state
        {
            self.PaintReplacedSelection(paint_info);
        }
    }

    // Chromium: text_combine_painter.cc:36-107. BoxFragmentPainter invokes
    // this after descendants and self-outline, with the wrapper's scale/skew
    // transform already restored by paint_walk.
    fn PaintTextCombine(&self, paint_info: &PaintInfo<'c, 'o>) {
        let phase = paint_info.phase;
        if matches!(
            phase,
            PaintPhase::kBlockBackground
                | PaintPhase::kForcedColorsModeBackplate
                | PaintPhase::kFloat
                | PaintPhase::kSelfBlockBackgroundOnly
                | PaintPhase::kDescendantBlockBackgroundsOnly
                | PaintPhase::kSelfOutlineOnly
        ) {
            return;
        }
        let mut decoration_painter = TextDecorationPainter::new(
            self.node,
            self.context,
            phase,
            None,
            paint_info.IsPrivacyPreserving(),
        );
        decoration_painter.SetSvgContextPaints(paint_info.GetSvgContextPaints());
        decoration_painter.Begin(TextDecorationPhase::Originating);
        decoration_painter.PaintExceptLineThrough();
        self.PaintTextCombineEmphasis(phase);
        decoration_painter.PaintOnlyLineThrough();
    }

    // Chromium: box_fragment_painter.cc:1966-1999. The retained style owns
    // the used forced-color inputs; Layout owns line-break identity and text
    // rectangles. Paint only groups paragraphs and records their fills.
    fn PaintBackplate(&self) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("backplate owner has a fragment");
        if !fragment.paint.style.forced_color_adjust_auto() || !fragment.paint.style.visible() {
            return;
        }
        let backplates = BuildBackplates(self.node);
        let Some(mut visual_rect) = backplates.first().copied() else {
            return;
        };
        for rect in backplates.iter().copied().skip(1) {
            visual_rect.union(rect);
        }
        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kForcedColorsModeBackplate,
        ) {
            return;
        }
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kForcedColorsModeBackplate,
            visual_rect.enclosing(),
        );
        let color = InvertColorIfNeeded(
            &fragment.paint.style,
            fragment.paint.style.forced_background_color(),
            ElementRole::kBackground,
        );
        for rect in backplates {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase: PaintPhase::kForcedColorsModeBackplate,
                    node_id: fragment.node_id,
                    rect,
                    color,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // Chromium gap/column decoration recording: box_fragment_painter.cc:1571-1641.
    fn PaintColumnRules(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(column_rules) = fragment.paint.column_rules.as_ref() else {
            return;
        };
        if column_rules.rules.is_empty() {
            return;
        }
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kColumnRules,
            DecorationVisualRect(self.node),
        );
        for rule in &column_rules.rules {
            let mut rect = PaintRect {
                x: self.node.paint_offset.x + rule.offset.x,
                y: self.node.paint_offset.y + rule.offset.y,
                width: rule.size.width,
                height: rule.size.height,
            };
            rect = self.node.PixelSnappedRect(&rect);
            if !IsVisible(self.node, &rect) {
                continue;
            }
            let color =
                InvertColorIfNeeded(&fragment.paint.style, rule.color, ElementRole::kBackground);
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color,
                    line_style: rule.line_style,
                    stroke_width: rule.stroke_width,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // Chromium: gap_decorations_painter.cc:222-527 and
    // box_fragment_painter.cc:1571-1641. Layout resolves intersection pairs,
    // insets, writing mode and rule-overlap ordering; Paint owns snapping,
    // culling, dark-mode color adjustment and display-item recording.
    fn PaintGapRules(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(gap_rules) = fragment.paint.gap_rules.as_ref() else {
            return;
        };
        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kColumnRules,
        ) {
            return;
        }
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kColumnRules,
            DecorationVisualRect(self.node),
        );
        for rule in &gap_rules.rules {
            let rect = self.node.PixelSnappedRect(&PaintRect {
                x: self.node.paint_offset.x + rule.offset.x,
                y: self.node.paint_offset.y + rule.offset.y,
                width: rule.size.width,
                height: rule.size.height,
            });
            if !IsVisible(self.node, &rect) {
                continue;
            }
            DrawBoxSide(
                self.node,
                self.context,
                phase,
                rect,
                rule.side,
                rule.color,
                rule.line_style,
            );
        }
    }

    fn PaintTablePartBackground(&self, paint_info: &PaintInfo<'c, 'o>) {
        let phase = paint_info.phase;
        fn collect_cells<'n, 'f>(
            parent: &'n PaintTreeNode<'f>,
            cells: &mut Vec<&'n PaintTreeNode<'f>>,
        ) {
            for child_owner in &parent.children {
                let child = child_owner.as_ref();
                if child
                    .fragment
                    .as_deref()
                    .expect("paint tree node has a fragment")
                    .paint
                    .display
                    == Display::kTableCell
                {
                    cells.push(child);
                } else {
                    collect_cells(child, cells);
                }
            }
        }
        let mut cells = Vec::new();
        collect_cells(self.node, &mut cells);
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint.style;
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        if background_color.alpha <= 0.0 && background_images.is_empty() {
            return;
        }
        let has_blended_layer = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        for cell in cells {
            let cell_fragment = cell
                .fragment
                .as_deref()
                .expect("paint tree node has a fragment");
            if !cell_fragment.paint.HasSource() || !cell_fragment.paint.style.visible() {
                continue;
            }
            let clip = PaintRect {
                x: cell.paint_offset.x,
                y: cell.paint_offset.y,
                width: cell_fragment.size.width,
                height: cell_fragment.size.height,
            };
            if !IsVisible(self.node, &clip) {
                continue;
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: clip,
                    ..Default::default()
                },
                self.node,
            );
            if has_blended_layer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSaveLayerBlend,
                        phase,
                        node_id: fragment.node_id,
                        blend_mode: PaintBlendMode::kNormal,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            if background_color.alpha > 0.0 {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: clip,
                        color: background_color,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            for layer in background_images.iter().rev() {
                self.PaintBackgroundImage(layer, paint_info, true, None);
            }
            if has_blended_layer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // Chromium: core/paint/box_fragment_painter.cc:1645-1743.
    // InlineBoxFragmentPainter delegates the same CSS decoration sequence to
    // this box painter, while fieldsets retain their specialized owner.
    pub(crate) fn PaintCssBoxDecoration(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        decoration_data: &BoxDecorationData,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let phase = paint_info.phase;
        let bleed_avoidance = decoration_data.GetBackgroundBleedAvoidance();

        if decoration_data.ShouldPaintShadow() {
            self.PaintBoxShadows(phase, false);
        }

        let clips_for_bleed = BleedAvoidanceIsClipping(bleed_avoidance);
        if clips_for_bleed {
            let border_radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: self
                        .node
                        .PixelSnappedRect(&BackgroundBoxRect(self.node, BackgroundBox::kBorderBox)),
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &border_radii,
                    ),
                    corner_radii: border_radii,
                    ..Default::default()
                },
                self.node,
            );
            if bleed_avoidance == BackgroundBleedAvoidance::kBackgroundBleedClipLayer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSaveLayerBlend,
                        phase,
                        node_id: fragment.node_id,
                        blend_mode: PaintBlendMode::kNormal,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }

        if matches!(
            fragment.paint.display,
            Display::kTableSection
                | Display::kTableHeaderGroup
                | Display::kTableFooterGroup
                | Display::kTableRow
        ) {
            self.PaintTablePartBackground(paint_info);
        } else if decoration_data.ShouldPaintBackground() {
            self.PaintBackground(paint_info, bleed_avoidance);
        }
        if decoration_data.ShouldPaintShadow() {
            self.PaintBoxShadows(phase, true);
        }
        if decoration_data.ShouldPaintBorder() {
            self.PaintBorder(phase, bleed_avoidance);
        }

        if clips_for_bleed {
            if bleed_avoidance == BackgroundBleedAvoidance::kBackgroundBleedClipLayer {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // cpp: paint/box_painter_base.cc:168-568
    pub(crate) fn PaintBoxShadows(&self, phase: PaintPhase, inset: bool) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint.style;
        let decoration_rect = DecorationRect(self.node);
        let decoration_box = self.node.PixelSnappedRect(&decoration_rect);
        let outer_size = Size {
            width: decoration_box.width,
            height: decoration_box.height,
        };
        // BoxPainterBase::PaintInsetBoxShadowWithBorderRect paints into the
        // pixel-snapped inner border shape.  Using the outer decoration box
        // makes the shadow cover the border and leaves the real inner edge
        // unshadowed (most visible for `inset 0 1px 0 ...`).
        let border_inset = inset
            .then(|| crate::contoured_border_geometry::IncludedBorderEdges(&fragment.paint))
            .unwrap_or_default();
        let radii = ResolveCornerRadii(
            style,
            &fragment.paint.border_sides,
            outer_size,
            border_inset,
        );
        for index in (0..style.box_shadow_count()).rev() {
            let shadow = style.box_shadow(index);
            if shadow.inset != inset || shadow.color.alpha <= 0.0 {
                continue;
            }
            if !shadow.offset.x.is_finite()
                || !shadow.offset.y.is_finite()
                || !shadow.blur_radius.is_finite()
                || shadow.blur_radius < 0.0
                || !shadow.spread.is_finite()
            {
                panic!("box shadow geometry is invalid");
            }
            if shadow.offset.x == 0.0
                && shadow.offset.y == 0.0
                && shadow.blur_radius == 0.0
                && shadow.spread == 0.0
            {
                continue;
            }
            let mut box_rect = PaintRect {
                x: decoration_box.x + border_inset.left,
                y: decoration_box.y + border_inset.top,
                width: (decoration_box.width - border_inset.left - border_inset.right).max(0.0),
                height: (decoration_box.height - border_inset.top - border_inset.bottom).max(0.0),
            };
            let bounds;
            if !inset {
                if box_rect.width + 2.0 * shadow.spread <= 0.0
                    || box_rect.height + 2.0 * shadow.spread <= 0.0
                {
                    continue;
                }
                let outset = shadow.spread.max(0.0) + 1.5 * shadow.blur_radius;
                bounds = PaintRect {
                    x: box_rect.x + shadow.offset.x - outset,
                    y: box_rect.y + shadow.offset.y - outset,
                    width: box_rect.width + 2.0 * outset,
                    height: box_rect.height + 2.0 * outset,
                };
            } else {
                let sides = &fragment.paint.border_sides;
                if !sides.left {
                    let amount = shadow.offset.x.max(0.0) + shadow.blur_radius;
                    box_rect.x -= amount;
                    box_rect.width += amount;
                }
                if !sides.top {
                    let amount = shadow.offset.y.max(0.0) + shadow.blur_radius;
                    box_rect.y -= amount;
                    box_rect.height += amount;
                }
                if !sides.right {
                    box_rect.width -= shadow.offset.x.min(0.0) - shadow.blur_radius;
                }
                if !sides.bottom {
                    box_rect.height -= shadow.offset.y.min(0.0) - shadow.blur_radius;
                }
                bounds = box_rect;
            }
            if !IsVisible(self.node, &bounds) {
                continue;
            }
            let clips_fragment_sides = !inset && !fragment.paint.border_sides.HasAllSides();
            if clips_fragment_sides {
                let blur_and_spread = (1.5 * shadow.blur_radius).ceil() + shadow.spread;
                let sides = &fragment.paint.border_sides;
                let left = if sides.left {
                    blur_and_spread - shadow.offset.x
                } else {
                    0.0
                };
                let top = if sides.top {
                    blur_and_spread - shadow.offset.y
                } else {
                    0.0
                };
                let right = if sides.right {
                    blur_and_spread + shadow.offset.x
                } else {
                    0.0
                };
                let bottom = if sides.bottom {
                    blur_and_spread + shadow.offset.y
                } else {
                    0.0
                };
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: PaintRect {
                            x: decoration_box.x - left,
                            y: decoration_box.y - top,
                            width: decoration_box.width + left + right,
                            height: decoration_box.height + top + bottom,
                        },
                        ..Default::default()
                    },
                    self.node,
                );
            }
            let shadow_radii = if inset {
                radii
            } else {
                crate::contoured_border_geometry::ExpandCornerRadiiUniform(
                    radii,
                    shadow.spread,
                    Size {
                        width: (box_rect.width + 2.0 * shadow.spread).max(0.0),
                        height: (box_rect.height + 2.0 * shadow.spread).max(0.0),
                    },
                )
            };
            let shadow_color = InvertColorIfNeeded(style, shadow.color, ElementRole::kBackground);
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawBoxShadow,
                    phase,
                    node_id: fragment.node_id,
                    rect: box_rect,
                    color: shadow_color,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &shadow_radii,
                    ),
                    // Outer shadow spread changes the casting shape's radii,
                    // while the excluded border-box hole retains the original
                    // radii. One shared value makes either the outer corners
                    // or the hole wrong whenever spread is nonzero.
                    corner_radii: shadow_radii,
                    inner_corner_radii: radii,
                    shadow_offset: shadow.offset,
                    blur_radius: shadow.blur_radius,
                    spread: shadow.spread,
                    inset: shadow.inset,
                    is_shadow: true,
                    shadow_has_opaque_background: style.background_color().alpha >= 1.0,
                    ..Default::default()
                },
                self.node,
            );
            if clips_fragment_sides {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
    }

    // cpp: paint/box_painter_base.cc:58-84,579-692,1152-1561
    // Chromium: box_fragment_painter.cc:1771-1803.
    pub(crate) fn PaintBackground(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        bleed_avoidance: BackgroundBleedAvoidance,
    ) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.background_transfers_to_view
            || fragment.paint.background_is_known_to_be_obscured
        {
            return;
        }
        let Some(style) = BackgroundStyleForPainting(fragment) else {
            return;
        };
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        let has_blended_layer = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let border_box = BackgroundBoxRect(self.node, BackgroundBox::kBorderBox);
        let border_radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
        let clips_for_bleed = BleedAvoidanceIsClipping(bleed_avoidance);
        let opens_outer_background_group = has_blended_layer;
        let has_outer_background_group = opens_outer_background_group || clips_for_bleed;
        // Blink paints the color and the bottom FillLayer image in one source
        // layer before applying the text mask.  Keeping them together is
        // observable at antialiased mask edges: masking each separately and
        // then compositing would apply the coverage twice.
        let special_color_image_layer = IsSpecialBackgroundClip(style.background_clip())
            && background_color.alpha > 0.0
            && background_images.iter().any(|layer| {
                layer.paints_background_color && self.CanPaintBackgroundImage(layer, paint_info)
            });
        let mut shares_bottom_layer_clip = false;
        let mut shared_clip_has_layer = false;
        if opens_outer_background_group {
            if border_radii.HasRadius() {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: self.node.PixelSnappedRect(&border_box),
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &border_radii,
                        ),
                        corner_radii: border_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        if background_color.alpha > 0.0
            && IsSpecialBackgroundClip(style.background_clip())
            && !special_color_image_layer
        {
            self.PaintSpecialClippedColor(background_color, style.background_clip(), phase);
        } else if background_color.alpha > 0.0 {
            let bleed_inset = (bleed_avoidance
                == BackgroundBleedAvoidance::kBackgroundBleedShrinkBackground)
                .then(|| BackgroundBleedInset(self.node))
                .flatten();
            let clips_background_stack_for_border =
                bleed_inset.is_some() && !background_images.is_empty();
            let shrink_for_border = style.background_clip() == BackgroundBox::kBorderBox
                && bleed_inset.is_some()
                && !clips_background_stack_for_border;
            let radii = if shrink_for_border {
                ResolveCornerRadii(
                    style,
                    &fragment.paint.border_sides,
                    Size {
                        width: border_box.width,
                        height: border_box.height,
                    },
                    bleed_inset.expect("shrink for border requires bleed inset"),
                )
            } else {
                BackgroundRadii(self.node, style.background_clip())
            };
            let mut unsnapped_rect = self
                .node
                .PixelSnappedRect(&BackgroundBoxRect(self.node, style.background_clip()));
            if shrink_for_border {
                let inset = bleed_inset.expect("shrink for border requires bleed inset");
                unsnapped_rect.x += inset.left;
                unsnapped_rect.y += inset.top;
                unsnapped_rect.width = (unsnapped_rect.width - inset.left - inset.right).max(0.0);
                unsnapped_rect.height = (unsnapped_rect.height - inset.top - inset.bottom).max(0.0);
            }
            let rect = unsnapped_rect;
            if IsVisible(self.node, &rect) {
                shares_bottom_layer_clip = !has_outer_background_group
                    && radii.HasRadius()
                    && !background_images.is_empty()
                    && background_images.last().unwrap().clip == style.background_clip();
                if shares_bottom_layer_clip {
                    let mut clip_rect = rect;
                    let mut clip_radii = radii;
                    if clips_background_stack_for_border {
                        let inset = bleed_inset.expect("clip for border requires bleed inset");
                        clip_rect.x += inset.left;
                        clip_rect.y += inset.top;
                        clip_rect.width = (clip_rect.width - inset.left - inset.right).max(0.0);
                        clip_rect.height = (clip_rect.height - inset.top - inset.bottom).max(0.0);
                        clip_radii = ResolveCornerRadii(
                            style,
                            &fragment.paint.border_sides,
                            Size {
                                width: border_box.width,
                                height: border_box.height,
                            },
                            inset,
                        );
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kSave,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kClipRoundedRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: clip_rect,
                            corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                                &clip_radii,
                            ),
                            corner_radii: clip_radii,
                            ..Default::default()
                        },
                        self.node,
                    );
                    if !clips_background_stack_for_border {
                        self.context.borrow_mut().Append(
                            DisplayItem {
                                r#type: DisplayItemType::kSaveLayerBlend,
                                phase,
                                node_id: fragment.node_id,
                                blend_mode: PaintBlendMode::kNormal,
                                ..Default::default()
                            },
                            self.node,
                        );
                        shared_clip_has_layer = true;
                    }
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: if radii.HasRadius()
                            && !shares_bottom_layer_clip
                            && !has_outer_background_group
                        {
                            DisplayItemType::kDrawRoundedRect
                        } else {
                            DisplayItemType::kDrawRect
                        },
                        phase,
                        node_id: fragment.node_id,
                        rect,
                        color: background_color,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &radii,
                        ),
                        corner_radii: radii,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
        let mut bottom_layer = true;
        for layer in background_images.iter().rev() {
            let outer_group_clips_layer =
                has_outer_background_group && layer.clip == BackgroundBox::kBorderBox;
            self.PaintBackgroundImage(
                layer,
                paint_info,
                outer_group_clips_layer || (shares_bottom_layer_clip && bottom_layer),
                (special_color_image_layer && layer.paints_background_color)
                    .then_some(background_color),
            );
            if shares_bottom_layer_clip && bottom_layer {
                if shared_clip_has_layer {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            bottom_layer = false;
        }
        if opens_outer_background_group {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            if border_radii.HasRadius() {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
    }

    /// BoxFragmentPainter::PaintCompositeBackgroundAttachmentFixed.  The
    /// drawing geometry is viewport-sized in the LayoutView transform; the
    /// element's background-clip is supplied by the independent property node
    /// built in PrePaint.
    fn PaintCompositedFixedBackground(&self, paint_info: &PaintInfo<'c, 'o>) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let properties = self
            .node
            .fixed_background_properties
            .as_ref()
            .expect("composited fixed background properties")
            .clone();
        let resources = fragment
            .paint
            .resources
            .as_ref()
            .expect("fixed background resources");
        let viewport = resources.viewport.expect("fixed background viewport");
        let zoom = resources.layout_zoom_factor.max(f64::MIN_POSITIVE);
        let rect = PaintRect {
            x: 0.0,
            y: 0.0,
            width: viewport.size.width as f64 * zoom,
            height: viewport.size.height as f64 * zoom,
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return;
        }
        let _properties =
            crate::paint_context::ScopedPaintChunkProperties::state(self.context, properties);
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kFixedAttachmentBackground,
            rect,
        );
        let style = &fragment.paint.style;
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        if !paints_background_images {
            if background_color.alpha > 0.0 {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect,
                        color: background_color,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        let layer = background_images
            .first()
            .expect("composited fixed background image layer");
        let isolates_layer = layer.blend_mode != PaintBlendMode::kNormal;
        if isolates_layer {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        if background_color.alpha > 0.0 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: background_color,
                    ..Default::default()
                },
                self.node,
            );
        }
        self.PaintBackgroundImageSourceWithGeometry(
            layer,
            paint_info,
            true,
            false,
            style,
            fragment.paint.effective_zoom,
            Some((rect, rect)),
            true,
        );
        if isolates_layer {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    /// ViewPainter::PaintRootGroup. The base backdrop is a drawing owned by
    /// the LayoutView's scrolling contents state, not a CSS canvas background.
    /// Keeping it in the paint artifact lets PaintArtifactCompositor upcast
    /// descendant content (including transformed SVG) into the root group,
    /// preserving Chromium's device-space raster origin.
    fn PaintRootGroup(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("root paint tree node has a fragment");
        if !fragment.paint.IsLayoutView() {
            return;
        }
        let Some(base_background_color) = BaseBackgroundColor(fragment) else {
            return;
        };
        let Some(rect) = ViewBackgroundRect(self.node, true) else {
            return;
        };
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        let id =
            CanvasBackgroundDisplayItemId(fragment, DisplayItemIdType::kDocumentRootBackdrop, 0);
        let client = crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment);
        let _properties = crate::paint_context::ScopedPaintChunkProperties::with_client(
            self.context,
            self.node.contents_properties.nodes.clone(),
            id,
            client,
        );
        let _drawing = DrawingRecorder::with_client(self.context, self.node, id, client, rect);
        let color = InvertColorIfNeeded(
            &fragment.paint.style,
            base_background_color,
            ElementRole::kBackground,
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase,
                node_id: fragment.node_id,
                rect,
                color,
                source_overwrite: fragment
                    .paint
                    .resources
                    .as_deref()
                    .is_some_and(|resources| resources.should_clear_document_background),
                ..Default::default()
            },
            self.node,
        );
    }

    /// ViewPainter::PaintRootElementGroup for a kPageContainer. The `@page`
    /// style covers the complete page, while background image positioning
    /// uses the page-border child and the document root's stitched size.
    fn PaintPageContainerBackground(&self, paint_info: &PaintInfo<'c, 'o>) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("page container paint tree node has a fragment");
        let Some(page_geometry) = fragment.paint.page_container_background_geometry else {
            return;
        };
        if ShouldForceWhiteBackgroundForPrintEconomy(fragment) {
            return;
        }
        let rect = self.node.PixelSnappedRect(&PaintRect {
            x: self.node.paint_offset.x,
            y: self.node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        });
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        let style = &fragment.paint.style;
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        let base_background_color = BaseBackgroundColor(fragment);
        let background_renderable = page_geometry.positioning_size.width > 0.0
            && page_geometry.positioning_size.height > 0.0;
        if !background_renderable && base_background_color.is_none() {
            return;
        }
        if base_background_color.is_none()
            && background_color.alpha <= 0.0
            && background_images.is_empty()
        {
            return;
        }
        let id = CanvasBackgroundDisplayItemId(fragment, DisplayItemIdType::kDocumentBackground, 0);
        let client = crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment);
        let _drawing = DrawingRecorder::with_client(self.context, self.node, id, client, rect);

        if let Some(base_background_color) = base_background_color {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: InvertColorIfNeeded(
                        style,
                        base_background_color,
                        ElementRole::kBackground,
                    ),
                    source_overwrite: fragment
                        .paint
                        .resources
                        .as_deref()
                        .is_some_and(|resources| resources.should_clear_document_background),
                    ..Default::default()
                },
                self.node,
            );
        }
        if !background_renderable {
            return;
        }
        let isolates_layers = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        if isolates_layers {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        if background_color.alpha > 0.0 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: background_color,
                    ..Default::default()
                },
                self.node,
            );
        }
        for layer in background_images.iter().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let positioning = if EffectiveBackgroundAttachment(self.node, layer)
                == BackgroundAttachment::kFixed
            {
                FixedBackgroundPositioningArea(self.node)
                    .unwrap_or_else(|| BackgroundPositioningBoxRect(self.node, layer.origin))
            } else {
                BackgroundPositioningBoxRect(self.node, layer.origin)
            };
            self.PaintBackgroundImageSourceWithGeometry(
                layer,
                paint_info,
                true,
                false,
                style,
                fragment.paint.effective_zoom,
                Some((positioning, rect)),
                true,
            );
        }
        if isolates_layers {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    /// ViewPainter::PaintRootElementGroup. Layout has already selected either
    /// the document element or first body background. CSS background-clip and
    /// border radii do not constrain the canvas; positioning still uses the
    /// root element box, while the paint area covers at least the viewport and
    /// (for scrolling layers) the complete scrollable contents.
    fn PaintCanvasBackground(&self, paint_info: &PaintInfo<'c, 'o>) {
        let background_state = CanvasBackgroundPaintState(self.node);
        let paints_separate_backdrop =
            !background_state.same_nodes(&self.node.contents_properties.nodes);
        let paints_separate_effect = !background_state
            .effect
            .lifecycle
            .same_node(&self.node.contents_properties.nodes.effect.lifecycle);
        self.PaintCanvasBackgroundWithState(
            paint_info,
            background_state,
            paints_separate_backdrop,
            paints_separate_effect,
        );
    }

    fn PaintCanvasBackgroundWithState(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        background_state: PropertyTreeState,
        paints_separate_backdrop: bool,
        paints_separate_effect: bool,
    ) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("root paint tree node has a fragment");
        let Some(canvas) = fragment.paint.canvas_background.as_ref() else {
            return;
        };
        let style = &canvas.style;
        if ShouldForceWhiteBackgroundForPrintEconomyWithStyle(fragment, style) {
            return;
        }
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        let has_fixed = background_images.iter().any(|layer| {
            (layer.shader.is_some() || layer.resource_id != 0)
                && EffectiveBackgroundAttachment(self.node, layer) == BackgroundAttachment::kFixed
        });
        let has_scrolling = background_images.iter().any(|layer| {
            (layer.shader.is_some() || layer.resource_id != 0)
                && EffectiveBackgroundAttachment(self.node, layer) != BackgroundAttachment::kFixed
        });
        if !(has_fixed && has_scrolling) {
            self.PaintCanvasBackgroundInOnePropertyState(
                paint_info,
                style,
                canvas.effective_zoom,
                has_fixed,
                background_state,
                paints_separate_backdrop,
                paints_separate_effect,
            );
            return;
        }

        // A mixed canvas cannot be represented by one PaintRecord: fixed
        // layers use the LayoutView border-box transform, while scroll/local
        // layers use ContentsProperties and its ScrollTranslation. Keep CSS
        // bottom-to-top order by assigning every layer its own stable record.
        let isolates_layers = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let group_effect = (isolates_layers && !paints_separate_effect).then(|| {
            let id = self.context.borrow_mut().AllocatePropertyId();
            Arc::new(CanvasBackgroundEffect(
                id,
                &background_state,
                background_state.clip.clone(),
                background_state.effect.clone(),
                PaintBlendMode::kNormal,
                true,
            ))
        });
        if group_effect.is_some() {
            // The unowned wrapper is the flat PaintRecord counterpart of the
            // retained isolation effect. LayerTile validates the two views and
            // lowers the effect across the differently transformed chunks.
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        let scrolling_rect = CanvasBackgroundRectInState(self.node, true, &background_state)
            .expect("root canvas background must have a scrolling paint rectangle");
        if scrolling_rect.width > 0.0
            && scrolling_rect.height > 0.0
            && IsVisible(self.node, &scrolling_rect)
            && CanvasBackgroundMarksFirstPaint(fragment, background_color)
        {
            self.context.borrow_mut().SetFirstPainted();
        }
        let canvas_color = if paints_separate_backdrop {
            background_color
        } else {
            CompositeColor(
                BaseBackgroundColor(fragment).unwrap_or_default(),
                background_color,
            )
        };
        if canvas_color.alpha > 0.0 {
            let properties = if let Some(group_effect) = &group_effect {
                let mut state = background_state.clone();
                state.effect = group_effect.clone();
                crate::paint_context::ScopedPaintChunkProperties::state(self.context, state)
            } else {
                crate::paint_context::ScopedPaintChunkProperties::state(
                    self.context,
                    background_state.clone(),
                )
            };
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.node, None);
            let _drawing = DrawingRecorder::with_client(
                self.context,
                self.node,
                CanvasBackgroundDisplayItemId(fragment, DisplayItemIdType::kDocumentBackground, 0),
                crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
                scrolling_rect,
            );
            if scrolling_rect.width > 0.0
                && scrolling_rect.height > 0.0
                && IsVisible(self.node, &scrolling_rect)
            {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: scrolling_rect,
                        color: canvas_color,
                        source_overwrite: !paints_separate_backdrop
                            && fragment
                                .paint
                                .resources
                                .as_deref()
                                .is_some_and(|resources| {
                                    resources.should_clear_document_background
                                }),
                        ..Default::default()
                    },
                    self.node,
                );
            }
            drop(_drawing);
            self.context.borrow_mut().EndEscapableGeometry(self.node);
            drop(properties);
        }
        for (layer_index, layer) in background_images.iter().enumerate().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let attachment = EffectiveBackgroundAttachment(self.node, layer);
            let fixed = attachment == BackgroundAttachment::kFixed;
            let mut base_state = background_state.clone();
            if fixed {
                base_state.transform = self.node.properties.nodes.transform.clone();
            }
            let rect = CanvasBackgroundRectInState(self.node, !fixed, &base_state)
                .expect("root canvas background layer must have a paint rectangle");
            if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
                continue;
            }
            let positioning = if fixed {
                FixedBackgroundPositioningArea(self.node).unwrap_or_else(|| {
                    CanvasBackgroundPositioningBoxRectInState(
                        self.node,
                        layer.origin,
                        &base_state,
                        rect,
                    )
                })
            } else {
                CanvasBackgroundPositioningBoxRectInState(
                    self.node,
                    layer.origin,
                    &base_state,
                    rect,
                )
            };
            let layer_effect = group_effect.as_ref().map(|group_effect| {
                if layer.blend_mode == PaintBlendMode::kNormal {
                    group_effect.clone()
                } else {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kSaveLayerBlend,
                            phase,
                            node_id: fragment.node_id,
                            blend_mode: layer.blend_mode,
                            ..Default::default()
                        },
                        self.node,
                    );
                    let id = self.context.borrow_mut().AllocatePropertyId();
                    Arc::new(CanvasBackgroundEffect(
                        id,
                        &base_state,
                        base_state.clip.clone(),
                        group_effect.clone(),
                        layer.blend_mode,
                        false,
                    ))
                }
            });
            let properties = if let Some(effect) = &layer_effect {
                let mut state = base_state;
                state.effect = effect.clone();
                crate::paint_context::ScopedPaintChunkProperties::state(self.context, state)
            } else {
                crate::paint_context::ScopedPaintChunkProperties::state(self.context, base_state)
            };
            let item_type = if fixed {
                DisplayItemIdType::kFixedAttachmentBackground
            } else {
                DisplayItemIdType::kDocumentBackground
            };
            if !fixed {
                self.context
                    .borrow_mut()
                    .BeginEscapableGeometry(self.node, None);
            }
            let drawing = DrawingRecorder::with_client(
                self.context,
                self.node,
                CanvasBackgroundDisplayItemId(fragment, item_type, layer_index + 1),
                crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
                rect,
            );
            let normalized_layer = layer_effect.as_ref().and_then(|_| {
                (layer.blend_mode != PaintBlendMode::kNormal).then(|| {
                    let mut layer = layer.clone();
                    layer.blend_mode = PaintBlendMode::kNormal;
                    layer
                })
            });
            self.PaintBackgroundImageSourceWithGeometry(
                normalized_layer.as_ref().unwrap_or(layer),
                paint_info,
                true,
                false,
                style,
                canvas.effective_zoom,
                Some((positioning, rect)),
                true,
            );
            drop(drawing);
            if !fixed {
                self.context.borrow_mut().EndEscapableGeometry(self.node);
            }
            drop(properties);
            if layer_effect.is_some() && layer.blend_mode != PaintBlendMode::kNormal {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
        if group_effect.is_some() {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    /// The second BoxFragmentPainter::PaintObject pass for a background whose
    /// canonical LayoutBox::BackgroundPaintLocation includes scrolling
    /// contents.  It is a distinct record in ContentsProperties; borders,
    /// shadows and theme decoration remain in border-box space.
    fn PaintContentsBackground(&self, paint_info: &PaintInfo<'c, 'o>) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.background_transfers_to_view {
            return;
        }
        let style = &fragment.paint.style;
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        let Some(scroll) = fragment.paint.scroll_container.as_ref() else {
            return;
        };
        let Some(client) =
            crate::paint_chunker::DisplayItemClientInfo::from_scrolling_background(fragment)
        else {
            return;
        };
        let border_rect = ContentsBackgroundBorderRect(self.node);
        let visual_rect = ScrollingBackgroundVisualRect(self.node);
        if border_rect.width <= 0.0
            || border_rect.height <= 0.0
            || !IsVisible(self.node, &visual_rect)
        {
            return;
        }
        self.context.borrow_mut().RegisterAuxiliaryClient(
            scroll.scrolling_background_client_id,
            fragment.paint.display_item_client_owner_node_id,
            std::sync::Arc::from(format!(
                "{} scrolling background",
                fragment.paint.display_item_client_debug_name
            )),
        );
        let skip_under_invalidation_cache = self
            .context
            .borrow()
            .PaintUnderInvalidationCheckingEnabled()
            && ShouldSkipPaintUnderInvalidationChecking(fragment);
        let cache_skipper =
            skip_under_invalidation_cache.then(|| DisplayItemCacheSkipper::new(self.context));
        let _drawing = DrawingRecorder::with_client(
            self.context,
            self.node,
            DisplayItemId {
                client_id: scroll.scrolling_background_client_id,
                r#type: DisplayItemIdType::PaintPhaseToClipType(
                    PaintPhase::kSelfBlockBackgroundOnly,
                ),
                fragment: fragment.paint.display_item_fragment,
            },
            client,
            visual_rect,
        );
        let isolates_layers = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let special_color_image_layer = IsSpecialBackgroundClip(style.background_clip())
            && background_color.alpha > 0.0
            && background_images.iter().any(|layer| {
                layer.paints_background_color && self.CanPaintBackgroundImage(layer, paint_info)
            });
        if isolates_layers {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        if background_color.alpha > 0.0 && !special_color_image_layer {
            let rect = BackgroundBoxRectWithin(self.node, style.background_clip(), border_rect);
            if IsSpecialBackgroundClip(style.background_clip()) {
                self.BeginSpecialBackgroundClip(phase, PaintBlendMode::kNormal, border_rect);
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: background_color,
                    ..Default::default()
                },
                self.node,
            );
            if IsSpecialBackgroundClip(style.background_clip()) {
                self.EndSpecialBackgroundClip(phase, style.background_clip());
            }
        }
        for layer in background_images.iter().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let positioning = BackgroundBoxRectWithin(self.node, layer.origin, border_rect);
            let clip = BackgroundBoxRectWithin(self.node, layer.clip, border_rect);
            if IsSpecialBackgroundClip(layer.clip) {
                self.BeginSpecialBackgroundClip(phase, layer.blend_mode, border_rect);
                if special_color_image_layer && layer.paints_background_color {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: border_rect,
                            color: background_color,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                let mut source = layer.clone();
                source.clip = BackgroundBox::kBorderBox;
                source.blend_mode = PaintBlendMode::kNormal;
                self.PaintBackgroundImageSourceWithGeometry(
                    &source,
                    paint_info,
                    false,
                    false,
                    style,
                    fragment.paint.effective_zoom,
                    Some((positioning, border_rect)),
                    true,
                );
                self.EndSpecialBackgroundClip(phase, layer.clip);
            } else {
                self.PaintBackgroundImageSourceWithGeometry(
                    layer,
                    paint_info,
                    false,
                    false,
                    style,
                    fragment.paint.effective_zoom,
                    Some((positioning, clip)),
                    true,
                );
            }
        }
        if isolates_layers {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
        drop(_drawing);
        drop(cache_skipper);
        let snapped_border_rect = self.node.PixelSnappedRect(&border_rect);
        if !paint_info.ShouldSkipBackground()
            && (fragment.paint.region_capture_crop_id.is_some()
                || !fragment.paint.tracked_element_sub_rects.is_empty())
        {
            self.RecordRegionCaptureAndTrackedElementDataForClient(
                snapped_border_rect,
                scroll.scrolling_background_client_id,
                client,
            );
        }
        self.RecordHitTestDataForClient(
            paint_info,
            snapped_border_rect,
            scroll.scrolling_background_client_id,
            client,
        );
    }

    fn PaintCanvasBackgroundInOnePropertyState(
        &self,
        paint_info: &PaintInfo<'c, 'o>,
        style: &PaintStyle,
        effective_zoom: f32,
        fixed: bool,
        mut background_state: PropertyTreeState,
        paints_separate_backdrop: bool,
        paints_separate_effect: bool,
    ) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("root paint tree node has a fragment");
        let background_images = style.background_images();
        let (background_color, paints_background_images) =
            BackgroundForPainting(fragment, style, &background_images);
        let background_images = if paints_background_images {
            background_images.as_slice()
        } else {
            &[]
        };
        if fixed {
            background_state.transform = self.node.properties.nodes.transform.clone();
        }
        let rect = CanvasBackgroundRectInState(self.node, !fixed, &background_state)
            .expect("root canvas background must have a paint rectangle");
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        if CanvasBackgroundMarksFirstPaint(fragment, background_color) {
            self.context.borrow_mut().SetFirstPainted();
        }
        let _properties = crate::paint_context::ScopedPaintChunkProperties::state(
            self.context,
            background_state.clone(),
        );
        let canvas_color = if paints_separate_backdrop {
            background_color
        } else {
            CompositeColor(
                BaseBackgroundColor(fragment).unwrap_or_default(),
                background_color,
            )
        };
        let item_type = if fixed {
            DisplayItemIdType::kFixedAttachmentBackground
        } else {
            DisplayItemIdType::kDocumentBackground
        };
        // ViewPainter records a scrolling canvas in ContentsProperties.
        // The layer's self-background is painted before its descendant clip,
        // so enter the same source overflow clip in the flat record wrapper.
        // Fixed canvas layers retain border-box properties and their clip.
        if !fixed {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.node, None);
        }
        let _drawing = DrawingRecorder::with_client(
            self.context,
            self.node,
            CanvasBackgroundDisplayItemId(fragment, item_type, 0),
            crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
            rect,
        );
        let isolates_layers = background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        if isolates_layers && !paints_separate_effect {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase,
                    node_id: fragment.node_id,
                    blend_mode: PaintBlendMode::kNormal,
                    ..Default::default()
                },
                self.node,
            );
        }
        if canvas_color.alpha > 0.0 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: canvas_color,
                    source_overwrite: !paints_separate_backdrop
                        && fragment
                            .paint
                            .resources
                            .as_deref()
                            .is_some_and(|resources| resources.should_clear_document_background),
                    ..Default::default()
                },
                self.node,
            );
        }
        for layer in background_images.iter().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let positioning = if fixed {
                FixedBackgroundPositioningArea(self.node).unwrap_or_else(|| {
                    CanvasBackgroundPositioningBoxRectInState(
                        self.node,
                        layer.origin,
                        &background_state,
                        rect,
                    )
                })
            } else {
                CanvasBackgroundPositioningBoxRectInState(
                    self.node,
                    layer.origin,
                    &background_state,
                    rect,
                )
            };
            self.PaintBackgroundImageSourceWithGeometry(
                layer,
                paint_info,
                true,
                false,
                style,
                effective_zoom,
                Some((positioning, rect)),
                true,
            );
        }
        if isolates_layers && !paints_separate_effect {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
        drop(_drawing);
        if !fixed {
            self.context.borrow_mut().EndEscapableGeometry(self.node);
        }
    }

    // box_painter_base.cc:1563-1605.  A text-clipped background is an
    // isolated source layer followed by a second layer whose alpha is applied
    // with DstIn.  Keeping this in the background DrawingRecorder is
    // important: the mask is part of the background display item, not a later
    // foreground effect that could also erase previously painted siblings.
    fn BeginSpecialBackgroundClip(
        &self,
        phase: PaintPhase,
        blend_mode: PaintBlendMode,
        mask_rect: PaintRect,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase,
                node_id: fragment.node_id,
                ..Default::default()
            },
            self.node,
        );
        let radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
        if radii.HasRadius() {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: mask_rect,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(&radii),
                    corner_radii: radii,
                    antialias: true,
                    ..Default::default()
                },
                self.node,
            );
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase,
                node_id: fragment.node_id,
                rect: mask_rect,
                ..Default::default()
            },
            self.node,
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSaveLayerBlend,
                phase,
                node_id: fragment.node_id,
                blend_mode,
                ..Default::default()
            },
            self.node,
        );
    }

    fn EndSpecialBackgroundClip(&self, phase: PaintPhase, clip: BackgroundBox) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSaveLayerDstIn,
                phase,
                node_id: fragment.node_id,
                ..Default::default()
            },
            self.node,
        );
        if matches!(
            clip,
            BackgroundBox::kBorderArea | BackgroundBox::kBorderAreaText
        ) {
            // BoxBorderPainter::PaintBorderArea ignores border color and
            // transparency while preserving the used
            // dotted/dashed/double/rounded border geometry.
            BoxBorderPainter::new(self.node, self.context, DecorationRect(self.node))
                .PaintBorderArea(PaintPhase::kTextClip);
        }
        if matches!(clip, BackgroundBox::kText | BackgroundBox::kBorderAreaText) {
            self.PaintTextClipMask();
        }
        for _ in 0..3 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    fn PaintSpecialClippedColor(&self, color: Color, clip: BackgroundBox, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let rect = self.node.PixelSnappedRect(&DecorationRect(self.node));
        if !IsVisible(self.node, &rect) {
            return;
        }
        self.BeginSpecialBackgroundClip(phase, PaintBlendMode::kNormal, rect);
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase,
                node_id: fragment.node_id,
                rect,
                color,
                ..Default::default()
            },
            self.node,
        );
        self.EndSpecialBackgroundClip(phase, clip);
    }

    fn CanPaintBackgroundImage(
        &self,
        layer: &BackgroundImageLayer,
        paint_info: &PaintInfo<'c, 'o>,
    ) -> bool {
        if !layer.can_render {
            return false;
        }
        if layer.shader.is_some() {
            return true;
        }
        let Some(image) = FindAvailableImage(self.node, layer.resource_id) else {
            return false;
        };
        !paint_info.IsPrivacyPreserving() || image.is_cors_same_origin
    }

    fn PaintSpecialClippedBackgroundImage(
        &self,
        layer: &BackgroundImageLayer,
        paint_info: &PaintInfo<'c, 'o>,
        background_color: Option<Color>,
    ) {
        let phase = paint_info.phase;
        let rect = self.node.PixelSnappedRect(&DecorationRect(self.node));
        if !IsVisible(self.node, &rect) {
            return;
        }
        self.BeginSpecialBackgroundClip(phase, layer.blend_mode, rect);
        if let Some(color) = background_color {
            let fragment = self
                .node
                .fragment
                .as_deref()
                .expect("paint tree node has a fragment");
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color,
                    ..Default::default()
                },
                self.node,
            );
        }
        let mut source = layer.clone();
        source.clip = BackgroundBox::kBorderBox;
        source.blend_mode = PaintBlendMode::kNormal;
        // The isolated source still uses the ordinary background geometry,
        // including local attachment, stitched decorations and rounded border
        // clipping.  Only the final effective clip is replaced by the text or
        // border-area mask below.
        self.PaintBackgroundImageSource(&source, paint_info, true, false);
        self.EndSpecialBackgroundClip(phase, layer.clip);
    }

    // box_fragment_painter.cc:2135-2175.  The special phase walks the
    // fragment descendants directly, including self-painting layers, because
    // the layer-tree walk has no independent kTextClip pass.  Geometry and
    // clips participate; opacity, blend, filters and masks deliberately do
    // not, since the mask represents text shape rather than composited color.
    fn PaintTextClipMask(&self) {
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.node, None);
        for child in &self.node.children {
            self.PaintTextClipSubtree(child);
        }
        self.context.borrow_mut().EndEscapableGeometry(self.node);
    }

    fn PaintTextClipSubtree(&self, node: &PaintTreeNode<'_>) {
        let Some(fragment) = node.fragment.as_deref() else {
            return;
        };
        if fragment.paint.hidden {
            return;
        }
        // Chromium's kTextClip block-child traversal excludes floating
        // descendants, including self-painting floating layers
        // (box_fragment_painter.cc:1160-1180).
        if fragment.paint.floating != FloatSide::kNone {
            return;
        }
        {
            let mut context = self.context.borrow_mut();
            context.BeginGeometry(node);
            context.BeginCssClip(node);
            context.BeginEscapableGeometry(node, None);
        }
        if fragment.paint.HasSource()
            && fragment.paint.style.visible()
            && matches!(
                fragment.kind,
                FragmentKind::kText | FragmentKind::kGeneratedText
            )
        {
            let input = PaintInfo::new(self.context, PaintPhase::kTextClip);
            let paint_state = crate::scoped_paint_state::ScopedPaintState::new(node, &input);
            BoxFragmentPainter::new(node, self.context).PaintText(paint_state.GetPaintInfo());
        }
        for child in &node.children {
            self.PaintTextClipSubtree(child);
        }
        {
            let mut context = self.context.borrow_mut();
            context.EndEscapableGeometry(node);
            context.EndCssClip(node);
            context.EndGeometry(node);
        }
    }

    fn PaintBackgroundImage(
        &self,
        layer: &BackgroundImageLayer,
        paint_info: &PaintInfo<'c, 'o>,
        already_clipped: bool,
        background_color: Option<Color>,
    ) {
        if !self.CanPaintBackgroundImage(layer, paint_info) {
            return;
        }
        if IsSpecialBackgroundClip(layer.clip) {
            self.PaintSpecialClippedBackgroundImage(layer, paint_info, background_color);
            return;
        }
        self.PaintBackgroundImageSource(layer, paint_info, already_clipped, true);
    }

    fn PaintBackgroundImageSource(
        &self,
        layer: &BackgroundImageLayer,
        paint_info: &PaintInfo<'c, 'o>,
        already_clipped: bool,
        apply_border_bleed_avoidance: bool,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        self.PaintBackgroundImageSourceWithGeometry(
            layer,
            paint_info,
            already_clipped,
            apply_border_bleed_avoidance,
            &fragment.paint.style,
            fragment.paint.effective_zoom,
            None,
            false,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn PaintBackgroundImageSourceWithGeometry(
        &self,
        layer: &BackgroundImageLayer,
        paint_info: &PaintInfo<'c, 'o>,
        already_clipped: bool,
        apply_border_bleed_avoidance: bool,
        paint_style: &PaintStyle,
        effective_zoom: f32,
        geometry_override: Option<(PaintRect, PaintRect)>,
        ignore_border_radius: bool,
    ) {
        if !self.CanPaintBackgroundImage(layer, paint_info) {
            return;
        }
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let attachment = EffectiveBackgroundAttachment(self.node, layer);
        let has_geometry_override = geometry_override.is_some();
        let (positioning, mut clip, local_outer_clip) = if let Some((positioning, clip)) =
            geometry_override
        {
            (positioning, clip, None)
        } else {
            match attachment {
                BackgroundAttachment::kScroll => (
                    BackgroundPositioningBoxRect(self.node, layer.origin),
                    BackgroundBoxRect(self.node, layer.clip),
                    None,
                ),
                BackgroundAttachment::kFixed => {
                    let positioning = FixedBackgroundPositioningArea(self.node)
                        .unwrap_or_else(|| BackgroundPositioningBoxRect(self.node, layer.origin));
                    (positioning, BackgroundBoxRect(self.node, layer.clip), None)
                }
                BackgroundAttachment::kLocal => {
                    let scrolled_border = LocalBackgroundBorderRect(self.node);
                    (
                        BackgroundBoxRectWithin(self.node, layer.origin, scrolled_border),
                        BackgroundBoxRectWithin(self.node, layer.clip, scrolled_border),
                        self.node
                            .local_inner_border_radius_clip
                            .map(|rect| (rect, self.node.local_inner_border_radius_radii))
                            .or_else(|| {
                                self.node
                                    .local_clip
                                    .map(|rect| (rect, self.node.local_clip_radii))
                            }),
                    )
                }
            }
        };
        // BackgroundImageGeometry keeps the image destination independent from
        // the later rounded-border bleed clip.  In particular, an opaque border
        // contracts the destination from this original paint rectangle rather
        // than from the already half-border-inset bleed rectangle.
        let destination_paint_rect = clip;
        let mut image_paint_rect = clip;
        let mut radii = if ignore_border_radius {
            PaintCornerRadii::default()
        } else {
            BackgroundRadiiForRect(self.node, layer.clip, clip)
        };
        if apply_border_bleed_avoidance
            && attachment != BackgroundAttachment::kLocal
            && layer.clip == BackgroundBox::kBorderBox
        {
            if let Some(bleed_inset) = BackgroundBleedInset(self.node) {
                clip.x += bleed_inset.left;
                clip.y += bleed_inset.top;
                clip.width = (clip.width - bleed_inset.left - bleed_inset.right).max(0.0);
                clip.height = (clip.height - bleed_inset.top - bleed_inset.bottom).max(0.0);
                let border_box = BackgroundBoxRect(self.node, BackgroundBox::kBorderBox);
                radii = ResolveCornerRadii(
                    paint_style,
                    &fragment.paint.border_sides,
                    Size {
                        width: border_box.width,
                        height: border_box.height,
                    },
                    bleed_inset,
                );
                let border = IncludedBorderEdges(&fragment.paint);
                image_paint_rect.x += border.left;
                image_paint_rect.y += border.top;
                image_paint_rect.width =
                    (image_paint_rect.width - border.left - border.right).max(0.0);
                image_paint_rect.height =
                    (image_paint_rect.height - border.top - border.bottom).max(0.0);
            }
        }
        if positioning.width <= 0.0
            || positioning.height <= 0.0
            || clip.width <= 0.0
            || clip.height <= 0.0
            || !IsVisible(self.node, &clip)
        {
            return;
        }
        if !layer.position.x.is_finite() || !layer.position.y.is_finite() {
            panic!("background position must be finite");
        }
        let image = if layer.shader.is_some() {
            None
        } else {
            Some(FindImage(self.node, layer.resource_id))
        };
        let image_orientation = image.map_or(image_resource::ImageOrientation::kDefault, |image| {
            if paint_style.respect_image_orientation() {
                image.orientation
            } else {
                image_resource::ImageOrientation::kDefault
            }
        });
        let tile_size = ResolveBackgroundTileSize(
            layer,
            &positioning,
            image,
            image_orientation,
            f64::from(effective_zoom),
        );
        if tile_size.width == 0.0 || tile_size.height == 0.0 {
            return;
        }
        let resolved = ResolveBackgroundTile(layer, &positioning, tile_size);
        let tile = resolved.tile_rect;
        let repeat_x = resolved.repeat_x;
        let repeat_y = resolved.repeat_y;
        let destination = ResolveBackgroundDestinationRects(
            self.node,
            layer,
            phase,
            attachment,
            destination_paint_rect,
            has_geometry_override,
        );
        let snapped_tile = self.node.PixelSnappedRect(&tile);
        let unsnapped_image_destination =
            ConstrainBackgroundDestinationToTile(destination.unsnapped, tile, repeat_x, repeat_y);
        let image_destination = ConstrainBackgroundDestinationToTile(
            destination.snapped,
            snapped_tile,
            repeat_x,
            repeat_y,
        );
        if image_destination.width <= 0.0 || image_destination.height <= 0.0 {
            return;
        }
        let bitmap_tile_scale = if let Some(image) = image {
            let (width, height) =
                image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
            Offset {
                x: tile.width / width,
                y: tile.height / height,
            }
        } else {
            resolved.tile_scale
        };
        let document_image_container_key = image.and_then(|image| {
            ResolveDocumentImageContainerKey(
                image,
                Size {
                    width: tile.width,
                    height: tile.height,
                },
                f64::from(effective_zoom),
                fragment.paint.resources.as_deref(),
                (!paint_style.color_scheme_flags_is_normal())
                    .then_some(u8::from(paint_style.used_dark_color_scheme())),
            )
        });
        if browser_tracing::enabled() {
            if let (Some(image), Some(container)) = (image, &document_image_container_key) {
                browser_tracing::instant(
                    "paint_record",
                    "PaintRecord.DocumentBackgroundContainer",
                    &[
                        ("tile_width", tile.width),
                        ("tile_height", tile.height),
                        ("image_width", f64::from(image.width)),
                        ("image_height", f64::from(image.height)),
                        ("container_width", f64::from(container.width)),
                        ("container_height", f64::from(container.height)),
                        ("size_mode", layer.size_mode as i32 as f64),
                    ],
                );
            }
        }
        let no_repeat_bitmap_destination = if image.is_some() && !repeat_x && !repeat_y {
            let image_origin = self.node.PixelSnappedOffset(Offset {
                x: tile.x,
                y: tile.y,
            });
            let full_destination = PaintRect {
                x: image_origin.x,
                y: image_origin.y,
                width: tile.width.round(),
                height: tile.height.round(),
            };
            let clipped_destination = IntersectPaintRects(image_destination, full_destination);
            if clipped_destination.width <= 0.0 || clipped_destination.height <= 0.0 {
                return;
            }
            Some((full_destination, clipped_destination))
        } else {
            None
        };
        let gradient_destination = layer.shader.as_ref().map(|_| {
            IntersectPaintRects(
                image_destination,
                self.node.PixelSnappedRect(&image_paint_rect),
            )
        });
        if gradient_destination
            .is_some_and(|destination| destination.width <= 0.0 || destination.height <= 0.0)
        {
            return;
        }
        let opens_attachment_clip = !already_clipped || local_outer_clip.is_some();
        if opens_attachment_clip {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
        if let Some((outer_clip, outer_radii)) = local_outer_clip {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: if outer_radii.HasRadius() {
                        DisplayItemType::kClipRoundedRect
                    } else {
                        DisplayItemType::kClipRect
                    },
                    phase,
                    node_id: fragment.node_id,
                    rect: outer_clip,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &outer_radii,
                    ),
                    corner_radii: outer_radii,
                    antialias: outer_radii.HasRadius(),
                    ..Default::default()
                },
                self.node,
            );
        }
        if !already_clipped || local_outer_clip.is_some() {
            let paint_clip = if radii.HasRadius() {
                self.node.PixelSnappedRect(&clip)
            } else {
                clip
            };
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: if radii.HasRadius() {
                        DisplayItemType::kClipRoundedRect
                    } else {
                        DisplayItemType::kClipRect
                    },
                    phase,
                    node_id: fragment.node_id,
                    rect: paint_clip,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(&radii),
                    corner_radii: radii,
                    antialias: radii.HasRadius(),
                    ..Default::default()
                },
                self.node,
            );
        }
        if let Some(shader) = layer.shader.as_ref() {
            let gradient_tile = snapped_tile;
            let gradient_destination = gradient_destination
                .expect("gradient destination must be resolved before recording");
            let mut item = DisplayItem {
                r#type: if repeat_x || repeat_y {
                    DisplayItemType::kDrawTiledGradient
                } else {
                    DisplayItemType::kDrawGradientRect
                },
                phase,
                node_id: fragment.node_id,
                rect: gradient_destination,
                tile_rect: gradient_tile,
                repeat_x,
                repeat_y,
                background_repeat_x: resolved.rule_x,
                background_repeat_y: resolved.rule_y,
                blend_mode: layer.blend_mode,
                tile_scale: resolved.tile_scale,
                tile_spacing: resolved.tile_spacing,
                ..Default::default()
            };
            item.paint_shader = Some(ResolvePaintShader(
                shader,
                fragment.paint.resources.as_deref(),
                Offset {
                    x: gradient_tile.x,
                    y: gradient_tile.y,
                },
                Some(Size {
                    width: gradient_tile.width,
                    height: gradient_tile.height,
                }),
            ));
            self.context.borrow_mut().Append(item, self.node);
        } else if repeat_x || repeat_y {
            let image = image.expect("bitmap background requires image");
            let (source_width, source_height) =
                image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
            let source_rect = PaintRect {
                x: 0.0,
                y: 0.0,
                width: source_width,
                height: source_height,
            };
            let use_dark_mode_for_image = GetImageAutoDarkMode(
                paint_style,
                fragment
                    .paint
                    .resources
                    .as_deref()
                    .map_or(1.0, |resources| resources.layout_zoom_factor),
                clip,
                source_rect,
            )
            .ShouldUseRasterDarkMode();
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawTiledImage,
                    phase,
                    node_id: fragment.node_id,
                    rect: image_destination,
                    source_rect,
                    // BackgroundImageGeometry pixel-snaps the bitmap phase
                    // before Image::DrawPattern.  Keeping the unsnapped box
                    // origin here makes a 1:1 image at a half-pixel layout
                    // position interpolate adjacent texels instead.
                    tile_rect: snapped_tile,
                    resource_id: image.id,
                    image_interpolation_quality: paint_style.image_interpolation_quality(),
                    dynamic_range_limit: paint_style.dynamic_range_limit(),
                    image_orientation,
                    use_dark_mode_for_image,
                    document_image_container_key: document_image_container_key.clone(),
                    repeat_x,
                    repeat_y,
                    background_repeat_x: resolved.rule_x,
                    background_repeat_y: resolved.rule_y,
                    blend_mode: layer.blend_mode,
                    tile_scale: bitmap_tile_scale,
                    tile_spacing: resolved.tile_spacing,
                    ..Default::default()
                },
                self.node,
            );
        } else {
            let image = image.expect("bitmap background requires image");
            let (source_width, source_height) =
                image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
            let (full_destination, image_destination) = no_repeat_bitmap_destination
                .expect("no-repeat bitmap destination must be resolved before recording");
            let full_source_rect = PaintRect {
                x: 0.0,
                y: 0.0,
                width: source_width,
                height: source_height,
            };
            let source_rect = CropBackgroundSourceRect(
                full_source_rect,
                full_destination,
                image_destination,
                unsnapped_image_destination,
                Size {
                    width: tile.width,
                    height: tile.height,
                },
            );
            let use_dark_mode_for_image = GetImageAutoDarkMode(
                paint_style,
                fragment
                    .paint
                    .resources
                    .as_deref()
                    .map_or(1.0, |resources| resources.layout_zoom_factor),
                image_destination,
                source_rect,
            )
            .ShouldUseRasterDarkMode();
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawImageRect,
                    phase,
                    node_id: fragment.node_id,
                    // BackgroundImageGeometry snaps the no-repeat origin before
                    // snapping the destination size; the phase retains its size.
                    rect: image_destination,
                    source_rect,
                    resource_id: image.id,
                    image_interpolation_quality: paint_style.image_interpolation_quality(),
                    dynamic_range_limit: paint_style.dynamic_range_limit(),
                    image_orientation,
                    use_dark_mode_for_image,
                    document_image_container_key,
                    background_repeat_x: resolved.rule_x,
                    background_repeat_y: resolved.rule_y,
                    blend_mode: layer.blend_mode,
                    tile_scale: bitmap_tile_scale,
                    tile_spacing: resolved.tile_spacing,
                    ..Default::default()
                },
                self.node,
            );
        }
        if opens_attachment_clip {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // cpp: paint/box_painter_base.cc:1745-1775
    // Chromium coordination: box_fragment_painter.cc:1721-1737.
    pub(crate) fn PaintBorder(&self, phase: PaintPhase, bleed_avoidance: BackgroundBleedAvoidance) {
        let paint = &self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment")
            .paint;
        if paint.has_collapsed_borders {
            return;
        }
        if NinePieceImagePainter::Paint(self.node, self.context, phase) {
            return;
        }
        let sides = &paint.border_sides;
        let widths = [
            if sides.top { paint.border.top } else { 0.0 },
            if sides.right { paint.border.right } else { 0.0 },
            if sides.bottom {
                paint.border.bottom
            } else {
                0.0
            },
            if sides.left { paint.border.left } else { 0.0 },
        ];
        let mut has_paintable_border = false;
        for (edge, width) in widths.into_iter().enumerate() {
            has_paintable_border |= width > 0.0
                && paint.border_styles[edge] != BorderLineStyle::kNone
                && paint.style.border_colors()[edge].alpha > 0.0;
        }
        if !has_paintable_border {
            return;
        }
        BoxBorderPainter::new(self.node, self.context, DecorationRect(self.node))
            .PaintBorder(phase, bleed_avoidance);
    }

    fn PaintText(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let phase = paint_info.phase;
        let privacy_preserving = paint_info.IsPrivacyPreserving();
        if !PaintsTextInPhase(phase) {
            return;
        }
        let style = &fragment.paint.style;
        // text_fragment_painter.cc:852-869. The native caret carries the
        // original DOM anchor/OffsetMapping boundary, independently of its
        // painter block or bidi-adjusted cursor. Layout matches ::first-letter
        // through the associated text owner and resolves the text style.
        let block_caret_text = if !fragment.paint.IsPrinting()
            && !paint_info.IsRenderingResourceSubtree()
            && phase != PaintPhase::kTextClip
            && foundation::RuntimeEnabledFeatures::CSSCaretColorWithOptionalSecondValueEnabled()
        {
            let context = self.context.borrow();
            context
                .caret
                .zip(context.TextSource(fragment.fragment_instance_id))
                .and_then(|(caret, cursor)| {
                    let (offset, color) = caret.BlockCaretTextPaint(cursor)?;
                    let local = cursor.TextRangeRect(offset, offset + 1)?;
                    Some((local, color))
                })
        } else {
            None
        };
        let svg_fill_context = crate::svg_context_paints::ResolveFor(style, false, paint_info);
        let svg_stroke_context = crate::svg_context_paints::ResolveFor(style, true, paint_info);
        let svg_fill = svg_fill_context
            .as_ref()
            .map_or(style.svg_fill, |paint| paint.color);
        let svg_stroke = svg_stroke_context
            .as_ref()
            .map_or(style.svg_stroke, |paint| paint.color);
        let svg_fill_server = svg_fill_context
            .as_ref()
            .map_or(style.svg_fill_server.as_ref(), |paint| {
                paint.server.as_ref()
            });
        let svg_stroke_server = svg_stroke_context
            .as_ref()
            .map_or(style.svg_stroke_server.as_ref(), |paint| {
                paint.server.as_ref()
            });
        let selection_style = &fragment.paint.selection_style;
        let text_clip = phase == PaintPhase::kTextClip;
        let force_white_background = ShouldForceWhiteBackgroundForPrintEconomy(fragment);
        let text_fill_color = if force_white_background {
            TextColorForWhiteBackground(style.text_fill_color())
        } else {
            style.text_fill_color()
        };
        let text_stroke_color = if force_white_background {
            TextColorForWhiteBackground(style.text_stroke_color())
        } else {
            style.text_stroke_color()
        };
        let mask_color = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let svg = fragment.paint.svg_text.as_ref();
        if svg.is_some_and(|svg| svg.hidden) {
            return;
        }
        if !self
            .context
            .borrow()
            .HasTextSource(fragment.fragment_instance_id)
            && fragment.paint.list_marker_symbol.is_none()
        {
            return;
        }
        // TextFragmentPainter records this text item, including its shadows,
        // symbol marker and decorations; glyph runs are paint-record ops.
        // text_fragment_painter.cc:429-444: source SelfInkOverflowRect is
        // moved by the physical fragment offset, then enclosed to pixels.
        let in_drawing = self.context.borrow().InDrawingRecorder();
        let drawing_type = DisplayItemIdType::PaintPhaseToDrawingType(phase);
        let visual_rect = (!in_drawing).then(|| {
            let mut visual_rect = FragmentVisualRect(self.node);
            // SkGlyphRunPainter's analytic/LCD coverage may touch the source
            // pixel immediately outside the geometric glyph bounds. Blink's
            // cc raster backing does not clip that coverage at a tight content
            // layer edge. Our retained layers can be content-tight, so carry
            // the same one-device-pixel raster fringe into their bounds.
            let scale = fragment
                .paint
                .resources
                .as_ref()
                .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
                .max(f64::MIN_POSITIVE);
            let fringe = 1.0 / scale;
            visual_rect.x -= fringe;
            visual_rect.y -= fringe;
            visual_rect.width += fringe * 2.0;
            visual_rect.height += fringe * 2.0;
            visual_rect
        });
        let control_selection_state = (!text_clip)
            .then(|| self.context.borrow().text_control_selection)
            .flatten()
            .filter(|state| fragment.paint.text_control_host == Some(state.host_id));
        let control_caret = (!text_clip)
            .then(|| self.context.borrow().caret)
            .flatten()
            .filter(|state| fragment.paint.text_control_host == state.text_control_host);
        let range_rect = |range: (u32, u32), selects_soft_line_break: bool| {
            self.context
                .borrow()
                .TextSource(fragment.fragment_instance_id)
                .and_then(|cursor| {
                    cursor.TextSelectionRangeRect(range.0, range.1, selects_soft_line_break)
                })
                .map(|local| {
                    let text_origin = TextPaintOrigin(self.node);
                    // ComputePhysicalBoxGeometry preserves selection_offset
                    // before applying the physical line-top snap, so adjacent
                    // selection backgrounds remain flush across line boxes.
                    PaintRect {
                        x: text_origin.x + local.x,
                        y: self.node.paint_offset.y + local.y,
                        width: local.width,
                        height: local.height,
                    }
                })
        };
        let control_selection = control_selection_state.and_then(|state| {
            let range = (state.start, state.end);
            let selects_soft_line_break = fragment
                .text_end
                .is_some_and(|fragment_end| range.1 > fragment_end);
            range_rect(range, selects_soft_line_break).map(|rect| (range, rect))
        });
        let document_selection = (!text_clip
            && fragment.paint.text_control_host.is_none()
            && !fragment.paint.style.user_select_none())
        .then(|| self.context.borrow().document_selection)
        .flatten()
        .and_then(|selection| {
            if selection.Collapsed() {
                return None;
            }
            let (start, end) = selection.Ordered();
            let order = fragment.paint.logical_tree_order;
            if order < start.tree_order || order > end.tree_order {
                return None;
            }
            let mut range_start = fragment.text_start?;
            let mut range_end = fragment.text_end?;
            if order == start.tree_order {
                range_start = range_start.max(start.offset);
            }
            if order == end.tree_order {
                range_end = range_end.min(end.offset);
            }
            let selects_soft_line_break =
                order < end.tree_order || order == end.tree_order && end.offset > range_end;
            (range_start < range_end).then_some(((range_start, range_end), selects_soft_line_break))
        })
        .and_then(|(range, selects_soft_line_break)| {
            range_rect(range, selects_soft_line_break).map(|rect| (range, rect))
        });
        let selection = if !text_clip && fragment.paint.list_marker_symbol.is_none() {
            control_selection.or(document_selection)
        } else {
            None
        };
        let selection_range = selection.map(|(range, _)| range);
        let selection_rect = selection.map(|(_, rect)| rect);
        // text_fragment_painter.cc:395-411,465-480. Selection painting needs a
        // non-empty range, but a zero-width fragment may still own one of the
        // document selection endpoints. Keep the bounds candidate independent
        // so its metadata survives cached drawing reuse as in Chromium.
        let selection_for_bounds = (phase == PaintPhase::kForeground
            && !fragment.paint.IsPrinting()
            && !text_clip
            && fragment.paint.list_marker_symbol.is_none()
            && !fragment.paint.is_ellipsis
            && self.context.borrow().selection_handles_visible)
            .then(|| {
                let fragment_start = fragment.text_start?;
                let fragment_end = fragment.text_end?;
                let control = control_selection_state.and_then(|state| {
                    let range = (state.start, state.end);
                    let state = RefineSelectionStateForTextFragment(
                        SelectionState::kStartAndEnd,
                        fragment_start,
                        fragment_end,
                        Some(range.0),
                        Some(range.1),
                    );
                    if state == SelectionState::kNone {
                        return None;
                    }
                    let selected = (fragment_start.max(range.0), fragment_end.min(range.1));
                    let selects_soft_line_break = range.1 > fragment_end;
                    range_rect(selected, selects_soft_line_break).map(|rect| (state, rect))
                });
                let document = (fragment.paint.text_control_host.is_none()
                    && !fragment.paint.style.user_select_none())
                .then(|| self.context.borrow().document_selection)
                .flatten()
                .and_then(|selection| {
                    if selection.Collapsed() {
                        return None;
                    }
                    let (start, end) = selection.Ordered();
                    let order = fragment.paint.logical_tree_order;
                    let state = if order == start.tree_order && order == end.tree_order {
                        SelectionState::kStartAndEnd
                    } else if order == start.tree_order {
                        SelectionState::kStart
                    } else if order == end.tree_order {
                        SelectionState::kEnd
                    } else if order > start.tree_order && order < end.tree_order {
                        SelectionState::kInside
                    } else {
                        SelectionState::kNone
                    };
                    let state = RefineSelectionStateForTextFragment(
                        state,
                        fragment_start,
                        fragment_end,
                        (order == start.tree_order).then_some(start.offset),
                        (order == end.tree_order).then_some(end.offset),
                    );
                    if state == SelectionState::kNone {
                        return None;
                    }
                    let selected = (
                        if order == start.tree_order {
                            fragment_start.max(start.offset)
                        } else {
                            fragment_start
                        },
                        if order == end.tree_order {
                            fragment_end.min(end.offset)
                        } else {
                            fragment_end
                        },
                    );
                    let selects_soft_line_break = order < end.tree_order
                        || order == end.tree_order && end.offset > selected.1;
                    range_rect(selected, selects_soft_line_break).map(|rect| (state, rect))
                });
                control.or(document)
            })
            .flatten();
        // Chromium: box_fragment_painter.cc:2016-2022. A selected <br> has no
        // ink overflow but still owns selection paint; every other text item
        // is culled by its source fragment visual rect.
        let item_visual_rect = FragmentVisualRect(self.node);
        let local_ink_overflow = PaintRect {
            x: item_visual_rect.x - self.node.paint_offset.x,
            y: item_visual_rect.y - self.node.paint_offset.y,
            width: item_visual_rect.width,
            height: item_visual_rect.height,
        };
        if !paint_info.IntersectsCullRect(local_ink_overflow, self.node.paint_offset)
            && !(fragment.paint.is_line_break && selection.is_some())
        {
            return;
        }
        let append_selection_background = |rect: PaintRect| {
            // highlight_painter.cc:63-75,882-900,1006-1028: transparent or
            // empty highlights do not record a draw, and the physical
            // background is pixel-snapped before it reaches GraphicsContext.
            if selection_style.background_color().alpha <= 0.0
                || rect.width <= 0.0
                || rect.height <= 0.0
            {
                return;
            }
            let rect = self.node.PixelSnappedRect(&rect);
            if rect.width <= 0.0 || rect.height <= 0.0 {
                return;
            }
            let color = InvertColorIfNeeded(
                &fragment.paint.style,
                selection_style.background_color(),
                ElementRole::kSelection,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color,
                    ..Default::default()
                },
                self.node,
            );
        };
        // text_fragment_painter.cc:54-62,480-493: when selection painting is
        // present, its complete text drawing uses LayoutText's independent
        // selection DisplayItemClient for cache identity and invalidation.
        let selection_client = selection_rect.and_then(|_| {
            crate::paint_chunker::DisplayItemClientInfo::from_selection(fragment).map(|client| {
                (
                    DisplayItemId {
                        client_id: fragment.paint.selection_client_id,
                        r#type: drawing_type,
                        fragment: fragment.paint.display_item_fragment,
                    },
                    client,
                )
            })
        });
        let _selection_bounds_recorder = selection_for_bounds.map(|(state, rect)| {
            SelectionBoundsRecorder::new(
                state,
                self.node.PixelSnappedRect(&rect),
                self.context,
                fragment.paint.direction,
                fragment.paint.writing_mode,
            )
        });
        let cached = visual_rect.is_some_and(|_| {
            if let Some((id, client)) = selection_client {
                self.context.borrow_mut().UseCachedDrawingWithClient(
                    id,
                    client,
                    Some((
                        fragment.paint.display_item_client_owner_node_id,
                        Arc::<str>::from(format!(
                            "{} selection",
                            fragment.paint.display_item_client_debug_name
                        )),
                    )),
                )
            } else {
                DrawingRecorder::UseCachedDrawingIfPossible(self.context, self.node, drawing_type)
            }
        });
        if cached {
            return;
        }
        let glyph_runs = self
            .context
            .borrow_mut()
            .TextGlyphRuns(fragment.fragment_instance_id);
        let _drawing = visual_rect.map(|visual_rect| {
            if let Some((id, client)) = selection_client {
                DrawingRecorder::with_client(self.context, self.node, id, client, visual_rect)
            } else {
                DrawingRecorder::new(self.context, self.node, drawing_type, visual_rect)
            }
        });
        // A selected forced line break has no glyph run, but Chromium gives its
        // background the scaled font's space width (inline_cursor.cc:529-537).
        // Do not manufacture glyphs or decorations for it; retain the real
        // selection background before leaving the text-proper path.
        if glyph_runs.is_empty() && fragment.paint.list_marker_symbol.is_none() {
            if let Some(rect) = selection_rect {
                append_selection_background(rect);
            }
            return;
        }
        let apply_svg_transform = svg.is_some_and(|svg| svg.has_transform);
        if apply_svg_transform {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            let mut concat = DisplayItem {
                r#type: DisplayItemType::kConcat,
                phase,
                node_id: fragment.node_id,
                ..Default::default()
            };
            concat.transform = RootSpaceTextTransform(self.node);
            self.context.borrow_mut().Append(concat, self.node);
        }
        if let Some(symbol) = fragment.paint.list_marker_symbol {
            if glyph_runs.is_empty() {
                panic!("symbol list marker has no shaped run");
            }
            let ascent = glyph_runs[0].baseline as i32;
            let two_thirds_ascent = ascent * 2 / 3;
            let bullet_width = (two_thirds_ascent + 1) / 2;
            let marker_top = 3 * (ascent - two_thirds_ascent) / 2;
            let unsnapped_x = self.node.paint_offset.x
                + if fragment.paint.list_marker_inside {
                    0.0
                } else {
                    1.0
                };
            let unsnapped_y = self.node.paint_offset.y + marker_top as f64;
            let marker_origin = self.node.PixelSnappedOffset(Offset {
                x: unsnapped_x,
                y: unsnapped_y,
            });
            let marker_rect = PaintRect {
                x: marker_origin.x,
                y: marker_origin.y,
                width: bullet_width as f64,
                height: bullet_width as f64,
            };
            let mut marker = DisplayItem {
                r#type: DisplayItemType::kDrawEllipse,
                phase,
                node_id: fragment.node_id,
                rect: marker_rect,
                color: if text_clip {
                    mask_color
                } else {
                    text_fill_color
                },
                ..Default::default()
            };
            if symbol == ListStyleType::kCircle {
                marker.r#type = DisplayItemType::kStrokeEllipse;
                marker.stroke_width = 1.0;
            } else if symbol == ListStyleType::kSquare {
                marker.r#type = DisplayItemType::kDrawRect;
            }
            if IsVisible(self.node, &marker_rect) {
                self.context.borrow_mut().Append(marker, self.node);
            }
            if apply_svg_transform {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        let apply_text_fit = svg.is_none() && fragment.paint.text_fit_scale != 1.0;
        let mut text_fit_origin = TextPaintOrigin(self.node);
        if let Some(run) = glyph_runs.first() {
            // text_fragment_painter.cc:531-542 constructs text_origin at the
            // scaled primary-font ascent, i.e. the same baseline retained by
            // the glyph run, before ApplyTextFitScale uses it as the fixed point.
            text_fit_origin.y += run.baseline;
        }
        if apply_text_fit {
            AppendTextFitTransform(
                self.context,
                self.node,
                phase,
                fragment.paint.text_fit_scale,
                text_fit_origin,
                true,
            );
        }
        let mut glyph_foregrounds = Vec::new();
        let mut block_caret_glyphs = Vec::new();
        let mut selection_glyph_shadows = Vec::new();
        let mut selection_glyph_foregrounds = Vec::new();
        for input_run in glyph_runs.iter() {
            let make_item = |color: Color,
                             shadow_offset: Offset,
                             blur_radius: f64,
                             is_shadow: bool,
                             stroke_glyphs: bool| {
                let color = InvertColorIfNeeded(
                    style,
                    color,
                    if fragment.paint.list_marker_symbol.is_some() {
                        ElementRole::kListSymbol
                    } else {
                        ElementRole::kForeground
                    },
                );
                let mut origin = TextPaintOrigin(self.node);
                let mut physical_box_offset = self.node.paint_offset;
                let mut text_line_top = 0.0;
                if let Some(text_line_top_offset) = fragment.paint.text_line_top_offset {
                    text_line_top = self.node.paint_offset.y + text_line_top_offset;
                    // ComputePhysicalBoxGeometry rounds the physical line top,
                    // not the fragment-local export inputs. Recompute the
                    // delta after paint-tree offsets are accumulated so a
                    // half-pixel line top produces Chromium's integer text
                    // baseline (for example 23.5 + 0.5 + 15 = 40).
                    let snap_delta = (text_line_top + 0.5).floor() - text_line_top;
                    origin.y += snap_delta;
                    physical_box_offset.y += snap_delta;
                }
                let rect = PaintRect {
                    x: self.node.paint_offset.x
                        + fragment.paint.text_combine_inline_offset
                        + shadow_offset.x,
                    y: self.node.paint_offset.y + shadow_offset.y,
                    width: fragment.size.width,
                    height: fragment.size.height,
                };
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kDrawGlyphRun,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color,
                    shadow_offset,
                    blur_radius,
                    is_shadow,
                    stroke_glyphs,
                    font_face_index: input_run.font_face_index,
                    font_variations: input_run.font_variations.clone(),
                    font_size: input_run.font_size,
                    baseline: input_run.baseline,
                    text_line_top,
                    text_line_height: fragment.paint.text_line_height.unwrap_or(0.0),
                    text_blob_origin: Offset {
                        x: origin.x + shadow_offset.x,
                        y: origin.y + input_run.baseline + shadow_offset.y,
                    },
                    horizontal: input_run.horizontal,
                    rtl: input_run.rtl,
                    synthetic_bold: input_run.synthetic_bold,
                    synthetic_italic: input_run.synthetic_italic,
                    font_smoothing: input_run.font_smoothing,
                    writing_mode: input_run.writing_mode,
                    glyphs: Arc::clone(&input_run.glyphs),
                    ..Default::default()
                };
                if std::env::var_os("RECHROM_TRACE_FONT_BINDING").is_some()
                    && !is_shadow
                    && !stroke_glyphs
                    && item.text_blob_origin.y < 60.0
                    && (item.font_size - 14.0).abs() < 0.01
                {
                    eprintln!(
                        "FONT_PAINT_ORIGIN node={} paint=({:.4},{:.4}) blob=({:.4},{:.4}) baseline={:.4} line_top={:.4} line_height={:.4} smoothing={:?} variations={:?} glyphs={:?}",
                        fragment.node_id,
                        self.node.paint_offset.x,
                        self.node.paint_offset.y,
                        item.text_blob_origin.x,
                        item.text_blob_origin.y,
                        item.baseline,
                        item.text_line_top,
                        item.text_line_height,
                        item.font_smoothing,
                        item.font_variations,
                        item.glyphs
                            .iter()
                            .map(|glyph| (glyph.id, glyph.offset.x, glyph.offset.y))
                            .collect::<Vec<_>>(),
                    );
                }
                if stroke_glyphs {
                    if svg.is_some() {
                        item.stroke_width = style.svg_stroke_width;
                        item.dash_intervals = style.svg_stroke_dash_array.clone();
                        item.dash_offset = style.svg_stroke_dash_offset;
                        item.svg_line_cap = style.svg_stroke_line_cap;
                        item.svg_line_join = style.svg_stroke_line_join;
                        item.miter_limit = style.svg_stroke_miter_limit;
                    } else {
                        item.stroke_width = style.text_stroke_width();
                    }
                }
                if input_run.writing_mode != WritingMode::kHorizontalTb {
                    item.transform = TextWritingModeTransform(
                        self.node,
                        input_run.writing_mode,
                        physical_box_offset,
                    );
                }
                // cc::PaintOp::GetBounds(DrawTextBlobOp) consumes blob ink,
                // independently of the containing decoration record's bound.
                // Layout supplied real shaped glyph ink even when decoration
                // overflow still requires the retained-layout bridge into the
                // Paint-owned InlinePaintContext. Keep that proof only for the
                // ordinary untransformed fill path.
                if svg.is_none()
                    && (fragment.paint.text_glyph_ink.is_some()
                        || fragment.paint.self_ink_overflow.is_some())
                    && fragment.paint.text_combine.is_none()
                    && fragment.paint.text_combine_inline_offset == 0.0
                    && input_run.writing_mode == WritingMode::kHorizontalTb
                    && !is_shadow
                    && !stroke_glyphs
                {
                    item.glyph_ink_rect = Some(FragmentVisualRect(self.node));
                }
                item
            };
            if let Some((_, color)) = block_caret_text {
                // kTextProperOnly: no shadows or decorations, and the same
                // glyph/ShapeResult owners as ordinary text. A cached whole
                // fragment blob is legal under the single-cell clip below.
                for component in style.svg_paint_order {
                    if component == SvgPaintComponent::kFill {
                        block_caret_glyphs.push(make_item(
                            color,
                            Offset::default(),
                            0.0,
                            false,
                            false,
                        ));
                    } else if component == SvgPaintComponent::kStroke
                        && style.text_stroke_width() > 0.0
                    {
                        block_caret_glyphs.push(make_item(
                            color,
                            Offset::default(),
                            0.0,
                            false,
                            true,
                        ));
                    }
                }
            }
            for index in (0..style.text_shadow_count()).rev() {
                let shadow = style.text_shadow(index);
                if text_clip || shadow.color.alpha <= 0.0 {
                    continue;
                }
                if shadow.inset
                    || shadow.spread != 0.0
                    || !shadow.offset.x.is_finite()
                    || !shadow.offset.y.is_finite()
                    || !shadow.blur_radius.is_finite()
                    || shadow.blur_radius < 0.0
                {
                    panic!("text shadow geometry is invalid");
                }
                let shadow_item =
                    make_item(shadow.color, shadow.offset, shadow.blur_radius, true, false);
                let mut bounds = shadow_item.rect;
                let blur_outset = 1.5 * shadow.blur_radius;
                bounds.x -= blur_outset;
                bounds.y -= blur_outset;
                bounds.width += 2.0 * blur_outset;
                bounds.height += 2.0 * blur_outset;
                let cull_bounds = if svg.is_some() {
                    FragmentVisualRect(self.node)
                } else {
                    bounds
                };
                if IsVisible(self.node, &cull_bounds) {
                    self.context.borrow_mut().Append(shadow_item, self.node);
                }
            }
            let rect = PaintRect {
                x: self.node.paint_offset.x + fragment.paint.text_combine_inline_offset,
                y: self.node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
            // SVG glyphs are positioned by a record-local textPath transform.
            // Their unrotated baseline box is not a visibility bound.
            let cull_bounds = if svg.is_some() {
                FragmentVisualRect(self.node)
            } else {
                rect
            };
            if IsVisible(self.node, &cull_bounds) {
                if text_clip {
                    // text_painter.cc:683-694: kTextClip forces every paint
                    // color opaque black, drops shadows and resets paint-order
                    // to normal while retaining the used stroke width.
                    glyph_foregrounds.push(make_item(
                        mask_color,
                        Offset::default(),
                        0.0,
                        false,
                        false,
                    ));
                    let stroke_width = if svg.is_some() {
                        style.svg_stroke_width
                    } else {
                        style.text_stroke_width()
                    };
                    if stroke_width > 0.0 {
                        glyph_foregrounds.push(make_item(
                            mask_color,
                            Offset::default(),
                            0.0,
                            false,
                            true,
                        ));
                    }
                } else if svg.is_none() {
                    // GraphicsContext's text paint order follows the same
                    // ComputedStyle paint-order used by SVG text. The normal
                    // value is fill then stroke; do not reverse it merely
                    // because the two modes are separate display items here.
                    for component in style.svg_paint_order {
                        if component == SvgPaintComponent::kFill && text_fill_color.alpha > 0.0 {
                            glyph_foregrounds.push(make_item(
                                text_fill_color,
                                Offset::default(),
                                0.0,
                                false,
                                false,
                            ));
                        } else if component == SvgPaintComponent::kStroke
                            && style.text_stroke_width() > 0.0
                            && text_stroke_color.alpha > 0.0
                        {
                            glyph_foregrounds.push(make_item(
                                text_stroke_color,
                                Offset::default(),
                                0.0,
                                false,
                                true,
                            ));
                        }
                    }
                } else {
                    for component in style.svg_paint_order {
                        if component == SvgPaintComponent::kFill
                            && (svg_fill_server.is_some_and(|server| {
                                !privacy_preserving || server.is_in_canvas_subtree
                            }) || svg_fill.is_some_and(|fill| fill.alpha > 0.0))
                        {
                            let mut item = make_item(
                                svg_fill.unwrap_or_default(),
                                Offset::default(),
                                0.0,
                                false,
                                false,
                            );
                            if let Some(server) = svg_fill_server
                                .filter(|server| !privacy_preserving || server.is_in_canvas_subtree)
                            {
                                item.color = svg_fill.unwrap_or_default();
                                let shader = if let Some(context) = &svg_fill_context {
                                    crate::svg_context_paints::ResolveShader(
                                        context,
                                        fragment.paint.resources.as_deref(),
                                        privacy_preserving,
                                    )
                                    .expect("admitted context paint server")
                                } else {
                                    ResolvePaintShader(
                                        server,
                                        fragment.paint.resources.as_deref(),
                                        Offset {
                                            x: item.rect.x,
                                            y: item.rect.y,
                                        },
                                        Some(Size {
                                            width: item.rect.width,
                                            height: item.rect.height,
                                        }),
                                    )
                                };
                                let mut shader =
                                    PrivacyPreservingPaintServer(&shader, privacy_preserving)
                                        .expect("admitted SVG fill paint server");
                                if style.auto_dark_mode_paused() {
                                    shader.apply_dark_mode = false;
                                }
                                item.paint_shader = Some(shader);
                            }
                            glyph_foregrounds.push(item);
                        } else if component == SvgPaintComponent::kStroke
                            && (svg_stroke_server.is_some_and(|server| {
                                !privacy_preserving || server.is_in_canvas_subtree
                            }) || svg_stroke.is_some_and(|stroke| stroke.alpha > 0.0))
                            && style.svg_stroke_width > 0.0
                        {
                            let mut item = make_item(
                                svg_stroke.unwrap_or_default(),
                                Offset::default(),
                                0.0,
                                false,
                                true,
                            );
                            if let Some(server) = svg_stroke_server
                                .filter(|server| !privacy_preserving || server.is_in_canvas_subtree)
                            {
                                item.color = svg_stroke.unwrap_or_default();
                                let shader = if let Some(context) = &svg_stroke_context {
                                    crate::svg_context_paints::ResolveShader(
                                        context,
                                        fragment.paint.resources.as_deref(),
                                        privacy_preserving,
                                    )
                                    .expect("admitted context paint server")
                                } else {
                                    ResolvePaintShader(
                                        server,
                                        fragment.paint.resources.as_deref(),
                                        Offset {
                                            x: item.rect.x,
                                            y: item.rect.y,
                                        },
                                        Some(Size {
                                            width: item.rect.width,
                                            height: item.rect.height,
                                        }),
                                    )
                                };
                                let mut shader =
                                    PrivacyPreservingPaintServer(&shader, privacy_preserving)
                                        .expect("admitted SVG stroke paint server");
                                if style.auto_dark_mode_paused() {
                                    shader.apply_dark_mode = false;
                                }
                                item.paint_shader = Some(shader);
                            }
                            glyph_foregrounds.push(item);
                        }
                    }
                }
            }
            if let Some(selection_range) = selection_range {
                let selected_glyphs = SliceGlyphsForSelection(
                    &input_run.glyphs,
                    selection_range,
                    fragment.text_end.unwrap_or(selection_range.1),
                );
                if selected_glyphs.is_empty() {
                    continue;
                }
                for index in (0..selection_style.text_shadow_count()).rev() {
                    let shadow = selection_style.text_shadow(index);
                    if shadow.color.alpha <= 0.0 {
                        continue;
                    }
                    if shadow.inset
                        || shadow.spread != 0.0
                        || !shadow.offset.x.is_finite()
                        || !shadow.offset.y.is_finite()
                        || !shadow.blur_radius.is_finite()
                        || shadow.blur_radius < 0.0
                    {
                        panic!("selection text shadow geometry is invalid");
                    }
                    let mut item =
                        make_item(shadow.color, shadow.offset, shadow.blur_radius, true, false);
                    item.glyphs = Arc::clone(&selected_glyphs);
                    selection_glyph_shadows.push(item);
                }
                for component in selection_style.svg_paint_order {
                    if component == SvgPaintComponent::kFill
                        && selection_style.text_fill_color().alpha > 0.0
                    {
                        let mut item = make_item(
                            selection_style.text_fill_color(),
                            Offset::default(),
                            0.0,
                            false,
                            false,
                        );
                        item.glyphs = Arc::clone(&selected_glyphs);
                        selection_glyph_foregrounds.push(item);
                    } else if component == SvgPaintComponent::kStroke
                        && selection_style.text_stroke_width() > 0.0
                        && selection_style.text_stroke_color().alpha > 0.0
                    {
                        let mut item = make_item(
                            selection_style.text_stroke_color(),
                            Offset::default(),
                            0.0,
                            false,
                            true,
                        );
                        item.stroke_width = selection_style.text_stroke_width();
                        item.glyphs = Arc::clone(&selected_glyphs);
                        selection_glyph_foregrounds.push(item);
                    }
                }
            }
        }
        if apply_text_fit {
            AppendTextFitTransform(
                self.context,
                self.node,
                phase,
                fragment.paint.text_fit_scale,
                text_fit_origin,
                false,
            );
        }
        if let Some(rect) = selection_rect {
            // HighlightPainter::PaintHighlightOverlays paints the originating
            // shadow first, then each highlight layer's background and shadow,
            // before any under/over decoration or text proper.
            append_selection_background(rect);
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    true,
                );
            }
            if !selection_glyph_shadows.is_empty() {
                let shadow_clip_rect = AdjustRectForShadow(rect, selection_style);
                let mut context = self.context.borrow_mut();
                context.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                context.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: shadow_clip_rect,
                        ..Default::default()
                    },
                    self.node,
                );
                for item in selection_glyph_shadows {
                    context.Append(item, self.node);
                }
                context.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    false,
                );
            }
        }
        let mut decoration_painter = TextDecorationPainter::new(
            self.node,
            self.context,
            phase,
            selection_rect,
            privacy_preserving,
        );
        decoration_painter.SetSvgContextPaints(paint_info.GetSvgContextPaints());
        decoration_painter.Begin(TextDecorationPhase::Originating);
        decoration_painter.PaintExceptLineThrough();
        if apply_text_fit {
            AppendTextFitTransform(
                self.context,
                self.node,
                phase,
                fragment.paint.text_fit_scale,
                text_fit_origin,
                true,
            );
        }
        for item in &glyph_foregrounds {
            self.context.borrow_mut().Append(item.clone(), self.node);
        }
        if fragment.text_paint_timing_non_whitespace {
            self.context.borrow_mut().SetTextPainted();
        }
        if self
            .node
            .SourceCursor()
            .is_some_and(|source| !source.TextPaintShouldSkipDrawing())
        {
            self.context
                .borrow_mut()
                .NotifyTextPaint(FragmentVisualRect(self.node));
        }

        // Repaint selected glyphs under the selection clip, preserving ligatures
        // and fallback runs rather than splitting glyph ids at character offsets.
        if let Some(rect) = selection_rect {
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    false,
                );
            }
            let mut selection_decoration_painter = TextDecorationPainter::new(
                self.node,
                self.context,
                phase,
                Some(rect),
                privacy_preserving,
            );
            selection_decoration_painter.SetSvgContextPaints(paint_info.GetSvgContextPaints());
            selection_decoration_painter.Begin(TextDecorationPhase::Selection);
            selection_decoration_painter.PaintExceptLineThrough();
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    true,
                );
            }
            if svg.is_some() {
                // highlight_painter.cc:961-972 accepts possible partial-
                // ligature overpaint for SVG because text transforms and
                // textPath geometry can invalidate a canvas-space clip.
                for item in selection_glyph_foregrounds {
                    self.context.borrow_mut().Append(item, self.node);
                }
            } else {
                // highlight_painter.cc:973-986 inflates the text-proper clip by
                // ceil(stroke_width / 2). Decoration and background clips keep
                // the uninflated selection range.
                let source = self
                    .context
                    .borrow()
                    .TextSource(fragment.fragment_instance_id);
                let item = source.and_then(|cursor| cursor.Cursor().InlineItem());
                let mut physical_box_offset = self.node.paint_offset;
                if fragment.paint.text_line_top_offset.is_some() {
                    physical_box_offset.y += fragment.paint.text_line_snap_delta.unwrap_or(0.0);
                }
                let highlight_rect = source
                    .and_then(|cursor| {
                        let range = selection_range.expect("selection rect has a logical range");
                        cursor.TextHighlightRangeRect(range.0, range.1)
                    })
                    .map(|local| PaintRect {
                        x: physical_box_offset.x + local.x,
                        y: physical_box_offset.y + local.y,
                        width: local.width,
                        height: local.height,
                    });
                let glyph_clip_rect = SelectionGlyphClipRect(
                    rect,
                    highlight_rect,
                    selection_style.text_stroke_width(),
                    selection_range.expect("selection rect has a logical range"),
                    fragment.text_start.zip(fragment.text_end),
                    fragment.paint.ink_overflow,
                    item,
                    physical_box_offset,
                    fragment.size,
                );
                {
                    let mut context = self.context.borrow_mut();
                    context.Append(
                        DisplayItem {
                            r#type: DisplayItemType::kSave,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
                    context.Append(
                        DisplayItem {
                            r#type: DisplayItemType::kClipRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: glyph_clip_rect,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                for item in selection_glyph_foregrounds {
                    self.context.borrow_mut().Append(item, self.node);
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    false,
                );
            }
            selection_decoration_painter.PaintOnlyLineThrough();
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    true,
                );
            }
        }
        if apply_text_fit {
            AppendTextFitTransform(
                self.context,
                self.node,
                phase,
                fragment.paint.text_fit_scale,
                text_fit_origin,
                false,
            );
        }
        let composition_ranges: Vec<_> = if fragment.paint.text_control_host.is_some() {
            control_caret
                .and_then(|state| state.composition)
                .into_iter()
                .collect()
        } else if !text_clip {
            self.context
                .borrow()
                .document_composition_markers
                .iter()
                .filter(|marker| marker.node_id == fragment.node_id)
                .map(|marker| (marker.start, marker.end))
                .collect()
        } else {
            Vec::new()
        };
        for mut rect in composition_ranges
            .into_iter()
            .filter_map(|range| range_rect(range, false))
        {
            rect.y += rect.height - 1.0;
            rect.height = 1.0;
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: style.color(),
                    ..Default::default()
                },
                self.node,
            );
        }
        decoration_painter.PaintOnlyLineThrough();
        // text_fragment_painter.cc:870-895. Our glyph ops retain their own
        // writing-mode transform, so clip the physical character cell before
        // those ops apply it (Blink's rotated context uses LineRelativeRect).
        if let Some((local, _)) = block_caret_text {
            let mut physical_offset = self.node.paint_offset;
            if let Some(line_top) = fragment.paint.text_line_top_offset {
                let top = physical_offset.y + line_top;
                physical_offset.y += (top + 0.5).floor() - top;
            }
            let mut context = self.context.borrow_mut();
            context.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            context.Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: PaintRect {
                        x: physical_offset.x + local.x,
                        y: physical_offset.y + local.y,
                        width: local.width,
                        height: local.height,
                    },
                    ..Default::default()
                },
                self.node,
            );
            drop(context);
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    true,
                );
            }
            for item in block_caret_glyphs {
                self.context.borrow_mut().Append(item, self.node);
            }
            if apply_text_fit {
                AppendTextFitTransform(
                    self.context,
                    self.node,
                    phase,
                    fragment.paint.text_fit_scale,
                    text_fit_origin,
                    false,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
        if apply_svg_transform {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // cpp: core/paint/text_combine_painter.cc:116-160
    fn PaintTextCombineEmphasis(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(emphasis) = fragment
            .paint
            .text_combine
            .as_ref()
            .and_then(|combine| combine.emphasis.as_ref())
        else {
            return;
        };
        let color = if phase == PaintPhase::kTextClip {
            Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            }
        } else if ShouldForceWhiteBackgroundForPrintEconomy(fragment) {
            TextColorForWhiteBackground(emphasis.color)
        } else {
            emphasis.color
        };
        let color = InvertColorIfNeeded(&fragment.paint.style, color, ElementRole::kForeground);
        let origin = Offset {
            x: self.node.paint_offset.x + emphasis.origin.x,
            y: self.node.paint_offset.y + emphasis.origin.y,
        };
        let rect = PaintRect {
            x: self.node.paint_offset.x,
            y: self.node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        for run in &emphasis.glyph_runs {
            let mut item = DisplayItem {
                r#type: DisplayItemType::kDrawGlyphRun,
                phase,
                node_id: fragment.node_id,
                rect,
                color,
                font_face_index: run.font_face_index,
                font_variations: run.font_variations.clone(),
                font_size: run.font_size,
                text_blob_origin: origin,
                horizontal: run.horizontal,
                rtl: run.rtl,
                synthetic_bold: run.synthetic_bold,
                synthetic_italic: run.synthetic_italic,
                font_smoothing: run.font_smoothing,
                writing_mode: run.writing_mode,
                glyphs: Arc::clone(&run.glyphs),
                ..Default::default()
            };
            if fragment.paint.writing_mode != WritingMode::kHorizontalTb {
                item.transform = TextWritingModeTransform(
                    self.node,
                    fragment.paint.writing_mode,
                    self.node.paint_offset,
                );
            }
            self.context.borrow_mut().Append(item, self.node);
        }
    }

    // Source: core/paint/image_painter.cc:66-115 and
    // core/html/html_area_element.cc:111-191.
    fn PaintAreaElementFocusRing(&self) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.IsPrinting() {
            return;
        }
        let Some(area) = fragment.paint.focused_image_map_area.as_ref() else {
            return;
        };
        if !area.has_outline || !area.outline_style_is_auto {
            return;
        }
        let path = ImageMapAreaPath(
            area,
            PaintRect {
                x: self.node.paint_offset.x,
                y: self.node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            },
            fragment.paint.effective_zoom as f64,
        );
        if path.is_empty() {
            return;
        }
        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kImageAreaFocusRing,
        ) {
            return;
        }
        let content_box = self.node.PixelSnappedRect(&PaintRect {
            x: self.node.paint_offset.x + fragment.paint.border.left + fragment.paint.padding.left,
            y: self.node.paint_offset.y + fragment.paint.border.top + fragment.paint.padding.top,
            width: fragment.content_size.width,
            height: fragment.content_size.height,
        });
        if content_box.width <= 0.0
            || content_box.height <= 0.0
            || !IsVisible(self.node, &content_box)
        {
            return;
        }
        // BoxDrawingRecorder uses the LayoutImage's source visual overflow.
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kImageAreaFocusRing,
            DecorationVisualRect(self.node),
        );
        let phase = PaintPhase::kOutline;
        let node_id = fragment.node_id;
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase,
                node_id,
                ..Default::default()
            },
            self.node,
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase,
                node_id,
                rect: content_box,
                antialias: false,
                ..Default::default()
            },
            self.node,
        );
        let stroke_width = FocusRingStrokeWidth(area.effective_zoom);
        let color = InvertColorIfNeeded(
            &fragment.paint.style,
            area.outline_color,
            ElementRole::kBackground,
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kStrokePath,
                phase,
                node_id,
                rect: content_box,
                path,
                even_odd: area.shape == ImageMapAreaShape::kPoly,
                color,
                stroke_width,
                corner_radius: stroke_width,
                antialias: true,
                ..Default::default()
            },
            self.node,
        );
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase,
                node_id,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: core/paint/replaced_painter.cc:213-269
    fn PaintReplacedSelection(&self, paint_info: &PaintInfo<'_, '_>) {
        if paint_info.phase != PaintPhase::kForeground {
            return;
        }
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.IsPrinting()
            || fragment.paint.replaced_selection_state == SelectionState::kNone
            || !fragment.paint.pre_paint_invalidation.can_be_selection_leaf
        {
            return;
        }
        let Some(local_rect) = fragment.paint.replaced_local_selection_visual_rect else {
            return;
        };
        let selection_rect = PaintRect {
            x: self.node.paint_offset.x + local_rect.offset.x,
            y: self.node.paint_offset.y + local_rect.offset.y,
            width: local_rect.size.width,
            height: local_rect.size.height,
        };
        if selection_rect.is_empty() {
            return;
        }

        // Declare this before the DrawingRecorder so cached and freshly
        // recorded selection tints both publish handle metadata after the
        // selected drawing closes.
        let _selection_bounds = self.context.borrow().selection_handles_visible.then(|| {
            SelectionBoundsRecorder::new(
                fragment.paint.replaced_selection_state,
                selection_rect,
                self.context,
                fragment.paint.direction,
                fragment.paint.writing_mode,
            )
        });
        if DrawingRecorder::UseCachedDrawingIfPossible(
            self.context,
            self.node,
            DisplayItemIdType::kSelectionTint,
        ) {
            return;
        }
        let selection_rect = self.node.PixelSnappedRect(&selection_rect);
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::kSelectionTint,
            selection_rect,
        );
        let color = InvertColorIfNeeded(
            &fragment.paint.style,
            fragment.paint.selection_style.background_color(),
            ElementRole::kBackground,
        );
        if color.alpha <= 0.0 || selection_rect.is_empty() {
            return;
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawRect,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                rect: selection_rect,
                color,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: core/paint/image_painter.cc:117-120
    fn PaintReplaced(&self, paint_info: &PaintInfo<'_, '_>) {
        let phase = paint_info.phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        // cpp: core/layout/layout_image.cc:293-299
        if fragment.paint.ChildPaintBlockedByDisplayLock() {
            return;
        }
        let style = &fragment.paint;
        let content_box = PaintRect {
            x: self.node.paint_offset.x + style.border.left + style.padding.left,
            y: self.node.paint_offset.y + style.border.top + style.padding.top,
            width: fragment.content_size.width,
            height: fragment.content_size.height,
        };
        if content_box.width <= 0.0 || content_box.height <= 0.0 {
            return;
        }
        // Source: core/paint/image_painter.cc:138-145. LayoutReplaced owns
        // object-fit/object-position resolution. Paint consumes its retained
        // ReplacedContentRect and never recomputes that geometry from style or
        // resource dimensions.
        let replaced_content = style
            .replaced_content
            .as_ref()
            .expect("replaced paint fragment has source replaced-content geometry");
        let paint_rect = PaintRect {
            x: self.node.paint_offset.x + replaced_content.offset.x,
            y: self.node.paint_offset.y + replaced_content.offset.y,
            width: replaced_content.size.width,
            height: replaced_content.size.height,
        };
        let clips_to_content_box = replaced_content.clips_to_content_box;
        let base_visual_rect = if clips_to_content_box {
            content_box
        } else {
            paint_rect
        };
        if let Some(shader) = style.style.replaced_image_shader.as_ref() {
            let destination = self.node.PixelSnappedRect(&paint_rect);
            let visual_rect = if clips_to_content_box {
                destination.intersection(self.node.PixelSnappedRect(&content_box))
            } else {
                destination
            };
            let painted_rect_changed = UpdateImagePaintedRect(self.node, base_visual_rect);
            if destination.width <= 0.0
                || destination.height <= 0.0
                || visual_rect.width <= 0.0
                || visual_rect.height <= 0.0
                || !IsVisible(self.node, &visual_rect)
            {
                return;
            }
            if UseCachedImageDrawingIfPossible(self.context, self.node, phase, painted_rect_changed)
            {
                return;
            }
            let _drawing = DrawingRecorder::new(
                self.context,
                self.node,
                DisplayItemIdType::PaintPhaseToDrawingType(phase),
                self.node.EnclosingRect(base_visual_rect),
            );
            if visual_rect != destination {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: visual_rect,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawGradientRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: destination,
                    tile_rect: destination,
                    paint_shader: Some(ResolvePaintShader(
                        shader,
                        fragment.paint.resources.as_deref(),
                        Offset {
                            x: destination.x,
                            y: destination.y,
                        },
                        Some(Size {
                            width: destination.width,
                            height: destination.height,
                        }),
                    )),
                    ..Default::default()
                },
                self.node,
            );
            if visual_rect != destination {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        // Source: image_painter.cc:121-136. HasImage() is a paint-time
        // resource query, not merely the presence of a retained resource id.
        // A failed, evicted or malformed frame therefore takes Chromium's
        // missing-image placeholder branch as well.
        let Some((id, image)) = style
            .style
            .image_resource_id
            .and_then(|id| FindAvailableReplacedImage(self.node, id).map(|image| (id, image)))
        else {
            if !fragment.paint.replaced_is_image {
                return;
            }
            // cpp: core/paint/image_painter.cc:130-132. A selection drag
            // snapshot must not manufacture the missing-image placeholder;
            // the dedicated paint phase is already preserved by paint_walk.
            if phase == PaintPhase::kSelectionDragImage {
                return;
            }
            // Chromium ImagePainter paints a one-pixel placeholder when a
            // LayoutImage has no decoded image. Layout exports the source
            // LayoutImage identity because NodeKind::kReplaced also includes
            // canvas/video/embedded content.
            // cpp: core/paint/image_painter.cc:121-136,184-195
            if let Some(outline_rect) = MissingImagePlaceholderRect(
                fragment.paint.replaced_is_image,
                content_box,
                self.node.PixelSnappedRect(&content_box),
            ) {
                let painted_rect_changed = UpdateImagePaintedRect(self.node, base_visual_rect);
                if IsVisible(self.node, &outline_rect) {
                    if UseCachedImageDrawingIfPossible(
                        self.context,
                        self.node,
                        phase,
                        painted_rect_changed,
                    ) {
                        return;
                    }
                    let _drawing = DrawingRecorder::new(
                        self.context,
                        self.node,
                        DisplayItemIdType::PaintPhaseToDrawingType(phase),
                        self.node.EnclosingRect(base_visual_rect),
                    );
                    let color = InvertColorIfNeeded(
                        &fragment.paint.style,
                        Color {
                            red: 211.0 / 255.0,
                            green: 211.0 / 255.0,
                            blue: 211.0 / 255.0,
                            alpha: 1.0,
                        },
                        ElementRole::kBorder,
                    );
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kStrokeRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: outline_rect,
                            color,
                            line_style: BorderLineStyle::kSolid,
                            stroke_width: 1.0,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
            }
            return;
        };
        // Source: image_painter.cc:126-129. Privacy-preserving subtree paint
        // must not expose pixels from a tainted cross-origin response.
        if paint_info.IsPrivacyPreserving() && !image.is_cors_same_origin {
            return;
        }
        // cpp: core/paint/image_painter.cc:203-218. The resource snapshot is
        // immutable for this paint pass, so the HasImage/GetImage checks above
        // share one validated value rather than fabricating a second lookup.
        let image_orientation = if style.style.respect_image_orientation() {
            image.orientation
        } else {
            image_resource::ImageOrientation::kDefault
        };
        let (source_width, source_height) =
            image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
        let mut destination = paint_rect;
        let mut source = PaintRect {
            x: 0.0,
            y: 0.0,
            width: source_width,
            height: source_height,
        };
        if destination.width <= 0.0 || destination.height <= 0.0 {
            return;
        }
        let document_image_container_key = ResolveDocumentImageContainerKey(
            image,
            Size {
                width: destination.width,
                height: destination.height,
            },
            f64::from(style.effective_zoom),
            fragment.paint.resources.as_deref(),
            (!style.style.color_scheme_flags_is_normal())
                .then_some(u8::from(style.style.used_dark_color_scheme())),
        );
        let mut visual_rect = base_visual_rect;
        visual_rect = SvgSpriteVisualRect(image, visual_rect, self.node.self_cull_rect);
        let painted_rect_changed = UpdateImagePaintedRect(self.node, visual_rect);
        if visual_rect.width <= 0.0
            || visual_rect.height <= 0.0
            || !IsVisible(self.node, &visual_rect)
        {
            return;
        }
        destination = self.node.PixelSnappedRect(&destination);
        if destination.width <= 0.0 || destination.height <= 0.0 {
            return;
        }
        if UseCachedImageDrawingIfPossible(self.context, self.node, phase, painted_rect_changed) {
            return;
        }
        // cpp: core/paint/image_painter.cc:175-183. Animated pixels can change
        // before invalidation reaches Paint. Under-invalidation diagnostics
        // therefore repaint them but keep the new display item uncacheable.
        let skip_animated_image_cache = self
            .context
            .borrow()
            .PaintUnderInvalidationCheckingEnabled()
            && image.MaybeAnimated();
        let _cache_skipper =
            skip_animated_image_cache.then(|| DisplayItemCacheSkipper::new(self.context));
        // ImagePainter::PaintIntoRect records the replaced foreground as one
        // drawing, independently of the image sampling/cropping commands.
        // Source: core/paint/image_painter.cc:198-200.
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(phase),
            self.node.EnclosingRect(visual_rect),
        );
        let snapped_visual_rect = self.node.PixelSnappedRect(&visual_rect);
        let needs_clipping = destination.x < snapped_visual_rect.x
            || destination.y < snapped_visual_rect.y
            || destination.x + destination.width
                > snapped_visual_rect.x + snapped_visual_rect.width
            || destination.y + destination.height
                > snapped_visual_rect.y + snapped_visual_rect.height;
        if needs_clipping {
            let clipped_left = destination.x.max(snapped_visual_rect.x);
            let clipped_top = destination.y.max(snapped_visual_rect.y);
            let clipped_right = (destination.x + destination.width)
                .min(snapped_visual_rect.x + snapped_visual_rect.width);
            let clipped_bottom = (destination.y + destination.height)
                .min(snapped_visual_rect.y + snapped_visual_rect.height);
            if clipped_right <= clipped_left || clipped_bottom <= clipped_top {
                return;
            }
            if clipped_left != destination.x
                || clipped_top != destination.y
                || clipped_right != destination.x + destination.width
                || clipped_bottom != destination.y + destination.height
            {
                let source_scale_x = source.width / destination.width;
                let source_scale_y = source.height / destination.height;
                source.x += (clipped_left - destination.x) * source_scale_x;
                source.y += (clipped_top - destination.y) * source_scale_y;
                source.width = (clipped_right - clipped_left) * source_scale_x;
                source.height = (clipped_bottom - clipped_top) * source_scale_y;
                destination = PaintRect {
                    x: clipped_left,
                    y: clipped_top,
                    width: clipped_right - clipped_left,
                    height: clipped_bottom - clipped_top,
                };
            }
        }
        if !IsVisible(self.node, &destination) {
            return;
        }
        // cpp: core/paint/image_painter.cc:246-249. Keep the DevTools event at
        // the final DrawImage boundary, after source clipping and orientation
        // have resolved the exact rectangles submitted to the display list.
        let mut trace = browser_tracing::span("devtools.timeline", "PaintImage");
        trace.set("node_id", fragment.node_id as f64);
        trace.set("src_x", source.x);
        trace.set("src_y", source.y);
        trace.set("src_width", source.width);
        trace.set("src_height", source.height);
        trace.set("dest_x", destination.x);
        trace.set("dest_y", destination.y);
        trace.set("dest_area", destination.width * destination.height);
        let image_decoding_mode = ImageDecodingModeForPainting(fragment, image);
        let use_dark_mode_for_image = GetImageAutoDarkMode(
            &style.style,
            fragment
                .paint
                .resources
                .as_deref()
                .map_or(1.0, |resources| resources.layout_zoom_factor),
            destination,
            source,
        )
        .ShouldUseRasterDarkMode();
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                phase,
                node_id: fragment.node_id,
                // cpp: core/paint/image_painter.cc:277-278
                rect: destination,
                source_rect: source,
                resource_id: id,
                image_interpolation_quality: style.style.image_interpolation_quality(),
                dynamic_range_limit: style.style.dynamic_range_limit(),
                image_decoding_mode,
                image_orientation,
                use_dark_mode_for_image,
                document_image_container_key,
                ..Default::default()
            },
            self.node,
        );
    }

    fn PaintOutline(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint.style;
        let outline_color =
            InvertColorIfNeeded(style, style.outline_color(), ElementRole::kBackground);
        let self_outline = fragment.paint.self_outline.as_ref();
        let exported_outline_rects = if self.node.outline_rects.is_empty() {
            self_outline
                .map(|outline| {
                    outline
                        .rects
                        .iter()
                        .map(|rect| PaintRect {
                            x: self.node.paint_offset.x + rect.offset.x,
                            y: self.node.paint_offset.y + rect.offset.y,
                            width: rect.size.width,
                            height: rect.size.height,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let outline_rects = if self.node.outline_rects.is_empty() {
            exported_outline_rects.as_slice()
        } else {
            self.node.outline_rects.as_slice()
        };
        let exact_outline_is_empty = self_outline.is_some() && outline_rects.is_empty();
        let outline_width = self_outline
            .map(|outline| outline.width)
            .unwrap_or_else(|| style.outline_width());
        let outline_offset = self_outline
            .map(|outline| outline.offset)
            .unwrap_or_else(|| style.outline_offset());
        let auto_outline = fragment
            .paint
            .form_control
            .as_ref()
            .is_some_and(|control| control.auto_focus_ring);
        let has_css_outline = self.node.paints_outline
            && !exact_outline_is_empty
            && style.outline_width() > 0.0
            && style.outline_color().alpha > 0.0
            && style.outline_style() != BorderLineStyle::kNone;
        let has_outline = self.node.paints_outline && (auto_outline || has_css_outline);
        // OutlinePainter::PaintOutlineRects has a single phase recorder for
        // the united outline, including outlines of multiline inline boxes.
        // Official core/paint/outline_painter.cc:998.
        let _drawing = has_outline.then(|| {
            DrawingRecorder::new(
                self.context,
                self.node,
                DisplayItemIdType::PaintPhaseToDrawingType(phase),
                DecorationVisualRect(self.node),
            )
        });
        if self.node.paints_outline
            && fragment
                .paint
                .form_control
                .as_ref()
                .is_some_and(|control| control.auto_focus_ring)
        {
            let source = PaintRect {
                x: self.node.paint_offset.x,
                y: self.node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
            if source.width >= 2.0 && source.height >= 2.0 && IsVisible(self.node, &source) {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kStrokeRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: PaintRect {
                            x: source.x + 1.0,
                            y: source.y + 1.0,
                            width: source.width - 2.0,
                            height: source.height - 2.0,
                        },
                        color: Color {
                            red: 0.0,
                            green: 95.0 / 255.0,
                            blue: 204.0 / 255.0,
                            alpha: 1.0,
                        },
                        line_style: BorderLineStyle::kSolid,
                        stroke_width: 2.0,
                        antialias: false,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
        if !has_css_outline {
            return;
        }
        if !outline_width.is_finite() || !outline_offset.is_finite() {
            panic!("outline geometry is invalid");
        }
        if outline_rects.len() > 1 {
            let expanded_rects = |additional_outset: f64| {
                let mut rects = Vec::with_capacity(outline_rects.len());
                for source in outline_rects {
                    let horizontal_offset = outline_offset.max(-source.width / 2.0);
                    let vertical_offset = outline_offset.max(-source.height / 2.0);
                    rects.push(ExpandedOutlineRect(
                        source,
                        horizontal_offset + additional_outset,
                        vertical_offset + additional_outset,
                    ));
                }
                rects
            };
            let outer_rects = expanded_rects(outline_width);
            let bounds = UnionRects(&outer_rects);
            if !IsVisible(self.node, &bounds) {
                return;
            }
            let mut outline = DisplayItem {
                phase,
                node_id: fragment.node_id,
                rect: bounds,
                color: outline_color,
                line_style: style.outline_style(),
                ..Default::default()
            };
            if matches!(
                style.outline_style(),
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            ) {
                outline.r#type = DisplayItemType::kStrokePath;
                outline.stroke_width = outline_width;
                AppendUnionBoundary(&expanded_rects(outline_width / 2.0), &mut outline.path);
                if style.outline_style() == BorderLineStyle::kDashed {
                    outline.dash_intervals = vec![3.0 * outline_width, 3.0 * outline_width];
                } else {
                    outline.dash_intervals = vec![0.0, 2.0 * outline_width];
                    outline.round_cap = true;
                }
            } else {
                outline.r#type = DisplayItemType::kDrawPath;
                outline.even_odd = true;
                AppendUnionBoundary(&outer_rects, &mut outline.path);
                if style.outline_style() == BorderLineStyle::kDouble && outline_width > 2.0 {
                    let third = (outline_width / 3.0).round();
                    AppendUnionBoundary(&expanded_rects(outline_width - third), &mut outline.path);
                    AppendUnionBoundary(&expanded_rects(third), &mut outline.path);
                }
                AppendUnionBoundary(&expanded_rects(0.0), &mut outline.path);
            }
            if !outline.path.is_empty() {
                self.context.borrow_mut().Append(outline, self.node);
            }
            return;
        }
        let source_box = outline_rects.first().copied().unwrap_or(PaintRect {
            x: self.node.paint_offset.x,
            y: self.node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        });
        let horizontal_offset = outline_offset.max(-source_box.width / 2.0);
        let vertical_offset = outline_offset.max(-source_box.height / 2.0);
        let center_outset_x = horizontal_offset + outline_width / 2.0;
        let center_outset_y = vertical_offset + outline_width / 2.0;
        let rect = ExpandedOutlineRect(&source_box, center_outset_x, center_outset_y);
        let outer_bounds = ExpandedOutlineRect(
            &source_box,
            horizontal_offset + outline_width,
            vertical_offset + outline_width,
        );
        if !IsVisible(self.node, &outer_bounds) {
            return;
        }
        let border_radii = ResolveCornerRadii(
            style,
            &fragment.paint.border_sides,
            Size {
                width: source_box.width,
                height: source_box.height,
            },
            Edges::default(),
        );
        if style.outline_style() == BorderLineStyle::kDouble && outline_width > 2.0 {
            let third = (outline_width / 3.0).round();
            let append_ring = |outer_additional: f64, inner_additional: f64| {
                let outer = ExpandedOutlineRect(
                    &source_box,
                    horizontal_offset + outer_additional,
                    vertical_offset + outer_additional,
                );
                let inner = ExpandedOutlineRect(
                    &source_box,
                    horizontal_offset + inner_additional,
                    vertical_offset + inner_additional,
                );
                let outer_radii = ExpandCornerRadiiAxes(
                    border_radii,
                    horizontal_offset + outer_additional,
                    vertical_offset + outer_additional,
                    Size {
                        width: outer.width,
                        height: outer.height,
                    },
                );
                let inner_radii = ExpandCornerRadiiAxes(
                    border_radii,
                    horizontal_offset + inner_additional,
                    vertical_offset + inner_additional,
                    Size {
                        width: inner.width,
                        height: inner.height,
                    },
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawDoubleRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: outer,
                        inner_rect: inner,
                        color: outline_color,
                        line_style: BorderLineStyle::kDouble,
                        corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &outer_radii,
                        ),
                        inner_corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: outer_radii,
                        inner_corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            };
            append_ring(outline_width, outline_width - third);
            append_ring(third, 0.0);
            return;
        }
        let outline_radii = ExpandCornerRadiiAxes(
            border_radii,
            center_outset_x,
            center_outset_y,
            Size {
                width: rect.width,
                height: rect.height,
            },
        );
        let mut outline = DisplayItem {
            r#type: DisplayItemType::kStrokeRect,
            phase,
            node_id: fragment.node_id,
            rect,
            color: outline_color,
            line_style: style.outline_style(),
            stroke_width: outline_width,
            corner_radius: crate::contoured_border_geometry::UniformCornerRadius(&outline_radii),
            corner_radii: outline_radii,
            ..Default::default()
        };
        if style.outline_style() == BorderLineStyle::kDashed {
            outline.dash_intervals = vec![3.0 * outline_width, 3.0 * outline_width];
        } else if style.outline_style() == BorderLineStyle::kDotted {
            outline.dash_intervals = vec![0.0, 2.0 * outline_width];
            outline.round_cap = true;
        }
        if matches!(
            style.outline_style(),
            BorderLineStyle::kDashed | BorderLineStyle::kDotted
        ) && border_radii.HasRadius()
        {
            let outer_outset_x = horizontal_offset + outline_width;
            let outer_outset_y = vertical_offset + outline_width;
            let half_used_width = (outline_width * 0.5) as i32;
            let stroked_outset_x = outer_outset_x - half_used_width as f64;
            let stroked_outset_y = outer_outset_y - half_used_width as f64;
            let snapped_border = self.node.PixelSnappedRect(&source_box);
            let snapped_border_radii = ResolveCornerRadii(
                style,
                &fragment.paint.border_sides,
                Size {
                    width: snapped_border.width,
                    height: snapped_border.height,
                },
                Edges::default(),
            );
            outline.rect = self.node.PixelSnappedRect(&ExpandedOutlineRect(
                &source_box,
                stroked_outset_x,
                stroked_outset_y,
            ));
            outline.corner_radii = ExpandCornerRadiiAxes(
                snapped_border_radii,
                stroked_outset_x,
                stroked_outset_y,
                Size {
                    width: outline.rect.width,
                    height: outline.rect.height,
                },
            );
            outline.corner_radius =
                crate::contoured_border_geometry::UniformCornerRadius(&outline.corner_radii);
            let outer_clip = self.node.PixelSnappedRect(&ExpandedOutlineRect(
                &source_box,
                outer_outset_x,
                outer_outset_y,
            ));
            let inner_clip = self.node.PixelSnappedRect(&ExpandedOutlineRect(
                &source_box,
                horizontal_offset,
                vertical_offset,
            ));
            let outer_clip_radii = ExpandCornerRadiiAxes(
                snapped_border_radii,
                outer_outset_x,
                outer_outset_y,
                Size {
                    width: outer_clip.width,
                    height: outer_clip.height,
                },
            );
            let inner_clip_radii = ExpandCornerRadiiAxes(
                snapped_border_radii,
                horizontal_offset,
                vertical_offset,
                Size {
                    width: inner_clip.width,
                    height: inner_clip.height,
                },
            );
            outline.dash_fit_thickness = outline_width;
            outline.stroke_width = outline_width * 2.2;
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRoundedRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: outer_clip,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &outer_clip_radii,
                    ),
                    corner_radii: outer_clip_radii,
                    ..Default::default()
                },
                self.node,
            );
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipOutRoundedRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: inner_clip,
                    corner_radius: crate::contoured_border_geometry::UniformCornerRadius(
                        &inner_clip_radii,
                    ),
                    corner_radii: inner_clip_radii,
                    ..Default::default()
                },
                self.node,
            );
            for side_path in RoundedBorderSideClips(&outer_clip, &inner_clip, &inner_clip_radii) {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipPath,
                        phase,
                        node_id: fragment.node_id,
                        path: side_path,
                        antialias: false,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(outline.clone(), self.node);
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kRestore,
                        phase,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
            return;
        }
        self.context.borrow_mut().Append(outline, self.node);
    }
}

fn ClampImageMapCoordinate(value: f64) -> f64 {
    LayoutUnit::from_f64(value).ToFloat() as f64
}

fn RectPath(left: f64, top: f64, right: f64, bottom: f64) -> Vec<PaintPathCommand> {
    if right <= left || bottom <= top {
        return Vec::new();
    }
    vec![
        PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: Offset { x: left, y: top },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset { x: right, y: top },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset {
                x: right,
                y: bottom,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kLineTo,
            point: Offset { x: left, y: bottom },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        },
    ]
}

fn EllipsePath(center_x: f64, center_y: f64, radius: f64) -> Vec<PaintPathCommand> {
    if radius <= 0.0 {
        return Vec::new();
    }
    const QUARTER_CIRCLE_WEIGHT: f64 = 0.7071067811865475244;
    vec![
        PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: Offset {
                x: center_x + radius,
                y: center_y,
            },
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: center_x + radius,
                y: center_y + radius,
            },
            point: Offset {
                x: center_x,
                y: center_y + radius,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: center_x - radius,
                y: center_y + radius,
            },
            point: Offset {
                x: center_x - radius,
                y: center_y,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: center_x - radius,
                y: center_y - radius,
            },
            point: Offset {
                x: center_x,
                y: center_y - radius,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: center_x + radius,
                y: center_y - radius,
            },
            point: Offset {
                x: center_x + radius,
                y: center_y,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..Default::default()
        },
        PaintPathCommand {
            verb: PaintPathVerb::kClose,
            ..Default::default()
        },
    ]
}

fn ImageMapAreaPath(
    area: &FocusedImageMapArea,
    border_box: PaintRect,
    image_effective_zoom: f64,
) -> Vec<PaintPathCommand> {
    if area.shape == ImageMapAreaShape::kDefault {
        return RectPath(
            border_box.x,
            border_box.y,
            border_box.x + border_box.width,
            border_box.y + border_box.height,
        );
    }
    let resolve = |value: f64, horizontal: bool| {
        ClampImageMapCoordinate(value) * image_effective_zoom
            + if horizontal {
                border_box.x
            } else {
                border_box.y
            }
    };
    match area.shape {
        ImageMapAreaShape::kRect if area.coordinates.len() >= 4 => RectPath(
            resolve(area.coordinates[0], true),
            resolve(area.coordinates[1], false),
            resolve(area.coordinates[2], true),
            resolve(area.coordinates[3], false),
        ),
        ImageMapAreaShape::kCircle if area.coordinates.len() >= 3 && area.coordinates[2] > 0.0 => {
            EllipsePath(
                resolve(area.coordinates[0], true),
                resolve(area.coordinates[1], false),
                ClampImageMapCoordinate(area.coordinates[2]) * image_effective_zoom,
            )
        }
        ImageMapAreaShape::kPoly if area.coordinates.len() >= 6 => {
            let point_count = area.coordinates.len() / 2;
            let mut path = Vec::with_capacity(point_count + 2);
            path.push(PaintPathCommand {
                verb: PaintPathVerb::kMoveTo,
                point: Offset {
                    x: resolve(area.coordinates[0], true),
                    y: resolve(area.coordinates[1], false),
                },
                ..Default::default()
            });
            for point in 1..point_count {
                path.push(PaintPathCommand {
                    verb: PaintPathVerb::kLineTo,
                    point: Offset {
                        x: resolve(area.coordinates[point * 2], true),
                        y: resolve(area.coordinates[point * 2 + 1], false),
                    },
                    ..Default::default()
                });
            }
            path.push(PaintPathCommand {
                verb: PaintPathVerb::kClose,
                ..Default::default()
            });
            path
        }
        _ => Vec::new(),
    }
}

fn FocusRingStrokeWidth(effective_zoom: f32) -> f64 {
    const WIDTH: f32 = 3.0;
    if effective_zoom >= 1.0 {
        (WIDTH * effective_zoom).floor().max(1.0) as f64
    } else {
        WIDTH as f64
    }
}

fn FragmentVisualRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    // core/paint/text_fragment_painter.cc:422-431. Layout owns the parent SVG
    // visual bound, already including glyph transforms, stroke and shadow.
    if let Some(svg) = &fragment.paint.svg_text {
        return node.EnclosingRect(PaintRect {
            x: node.paint_offset.x + svg.visual_rect.offset.x,
            y: node.paint_offset.y + svg.visual_rect.offset.y,
            width: svg.visual_rect.size.width,
            height: svg.visual_rect.size.height,
        });
    }
    let mut paint_offset = node.paint_offset;
    // text_fragment_painter.cc:98-99 shares this physical-line snap with the
    // text origin; moving the glyphs must move their visual rect as well.
    if fragment.paint.text_line_top_offset.is_some() {
        paint_offset.y += fragment.paint.text_line_snap_delta.unwrap_or(0.0);
    }
    if let Some(rect) = fragment.paint.self_ink_overflow {
        return node.EnclosingRect(PaintRect {
            x: paint_offset.x + rect.offset.x,
            y: paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        });
    }
    // Preserve real glyph overhangs while the decoration/marker branch still
    // needs the retained-layout bridge into the Paint-owned InlinePaintContext.
    let mut rect = PaintRect {
        x: paint_offset.x,
        y: paint_offset.y,
        width: fragment.size.width,
        height: fragment.size.height,
    };
    if let Some(glyph) = fragment.paint.text_glyph_ink {
        rect.union(PaintRect {
            x: paint_offset.x + glyph.offset.x,
            y: paint_offset.y + glyph.offset.y,
            width: glyph.size.width,
            height: glyph.size.height,
        });
    }
    let source = rect;
    for index in 0..fragment.paint.style.text_shadow_count() {
        let shadow = fragment.paint.style.text_shadow(index);
        // shadow_data.cc:32-42, shadow_list.cc:39-47. Include negative spread;
        // quantize blur extent before adding offsets and fragment coordinates.
        if shadow.inset {
            continue;
        }
        let extent = (1.5 * shadow.blur_radius).ceil() + shadow.spread;
        rect.union(PaintRect {
            x: source.x + shadow.offset.x - extent,
            y: source.y + shadow.offset.y - extent,
            width: source.width + 2.0 * extent,
            height: source.height + 2.0 * extent,
        });
    }
    node.EnclosingRect(rect)
}

fn UsesCompositedFixedBackground(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let background_images = fragment.paint.style.background_images();
    fragment.paint.can_composite_background_attachment_fixed
        && !fragment.paint.background_transfers_to_view
        && !node.fixed_background_acts_as_scroll
        && node.fixed_background_properties.is_some()
        && fragment.paint.background_paint_location.PaintsBorderBox()
        && background_images.len() == 1
        && background_images[0].attachment == BackgroundAttachment::kFixed
        && (background_images[0].shader.is_some() || background_images[0].resource_id != 0)
}

pub(crate) fn DecorationVisualRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    // BoxPainter::VisualRect:124-129; inline boxes use their item ink overflow.
    let rect = fragment
        .paint
        .box_self_visual_overflow
        .or(fragment.paint.ink_overflow)
        .expect("painted box fragment has source visual overflow");
    // Source: core/paint/box_fragment_painter.cc:3021-3030 and
    // core/paint/box_painter.cc:124-129. Layout owns visual overflow; Paint
    // only applies the fragment paint offset and enclosing-pixel conversion.
    node.EnclosingRect(PaintRect {
        x: node.paint_offset.x + rect.offset.x,
        y: node.paint_offset.y + rect.offset.y,
        width: rect.size.width,
        height: rect.size.height,
    })
}

// Source: box_fragment_painter.cc passes ToPixelSnappedRect(paint_rect) to
// ObjectPainter::RecordHitTestData and tracked-element metadata. This is the
// border box, not the enclosing visual-overflow rect used by DrawingRecorder.
fn PixelSnappedBorderBoxRect(node: &PaintTreeNode<'_>, size: Size) -> PaintRect {
    node.PixelSnappedRect(&PaintRect {
        x: node.paint_offset.x,
        y: node.paint_offset.y,
        width: size.width,
        height: size.height,
    })
}

/// A canvas background with both fixed and scrolling layers spans more than
/// one transform state, so its blend group cannot live inside either layer's
/// PaintRecord.  Give the group a retained Effect node and let LayerTile bind
/// its matching flat save-layer scope across all participating chunks.
fn CanvasBackgroundEffect(
    id: u64,
    local_state: &PropertyTreeState,
    output_clip: Arc<ClipPaintPropertyNode>,
    parent: Arc<EffectPaintPropertyNode>,
    blend_mode: PaintBlendMode,
    isolates_blending: bool,
) -> EffectPaintPropertyNode {
    EffectPaintPropertyNode {
        compositor: Default::default(),
        backdrop_filter_bounds: None,
        self_or_ancestor_participates_in_view_transition: false,
        lifecycle: PaintPropertyNodeLifecycle::default(),
        id,
        parent: Some(parent),
        local_transform_space: local_state.transform.clone(),
        output_clip: Some(output_clip),
        filter_reference_box: None,
        filter_viewport: None,
        filter_space_origin: Offset::default(),
        filter_reference_scale: 1.0,
        opacity: 1.0,
        blend_mode,
        filters: Default::default(),
        backdrop_filters: Default::default(),
        backdrop_filter_has_reference: false,
        backdrop_filter_clip: None,
        isolates_blending,
        has_mask: false,
        is_mask: false,
        direct_compositing_reasons: Default::default(),
    }
    .with_filter_presence()
    .with_inherited_view_transition()
}

fn CanvasBackgroundDisplayItemId(
    fragment: &layoutng_assembly::fragment_tree::FragmentPaintData,
    item_type: DisplayItemIdType,
    layer_ordinal: usize,
) -> DisplayItemId {
    // LayoutView never fragments. Use ScopedDisplayItemFragment's occurrence
    // field to give each independently transformed canvas layer a stable
    // semantic record identity while retaining the source LayoutView client.
    let occurrence = u32::try_from(layer_ordinal).expect("canvas layer count must fit u32");
    DisplayItemId {
        client_id: fragment.paint.display_item_client_id,
        r#type: item_type,
        fragment: fragment
            .paint
            .display_item_fragment
            .checked_add(occurrence)
            .expect("canvas display item fragment must fit u32"),
    }
}

fn CanvasBackgroundRect(node: &PaintTreeNode<'_>, scrolling: bool) -> Option<PaintRect> {
    let fragment = node.fragment.as_deref()?;
    fragment.paint.canvas_background.as_ref()?;
    ViewBackgroundRect(node, scrolling)
}

fn FindPaintNodeById<'n, 'f>(
    node: &'n PaintTreeNode<'f>,
    node_id: u64,
) -> Option<&'n PaintTreeNode<'f>> {
    if node_id == 0 {
        return None;
    }
    if node
        .fragment
        .as_deref()
        .is_some_and(|fragment| fragment.node_id == node_id)
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| FindPaintNodeById(child, node_id))
}

/// ViewPainter uses the document element's LocalBorderBoxProperties but
/// replaces its CSS transform with PreTransform. This preserves paint-offset
/// and sticky translations as well as the element's clip/effect state.
fn CanvasBackgroundPaintState(node: &PaintTreeNode<'_>) -> PropertyTreeState {
    let document_element_node_id = node
        .fragment
        .as_deref()
        .and_then(|fragment| fragment.paint.canvas_background.as_ref())
        .map_or(0, |canvas| canvas.document_element_node_id);
    let Some(document_element) = FindPaintNodeById(node, document_element_node_id) else {
        return node.contents_properties.nodes.clone();
    };
    let mut state = document_element.properties.nodes.clone();
    state.transform = document_element.pre_transform.clone();
    state
}

fn CanvasBackgroundRectInState(
    node: &PaintTreeNode<'_>,
    scrolling: bool,
    destination: &PropertyTreeState,
) -> Option<PaintRect> {
    let rect = CanvasBackgroundRect(node, scrolling)?;
    SourceToDestinationRect(
        &node.contents_properties.nodes.transform,
        &destination.transform,
        rect,
    )
}

/// ViewPainter's BoxBackgroundPaintContext uses the document element's
/// stitched border box for image sizing and positioning even though the draw
/// destination is the LayoutView canvas. The exported paint offset is already
/// in LayoutView contents space, matching the ordinary (unmapped) source path.
fn CanvasBackgroundPositioningBoxRect(
    node: &PaintTreeNode<'_>,
    box_type: BackgroundBox,
) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("canvas background requires a fragment");
    let canvas = fragment
        .paint
        .canvas_background
        .as_ref()
        .expect("canvas background positioning requires document-element geometry");
    let mut rect = PaintRect {
        x: canvas.document_element_paint_offset.x,
        y: canvas.document_element_paint_offset.y,
        width: canvas.document_element_size.width,
        height: canvas.document_element_size.height,
    };
    let box_type = UsedBackgroundGeometryBox(box_type);
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    rect = ContractPaintRect(rect, canvas.document_element_border);
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    ContractPaintRect(rect, canvas.document_element_padding)
}

/// With a remapped document-element paint state the root paint offset is
/// already encoded by the property tree. Chromium therefore uses the mapped
/// canvas origin as BackgroundImageGeometry's offset instead of subtracting
/// the document element's LayoutView-space paint offset.
fn CanvasBackgroundPositioningBoxRectInState(
    node: &PaintTreeNode<'_>,
    box_type: BackgroundBox,
    destination: &PropertyTreeState,
    mapped_canvas_rect: PaintRect,
) -> PaintRect {
    if node
        .contents_properties
        .nodes
        .transform
        .lifecycle
        .same_node(&destination.transform.lifecycle)
    {
        return CanvasBackgroundPositioningBoxRect(node, box_type);
    }
    let fragment = node
        .fragment
        .as_deref()
        .expect("canvas background requires a fragment");
    let canvas = fragment
        .paint
        .canvas_background
        .as_ref()
        .expect("canvas background positioning requires document-element geometry");
    let mut rect = PaintRect {
        x: mapped_canvas_rect.x,
        y: mapped_canvas_rect.y,
        width: canvas.document_element_size.width,
        height: canvas.document_element_size.height,
    };
    let box_type = UsedBackgroundGeometryBox(box_type);
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    rect = ContractPaintRect(rect, canvas.document_element_border);
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    ContractPaintRect(rect, canvas.document_element_padding)
}

fn ViewBackgroundRect(node: &PaintTreeNode<'_>, scrolling: bool) -> Option<PaintRect> {
    let fragment = node.fragment.as_deref()?;
    let source_background = fragment.paint.view_background_rect.map(|rect| PaintRect {
        x: node.paint_offset.x + rect.offset.x,
        y: node.paint_offset.y + rect.offset.y,
        width: rect.size.width,
        height: rect.size.height,
    });
    if !scrolling {
        return Some(
            node.PixelSnappedRect(&source_background.unwrap_or(PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            })),
        );
    }
    let viewport = fragment
        .paint
        .resources
        .as_deref()
        .and_then(|resources| {
            resources.viewport.map(|viewport| {
                let zoom = resources.layout_zoom_factor.max(f64::MIN_POSITIVE);
                Size {
                    width: viewport.size.width as f64 * zoom,
                    height: viewport.size.height as f64 * zoom,
                }
            })
        })
        .unwrap_or(fragment.size);
    let mut rect = PaintRect {
        x: node.paint_offset.x,
        y: node.paint_offset.y,
        width: viewport
            .width
            .max(fragment.size.width)
            .max(fragment.paint.scroll_size.width),
        height: viewport
            .height
            .max(fragment.size.height)
            .max(fragment.paint.scroll_size.height),
    };
    if let Some(source_background) = source_background {
        rect.union(source_background);
    }
    Some(node.PixelSnappedRect(&rect))
}

fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

pub(crate) fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(self_cull_rect) = node.self_cull_rect else {
        return true;
    };
    Intersects(rect, &self_cull_rect)
}

fn ExpandedOutlineRect(
    rect: &PaintRect,
    horizontal_outset: f64,
    vertical_outset: f64,
) -> PaintRect {
    PaintRect {
        x: rect.x - horizontal_outset,
        y: rect.y - vertical_outset,
        width: rect.width + 2.0 * horizontal_outset,
        height: rect.height + 2.0 * vertical_outset,
    }
}

fn UnionRects(rects: &[PaintRect]) -> PaintRect {
    let mut result = rects[0];
    for rect in rects.iter().skip(1) {
        let left = result.x.min(rect.x);
        let top = result.y.min(rect.y);
        let right = (result.x + result.width).max(rect.x + rect.width);
        let bottom = (result.y + result.height).max(rect.y + rect.height);
        result = PaintRect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        };
    }
    result
}

#[derive(Clone, Copy, Debug)]
struct BoundaryPoint(f64, f64);

impl PartialEq for BoundaryPoint {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for BoundaryPoint {}
impl PartialOrd for BoundaryPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BoundaryPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        fn compare(left: f64, right: f64) -> Ordering {
            if left == right {
                Ordering::Equal
            } else {
                left.total_cmp(&right)
            }
        }
        compare(self.0, other.0).then_with(|| compare(self.1, other.1))
    }
}

fn AppendUnionBoundary(input: &[PaintRect], path: &mut Vec<PaintPathCommand>) {
    let mut rects = Vec::new();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for rect in input {
        if rect.width <= 0.0 || rect.height <= 0.0 {
            continue;
        }
        rects.push(*rect);
        xs.push(rect.x);
        xs.push(rect.x + rect.width);
        ys.push(rect.y);
        ys.push(rect.y + rect.height);
    }
    if rects.is_empty() {
        return;
    }
    xs.sort_by(f64::total_cmp);
    xs.dedup_by(|left, right| *left == *right);
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|left, right| *left == *right);
    let columns = xs.len() - 1;
    let rows = ys.len() - 1;
    let mut covered = vec![false; columns * rows];
    let cell = |x: usize, y: usize| y * columns + x;
    for y in 0..rows {
        let middle_y = (ys[y] + ys[y + 1]) / 2.0;
        for x in 0..columns {
            let middle_x = (xs[x] + xs[x + 1]) / 2.0;
            covered[cell(x, y)] = rects.iter().any(|rect| {
                middle_x >= rect.x
                    && middle_x < rect.x + rect.width
                    && middle_y >= rect.y
                    && middle_y < rect.y + rect.height
            });
        }
    }

    let mut edges: BTreeMap<BoundaryPoint, Vec<BoundaryPoint>> = BTreeMap::new();
    let mut add_edge = |start: BoundaryPoint, end: BoundaryPoint| {
        edges.entry(start).or_default().push(end);
    };
    for y in 0..rows {
        for x in 0..columns {
            if !covered[cell(x, y)] {
                continue;
            }
            if y == 0 || !covered[cell(x, y - 1)] {
                add_edge(BoundaryPoint(xs[x], ys[y]), BoundaryPoint(xs[x + 1], ys[y]));
            }
            if x + 1 == columns || !covered[cell(x + 1, y)] {
                add_edge(
                    BoundaryPoint(xs[x + 1], ys[y]),
                    BoundaryPoint(xs[x + 1], ys[y + 1]),
                );
            }
            if y + 1 == rows || !covered[cell(x, y + 1)] {
                add_edge(
                    BoundaryPoint(xs[x + 1], ys[y + 1]),
                    BoundaryPoint(xs[x], ys[y + 1]),
                );
            }
            if x == 0 || !covered[cell(x - 1, y)] {
                add_edge(BoundaryPoint(xs[x], ys[y + 1]), BoundaryPoint(xs[x], ys[y]));
            }
        }
    }

    let mut remaining = 0;
    for destinations in edges.values() {
        remaining += destinations.len();
    }
    while remaining != 0 {
        let start = *edges.first_key_value().unwrap().0;
        let mut current = start;
        let mut previous: Option<BoundaryPoint> = None;
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kMoveTo,
            point: Offset {
                x: current.0,
                y: current.1,
            },
            ..Default::default()
        });
        let max_steps = remaining + 1;
        for _guard in 0..max_steps {
            let Some(destinations) = edges.get(&current) else {
                panic!("outline union produced an open boundary");
            };
            if destinations.is_empty() {
                panic!("outline union produced an open boundary");
            }
            let mut selected = destinations.len() - 1;
            if let Some(previous) = previous {
                if destinations.len() > 1 {
                    let direction = |from: BoundaryPoint, to: BoundaryPoint| {
                        if to.0 > from.0 {
                            0
                        } else if to.1 > from.1 {
                            1
                        } else if to.0 < from.0 {
                            2
                        } else {
                            3
                        }
                    };
                    let incoming = direction(previous, current);
                    let mut best_score = 4;
                    for (candidate, &destination) in destinations.iter().enumerate() {
                        let turn = (direction(current, destination) - incoming + 4) % 4;
                        let score = if turn == 1 {
                            0
                        } else if turn == 0 {
                            1
                        } else if turn == 3 {
                            2
                        } else {
                            3
                        };
                        if score < best_score {
                            best_score = score;
                            selected = candidate;
                        }
                    }
                }
            }
            let (next, empty) = {
                let destinations = edges.get_mut(&current).unwrap();
                let next = destinations.remove(selected);
                (next, destinations.is_empty())
            };
            if empty {
                edges.remove(&current);
            }
            remaining -= 1;
            path.push(PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: Offset {
                    x: next.0,
                    y: next.1,
                },
                ..Default::default()
            });
            previous = Some(current);
            current = next;
            if current == start {
                path.push(PaintPathCommand {
                    verb: PaintPathVerb::kClose,
                    ..Default::default()
                });
                break;
            }
        }
    }
}

fn DecorationRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    if let Some(rect) = fragment.paint.first_line_decoration_rect {
        return PaintRect {
            x: node.paint_offset.x + rect.offset.x,
            y: node.paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        };
    }
    if fragment.paint.table.is_some() {
        return TablePainter::DecorationRect(node);
    }
    FieldsetPainter::DecorationRect(node)
}

fn IsSpecialBackgroundClip(box_type: BackgroundBox) -> bool {
    matches!(
        box_type,
        BackgroundBox::kText | BackgroundBox::kBorderArea | BackgroundBox::kBorderAreaText
    )
}

/// Resolve the geometry-box aliases only where a concrete CSS layout box is
/// available.  The semantic value itself remains intact on PaintStyleData so
/// special text/border-area behavior is not lost at the package boundary.
fn UsedBackgroundGeometryBox(box_type: BackgroundBox) -> BackgroundBox {
    match box_type {
        BackgroundBox::kPaddingBox => BackgroundBox::kPaddingBox,
        // For an element with an associated CSS layout box, fill-box computes
        // to content-box while stroke-box/view-box compute to border-box.
        BackgroundBox::kContentBox | BackgroundBox::kFillBox => BackgroundBox::kContentBox,
        BackgroundBox::kBorderBox
        | BackgroundBox::kStrokeBox
        | BackgroundBox::kViewBox
        | BackgroundBox::kNoClip
        | BackgroundBox::kText
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => BackgroundBox::kBorderBox,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BackgroundDestinationRects {
    unsnapped: PaintRect,
    snapped: PaintRect,
}

fn ContractPaintRect(mut rect: PaintRect, edges: Edges) -> PaintRect {
    rect.x += edges.left;
    rect.y += edges.top;
    rect.width = (rect.width - edges.left - edges.right).max(0.0);
    rect.height = (rect.height - edges.top - edges.bottom).max(0.0);
    rect
}

fn IntersectPaintRects(a: PaintRect, b: PaintRect) -> PaintRect {
    let left = a.x.max(b.x);
    let top = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    PaintRect {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

fn BorderEdgeObscuresBackground(
    present: bool,
    width: f64,
    color: Color,
    mut style: BorderLineStyle,
) -> bool {
    // BorderEdge::EffectiveStyle collapses narrow multi-line borders to solid
    // before ObscuresBackground() examines the style.
    if (style == BorderLineStyle::kDouble && width < 3.0)
        || (matches!(style, BorderLineStyle::kGroove | BorderLineStyle::kRidge) && width <= 1.0)
    {
        style = BorderLineStyle::kSolid;
    }
    present
        && color.alpha >= 1.0
        && !matches!(
            style,
            BorderLineStyle::kHidden
                | BorderLineStyle::kDotted
                | BorderLineStyle::kDashed
                | BorderLineStyle::kDouble
        )
}

// Chromium: background_image_geometry.cc:209-280 and
// box_background_paint_context.cc:248-297.  Keep both rectangles: source
// positioning uses the unsnapped intent while raster work uses border-aligned
// snapped bounds.
fn ResolveBackgroundDestinationRects(
    node: &PaintTreeNode<'_>,
    layer: &BackgroundImageLayer,
    phase: PaintPhase,
    attachment: BackgroundAttachment,
    paint_rect: PaintRect,
    has_geometry_override: bool,
) -> BackgroundDestinationRects {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let paint = &fragment.paint;
    let mut result = BackgroundDestinationRects {
        unsnapped: paint_rect,
        snapped: node.PixelSnappedRect(&paint_rect),
    };

    // Fixed backgrounds use the viewport destination directly.  Border-derived
    // contraction is also unsafe outside an ordinary self-background pass and
    // for the same exceptional box geometries rejected by Blink's context.
    let disallow_border_derived_adjustment = attachment == BackgroundAttachment::kFixed
        || has_geometry_override
        || !paint_model::paint_phase::ShouldPaintSelfBlockBackground(phase)
        || paint.display == Display::kTableCell
        || paint.is_page_container
        || paint.is_page_border_box
        || paint.stitched_decoration.is_some()
        || node.paint_layer_is_fragmented
        || paint.style.border_shape_unsupported
        || paint.style.border_image.is_some()
        || paint.has_collapsed_borders;

    if layer.clip == BackgroundBox::kNoClip
        || UsedBackgroundGeometryBox(layer.clip) != BackgroundBox::kBorderBox
        || disallow_border_derived_adjustment
    {
        return result;
    }

    let border = IncludedBorderEdges(paint);
    let inner = ContractPaintRect(paint_rect, border);
    let snapped_inner = node.PixelSnappedRect(&inner);
    let sides = &paint.border_sides;
    let colors = paint.style.border_colors();
    let obscures = [
        BorderEdgeObscuresBackground(sides.top, border.top, colors[0], paint.border_styles[0]),
        BorderEdgeObscuresBackground(sides.right, border.right, colors[1], paint.border_styles[1]),
        BorderEdgeObscuresBackground(
            sides.bottom,
            border.bottom,
            colors[2],
            paint.border_styles[2],
        ),
        BorderEdgeObscuresBackground(sides.left, border.left, colors[3], paint.border_styles[3]),
    ];

    if obscures[0] {
        let bottom = result.unsnapped.y + result.unsnapped.height;
        result.unsnapped.y += border.top;
        result.unsnapped.height = (bottom - result.unsnapped.y).max(0.0);
        let bottom = result.snapped.y + result.snapped.height;
        result.snapped.y = snapped_inner.y;
        result.snapped.height = (bottom - result.snapped.y).max(0.0);
    }
    if obscures[1] {
        result.unsnapped.width = (result.unsnapped.width - border.right).max(0.0);
        result.snapped.width = (snapped_inner.x + snapped_inner.width - result.snapped.x).max(0.0);
    }
    if obscures[2] {
        result.unsnapped.height = (result.unsnapped.height - border.bottom).max(0.0);
        result.snapped.height =
            (snapped_inner.y + snapped_inner.height - result.snapped.y).max(0.0);
    }
    if obscures[3] {
        let right = result.unsnapped.x + result.unsnapped.width;
        result.unsnapped.x += border.left;
        result.unsnapped.width = (right - result.unsnapped.x).max(0.0);
        let right = result.snapped.x + result.snapped.width;
        result.snapped.x = snapped_inner.x;
        result.snapped.width = (right - result.snapped.x).max(0.0);
    }
    result
}

fn ConstrainBackgroundDestinationToTile(
    mut destination: PaintRect,
    tile: PaintRect,
    repeat_x: bool,
    repeat_y: bool,
) -> PaintRect {
    if !repeat_x {
        let clipped = IntersectPaintRects(
            PaintRect {
                y: destination.y,
                height: destination.height,
                ..tile
            },
            destination,
        );
        destination.x = clipped.x;
        destination.width = clipped.width;
    }
    if !repeat_y {
        let clipped = IntersectPaintRects(
            PaintRect {
                x: destination.x,
                width: destination.width,
                ..tile
            },
            destination,
        );
        destination.y = clipped.y;
        destination.height = clipped.height;
    }
    destination
}

fn CropBackgroundSourceRect(
    source: PaintRect,
    full_destination: PaintRect,
    destination: PaintRect,
    unsnapped_destination: PaintRect,
    tile_size: Size,
) -> PaintRect {
    if tile_size.width <= 0.0 || tile_size.height <= 0.0 {
        return PaintRect::default();
    }
    let scale_x = source.width / tile_size.width;
    let scale_y = source.height / tile_size.height;
    PaintRect {
        x: source.x + (destination.x - full_destination.x) * scale_x,
        y: source.y + (destination.y - full_destination.y) * scale_y,
        width: unsnapped_destination.width * scale_x,
        height: unsnapped_destination.height * scale_y,
    }
}

fn BackgroundBoxRect(node: &PaintTreeNode<'_>, box_type: BackgroundBox) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let style = &fragment.paint;
    let decoration = DecorationRect(node);
    let x = decoration.x;
    let y = decoration.y;
    let box_type = UsedBackgroundGeometryBox(box_type);
    if box_type == BackgroundBox::kBorderBox {
        return decoration;
    }
    let border = IncludedBorderEdges(style);
    if box_type == BackgroundBox::kPaddingBox {
        return PaintRect {
            x: x + border.left,
            y: y + border.top,
            width: (decoration.width - border.left - border.right).max(0.0),
            height: (decoration.height - border.top - border.bottom).max(0.0),
        };
    }
    PaintRect {
        x: x + border.left + style.padding.left,
        y: y + border.top + style.padding.top,
        width: fragment.content_size.width,
        height: fragment.content_size.height,
    }
}

fn BackgroundBoxRectWithin(
    node: &PaintTreeNode<'_>,
    box_type: BackgroundBox,
    mut rect: PaintRect,
) -> PaintRect {
    let box_type = UsedBackgroundGeometryBox(box_type);
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    let paint = &node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .paint;
    let border = IncludedBorderEdges(paint);
    rect.x += border.left;
    rect.y += border.top;
    rect.width = (rect.width - border.left - border.right).max(0.0);
    rect.height = (rect.height - border.top - border.bottom).max(0.0);
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    rect.x += paint.padding.left;
    rect.y += paint.padding.top;
    rect.width = (rect.width - paint.padding.left - paint.padding.right).max(0.0);
    rect.height = (rect.height - paint.padding.top - paint.padding.bottom).max(0.0);
    rect
}

fn EffectiveBackgroundAttachment(
    node: &PaintTreeNode<'_>,
    layer: &BackgroundImageLayer,
) -> BackgroundAttachment {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    match layer.attachment {
        BackgroundAttachment::kFixed
            if fragment.paint.is_page_border_box
                && !fragment
                    .paint
                    .page_document_background_geometry
                    .is_some_and(|geometry| geometry.has_background_fixed_to_viewport) =>
        {
            BackgroundAttachment::kScroll
        }
        BackgroundAttachment::kFixed
            if fragment.paint.is_page_container
                && !fragment
                    .paint
                    .page_container_background_geometry
                    .is_some_and(|geometry| geometry.has_background_fixed_to_viewport) =>
        {
            BackgroundAttachment::kScroll
        }
        BackgroundAttachment::kFixed if node.fixed_background_acts_as_scroll => {
            BackgroundAttachment::kScroll
        }
        // A local attachment only establishes scrolling geometry on an actual
        // source scroll owner. Without one, Chromium paints it in border-box
        // space and it is visually equivalent to scroll attachment.
        BackgroundAttachment::kLocal if fragment.paint.scroll_container.is_none() => {
            BackgroundAttachment::kScroll
        }
        attachment => attachment,
    }
}

fn FixedBackgroundPositioningArea(node: &PaintTreeNode<'_>) -> Option<PaintRect> {
    let fragment = node.fragment.as_deref()?;
    let resources = fragment.paint.resources.as_ref()?;
    let viewport = resources.viewport?;
    let zoom = resources.layout_zoom_factor.max(f64::MIN_POSITIVE);
    // GeometryMapper maps the viewport origin into the current transform
    // space. As in Chromium, the viewport size remains in LayoutZoomFactor
    // coordinates; the root transform subsequently affects the fixed image.
    let mapped_origin = MapRectWithTransforms(
        PaintRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        },
        &node.transforms,
        true,
    )?;
    Some(PaintRect {
        x: mapped_origin.x + node.scroll_offset.x,
        y: mapped_origin.y + node.scroll_offset.y,
        width: viewport.size.width as f64 * zoom,
        height: viewport.size.height as f64 * zoom,
    })
}

fn ContentsBackgroundBorderRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let border = IncludedBorderEdges(&fragment.paint);
    let mut rect = BackgroundPositioningBoxRect(node, BackgroundBox::kBorderBox);
    // LayoutBox::ScrollWidth/Height describe the scrolling padding area. The
    // background painter adds the border widths while recording the local
    // layer into scrolling-contents space.
    rect.width = fragment.paint.scroll_size.width + border.left + border.right;
    rect.height = fragment.paint.scroll_size.height + border.top + border.bottom;
    rect
}

fn ScrollingBackgroundVisualRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let local = fragment
        .paint
        .scroll_container
        .as_ref()
        .expect("scrolling background requires a scroll container")
        .scrolling_background_visual_rect;
    PaintRect {
        x: node.paint_offset.x + local.offset.x,
        y: node.paint_offset.y + local.offset.y,
        width: local.size.width,
        height: local.size.height,
    }
}

// cpp: paint/box_model_object_painter.cc:39-60
// cpp: paint/box_fragment_painter.cc:2177-2202
fn LocalBackgroundBorderRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let mut rect = ContentsBackgroundBorderRect(node);
    // BoxModelObjectPainter::AdjustRectForScrolledContent and the fragment
    // equivalent both consume the source pixel-snapped scroll displacement.
    // Border-box painting has no ScrollTranslation property, so apply that
    // displacement explicitly to the enlarged local-background rectangle.
    rect.x -= fragment.paint.pixel_snapped_scrolled_content_offset.x;
    rect.y -= fragment.paint.pixel_snapped_scrolled_content_offset.y;
    rect
}

fn BackgroundPositioningBoxRect(node: &PaintTreeNode<'_>, box_type: BackgroundBox) -> PaintRect {
    let paint = &node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .paint;
    let box_type = UsedBackgroundGeometryBox(box_type);
    if let Some(page) = paint
        .is_page_container
        .then_some(paint.page_container_background_geometry)
        .flatten()
    {
        let mut rect = PaintRect {
            x: page.root_element_paint_offset.x + page.positioning_box_offset.x,
            y: page.root_element_paint_offset.y + page.positioning_box_offset.y,
            width: page.positioning_size.width,
            height: page.positioning_size.height,
        };
        if box_type == BackgroundBox::kBorderBox {
            return rect;
        }
        rect = ContractPaintRect(rect, page.border);
        if box_type == BackgroundBox::kPaddingBox {
            return rect;
        }
        return ContractPaintRect(rect, page.padding);
    }
    if let Some(page) = paint
        .is_page_border_box
        .then_some(paint.page_document_background_geometry)
        .flatten()
    {
        let mut rect = PaintRect {
            x: node.paint_offset.x - page.positioning_area_offset.x,
            y: node.paint_offset.y - page.positioning_area_offset.y,
            width: page.positioning_size.width,
            height: page.positioning_size.height,
        };
        if box_type == BackgroundBox::kBorderBox {
            return rect;
        }
        rect = ContractPaintRect(rect, page.border);
        if box_type == BackgroundBox::kPaddingBox {
            return rect;
        }
        return ContractPaintRect(rect, page.padding);
    }
    let Some(stitched) = paint.stitched_decoration.as_ref() else {
        return BackgroundBoxRect(node, box_type);
    };
    let mut rect = PaintRect {
        x: node.paint_offset.x + stitched.fragment_origin.x - stitched.fragment_offset.x,
        y: node.paint_offset.y + stitched.fragment_origin.y - stitched.fragment_offset.y,
        width: stitched.size.width,
        height: stitched.size.height,
    };
    if box_type == BackgroundBox::kBorderBox {
        return rect;
    }
    rect = PaintRect {
        x: rect.x + paint.border.left,
        y: rect.y + paint.border.top,
        width: (rect.width - paint.border.left - paint.border.right).max(0.0),
        height: (rect.height - paint.border.top - paint.border.bottom).max(0.0),
    };
    if box_type == BackgroundBox::kPaddingBox {
        return rect;
    }
    PaintRect {
        x: rect.x + paint.padding.left,
        y: rect.y + paint.padding.top,
        width: (rect.width - paint.padding.left - paint.padding.right).max(0.0),
        height: (rect.height - paint.padding.top - paint.padding.bottom).max(0.0),
    }
}

fn BackgroundInsets(node: &PaintTreeNode<'_>, box_type: BackgroundBox) -> Edges {
    let style = &node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .paint;
    let box_type = UsedBackgroundGeometryBox(box_type);
    if box_type == BackgroundBox::kBorderBox {
        return Edges::default();
    }
    let mut inset = IncludedBorderEdges(style);
    if box_type == BackgroundBox::kContentBox {
        inset.top += style.padding.top;
        inset.right += style.padding.right;
        inset.bottom += style.padding.bottom;
        inset.left += style.padding.left;
    }
    inset
}

pub(crate) fn BackgroundRadii(
    node: &PaintTreeNode<'_>,
    box_type: BackgroundBox,
) -> PaintCornerRadii {
    let decoration = DecorationRect(node);
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    ResolveCornerRadii(
        &fragment.paint.style,
        &fragment.paint.border_sides,
        Size {
            width: decoration.width,
            height: decoration.height,
        },
        BackgroundInsets(node, box_type),
    )
}

fn BackgroundRadiiForRect(
    node: &PaintTreeNode<'_>,
    box_type: BackgroundBox,
    rect: PaintRect,
) -> PaintCornerRadii {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    ResolveCornerRadii(
        &fragment.paint.style,
        &fragment.paint.border_sides,
        Size {
            width: rect.width
                + BackgroundInsets(node, box_type).left
                + BackgroundInsets(node, box_type).right,
            height: rect.height
                + BackgroundInsets(node, box_type).top
                + BackgroundInsets(node, box_type).bottom,
        },
        BackgroundInsets(node, box_type),
    )
}

fn BackgroundBleedInset(node: &PaintTreeNode<'_>) -> Option<Edges> {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let style = &fragment.paint.style;
    if !BackgroundRadii(node, BackgroundBox::kBorderBox).HasRadius()
        || fragment.paint.source_kind == NodeKind::kFieldset
    {
        return None;
    }
    let border = IncludedBorderEdges(&fragment.paint);
    let widths = [border.top, border.right, border.bottom, border.left];
    let mut fraction = 0.5;
    for (edge, width) in widths.into_iter().enumerate() {
        let border_style = fragment.paint.border_styles[edge];
        if width <= 0.0
            || style.border_colors()[edge].alpha < 1.0
            || border_style == BorderLineStyle::kNone
            || border_style == BorderLineStyle::kDashed
            || border_style == BorderLineStyle::kDotted
        {
            return None;
        }
        if border_style == BorderLineStyle::kDouble {
            fraction = 1.0 / 6.0;
        }
    }
    Some(Edges {
        top: border.top * fraction,
        right: border.right * fraction,
        bottom: border.bottom * fraction,
        left: border.left * fraction,
    })
}

pub(crate) fn BackgroundLayerOccludesFollowingLayers(layer: &BackgroundImageLayer) -> bool {
    layer.image_occludes_next_layers
}

fn FindImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> &'a PaintImage {
    FindAvailableImage(node, id).unwrap_or_else(|| {
        panic!("paint image must exist with dimensions, scale and width*height RGBA bytes")
    })
}

fn FindAvailableReplacedImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> Option<&'a PaintImage> {
    FindAvailableImage(node, id)
}

fn FindAvailableImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> Option<&'a PaintImage> {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(resources) = fragment.paint.resources.as_ref() else {
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            eprintln!("paint-image-profile id={id} resources=none");
        }
        return None;
    };
    let images = &resources.images;
    let Some(found) = images.iter().find(|image| image.id == id) else {
        if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
            eprintln!(
                "paint-image-profile id={id} missing catalog_len={} catalog_ids={:?}",
                images.len(),
                images.iter().map(|image| image.id).collect::<Vec<_>>()
            );
        }
        return None;
    };
    let available = IsPaintImageAvailable(found);
    if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() && !available {
        eprintln!(
            "paint-image-profile id={id} unavailable size={}x{} scale={} bitmap_bytes={:?} document={}",
            found.width,
            found.height,
            found.resolution_scale,
            found.BitmapPixels().map(|pixels| pixels.len()),
            found.IsDocumentImage()
        );
    }
    available.then_some(found)
}

fn IsPaintImageAvailable(image: &PaintImage) -> bool {
    let bitmap_storage_is_valid = image.BitmapPixels().is_none_or(|pixels| {
        (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            == Some(pixels.len())
    });
    image.width != 0
        && image.height != 0
        && image.resolution_scale.is_finite()
        && image.resolution_scale > 0.0
        && bitmap_storage_is_valid
}

// Source: text_painter.cc:361-369,801-815. Decorations remain outside this
// state; shadows and glyph passes enter it independently in source paint order.
fn AppendTextFitTransform(
    context: &RefCell<PaintContext<'_>>,
    node: &PaintTreeNode<'_>,
    phase: PaintPhase,
    scale: f64,
    text_origin: Offset,
    begin: bool,
) {
    if !scale.is_finite() || scale <= 0.0 {
        panic!("text-fit scale must be finite and positive");
    }
    if !begin {
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase,
                node_id: node.fragment.as_deref().map_or(0, |f| f.node_id),
                ..Default::default()
            },
            node,
        );
        return;
    }
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kSave,
            phase,
            node_id: fragment.node_id,
            ..Default::default()
        },
        node,
    );
    let transform = ComputeTextFitTransform(scale, text_origin);
    context.borrow_mut().Append(
        DisplayItem {
            r#type: DisplayItemType::kConcat,
            phase,
            node_id: fragment.node_id,
            transform,
            ..Default::default()
        },
        node,
    );
}

// Source: text_painter.cc:463-487,989-999. ShapeResult::Slice retains every
// glyph in a cluster that intersects the selected character range. The clip
// then splits a partially selected ligature geometrically.
fn SliceGlyphsForSelection(
    glyphs: &Arc<Vec<PaintGlyph>>,
    selection: (u32, u32),
    text_end: u32,
) -> Arc<Vec<PaintGlyph>> {
    if glyphs.is_empty() {
        return Arc::clone(glyphs);
    }
    let selection = (selection.0.min(selection.1), selection.0.max(selection.1));
    let mut cluster_starts: Vec<u32> = glyphs.iter().map(|glyph| glyph.character_index).collect();
    cluster_starts.sort_unstable();
    cluster_starts.dedup();
    let selected: Vec<PaintGlyph> = glyphs
        .iter()
        .filter(|glyph| {
            let cluster_start = glyph.character_index;
            let next = cluster_starts.partition_point(|start| *start <= cluster_start);
            let cluster_end = cluster_starts
                .get(next)
                .copied()
                .unwrap_or_else(|| text_end.max(cluster_start.saturating_add(1)));
            cluster_start < selection.1 && cluster_end > selection.0
        })
        .cloned()
        .collect();
    if selected.len() == glyphs.len() {
        Arc::clone(glyphs)
    } else {
        Arc::new(selected)
    }
}

// Source: highlight_painter.cc:952-986,1112-1131. For ordinary horizontal
// text, retain the line-relative range through stroke inflation and the
// fragment-edge ink-overflow adjustments, then apply ClipToPartRect's source
// unit/non-unit used-font block offset and height.
fn SelectionGlyphClipRect(
    rect: PaintRect,
    line_relative_range: Option<PaintRect>,
    stroke_width: f64,
    selection: (u32, u32),
    fragment_range: Option<(u32, u32)>,
    ink_overflow: Option<layoutng_assembly::fragment_tree::FragmentPaintRect>,
    item: Option<&layoutng_assembly::fragment_item::FragmentItem>,
    physical_box_offset: Offset,
    physical_box_size: Size,
) -> PaintRect {
    let stroke_outset = (stroke_width / 2.0).ceil().max(0.0);
    let fallback = || PaintRect {
        x: rect.x - stroke_outset,
        y: rect.y - stroke_outset,
        width: rect.width + stroke_outset * 2.0,
        height: rect.height + stroke_outset * 2.0,
    };
    let (
        Some(item),
        Some(ink_overflow),
        Some((fragment_start, fragment_end)),
        Some(line_relative_range),
    ) = (item, ink_overflow, fragment_range, line_relative_range)
    else {
        return fallback();
    };
    if item.IsSvgText() {
        return fallback();
    }

    // highlight_painter.cc:50-60,1065-1069: the range comes from source
    // ShapeResult positions in line-relative space, then reuses the physical
    // text-box origin as its line-relative origin.
    let mut line_rect = LineRelativeRect::Create(
        &PhysicalRect::new(
            PhysicalOffset::new(
                LayoutUnit::from_f64(line_relative_range.x),
                LayoutUnit::from_f64(line_relative_range.y),
            ),
            PhysicalSize::new(
                LayoutUnit::from_f64(line_relative_range.width),
                LayoutUnit::from_f64(line_relative_range.height),
            ),
        ),
        None,
    );
    line_rect.Inflate(LayoutUnit::FromFloatCeil(stroke_width as f32 / 2.0));
    if selection.0 == fragment_start {
        line_rect.AdjustLineStartToInkOverflow(item);
    }
    if selection.1 == fragment_end {
        line_rect.AdjustLineEndToInkOverflow(item);
    }
    let mut line_rect = line_rect.EnclosingLineRelativeRect();
    let used_font = item.GetUsedFont();
    let scale = f64::from(used_font.ScalingFactor());
    if scale != 1.0 {
        let primary = used_font.PrimaryFont();
        if primary.is_null() {
            return fallback();
        }
        let unscaled_ascent = f64::from(unsafe { &*primary }.GetFontMetrics().FloatAscent());
        let local_y_scaled = item.InkOverflowRect().Y().ToDouble();
        line_rect.offset.line_over +=
            LayoutUnit::from_f64(local_y_scaled / scale - unscaled_ascent * (1.0 - scale));
        line_rect.size.block_size /= scale as f32;
    } else {
        line_rect.offset.line_over += LayoutUnit::from_f64(ink_overflow.offset.y);
    }
    let line_rect = line_rect.ToRectF();
    let physical = if item.IsHorizontal() {
        line_rect
    } else {
        let rotation = TextWritingModeRotation(
            item.GetWritingMode(),
            physical_box_offset,
            physical_box_size,
        );
        rotation.MapRect(line_rect)
    };
    PaintRect {
        x: f64::from(physical.x()),
        y: f64::from(physical.y()),
        width: f64::from(physical.width()),
        height: f64::from(physical.height()),
    }
}

// Source: text_painter.cc:474-488; shadow_list.cc:39-52;
// shadow_data.cc:32-43. Keep the original selection rect and take the maximum
// non-inset raster outset on each side across the complete shadow list.
fn AdjustRectForShadow(mut rect: PaintRect, style: &PaintStyle) -> PaintRect {
    let mut left: f64 = 0.0;
    let mut right: f64 = 0.0;
    let mut top: f64 = 0.0;
    let mut bottom: f64 = 0.0;
    for index in 0..style.text_shadow_count() {
        let shadow = style.text_shadow(index);
        if shadow.inset {
            continue;
        }
        let blur_and_spread = (1.5 * shadow.blur_radius).ceil() + shadow.spread;
        left = left.max(blur_and_spread - shadow.offset.x);
        right = right.max(blur_and_spread + shadow.offset.x);
        top = top.max(blur_and_spread - shadow.offset.y);
        bottom = bottom.max(blur_and_spread + shadow.offset.y);
    }
    rect.x -= left;
    rect.y -= top;
    rect.width += left + right;
    rect.height += top + bottom;
    rect
}

// Source: text_painter.cc:361-369.
fn ComputeTextFitTransform(scale: f64, text_origin: Offset) -> TransformMatrix {
    let mut transform = TransformMatrix::default();
    transform.values[0] = scale;
    transform.values[5] = scale;
    transform.values[12] = text_origin.x - scale * text_origin.x;
    transform.values[13] = text_origin.y - scale * text_origin.y;
    transform
}

fn TextWritingModeTransform(
    node: &PaintTreeNode<'_>,
    writing_mode: WritingMode,
    physical_box_offset: Offset,
) -> TransformMatrix {
    if writing_mode == WritingMode::kHorizontalTb {
        return TransformMatrix::default();
    }
    let size = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .size;
    let writing_mode = match writing_mode {
        WritingMode::kHorizontalTb => StyleWritingMode::kHorizontalTb,
        WritingMode::kVerticalRl => StyleWritingMode::kVerticalRl,
        WritingMode::kVerticalLr => StyleWritingMode::kVerticalLr,
        WritingMode::kSidewaysLr => StyleWritingMode::kSidewaysLr,
    };
    let rotation = TextWritingModeRotation(writing_mode, physical_box_offset, size);
    let mut transform = TransformMatrix::default();
    transform.values[0] = rotation.A();
    transform.values[1] = rotation.B();
    transform.values[4] = rotation.C();
    transform.values[5] = rotation.D();
    transform.values[12] = rotation.E();
    transform.values[13] = rotation.F();
    transform
}

// text_fragment_painter.cc:383-393. The line box shares its origin between
// physical and line-relative spaces. Chromium callers retain the five-valued
// mode, so LineRelativeRect also preserves the sideways-lr distinction.
fn TextWritingModeRotation(
    writing_mode: StyleWritingMode,
    physical_box_offset: Offset,
    physical_box_size: Size,
) -> AffineTransform {
    let physical_box = PhysicalRect::new(
        PhysicalOffset::new(
            LayoutUnit::from_f64(physical_box_offset.x),
            LayoutUnit::from_f64(physical_box_offset.y),
        ),
        PhysicalSize::new(
            LayoutUnit::from_f64(physical_box_size.width),
            LayoutUnit::from_f64(physical_box_size.height),
        ),
    );
    LineRelativeRect::CreateFromLineBox(&physical_box, false)
        .ComputeRelativeToPhysicalTransform(writing_mode)
}

fn RootSpaceTextTransform(node: &PaintTreeNode<'_>) -> TransformMatrix {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(svg) = fragment.paint.svg_text.as_ref() else {
        return TransformMatrix::default();
    };
    if !svg.has_transform {
        return TransformMatrix::default();
    }
    let local = MultiplyTransforms(
        &TranslationTransform(node.paint_offset.x, node.paint_offset.y),
        &MultiplyTransforms(
            &svg.local_transform,
            &TranslationTransform(-node.paint_offset.x, -node.paint_offset.y),
        ),
    );
    if svg.scaling_factor == 1.0 {
        return local;
    }
    if !svg.scaling_factor.is_finite() || svg.scaling_factor <= 0.0 {
        panic!("SVG text scaling factor must be positive");
    }
    let mut scale = TransformMatrix::default();
    scale.values[0] = 1.0 / svg.scaling_factor;
    scale.values[5] = 1.0 / svg.scaling_factor;
    MultiplyTransforms(&scale, &local)
}

pub(crate) fn TextPaintOrigin(node: &PaintTreeNode<'_>) -> Offset {
    let mut origin = node.paint_offset;
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    if let Some(svg) = fragment.paint.svg_text.as_ref() {
        origin.x += svg.glyph_origin.x;
        origin.y += svg.glyph_origin.y;
    }
    origin.x += fragment.paint.text_combine_inline_offset;
    origin
}

#[cfg(test)]
mod background_attachment_tests {
    use std::cell::RefCell;
    use std::sync::Arc;

    use layoutng_assembly::fragment_tree::{
        CanvasBackgroundPaintData, FragmentKind, FragmentPaintData, FragmentPaintRect, GapRule,
        GapRulePaintData, GapRuleSide, MathMLPaintData, MathMLPaintKind, PageBackgroundPaintData,
        PageContainerBackgroundPaintData, PaintGlyph, PaintResources, ReplacedContentPaintData,
        ScrollContainerPaintData, TextCombineEmphasisPaintData, TextCombinePaintData,
    };
    use layoutng_assembly::internal::layout_input::{
        BorderLineStyle, Display, Edges, NodeKind, Offset, PaintImage, PaintShadow, PaintStyle,
        Size, ViewportGeometry,
    };
    use layoutng_assembly::internal::layout_input_types::{Color, IntSize};
    use layoutng_assembly::internal::paint_input::{
        BackgroundAttachment, BackgroundBox, BackgroundImageLayer,
    };
    use layoutng_assembly::internal::selection_state::SelectionState;

    use crate::geometry_mapper::TranslationTransform;
    use crate::paint_info::PaintInfo;
    use crate::{PaintPhase, PaintRect};

    use super::{
        AdjustRectForShadow, BackgroundBleedAvoidance, BackgroundBoxRectWithin,
        BackgroundForPainting, BackgroundPositioningBoxRect, BackgroundStyleForPainting,
        BaseBackgroundColor, BorderEdgeObscuresBackground, BoxFragmentPainter, BuildBackplates,
        CanvasBackgroundPaintState, CanvasBackgroundPositioningBoxRect,
        CanvasBackgroundPositioningBoxRectInState, CanvasBackgroundRectInState,
        ComputeTextFitTransform, CropBackgroundSourceRect, EffectiveBackgroundAttachment,
        FixedBackgroundPositioningArea, GetHitTestOpaqueness, ImageDecodingModeForPainting,
        IsPaintImageAvailable, IsVisibleToPaint, LocalBackgroundBorderRect,
        MissingImagePlaceholderRect, PaintTreeNode, PaintsCaretAfterChildren,
        PaintsCaretBeforeChildren, PaintsOverflowControlsBeforeContents,
        PaintsSelfOutlineAfterChildren, PaintsSelfOutlineBeforeChildren, PaintsTextInPhase,
        PixelSnappedBorderBoxRect, ResolveBackgroundDestinationRects,
        ScrollingBackgroundVisualRect, ShouldForceWhiteBackgroundForPrintEconomy,
        ShouldRecordHitTestData, ShouldRecordScrollHitTestDataForPaint,
        ShouldSkipPaintUnderInvalidationChecking, SliceGlyphsForSelection, SvgSpriteVisualRect,
        TextColorForWhiteBackground, UpdateImagePaintedRect, UseCachedImageDrawingIfPossible,
        UsedBackgroundGeometryBox,
    };

    #[test]
    fn border_box_metadata_pixel_snaps_fractional_paint_offset() {
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: -0.25, y: 0.0 };
        assert_eq!(
            PixelSnappedBorderBoxRect(
                &node,
                Size {
                    width: 200.0,
                    height: 36.0,
                },
            ),
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 36.0,
            }
        );
    }

    #[test]
    fn print_economy_darkens_near_white_text_and_honors_background_setting() {
        let adjusted = TextColorForWhiteBackground(Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.4,
        });
        assert_eq!(adjusted.red, 171.0 / 255.0);
        assert_eq!(adjusted.green, 171.0 / 255.0);
        assert_eq!(adjusted.blue, 171.0 / 255.0);
        assert_eq!(adjusted.alpha, 0.4);
        let black = Color {
            alpha: 1.0,
            ..Default::default()
        };
        assert_eq!(TextColorForWhiteBackground(black), black);

        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_printing = true;
        assert!(ShouldForceWhiteBackgroundForPrintEconomy(&fragment));
        fragment.paint.should_print_backgrounds = true;
        assert!(!ShouldForceWhiteBackgroundForPrintEconomy(&fragment));
        fragment.paint.should_print_backgrounds = false;
        fragment.paint.style.print_color_adjust_economy = false;
        assert!(!ShouldForceWhiteBackgroundForPrintEconomy(&fragment));
    }

    #[test]
    fn printing_suppresses_only_the_white_base_background() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_printing = true;
        fragment.paint.resources = Some(Arc::new(PaintResources::default()));
        assert_eq!(BaseBackgroundColor(&fragment), None);

        Arc::make_mut(fragment.paint.resources.as_mut().unwrap()).base_background_color = Color {
            red: 0.25,
            green: 0.5,
            blue: 0.75,
            alpha: 1.0,
        };
        assert_eq!(
            BaseBackgroundColor(&fragment),
            Some(Color {
                red: 0.25,
                green: 0.5,
                blue: 0.75,
                alpha: 1.0,
            })
        );

        Arc::make_mut(fragment.paint.resources.as_mut().unwrap())
            .base_background_color
            .alpha = 0.0;
        assert_eq!(BaseBackgroundColor(&fragment), None);
    }

    #[test]
    fn box_under_invalidation_cache_skip_policy_matches_source_cases() {
        use layoutng_assembly::fragment_tree::FormControlPaintData;
        use layoutng_style::style::appearance::AppearanceValue;

        let mut fragment = FragmentPaintData::default();
        assert!(!ShouldSkipPaintUnderInvalidationChecking(&fragment));

        fragment
            .paint
            .pre_paint_invalidation
            .should_delay_full_paint_invalidation = true;
        assert!(ShouldSkipPaintUnderInvalidationChecking(&fragment));
        fragment
            .paint
            .pre_paint_invalidation
            .should_delay_full_paint_invalidation = false;

        fragment.paint.form_control = Some(Arc::new(FormControlPaintData {
            appearance: AppearanceValue::kMediaSlider,
            ..FormControlPaintData::default()
        }));
        assert!(ShouldSkipPaintUnderInvalidationChecking(&fragment));

        fragment.paint.form_control = Some(Arc::new(FormControlPaintData {
            appearance: AppearanceValue::kProgressBar,
            is_progress: true,
            indeterminate: true,
            ..FormControlPaintData::default()
        }));
        assert!(ShouldSkipPaintUnderInvalidationChecking(&fragment));

        fragment.paint.form_control = Some(Arc::new(FormControlPaintData {
            appearance: AppearanceValue::kProgressBar,
            is_progress: false,
            indeterminate: true,
            ..FormControlPaintData::default()
        }));
        assert!(!ShouldSkipPaintUnderInvalidationChecking(&fragment));
    }

    #[test]
    fn delayed_box_background_is_repainted_uncacheably_under_invalidation_checking() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;

        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 23;
        fragment.size = Size {
            width: 30.0,
            height: 20.0,
        };
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            size: fragment.size,
            ..FragmentPaintRect::default()
        });
        fragment.paint.establishes_paint_state = true;
        fragment.paint.has_source = true;
        fragment.paint.has_background = true;
        fragment.paint.style.background_color = Color {
            red: 0.25,
            green: 0.5,
            blue: 0.75,
            alpha: 1.0,
        };
        fragment.paint.display_item_client_id = 23;
        fragment.paint.display_item_client_is_cacheable = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kNone);
        fragment
            .paint
            .pre_paint_invalidation
            .should_delay_full_paint_invalidation = true;

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            context
                .borrow_mut()
                .SetPaintUnderInvalidationCheckingForTesting(true);
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }

        let background = recording
            .display_items
            .iter()
            .find(|item| item.draws_content)
            .expect("box decoration background drawing");
        assert!(!background.is_cacheable);
        assert_eq!(
            background.paint_invalidation_reason,
            Some(foundation::PaintInvalidationReason::kUncacheable)
        );
    }

    #[test]
    fn mathml_operator_suppresses_only_its_inline_fragment_children() {
        let mut parent_fragment = FragmentPaintData::default();
        parent_fragment.paint.mathml = Some(Arc::new(MathMLPaintData {
            kind: MathMLPaintKind::kOperator,
            ..MathMLPaintData::default()
        }));
        let mut parent = PaintTreeNode::default();
        parent.SetOwnedFragmentForTesting(&parent_fragment);

        let mut line_fragment = FragmentPaintData::default();
        line_fragment.kind = layoutng_assembly::fragment_tree::FragmentKind::kLine;
        let mut line = PaintTreeNode::default();
        line.SetOwnedFragmentForTesting(&line_fragment);
        assert!(!BoxFragmentPainter::ShouldPaintChildFragment(
            &parent, &line
        ));

        let box_fragment = FragmentPaintData::default();
        let mut box_child = PaintTreeNode::default();
        box_child.SetOwnedFragmentForTesting(&box_fragment);
        assert!(BoxFragmentPainter::ShouldPaintChildFragment(
            &parent, &box_child
        ));

        Arc::make_mut(parent_fragment.paint.mathml.as_mut().unwrap()).kind =
            MathMLPaintKind::kRadical;
        parent.SetOwnedFragmentForTesting(&parent_fragment);
        assert!(BoxFragmentPainter::ShouldPaintChildFragment(&parent, &line));
    }

    #[test]
    fn page_border_box_paints_the_exported_document_background() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let anonymous_page_color = Color {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let document_color = Color {
            red: 0.0,
            green: 0.25,
            blue: 0.75,
            alpha: 1.0,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 29;
        fragment.size = Size {
            width: 40.0,
            height: 30.0,
        };
        fragment.paint.is_page_border_box = true;
        fragment.paint.style.background_color = anonymous_page_color;
        let mut document_style = PaintStyle::default();
        document_style.background_color = document_color;
        fragment.paint.page_document_background_style = Some(document_style);

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context)
                .PaintBackground(&paint_info, BackgroundBleedAvoidance::kBackgroundBleedNone);
            context.borrow_mut().Finish();
        }

        let background = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawRect)
            .expect("page document background");
        assert_eq!(background.color, document_color);
        assert_ne!(background.color, anonymous_page_color);

        fragment.paint.page_document_background_style = None;
        assert!(BackgroundStyleForPainting(&fragment).is_none());
    }

    #[test]
    fn page_background_geometry_uses_the_stitched_document_and_page_offset() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_page_border_box = true;
        fragment.paint.page_document_background_geometry = Some(PageBackgroundPaintData {
            positioning_size: Size {
                width: 600.0,
                height: 900.0,
            },
            positioning_area_offset: Offset { x: 20.0, y: 400.0 },
            border: Edges {
                top: 7.0,
                right: 11.0,
                bottom: 13.0,
                left: 5.0,
            },
            padding: Edges {
                top: 3.0,
                right: 4.0,
                bottom: 6.0,
                left: 2.0,
            },
            has_background_fixed_to_viewport: false,
        });
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 100.0, y: 200.0 };
        node.SetOwnedFragmentForTesting(&fragment);

        assert_eq!(
            BackgroundPositioningBoxRect(&node, BackgroundBox::kBorderBox),
            PaintRect {
                x: 80.0,
                y: -200.0,
                width: 600.0,
                height: 900.0,
            }
        );
        assert_eq!(
            BackgroundPositioningBoxRect(&node, BackgroundBox::kPaddingBox),
            PaintRect {
                x: 85.0,
                y: -193.0,
                width: 584.0,
                height: 880.0,
            }
        );
        assert_eq!(
            BackgroundPositioningBoxRect(&node, BackgroundBox::kContentBox),
            PaintRect {
                x: 87.0,
                y: -190.0,
                width: 578.0,
                height: 871.0,
            }
        );

        let fixed = BackgroundImageLayer {
            attachment: BackgroundAttachment::kFixed,
            ..BackgroundImageLayer::default()
        };
        assert_eq!(
            EffectiveBackgroundAttachment(&node, &fixed),
            BackgroundAttachment::kScroll
        );
        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .page_document_background_geometry
            .as_mut()
            .unwrap()
            .has_background_fixed_to_viewport = true;
        assert_eq!(
            EffectiveBackgroundAttachment(&node, &fixed),
            BackgroundAttachment::kFixed
        );
    }

    #[test]
    fn page_container_background_uses_page_border_geometry_and_page_clip() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let page_color = Color {
            red: 0.2,
            green: 0.4,
            blue: 0.6,
            alpha: 1.0,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 37;
        fragment.size = Size {
            width: 800.0,
            height: 1000.0,
        };
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_page_container = true;
        fragment.paint.style.background_color = page_color;
        fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: false,
            ..PaintResources::default()
        }));
        fragment.paint.page_container_background_geometry =
            Some(PageContainerBackgroundPaintData {
                positioning_size: Size {
                    width: 620.0,
                    height: 1800.0,
                },
                root_element_paint_offset: Offset { x: 30.0, y: 40.0 },
                positioning_box_offset: Offset { x: 90.0, y: 120.0 },
                border: Edges {
                    top: 7.0,
                    right: 11.0,
                    bottom: 13.0,
                    left: 5.0,
                },
                padding: Edges {
                    top: 3.0,
                    right: 4.0,
                    bottom: 6.0,
                    left: 2.0,
                },
                has_background_fixed_to_viewport: false,
            });
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 10.0, y: 20.0 };
        node.SetOwnedFragmentForTesting(&fragment);

        assert_eq!(
            BackgroundPositioningBoxRect(&node, BackgroundBox::kBorderBox),
            PaintRect {
                x: 120.0,
                y: 160.0,
                width: 620.0,
                height: 1800.0,
            }
        );
        assert_eq!(
            BackgroundPositioningBoxRect(&node, BackgroundBox::kContentBox),
            PaintRect {
                x: 127.0,
                y: 170.0,
                width: 598.0,
                height: 1771.0,
            }
        );
        let fixed = BackgroundImageLayer {
            attachment: BackgroundAttachment::kFixed,
            ..BackgroundImageLayer::default()
        };
        assert_eq!(
            EffectiveBackgroundAttachment(&node, &fixed),
            BackgroundAttachment::kScroll
        );

        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        let draw = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawRect)
            .expect("@page background drawing");
        assert_eq!(draw.color, page_color);
        assert_eq!(
            draw.rect,
            PaintRect {
                x: 10.0,
                y: 20.0,
                width: 800.0,
                height: 1000.0,
            }
        );

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .page_container_background_geometry
            .as_mut()
            .unwrap()
            .positioning_size = Size::default();
        node.fragment.as_mut().unwrap().paint.resources = Some(Arc::new(PaintResources::default()));
        let mut fallback_recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut fallback_recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        let fallback_draws = fallback_recording
            .items
            .iter()
            .filter(|item| item.r#type == DisplayItemType::kDrawRect)
            .collect::<Vec<_>>();
        assert_eq!(fallback_draws.len(), 1);
        assert_eq!(
            fallback_draws[0].color,
            Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            }
        );
    }

    #[test]
    fn paginated_root_does_not_paint_its_canvas_background() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;

        let mut fragment = FragmentPaintData::default();
        fragment.kind = FragmentKind::kBox;
        fragment.size = Size {
            width: 800.0,
            height: 1000.0,
        };
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_layout_view = true;
        fragment.paint.is_paginated_root = true;
        fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: true,
            ..PaintResources::default()
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.is_root = true;

        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(recording.items.is_empty());
        assert!(recording.display_items.is_empty());
    }

    #[test]
    fn print_economy_and_forced_colors_rewrite_background_sources() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_printing = true;
        fragment.paint.style.background_color = Color {
            red: 0.2,
            green: 0.4,
            blue: 0.6,
            alpha: 0.5,
        };
        let (color, paints_images) = BackgroundForPainting(&fragment, &fragment.paint.style, &[]);
        assert_eq!(
            color,
            Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 1.0,
            }
        );
        assert!(!paints_images);

        fragment.paint.style.background_color = Color::default();
        let non_bottom_image = BackgroundImageLayer::default();
        let (color, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&non_bottom_image),
        );
        assert_eq!(color, Color::default());
        assert!(!paints_images);

        let bottom_image = BackgroundImageLayer {
            paints_background_color: true,
            ..Default::default()
        };
        let (color, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&bottom_image),
        );
        assert_eq!(color.alpha, 1.0);
        assert_eq!((color.red, color.green, color.blue), (1.0, 1.0, 1.0));
        assert!(!paints_images);

        fragment.paint.is_page_border_box = true;
        let (color, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&bottom_image),
        );
        assert_eq!(color, Color::default());
        assert!(!paints_images);
        fragment.paint.is_page_border_box = false;

        fragment.paint.should_print_backgrounds = true;
        let (color, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&bottom_image),
        );
        assert_eq!(color, Color::default());
        assert!(paints_images);

        fragment.paint.in_forced_colors_mode = true;
        fragment.paint.box_fragment_is_inline_box = true;
        let (_, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&bottom_image),
        );
        assert!(!paints_images);
        fragment.paint.box_fragment_is_inline_box = false;
        let (_, paints_images) = BackgroundForPainting(
            &fragment,
            &fragment.paint.style,
            std::slice::from_ref(&bottom_image),
        );
        assert!(paints_images);

        fragment.paint.in_forced_colors_mode = false;
        fragment.paint.style.force_dark = true;
        fragment.paint.style.background_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 0.5,
        };
        let (color, _) = BackgroundForPainting(&fragment, &fragment.paint.style, &[]);
        assert!((color.red - 18.0 / 255.0).abs() < 0.00001);
        assert_eq!(color.alpha, 0.5);
    }

    #[test]
    fn background_image_admission_checks_renderability_and_privacy_origin() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;
        use image_resource::{PaintImage, PaintImageContent};
        use paint_model::paint_flags::PaintFlag;

        let mut fragment = FragmentPaintData::default();
        fragment.paint.resources = Some(Arc::new(PaintResources {
            images: vec![PaintImage {
                id: 7,
                width: 1,
                height: 1,
                is_cors_same_origin: false,
                content: PaintImageContent::Bitmap(Arc::new(vec![0, 0, 0, 255])),
                ..Default::default()
            }],
            ..Default::default()
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let ordinary = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
        let private = PaintInfo::new_with_flags(
            &context,
            PaintPhase::kSelfBlockBackgroundOnly,
            false,
            PaintFlag::kPrivacyPreserving,
        );
        let painter = BoxFragmentPainter::new(&node, &context);
        let mut layer = BackgroundImageLayer {
            resource_id: 7,
            can_render: true,
            ..Default::default()
        };
        assert!(painter.CanPaintBackgroundImage(&layer, &ordinary));
        assert!(!painter.CanPaintBackgroundImage(&layer, &private));
        layer.can_render = false;
        assert!(!painter.CanPaintBackgroundImage(&layer, &ordinary));
    }

    #[test]
    fn background_destination_contracts_only_obscuring_border_edges() {
        let opaque = Color {
            alpha: 1.0,
            ..Color::default()
        };
        let mut fragment = FragmentPaintData::default();
        fragment.paint.border = Edges {
            top: 1.2,
            right: 2.2,
            bottom: 4.0,
            left: 3.4,
        };
        fragment.paint.border_styles = [
            BorderLineStyle::kSolid,
            BorderLineStyle::kDashed,
            BorderLineStyle::kDouble,
            BorderLineStyle::kGroove,
        ];
        fragment.paint.style.border_colors = [opaque; 4];
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.pixel_snap_scale = 1.0;
        let layer = BackgroundImageLayer {
            clip: BackgroundBox::kBorderBox,
            ..BackgroundImageLayer::default()
        };

        let destination = ResolveBackgroundDestinationRects(
            &node,
            &layer,
            PaintPhase::kSelfBlockBackgroundOnly,
            layoutng_assembly::internal::paint_input::BackgroundAttachment::kScroll,
            PaintRect {
                x: 0.25,
                y: 0.25,
                width: 100.0,
                height: 80.0,
            },
            false,
        );

        assert_eq!(
            destination.unsnapped,
            PaintRect {
                x: 3.65,
                y: 1.45,
                width: 96.6,
                height: 78.8,
            }
        );
        assert_eq!(
            destination.snapped,
            PaintRect {
                x: 4.0,
                y: 1.0,
                width: 96.0,
                height: 79.0,
            }
        );
    }

    #[test]
    fn background_destination_rejects_unsafe_border_derived_adjustment() {
        let opaque = Color {
            alpha: 1.0,
            ..Color::default()
        };
        let mut fragment = FragmentPaintData::default();
        fragment.paint.border = Edges {
            top: 5.0,
            right: 5.0,
            bottom: 5.0,
            left: 5.0,
        };
        fragment.paint.border_styles = [BorderLineStyle::kSolid; 4];
        fragment.paint.style.border_colors = [opaque; 4];
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let layer = BackgroundImageLayer {
            clip: BackgroundBox::kBorderBox,
            ..BackgroundImageLayer::default()
        };
        let rect = PaintRect {
            x: 0.25,
            y: 0.25,
            width: 20.0,
            height: 20.0,
        };

        let destination = ResolveBackgroundDestinationRects(
            &node,
            &layer,
            PaintPhase::kSelfBlockBackgroundOnly,
            layoutng_assembly::internal::paint_input::BackgroundAttachment::kScroll,
            rect,
            true,
        );
        assert_eq!(destination.unsnapped, rect);
        assert_eq!(destination.snapped, node.PixelSnappedRect(&rect));
    }

    #[test]
    fn background_geometry_aliases_and_subset_crop_match_chromium() {
        assert_eq!(
            UsedBackgroundGeometryBox(BackgroundBox::kFillBox),
            BackgroundBox::kContentBox
        );
        assert_eq!(
            UsedBackgroundGeometryBox(BackgroundBox::kStrokeBox),
            BackgroundBox::kBorderBox
        );
        assert!(BorderEdgeObscuresBackground(
            true,
            2.0,
            Color {
                alpha: 1.0,
                ..Color::default()
            },
            BorderLineStyle::kDouble,
        ));
        assert!(!BorderEdgeObscuresBackground(
            true,
            3.0,
            Color {
                alpha: 1.0,
                ..Color::default()
            },
            BorderLineStyle::kDouble,
        ));
        assert_eq!(
            CropBackgroundSourceRect(
                PaintRect {
                    width: 200.0,
                    height: 100.0,
                    ..PaintRect::default()
                },
                PaintRect {
                    x: 10.0,
                    y: 20.0,
                    width: 100.0,
                    height: 50.0,
                },
                PaintRect {
                    x: 20.0,
                    y: 25.0,
                    width: 50.0,
                    height: 25.0,
                },
                PaintRect {
                    x: 20.0,
                    y: 25.0,
                    width: 50.0,
                    height: 25.0,
                },
                Size {
                    width: 100.0,
                    height: 50.0,
                },
            ),
            PaintRect {
                x: 20.0,
                y: 10.0,
                width: 100.0,
                height: 50.0,
            }
        );
    }

    #[test]
    fn forced_dark_box_shadow_records_the_inverted_background_role_color() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 40.0,
            height: 40.0,
        };
        fragment.paint.style.force_dark = true;
        fragment.paint.style.box_shadows = vec![PaintShadow {
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
                alpha: 0.5,
            },
            offset: Offset { x: 1.0, y: 1.0 },
            blur_radius: 2.0,
            ..Default::default()
        }];
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        BoxFragmentPainter::new(&node, &context)
            .PaintBoxShadows(PaintPhase::kSelfBlockBackgroundOnly, false);

        drop(context);
        let shadow = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawBoxShadow)
            .expect("box shadow draw");
        assert!((shadow.color.red - 18.0 / 255.0).abs() < 0.00001);
        assert_eq!(shadow.color.alpha, 0.5);
    }

    #[test]
    fn gap_rules_preserve_resolved_overlap_order_and_snap_in_paint_space() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let first_color = Color {
            red: 1.0,
            alpha: 1.0,
            ..Default::default()
        };
        let second_color = Color {
            blue: 1.0,
            alpha: 1.0,
            ..Default::default()
        };
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 80.0,
            height: 60.0,
        };
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: fragment.size,
        });
        fragment.paint.gap_rules = Some(Arc::new(GapRulePaintData {
            rules: vec![
                GapRule {
                    offset: Offset { x: 9.2, y: 1.2 },
                    size: Size {
                        width: 2.0,
                        height: 30.0,
                    },
                    color: first_color,
                    line_style: BorderLineStyle::kSolid,
                    side: GapRuleSide::Left,
                },
                GapRule {
                    offset: Offset { x: 1.2, y: 19.2 },
                    size: Size {
                        width: 40.0,
                        height: 3.0,
                    },
                    color: second_color,
                    line_style: BorderLineStyle::kDashed,
                    side: GapRuleSide::Top,
                },
            ],
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 0.4, y: 0.4 };
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        BoxFragmentPainter::new(&node, &context)
            .PaintGapRules(PaintPhase::kSelfBlockBackgroundOnly);
        drop(context);

        let rules = recording
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item.r#type,
                    DisplayItemType::kDrawRect | DisplayItemType::kStrokeLine
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].color, first_color);
        assert_eq!(rules[1].color, second_color);
        assert_eq!(
            rules[0].rect,
            PaintRect {
                x: 10.0,
                y: 2.0,
                width: 2.0,
                height: 30.0
            }
        );
        assert_eq!(rules[1].r#type, DisplayItemType::kStrokeLine);
        assert_eq!(rules[1].line_style, BorderLineStyle::kDashed);
        assert_eq!(rules[1].stroke_width, 3.0);
        assert_eq!(rules[1].dash_intervals, vec![6.0, 2.5]);
    }

    #[test]
    fn gap_rule_decoration_admission_honors_the_paint_info_skip_flag() {
        use crate::box_decoration_data::BoxDecorationData;
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;

        let mut fragment = FragmentPaintData::default();
        fragment.paint.gap_rules = Some(Arc::new(GapRulePaintData::default()));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let mut paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
        assert!(BoxDecorationData::new(&node, &paint_info).ShouldPaintGapDecorations());
        paint_info.SetSkipsGapDecorations(true);
        assert!(!BoxDecorationData::new(&node, &paint_info).ShouldPaintGapDecorations());
    }

    #[test]
    fn gap_rule_box_side_lowers_double_and_groove_styles() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let color = Color {
            red: 0.5,
            green: 0.5,
            blue: 0.5,
            alpha: 1.0,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 60.0,
        };
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: fragment.size,
        });
        fragment.paint.gap_rules = Some(Arc::new(GapRulePaintData {
            rules: vec![
                GapRule {
                    offset: Offset { x: 4.0, y: 5.0 },
                    size: Size {
                        width: 30.0,
                        height: 5.0,
                    },
                    color,
                    line_style: BorderLineStyle::kDouble,
                    side: GapRuleSide::Top,
                },
                GapRule {
                    offset: Offset { x: 50.0, y: 5.0 },
                    size: Size {
                        width: 4.0,
                        height: 30.0,
                    },
                    color,
                    line_style: BorderLineStyle::kGroove,
                    side: GapRuleSide::Left,
                },
            ],
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        BoxFragmentPainter::new(&node, &context)
            .PaintGapRules(PaintPhase::kSelfBlockBackgroundOnly);
        drop(context);

        let fills = recording
            .items
            .iter()
            .filter(|item| item.r#type == DisplayItemType::kDrawRect)
            .collect::<Vec<_>>();
        assert_eq!(fills.len(), 4);
        assert_eq!(fills[0].rect.height, 2.0);
        assert_eq!(fills[1].rect.y, 8.0);
        assert_eq!(fills[1].rect.height, 2.0);
        assert_eq!(fills[2].rect.width, 2.0);
        assert_eq!(fills[3].rect.width, 2.0);
        assert_ne!(fills[2].color, fills[3].color);
    }

    #[test]
    fn gap_rule_dots_use_source_endpoint_and_path_fitting_rules() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let color = Color {
            red: 0.25,
            green: 0.5,
            blue: 0.75,
            alpha: 1.0,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 80.0,
            height: 20.0,
        };
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: fragment.size,
        });
        fragment.paint.gap_rules = Some(Arc::new(GapRulePaintData {
            rules: vec![
                GapRule {
                    offset: Offset { x: 2.0, y: 4.0 },
                    size: Size {
                        width: 8.0,
                        height: 1.0,
                    },
                    color,
                    line_style: BorderLineStyle::kDotted,
                    side: GapRuleSide::Top,
                },
                GapRule {
                    offset: Offset { x: 20.0, y: 4.0 },
                    size: Size {
                        width: 20.0,
                        height: 4.0,
                    },
                    color,
                    line_style: BorderLineStyle::kDotted,
                    side: GapRuleSide::Top,
                },
                GapRule {
                    offset: Offset { x: 50.0, y: 4.0 },
                    size: Size {
                        width: 20.0,
                        height: 2.0,
                    },
                    color,
                    line_style: BorderLineStyle::kDouble,
                    side: GapRuleSide::Top,
                },
            ],
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        BoxFragmentPainter::new(&node, &context)
            .PaintGapRules(PaintPhase::kSelfBlockBackgroundOnly);
        drop(context);

        assert_eq!(recording.items.len(), 4);
        assert_eq!(recording.items[0].r#type, DisplayItemType::kDrawRect);
        assert_eq!(recording.items[0].rect.width, 2.0);
        assert_eq!(recording.items[0].rect.height, 1.0);
        assert_eq!(recording.items[1].r#type, DisplayItemType::kStrokeLine);
        assert_eq!(recording.items[1].rect.x, 5.0);
        assert_eq!(recording.items[1].rect.y, 4.5);
        assert_eq!(recording.items[1].rect.width, 5.0);
        assert_eq!(recording.items[1].dash_intervals, vec![1.0, 1.0]);
        assert!(!recording.items[1].round_cap);
        assert_eq!(recording.items[2].r#type, DisplayItemType::kStrokeLine);
        assert_eq!(recording.items[2].rect.x, 22.0);
        assert_eq!(recording.items[2].rect.width, 16.0);
        assert_eq!(
            recording.items[2].dash_intervals,
            vec![0.0, 7.989999771118164]
        );
        assert!(recording.items[2].round_cap);
        assert_eq!(recording.items[3].r#type, DisplayItemType::kDrawRect);
        assert_eq!(recording.items[3].line_style, BorderLineStyle::kSolid);
        assert_eq!(recording.items[3].rect.height, 2.0);
    }

    #[test]
    fn selection_slice_expands_a_partial_cluster_to_all_of_its_glyphs() {
        let glyphs = Arc::new(vec![
            PaintGlyph {
                id: 10,
                character_index: 0,
                ..Default::default()
            },
            PaintGlyph {
                id: 11,
                character_index: 1,
                ..Default::default()
            },
            PaintGlyph {
                id: 12,
                character_index: 1,
                ..Default::default()
            },
            PaintGlyph {
                id: 13,
                character_index: 3,
                ..Default::default()
            },
        ]);
        let selected = SliceGlyphsForSelection(&glyphs, (2, 3), 4);
        assert_eq!(
            selected.iter().map(|glyph| glyph.id).collect::<Vec<_>>(),
            vec![11, 12]
        );
        let selected = SliceGlyphsForSelection(&glyphs, (3, 4), 4);
        assert_eq!(
            selected.iter().map(|glyph| glyph.id).collect::<Vec<_>>(),
            vec![13]
        );
    }

    #[test]
    fn selection_shadow_clip_uses_maximum_outset_on_each_side() {
        let rect = crate::PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let mut style = PaintStyle::default();
        style.text_shadows = vec![
            PaintShadow {
                offset: Offset { x: 4.0, y: 5.0 },
                ..Default::default()
            },
            PaintShadow {
                offset: Offset { x: -2.0, y: -3.0 },
                blur_radius: 2.0,
                ..Default::default()
            },
        ];
        let adjusted = AdjustRectForShadow(rect, &style);
        assert_eq!(
            adjusted,
            crate::PaintRect {
                x: 5.0,
                y: 14.0,
                width: 39.0,
                height: 51.0,
            }
        );
    }

    #[test]
    fn text_fit_transform_scales_around_the_text_origin() {
        let origin = Offset { x: 24.0, y: 40.0 };
        let transform = ComputeTextFitTransform(1.25, origin);
        assert_eq!(transform.values[0], 1.25);
        assert_eq!(transform.values[5], 1.25);
        assert_eq!(transform.values[12], -6.0);
        assert_eq!(transform.values[13], -10.0);
        assert_eq!(
            transform.values[0] * origin.x + transform.values[12],
            origin.x
        );
        assert_eq!(
            transform.values[5] * origin.y + transform.values[13],
            origin.y
        );
    }

    #[test]
    fn non_overlay_controls_precede_scrolling_background_only_in_background_phases() {
        use crate::PaintPhase;

        assert!(PaintsOverflowControlsBeforeContents(
            PaintPhase::kBlockBackground,
            false,
        ));
        assert!(PaintsOverflowControlsBeforeContents(
            PaintPhase::kSelfBlockBackgroundOnly,
            false,
        ));
        assert!(!PaintsOverflowControlsBeforeContents(
            PaintPhase::kForeground,
            false,
        ));
        assert!(!PaintsOverflowControlsBeforeContents(
            PaintPhase::kBlockBackground,
            true,
        ));
    }

    #[test]
    fn hidden_for_paint_and_css_visibility_are_distinct_for_table_parts() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.style.SetVisibleForTesting(false);
        fragment.paint.display = Display::kBlock;
        assert!(!IsVisibleToPaint(&fragment));

        fragment.paint.display = Display::kTableRow;
        assert!(IsVisibleToPaint(&fragment));
        fragment.paint.display = Display::kTableSection;
        assert!(IsVisibleToPaint(&fragment));
        fragment.paint.display = Display::kTableColumn;
        assert!(IsVisibleToPaint(&fragment));

        fragment.paint.hidden = true;
        assert!(!IsVisibleToPaint(&fragment));
        fragment.paint.style.SetVisibleForTesting(true);
        assert!(!IsVisibleToPaint(&fragment));
    }

    #[test]
    fn text_item_phase_admission_matches_box_fragment_painter() {
        assert!(PaintsTextInPhase(PaintPhase::kForeground));
        assert!(PaintsTextInPhase(PaintPhase::kSelectionDragImage));
        assert!(PaintsTextInPhase(PaintPhase::kTextClip));
        assert!(PaintsTextInPhase(PaintPhase::kMask));
        assert!(!PaintsTextInPhase(PaintPhase::kBlockBackground));
        assert!(!PaintsTextInPhase(PaintPhase::kForcedColorsModeBackplate));
        assert!(!PaintsTextInPhase(PaintPhase::kOutline));
    }

    #[test]
    fn fragment_admission_uses_filtered_ink_overflow_and_keeps_paginated_root() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;

        let filtered = FragmentPaintRect {
            offset: Offset { x: 10.0, y: 10.0 },
            size: Size {
                width: 20.0,
                height: 20.0,
            },
        };
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        let mut fragment = FragmentPaintData::default();
        fragment.paint.filtered_ink_overflow = Some(filtered);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.self_cull_rect = Some(crate::PaintRect {
            x: 100.0,
            y: 100.0,
            width: 10.0,
            height: 10.0,
        });
        assert!(!BoxFragmentPainter::ShouldPaintFragment(&node, &paint_info));

        let mut paginated_fragment = FragmentPaintData::default();
        paginated_fragment.paint.filtered_ink_overflow = Some(filtered);
        paginated_fragment.paint.is_paginated_root = true;
        let mut paginated_node = PaintTreeNode::default();
        paginated_node.SetOwnedFragmentForTesting(&paginated_fragment);
        paginated_node.self_cull_rect = node.self_cull_rect;
        assert!(BoxFragmentPainter::ShouldPaintFragment(
            &paginated_node,
            &paint_info
        ));
    }

    #[test]
    fn child_admission_uses_contents_paint_info_and_source_item_offset() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;
        use paint_model::CullRect;

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new_with_cull(
            &context,
            CullRect::FromRect(crate::PaintRect {
                x: 29.0,
                y: 39.0,
                width: 4.0,
                height: 4.0,
            }),
            PaintPhase::kForeground,
        );
        let mut fragment = FragmentPaintData::default();
        fragment.paint.ink_overflow = Some(FragmentPaintRect {
            offset: Offset { x: 10.0, y: 10.0 },
            size: Size {
                width: 2.0,
                height: 2.0,
            },
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 20.0, y: 30.0 };
        assert!(BoxFragmentPainter::ChildIntersectsCullRect(
            &node,
            &paint_info
        ));

        node.paint_offset = Offset { x: 100.0, y: 100.0 };
        assert!(!BoxFragmentPainter::ChildIntersectsCullRect(
            &node,
            &paint_info
        ));

        let mut line_break = FragmentPaintData::default();
        line_break.paint.ink_overflow = fragment.paint.ink_overflow;
        line_break.paint.is_line_break = true;
        let mut line_break_node = PaintTreeNode::default();
        line_break_node.SetOwnedFragmentForTesting(&line_break);
        line_break_node.paint_offset = node.paint_offset;
        assert!(BoxFragmentPainter::ChildIntersectsCullRect(
            &line_break_node,
            &paint_info
        ));
    }

    #[test]
    fn line_child_cull_rect_checks_only_the_block_direction() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;
        use paint_model::CullRect;

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new_with_cull(
            &context,
            CullRect::FromRect(crate::PaintRect {
                x: 0.0,
                y: 0.0,
                width: 20.0,
                height: 20.0,
            }),
            PaintPhase::kForeground,
        );
        let mut fragment = FragmentPaintData::default();
        fragment.kind = layoutng_assembly::fragment_tree::FragmentKind::kLine;
        fragment.paint.ink_overflow = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: Size {
                width: 10.0,
                height: 10.0,
            },
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);

        node.paint_offset = Offset { x: 100.0, y: 5.0 };
        assert!(BoxFragmentPainter::ChildIntersectsCullRect(
            &node,
            &paint_info
        ));
        node.paint_offset.y = 100.0;
        assert!(!BoxFragmentPainter::ChildIntersectsCullRect(
            &node,
            &paint_info
        ));

        let mut vertical_fragment = fragment.clone();
        vertical_fragment.paint.writing_mode =
            layoutng_assembly::internal::layout_input::WritingMode::kVerticalRl;
        let mut vertical_node = PaintTreeNode::default();
        vertical_node.SetOwnedFragmentForTesting(&vertical_fragment);
        vertical_node.paint_offset = Offset { x: 5.0, y: 100.0 };
        assert!(BoxFragmentPainter::ChildIntersectsCullRect(
            &vertical_node,
            &paint_info
        ));
        vertical_node.paint_offset.x = 100.0;
        assert!(!BoxFragmentPainter::ChildIntersectsCullRect(
            &vertical_node,
            &paint_info
        ));
    }

    #[test]
    fn forced_colors_backplates_split_paragraphs_and_exclude_atomic_inline_text() {
        fn text_fragment(x: f64, width: f64) -> (FragmentPaintData, f64) {
            let mut fragment = FragmentPaintData::default();
            fragment.kind = layoutng_assembly::fragment_tree::FragmentKind::kText;
            fragment.size = Size {
                width,
                height: 10.0,
            };
            (fragment, x)
        }

        fn node<'a>(fragment: &'a FragmentPaintData, x: f64) -> Box<PaintTreeNode<'a>> {
            let mut node = PaintTreeNode::default();
            node.SetOwnedFragmentForTesting(fragment);
            node.paint_offset.x = x;
            Box::new(node)
        }

        let root_fragment = FragmentPaintData::default();
        let (first, first_x) = text_fragment(10.0, 5.0);
        let (mut break_one, break_one_x) = text_fragment(20.0, 0.0);
        break_one.paint.is_line_break = true;
        let (mut break_two, break_two_x) = text_fragment(21.0, 0.0);
        break_two.paint.is_line_break = true;
        let (second, second_x) = text_fragment(30.0, 5.0);
        let (mut one_break, one_break_x) = text_fragment(36.0, 0.0);
        one_break.paint.is_line_break = true;
        let (third, third_x) = text_fragment(40.0, 5.0);
        let atomic_fragment = {
            let mut fragment = FragmentPaintData::default();
            fragment.paint.painted_atomically = true;
            fragment
        };
        let (atomic_text, atomic_text_x) = text_fragment(100.0, 20.0);

        let mut root = PaintTreeNode::default();
        root.SetOwnedFragmentForTesting(&root_fragment);
        root.children.push(node(&first, first_x));
        root.children.push(node(&break_one, break_one_x));
        root.children.push(node(&break_two, break_two_x));
        root.children.push(node(&second, second_x));
        root.children.push(node(&one_break, one_break_x));
        root.children.push(node(&third, third_x));
        let mut atomic = node(&atomic_fragment, 100.0);
        atomic.children.push(node(&atomic_text, atomic_text_x));
        root.children.push(atomic);

        assert_eq!(
            BuildBackplates(&root),
            vec![
                crate::PaintRect {
                    x: 10.0,
                    y: 0.0,
                    width: 5.0,
                    height: 10.0,
                },
                crate::PaintRect {
                    x: 30.0,
                    y: 0.0,
                    width: 15.0,
                    height: 10.0,
                },
            ]
        );
    }

    #[test]
    fn forced_colors_backplate_records_the_exported_document_color() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};

        let color = Color {
            red: 12.0 / 255.0,
            green: 34.0 / 255.0,
            blue: 56.0 / 255.0,
            alpha: 1.0,
        };
        let mut root_fragment = FragmentPaintData::default();
        root_fragment.paint.has_source = true;
        root_fragment.paint.is_inline_formatting_context = true;
        root_fragment.paint.in_forced_colors_mode = true;
        root_fragment.paint.style.forced_color_adjust_auto = true;
        root_fragment.paint.style.forced_background_color = color;
        let (text, text_x) = {
            let mut fragment = FragmentPaintData::default();
            fragment.kind = layoutng_assembly::fragment_tree::FragmentKind::kText;
            fragment.size = Size {
                width: 18.0,
                height: 9.0,
            };
            (fragment, 7.0)
        };
        let mut root = PaintTreeNode::default();
        root.SetOwnedFragmentForTesting(&root_fragment);
        let mut text_node = PaintTreeNode::default();
        text_node.SetOwnedFragmentForTesting(&text);
        text_node.paint_offset = Offset { x: text_x, y: 11.0 };
        root.children.push(Box::new(text_node));

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForcedColorsModeBackplate);
        BoxFragmentPainter::new(&root, &context).Paint(&paint_info);
        drop(paint_info);
        drop(context);

        let draws = recording
            .items
            .iter()
            .filter(|item| item.r#type == DisplayItemType::kDrawRect)
            .collect::<Vec<_>>();
        assert_eq!(draws.len(), 1);
        assert_eq!(draws[0].phase, PaintPhase::kForcedColorsModeBackplate);
        assert_eq!(draws[0].color, color);
        assert_eq!(
            draws[0].rect,
            crate::PaintRect {
                x: 7.0,
                y: 11.0,
                width: 18.0,
                height: 9.0,
            }
        );
    }

    #[test]
    fn root_group_records_the_opaque_base_backdrop_without_a_css_background() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording, PaintRect};
        use paint_model::display_item_id::DisplayItemIdType;

        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 320.0,
            height: 210.0,
        };
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_layout_view = true;
        fragment.paint.style.SetVisibleForTesting(true);
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            size: fragment.size,
            ..FragmentPaintRect::default()
        });
        fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: true,
            ..PaintResources::default()
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.is_root = true;
        let parent = node.contents_properties.nodes.transform.clone();
        node.contents_properties.nodes.transform =
            Arc::new(crate::paint_property_tree::TransformPaintPropertyNode {
                id: 41,
                parent: Some(parent),
                ..(*node.contents_properties.nodes.transform).clone()
            });

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        BoxFragmentPainter::new(&node, &context)
            .PaintRootGroup(PaintPhase::kSelfBlockBackgroundOnly);
        drop(context);

        assert_eq!(recording.display_items.len(), 1);
        assert_eq!(
            recording.display_items[0].id.r#type,
            DisplayItemIdType::kDocumentRootBackdrop
        );
        assert_eq!(
            recording.display_items[0].visual_rect,
            PaintRect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 210.0,
            }
        );
        assert_eq!(recording.items.len(), 1);
        assert_eq!(recording.items[0].r#type, DisplayItemType::kDrawRect);
        assert_eq!(
            recording.items[0].phase,
            PaintPhase::kSelfBlockBackgroundOnly
        );
        assert_eq!(recording.items[0].color.alpha, 1.0);
        assert_eq!(recording.items[0].color.red, 1.0);
        assert!(recording.items[0].source_overwrite);
        assert_eq!(recording.chunks.len(), 1);
        assert_eq!(
            recording.chunks[0].id.r#type,
            DisplayItemIdType::kDocumentRootBackdrop
        );
        assert!(Arc::ptr_eq(
            &recording.chunks[0].properties.transform,
            &node.contents_properties.nodes.transform,
        ));
        assert_eq!(
            recording.chunks[0].bounds,
            recording.display_items[0].visual_rect
        );
        assert_eq!(
            recording.chunks[0].rect_known_to_be_opaque,
            recording.display_items[0].visual_rect,
        );

        let mut skipped = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut skipped));
            let mut paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            paint_info.SetSkipsBackground(true);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(skipped.items.is_empty());
        assert!(skipped.display_items.is_empty());
    }

    #[test]
    fn layout_view_records_only_special_hit_test_data_over_its_source_background_rect() {
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;

        let mut fragment = FragmentPaintData::default();
        fragment.kind = FragmentKind::kBox;
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_layout_view = true;
        fragment.paint.style.SetVisibleForTesting(true);
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            size: fragment.size,
            ..FragmentPaintRect::default()
        });
        fragment.paint.display_item_client_id = 91;
        fragment.paint.view_background_rect = Some(FragmentPaintRect {
            offset: Offset { x: 3.0, y: 4.0 },
            size: Size {
                width: 200.0,
                height: 120.0,
            },
        });
        fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: false,
            ..PaintResources::default()
        }));
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 10.0, y: 20.0 };
        node.is_root = true;
        node.SetOwnedFragmentForTesting(&fragment);

        let mut ordinary = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut ordinary));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(ordinary.chunks.iter().all(|chunk| {
            chunk
                .compositor
                .as_deref()
                .and_then(|metadata| metadata.hit_test_data.as_deref())
                .is_none()
        }));

        node.fragment
            .as_mut()
            .unwrap()
            .paint
            .effective_allowed_touch_action = foundation::TouchAction::kPanX;
        node.fragment.as_mut().unwrap().paint.region_capture_crop_id = Some([17, 19]);
        let mut special = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut special));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        let hit = special
            .chunks
            .iter()
            .find_map(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.hit_test_data.as_deref())
            })
            .expect("LayoutView special hit-test metadata");
        let expected = PaintRect {
            x: 13.0,
            y: 24.0,
            width: 200.0,
            height: 120.0,
        };
        assert_eq!(
            hit.touch_action_rects,
            vec![(expected, foundation::TouchAction::kPanX.bits() as u32)]
        );
        let capture = special
            .chunks
            .iter()
            .find_map(|chunk| {
                chunk
                    .compositor
                    .as_deref()
                    .and_then(|metadata| metadata.region_capture_data.as_deref())
            })
            .expect("LayoutView region-capture metadata");
        assert_eq!(capture.as_slice(), &[([17, 19], expected)]);

        let fragment = node.fragment.as_mut().unwrap();
        fragment.paint.effective_allowed_touch_action = foundation::TouchAction::kAuto;
        fragment.paint.scroll_size = Size {
            width: 300.0,
            height: 180.0,
        };
        fragment.paint.scroll_container = Some(ScrollContainerPaintData {
            compositor_element_id: Default::default(),
            scrollable_axes: [true, true],
            scroll_origin: Offset::default(),
            scroll_position: Offset::default(),
            background_needs_repaint_on_scroll: None,
            prefers_non_composited_scrolling: None,
            scroll_snap_type: None,
            container_rect: FragmentPaintRect::default(),
            scrolling_background_visual_rect: FragmentPaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
            scrolling_background_client_id: 177,
            scrolling_background_client_invalidation_reason: None,
            scrolling_background_client_is_cacheable: true,
            scrolling_background_client_is_just_created: std::cell::Cell::new(false),
        });
        let mut contents = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut contents));
            let mut paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            paint_info.SetIsPaintingBackgroundInContentsSpace(true);
            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        let expected_contents = PaintRect {
            x: 10.0,
            y: 20.0,
            width: 300.0,
            height: 180.0,
        };
        let contents_chunk = contents
            .chunks
            .iter()
            .find(|chunk| chunk.id.client_id == 177)
            .expect("scrolling-background metadata client");
        assert_eq!(contents_chunk.bounds, expected_contents);
        let contents_capture = contents_chunk
            .compositor
            .as_deref()
            .and_then(|metadata| metadata.region_capture_data.as_deref())
            .expect("scrolling-background region-capture metadata");
        assert_eq!(
            contents_capture.as_slice(),
            &[([17, 19], expected_contents)]
        );
    }

    #[test]
    fn scroll_hit_test_metadata_respects_omit_compositing_and_canvas_exception() {
        assert!(ShouldRecordScrollHitTestDataForPaint(false, false, false));
        assert!(!ShouldRecordScrollHitTestDataForPaint(true, false, false));
        assert!(!ShouldRecordScrollHitTestDataForPaint(true, true, false));
        assert!(!ShouldRecordScrollHitTestDataForPaint(true, false, true));
        assert!(ShouldRecordScrollHitTestDataForPaint(true, true, true));
    }

    #[test]
    fn missing_image_placeholder_uses_layout_image_identity_and_half_pixel_inset() {
        let content = crate::PaintRect {
            x: 10.2,
            y: 20.2,
            width: 30.4,
            height: 40.4,
        };
        let snapped = crate::PaintRect {
            x: 10.0,
            y: 20.0,
            width: 31.0,
            height: 41.0,
        };
        assert_eq!(
            MissingImagePlaceholderRect(true, content, snapped),
            Some(crate::PaintRect {
                x: 10.5,
                y: 20.5,
                width: 30.0,
                height: 40.0,
            })
        );
        assert_eq!(MissingImagePlaceholderRect(false, content, snapped), None);
        assert_eq!(
            MissingImagePlaceholderRect(
                true,
                crate::PaintRect {
                    width: 2.0,
                    ..content
                },
                snapped,
            ),
            None
        );
    }

    #[test]
    fn svg_sprite_visual_rect_uses_only_a_fully_contained_local_cull_rect() {
        use image_resource::{
            ContainerKey, DocumentPaintContent, DocumentPaintRecord, PaintImageContent,
        };
        use layoutng_assembly::internal::layout_input::PaintImage;

        let record: Arc<dyn DocumentPaintRecord> = Arc::new(0u8);
        let svg = PaintImage {
            content: PaintImageContent::Document(Arc::new(DocumentPaintContent::new(
                ContainerKey::default(),
                record,
            ))),
            ..Default::default()
        };
        let bitmap = PaintImage {
            content: PaintImageContent::Bitmap(Arc::new(Vec::new())),
            ..Default::default()
        };
        let visual = crate::PaintRect {
            x: 10.25,
            y: 20.25,
            width: 100.0,
            height: 80.0,
        };
        let local_cull = crate::PaintRect {
            x: 30.0,
            y: 40.0,
            width: 20.0,
            height: 10.0,
        };
        assert_eq!(
            SvgSpriteVisualRect(&svg, visual, Some(local_cull)),
            crate::PaintRect {
                x: 30.0,
                y: 40.0,
                width: 20.0,
                height: 10.0,
            }
        );
        assert_eq!(
            SvgSpriteVisualRect(&bitmap, visual, Some(local_cull)),
            visual
        );

        let partially_outside = crate::PaintRect {
            x: 100.0,
            y: 60.0,
            width: 20.0,
            height: 20.0,
        };
        assert_eq!(
            SvgSpriteVisualRect(&svg, visual, Some(partially_outside)),
            visual
        );
    }

    #[test]
    fn replaced_image_availability_rejects_null_or_malformed_frames() {
        use image_resource::PaintImageContent;
        use layoutng_assembly::internal::layout_input::PaintImage;

        assert!(!IsPaintImageAvailable(&PaintImage::default()));
        assert!(!IsPaintImageAvailable(&PaintImage {
            width: 1,
            height: 1,
            content: PaintImageContent::Bitmap(Arc::new(vec![0; 3])),
            ..Default::default()
        }));
        assert!(IsPaintImageAvailable(&PaintImage {
            width: 1,
            height: 1,
            content: PaintImageContent::Bitmap(Arc::new(vec![0; 4])),
            ..Default::default()
        }));
    }

    #[test]
    fn forced_dark_replaced_icon_reaches_the_raster_image_flag() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};
        use image_resource::{PaintImage, PaintImageContent};

        let mut fragment = FragmentPaintData::default();
        fragment.content_size = Size {
            width: 128.0,
            height: 128.0,
        };
        fragment.paint.replaced_is_image = true;
        fragment.paint.replaced_content = Some(ReplacedContentPaintData {
            size: fragment.content_size,
            is_image: true,
            ..Default::default()
        });
        fragment.paint.style.image_resource_id = Some(7);
        fragment.paint.style.force_dark = true;
        fragment.paint.resources = Some(Arc::new(PaintResources {
            images: vec![PaintImage {
                id: 7,
                width: 100,
                height: 100,
                content: PaintImageContent::Bitmap(Arc::new(vec![0; 100 * 100 * 4])),
                ..Default::default()
            }],
            // 128 layout pixels become the 64 CSS-pixel icon threshold.
            layout_zoom_factor: 2.0,
            ..Default::default()
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        let image = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawImageRect)
            .expect("replaced image draw");
        assert!(image.use_dark_mode_for_image);
    }

    #[test]
    fn layout_image_paint_entry_is_recorded_even_without_an_image_drawing() {
        use crate::paint_context::PaintContext;
        use crate::{PaintPhase, PaintRecording};

        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.style.SetVisibleForTesting(true);
        fragment.paint.replaced_is_image = true;
        fragment.paint.style.image_resource_id = Some(7);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        BoxFragmentPainter::new(&node, &context).Paint(&paint_info);

        drop(paint_info);
        drop(context);
        assert_eq!(
            recording
                .PaintedLayoutImageResourceIds()
                .collect::<Vec<_>>(),
            vec![7]
        );
        assert!(recording.items.is_empty());
    }

    #[test]
    fn unavailable_layout_image_resource_paints_the_missing_image_placeholder() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};
        use image_resource::{PaintImage, PaintImageContent};

        let mut fragment = FragmentPaintData::default();
        fragment.content_size = Size {
            width: 30.0,
            height: 40.0,
        };
        fragment.paint.replaced_is_image = true;
        fragment.paint.style.force_dark = true;
        fragment.paint.replaced_content = Some(ReplacedContentPaintData {
            size: fragment.content_size,
            is_image: true,
            ..Default::default()
        });
        fragment.paint.style.image_resource_id = Some(7);
        fragment.paint.resources = Some(Arc::new(PaintResources {
            fonts: Default::default(),
            images: vec![PaintImage {
                id: 7,
                width: 1,
                height: 1,
                content: PaintImageContent::Bitmap(Arc::new(vec![0; 3])),
                ..Default::default()
            }],
            canvases: Vec::new(),
            device_pixel_ratio: 1.0,
            preferred_color_scheme: 0,
            layout_zoom_factor: 1.0,
            base_background_color: Color::default(),
            paints_opaque_base_background: false,
            should_clear_document_background: true,
            viewport: None,
        }));
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
        BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(context);
        let placeholder = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kStrokeRect)
            .expect("missing image placeholder");
        assert!(placeholder.color.red < 211.0 / 255.0);
    }

    #[test]
    fn unavailable_layout_image_skips_placeholder_for_selection_drag_image() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};

        let mut fragment = FragmentPaintData::default();
        fragment.content_size = Size {
            width: 30.0,
            height: 40.0,
        };
        fragment.paint.replaced_is_image = true;
        fragment.paint.replaced_content = Some(ReplacedContentPaintData {
            size: fragment.content_size,
            is_image: true,
            ..Default::default()
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kSelectionDragImage);

        BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(paint_info);
        drop(context);
        assert!(!recording
            .items
            .iter()
            .any(|item| item.r#type == DisplayItemType::kStrokeRect));
        assert!(fragment.paint.image_last_paint_rect.get().is_none());
    }

    #[test]
    fn selection_drag_image_routes_only_selected_replaced_content() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};
        use image_resource::{PaintImage, PaintImageContent};

        for (selection_state, should_draw) in [
            (SelectionState::kNone, false),
            (SelectionState::kStartAndEnd, true),
        ] {
            let mut fragment = FragmentPaintData::default();
            fragment.content_size = Size {
                width: 30.0,
                height: 40.0,
            };
            fragment.paint.has_source = true;
            fragment.paint.establishes_paint_state = true;
            fragment.paint.source_kind = NodeKind::kReplaced;
            fragment.paint.replaced_is_image = true;
            fragment.paint.replaced_selection_state = selection_state;
            fragment.paint.replaced_content = Some(ReplacedContentPaintData {
                size: fragment.content_size,
                is_image: true,
                ..Default::default()
            });
            fragment.paint.style.image_resource_id = Some(7);
            fragment.paint.resources = Some(Arc::new(PaintResources {
                fonts: Default::default(),
                images: vec![PaintImage {
                    id: 7,
                    width: 1,
                    height: 1,
                    content: PaintImageContent::Bitmap(Arc::new(vec![0, 0, 0, 255])),
                    ..Default::default()
                }],
                canvases: Vec::new(),
                device_pixel_ratio: 1.0,
                preferred_color_scheme: 0,
                layout_zoom_factor: 1.0,
                base_background_color: Color::default(),
                paints_opaque_base_background: false,
                should_clear_document_background: true,
                viewport: None,
            }));
            let mut node = PaintTreeNode::default();
            node.SetOwnedFragmentForTesting(&fragment);
            let mut recording = PaintRecording::default();
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelectionDragImage);

            BoxFragmentPainter::new(&node, &context).Paint(&paint_info);

            drop(context);
            assert_eq!(
                recording
                    .items
                    .iter()
                    .any(|item| item.r#type == DisplayItemType::kDrawImageRect),
                should_draw,
                "selection_state={selection_state:?}"
            );
        }
    }

    #[test]
    fn selected_replaced_leaf_paints_exported_selection_rect_after_contents() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};

        let selection_color = Color {
            red: 0.1,
            green: 0.2,
            blue: 0.3,
            alpha: 0.75,
        };
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.establishes_paint_state = true;
        fragment.paint.source_kind = NodeKind::kReplaced;
        fragment.paint.replaced_selection_state = SelectionState::kStartAndEnd;
        fragment.paint.replaced_local_selection_visual_rect = Some(FragmentPaintRect {
            offset: Offset { x: 2.0, y: 3.0 },
            size: Size {
                width: 10.0,
                height: 20.0,
            },
        });
        fragment.paint.pre_paint_invalidation.can_be_selection_leaf = true;
        fragment.paint.selection_style.background_color = selection_color;
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 5.0, y: 7.0 };
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

        BoxFragmentPainter::new(&node, &context).PaintAfterChildren(&paint_info);

        drop(context);
        let tint = recording
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kDrawRect)
            .expect("selection tint");
        assert_eq!(
            tint.rect,
            PaintRect {
                x: 7.0,
                y: 10.0,
                width: 10.0,
                height: 20.0,
            }
        );
        assert_eq!(tint.color, selection_color);
    }

    #[test]
    fn privacy_preserving_layout_image_paint_requires_same_origin_pixels() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintPhase, PaintRecording};
        use image_resource::{PaintImage, PaintImageContent};
        use paint_model::paint_flags::PaintFlag;

        for (is_cors_same_origin, should_draw) in [(false, false), (true, true)] {
            let mut fragment = FragmentPaintData::default();
            fragment.content_size = Size {
                width: 30.0,
                height: 40.0,
            };
            fragment.paint.replaced_is_image = true;
            fragment.paint.replaced_content = Some(ReplacedContentPaintData {
                size: fragment.content_size,
                is_image: true,
                ..Default::default()
            });
            fragment.paint.style.image_resource_id = Some(7);
            fragment.paint.resources = Some(Arc::new(PaintResources {
                fonts: Default::default(),
                images: vec![PaintImage {
                    id: 7,
                    width: 1,
                    height: 1,
                    is_cors_same_origin,
                    content: PaintImageContent::Bitmap(Arc::new(vec![0, 0, 0, 255])),
                    ..Default::default()
                }],
                canvases: Vec::new(),
                device_pixel_ratio: 1.0,
                preferred_color_scheme: 0,
                layout_zoom_factor: 1.0,
                base_background_color: Color::default(),
                paints_opaque_base_background: false,
                should_clear_document_background: true,
                viewport: None,
            }));
            let mut node = PaintTreeNode::default();
            node.SetOwnedFragmentForTesting(&fragment);
            let mut recording = PaintRecording::default();
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new_with_flags(
                &context,
                PaintPhase::kForeground,
                false,
                PaintFlag::kPrivacyPreserving,
            );

            BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

            drop(paint_info);
            drop(context);
            assert_eq!(
                recording
                    .items
                    .iter()
                    .any(|item| item.r#type == DisplayItemType::kDrawImageRect),
                should_draw,
                "same_origin={is_cors_same_origin}"
            );
            assert_eq!(
                fragment.paint.image_last_paint_rect.get().is_some(),
                should_draw,
                "same_origin={is_cors_same_origin}"
            );
        }
    }

    #[test]
    fn under_invalidation_diagnostic_marks_only_animated_layout_image_uncacheable() {
        use crate::paint_context::PaintContext;
        use crate::{PaintPhase, PaintRecording};
        use image_resource::{
            PaintImage, PaintImageAnimationInfo, PaintImageAnimationMetadata,
            PaintImageAnimationType, PaintImageCompletionState, PaintImageContent,
            PaintImageMetadata,
        };

        for (animation_type, should_cache) in [
            (PaintImageAnimationType::Static, true),
            (PaintImageAnimationType::Animated, false),
        ] {
            let mut fragment = FragmentPaintData::default();
            fragment.content_size = Size {
                width: 30.0,
                height: 40.0,
            };
            fragment.paint.replaced_is_image = true;
            fragment.paint.display_item_client_is_cacheable = true;
            fragment.paint.display_item_client_invalidation_reason =
                Some(foundation::PaintInvalidationReason::kNone);
            fragment.paint.replaced_content = Some(ReplacedContentPaintData {
                size: fragment.content_size,
                is_image: true,
                ..Default::default()
            });
            fragment.paint.style.image_resource_id = Some(7);
            fragment.paint.resources = Some(Arc::new(PaintResources {
                fonts: Default::default(),
                images: vec![PaintImage {
                    id: 7,
                    width: 1,
                    height: 1,
                    compositor_metadata: Ok(PaintImageMetadata {
                        animation: PaintImageAnimationMetadata::Known(PaintImageAnimationInfo {
                            animation_type,
                            repetition_count: 0,
                            frame_count: if animation_type == PaintImageAnimationType::Animated {
                                2
                            } else {
                                1
                            },
                        }),
                        completion_state: PaintImageCompletionState::Done,
                        is_multipart: false,
                    }),
                    content: PaintImageContent::Bitmap(Arc::new(vec![0, 0, 0, 255])),
                    ..Default::default()
                }],
                canvases: Vec::new(),
                device_pixel_ratio: 1.0,
                preferred_color_scheme: 0,
                layout_zoom_factor: 1.0,
                base_background_color: Color::default(),
                paints_opaque_base_background: false,
                should_clear_document_background: true,
                viewport: None,
            }));
            let mut node = PaintTreeNode::default();
            node.SetOwnedFragmentForTesting(&fragment);
            let mut recording = PaintRecording::default();
            let context = RefCell::new(PaintContext::new(&mut recording));
            context
                .borrow_mut()
                .SetPaintUnderInvalidationCheckingForTesting(true);
            let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);

            BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

            drop(paint_info);
            drop(context);
            assert_eq!(recording.display_items.len(), 1);
            assert_eq!(recording.display_items[0].is_cacheable, should_cache);
            assert_eq!(
                recording.display_items[0].paint_invalidation_reason,
                Some(if should_cache {
                    foundation::PaintInvalidationReason::kNone
                } else {
                    foundation::PaintInvalidationReason::kUncacheable
                })
            );
        }
    }

    #[test]
    fn layout_image_decoding_mode_tracks_content_transitions_and_explicit_preference() {
        let fragment = FragmentPaintData::default();
        let image_1 = PaintImage {
            paint_image_id: image_resource::PaintImageId(1),
            ..Default::default()
        };
        let image_2 = PaintImage {
            paint_image_id: image_resource::PaintImageId(2),
            ..Default::default()
        };
        assert_eq!(
            ImageDecodingModeForPainting(&fragment, &image_1),
            image_resource::ImageDecodingMode::kUnspecified
        );
        assert_eq!(
            ImageDecodingModeForPainting(&fragment, &image_2),
            image_resource::ImageDecodingMode::kSync,
            "an unspecified content transition is forced synchronous"
        );

        let mut explicit = FragmentPaintData::default();
        explicit.paint.image_decoding_mode = image_resource::ImageDecodingMode::kAsync;
        assert_eq!(
            ImageDecodingModeForPainting(&explicit, &image_1),
            image_resource::ImageDecodingMode::kAsync
        );
        assert_eq!(
            ImageDecodingModeForPainting(&explicit, &image_2),
            image_resource::ImageDecodingMode::kAsync,
            "an authored async preference wins across a content transition"
        );
    }

    #[test]
    fn image_painted_rect_invalidates_only_when_the_retained_value_changes() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.replaced_is_image = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let first = crate::PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        assert!(UpdateImagePaintedRect(&node, first));
        assert_eq!(
            fragment.paint.image_last_paint_rect.get(),
            Some(FragmentPaintRect {
                offset: Offset { x: 10.0, y: 20.0 },
                size: Size {
                    width: 30.0,
                    height: 40.0,
                },
            })
        );
        assert!(!UpdateImagePaintedRect(&node, first));
        assert!(UpdateImagePaintedRect(
            &node,
            crate::PaintRect {
                width: 20.0,
                ..first
            }
        ));
    }

    #[test]
    fn display_lock_blocks_layout_image_content_before_recording() {
        use crate::paint_context::PaintContext;
        use crate::{PaintPhase, PaintRecording};

        let mut fragment = FragmentPaintData::default();
        fragment.paint.replaced_is_image = true;
        fragment.paint.child_paint_blocked_by_display_lock = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));

        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
        BoxFragmentPainter::new(&node, &context).PaintReplaced(&paint_info);

        drop(context);
        assert!(recording.display_items.is_empty());
        assert!(recording.items.is_empty());
        assert_eq!(fragment.paint.image_last_paint_rect.get(), None);
    }

    #[test]
    fn image_drawing_cache_reuses_only_an_unchanged_painted_rect() {
        use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
        use crate::paint_context::PaintContext;
        use crate::{
            DisplayItem, PaintPhase, PaintRecording, RasterEffectOutset, RecordedDisplayItem,
            RecordedDisplayItemKind,
        };

        let mut fragment = FragmentPaintData::default();
        fragment.paint.display_item_client_id = 9;
        fragment.paint.display_item_client_is_cacheable = true;
        fragment.paint.display_item_client_invalidation_reason =
            Some(foundation::PaintInvalidationReason::kNone);
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let id = DisplayItemId {
            client_id: 9,
            r#type: DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kForeground),
            fragment: 0,
        };
        let visual_rect = crate::PaintRect {
            x: 10.0,
            y: 20.0,
            width: 30.0,
            height: 40.0,
        };
        let mut previous = PaintRecording::default();
        previous.items = Arc::new(vec![DisplayItem {
            rect: visual_rect,
            ..Default::default()
        }]);
        previous.display_items.push(RecordedDisplayItem {
            is_cacheable: true,
            is_tombstone: false,
            paint_invalidation_reason: Some(foundation::PaintInvalidationReason::kNone),
            scrollbar: None,
            foreign_layer: None,
            kind: RecordedDisplayItemKind::Drawing,
            id,
            visual_rect,
            visual_rect_is_accurate: true,
            draws_content: true,
            raster_effect_outset: RasterEffectOutset::kNone,
            record_rect_known_to_be_opaque: Default::default(),
            record_background: Default::default(),
            record_has_text: false,
            record_begin: 0,
            record_end: 1,
            scroll_translation: None,
        });
        let previous = Arc::new(previous);

        let mut reused = PaintRecording::default();
        let context = RefCell::new(PaintContext::WithEditingAndCache(
            &mut reused,
            Default::default(),
            Some(Arc::clone(&previous)),
        ));
        assert!(UseCachedImageDrawingIfPossible(
            &context,
            &node,
            PaintPhase::kForeground,
            false,
        ));
        drop(context);
        assert_eq!(reused.display_items.len(), 1);

        let mut changed = PaintRecording::default();
        let context = RefCell::new(PaintContext::WithEditingAndCache(
            &mut changed,
            Default::default(),
            Some(previous),
        ));
        assert!(!UseCachedImageDrawingIfPossible(
            &context,
            &node,
            PaintPhase::kForeground,
            true,
        ));
        drop(context);
        assert!(changed.display_items.is_empty());
    }

    #[test]
    fn table_parts_do_not_record_scroll_hit_test_data() {
        use layoutng_assembly::internal::layout_input::Display;

        assert!(!ShouldRecordHitTestData(Display::kTableRow));
        assert!(!ShouldRecordHitTestData(Display::kTableSection));
        assert!(!ShouldRecordHitTestData(Display::kTableHeaderGroup));
        assert!(!ShouldRecordHitTestData(Display::kTableFooterGroup));
        assert!(ShouldRecordHitTestData(Display::kBlock));
    }

    #[test]
    fn hit_test_opaqueness_uses_source_visibility_and_svg_child_identity() {
        use crate::paint_engine::PaintHitTestOpaqueness;

        let mut fragment = FragmentPaintData::default();
        assert_eq!(
            GetHitTestOpaqueness(&fragment),
            PaintHitTestOpaqueness::Opaque
        );

        fragment.paint.source_kind = NodeKind::kSvgShape;
        fragment.paint.is_svg_child = true;
        assert_eq!(
            GetHitTestOpaqueness(&fragment),
            PaintHitTestOpaqueness::Mixed
        );

        fragment.paint.source_kind = NodeKind::kSvgGroup;
        fragment.paint.is_svg_child = false;
        assert_eq!(
            GetHitTestOpaqueness(&fragment),
            PaintHitTestOpaqueness::Opaque
        );

        fragment.paint.visible_to_hit_testing = false;
        assert_eq!(
            GetHitTestOpaqueness(&fragment),
            PaintHitTestOpaqueness::Transparent
        );
    }

    #[test]
    fn ordinary_outline_is_deferred_but_self_outline_is_direct() {
        use crate::PaintPhase;

        assert!(PaintsSelfOutlineAfterChildren(PaintPhase::kOutline));
        assert!(!PaintsSelfOutlineBeforeChildren(PaintPhase::kOutline));
        assert!(PaintsSelfOutlineBeforeChildren(
            PaintPhase::kSelfOutlineOnly,
        ));
        assert!(!PaintsSelfOutlineAfterChildren(
            PaintPhase::kSelfOutlineOnly,
        ));
        assert!(!PaintsSelfOutlineBeforeChildren(
            PaintPhase::kDescendantOutlinesOnly,
        ));
        assert!(!PaintsSelfOutlineAfterChildren(
            PaintPhase::kDescendantOutlinesOnly,
        ));
    }

    #[test]
    fn exact_single_outline_rect_and_metrics_drive_recording() {
        use crate::paint_context::PaintContext;
        use crate::paint_engine::DisplayItemType;
        use crate::{PaintPhase, PaintRecording, PaintRect};
        use layoutng_assembly::fragment_tree::{FragmentPaintRect, SelfOutlinePaintData};
        use layoutng_assembly::internal::layout_input::{BorderLineStyle, Offset, Size};
        use layoutng_assembly::internal::layout_input_types::Color;

        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 7;
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            size: fragment.size,
            ..FragmentPaintRect::default()
        });
        fragment.paint.style.outline_width = 2.0;
        fragment.paint.style.outline_offset = 0.0;
        fragment.paint.style.outline_style = BorderLineStyle::kSolid;
        fragment.paint.style.outline_color = Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        fragment.paint.style.force_dark = true;
        fragment.paint.self_outline = Some(SelfOutlinePaintData {
            rects: vec![FragmentPaintRect {
                offset: Offset { x: 10.0, y: 20.0 },
                size: Size {
                    width: 30.0,
                    height: 40.0,
                },
            }],
            width: 6.0,
            offset: 2.0,
        });

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 100.0, y: 200.0 };
        let mut output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut output));
            BoxFragmentPainter::new(&node, &context).PaintOutline(PaintPhase::kSelfOutlineOnly);
            context.borrow_mut().Finish();
        }

        let outline = output
            .items
            .iter()
            .find(|item| item.r#type == DisplayItemType::kStrokeRect)
            .expect("exact outline should record a stroke");
        assert_eq!(
            outline.rect,
            PaintRect {
                x: 105.0,
                y: 215.0,
                width: 40.0,
                height: 50.0,
            }
        );
        assert_eq!(outline.stroke_width, 6.0);
        assert!((outline.color.red - 18.0 / 255.0).abs() < 0.00001);
    }

    #[test]
    fn fieldset_mask_dispatch_uses_the_legend_contracted_reference_rect() {
        use crate::display_item_id::DisplayItemIdType;
        use crate::paint_context::PaintContext;
        use crate::PaintRecording;
        use layoutng_assembly::fragment_tree::FieldsetPaintData;
        use layoutng_assembly::internal::paint_input::NinePieceImagePaint;

        let mut fragment = FragmentPaintData::default();
        fragment.node_id = 17;
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.fieldset = Some(Arc::new(FieldsetPaintData {
            border_outsets: Edges {
                top: 7.0,
                right: 4.0,
                bottom: 3.0,
                left: 6.0,
            },
            ..FieldsetPaintData::default()
        }));
        fragment.paint.style.mask_box_image = Some(NinePieceImagePaint {
            outsets: Edges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            ..NinePieceImagePaint::default()
        });

        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 20.0, y: 30.0 };
        node.applies_mask = true;
        let mut output = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut output));
            BoxFragmentPainter::new(&node, &context).PaintMask(
                DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kMask),
            );
            context.borrow_mut().Finish();
        }

        assert_eq!(output.display_items.len(), 1);
        assert_eq!(
            output.display_items[0].visual_rect,
            PaintRect {
                x: 21.0,
                y: 35.0,
                width: 98.0,
                height: 76.0,
            }
        );
        assert!(!output.display_items[0].draws_content);
    }

    #[test]
    fn text_combine_caret_is_always_unscaled_before_children() {
        use layoutng_assembly::internal::caret_rect::CaretShape;

        for shape in [
            CaretShape::kBar,
            CaretShape::kBlock,
            CaretShape::kUnderscore,
        ] {
            assert!(PaintsCaretBeforeChildren(Some(shape), true));
            assert!(!PaintsCaretAfterChildren(Some(shape), true));
        }
        assert!(PaintsCaretBeforeChildren(Some(CaretShape::kBlock), false));
        assert!(PaintsCaretAfterChildren(Some(CaretShape::kBar), false));
        assert!(PaintsCaretAfterChildren(
            Some(CaretShape::kUnderscore),
            false,
        ));
        assert!(!PaintsCaretAfterChildren(None, false));
    }

    #[test]
    fn text_combine_parent_paint_follows_children_and_the_restored_transform() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItem, DisplayItemType, PaintPhase, PaintRecording};

        let mut fragment = FragmentPaintData::default();
        fragment.paint.establishes_paint_state = true;
        fragment.paint.is_printing = true;
        fragment.paint.text_combine = Some(TextCombinePaintData {
            emphasis: Some(TextCombineEmphasisPaintData {
                glyph_runs: vec![Default::default()],
                color: Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                },
                ..TextCombineEmphasisPaintData::default()
            }),
            ..Default::default()
        });
        fragment.size = Size {
            width: 20.0,
            height: 16.0,
        };
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.applies_transform = true;
        node.local_transforms.push(TranslationTransform(3.0, 0.0));

        let mut recording = PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut recording));
        let paint_info = PaintInfo::new(&context, PaintPhase::kForeground);
        context.borrow_mut().BeginGeometry(&node);

        let painter = BoxFragmentPainter::new(&node, &context);
        painter.Paint(&paint_info);
        context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase: PaintPhase::kForeground,
                ..Default::default()
            },
            &node,
        );
        context.borrow_mut().EndGeometry(&node);
        painter.PaintAfterChildren(&paint_info);
        drop(context);

        let types: Vec<_> = recording.items.iter().map(|item| item.r#type).collect();
        assert_eq!(
            types,
            vec![
                DisplayItemType::kSave,
                DisplayItemType::kConcat,
                DisplayItemType::kClipRect,
                DisplayItemType::kRestore,
                DisplayItemType::kDrawGlyphRun,
            ]
        );
        let emphasis = recording.items.last().expect("emphasis item");
        assert_eq!(emphasis.color.red, 171.0 / 255.0);
        assert_eq!(emphasis.color.green, 171.0 / 255.0);
        assert_eq!(emphasis.color.blue, 171.0 / 255.0);
    }

    #[test]
    fn fixed_and_local_backgrounds_use_viewport_zoom_and_scroll_geometry() {
        let mut fixed_fragment = FragmentPaintData::default();
        fixed_fragment.paint.resources = Some(Arc::new(PaintResources {
            layout_zoom_factor: 1.5,
            viewport: Some(ViewportGeometry {
                size: IntSize {
                    width: 320,
                    height: 200,
                },
                ..ViewportGeometry::default()
            }),
            ..PaintResources::default()
        }));
        let mut fixed = PaintTreeNode::default();
        fixed.SetOwnedFragmentForTesting(&fixed_fragment);
        fixed.scroll_offset = Offset { x: 7.0, y: 11.0 };
        fixed.transforms.push(TranslationTransform(5.0, 9.0));
        let viewport = FixedBackgroundPositioningArea(&fixed).unwrap();
        assert_eq!((viewport.x, viewport.y), (2.0, 2.0));
        assert_eq!((viewport.width, viewport.height), (480.0, 300.0));

        let mut local_fragment = FragmentPaintData::default();
        local_fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        local_fragment.paint.scroll_offset = Offset { x: 13.75, y: 17.25 };
        local_fragment.paint.pixel_snapped_scrolled_content_offset = Offset { x: 13.0, y: 17.0 };
        local_fragment.paint.scroll_size = Size {
            width: 240.0,
            height: 180.0,
        };
        local_fragment.paint.border.left = 2.0;
        local_fragment.paint.border.right = 3.0;
        local_fragment.paint.border.top = 4.0;
        local_fragment.paint.border.bottom = 5.0;
        local_fragment.paint.padding.left = 6.0;
        local_fragment.paint.padding.right = 7.0;
        local_fragment.paint.padding.top = 8.0;
        local_fragment.paint.padding.bottom = 9.0;
        local_fragment.paint.scroll_container = Some(ScrollContainerPaintData {
            compositor_element_id: Default::default(),
            scrollable_axes: [true, true],
            scroll_origin: Offset::default(),
            scroll_position: local_fragment.paint.scroll_offset,
            background_needs_repaint_on_scroll: None,
            prefers_non_composited_scrolling: None,
            scroll_snap_type: None,
            container_rect: FragmentPaintRect::default(),
            scrolling_background_visual_rect: FragmentPaintRect {
                offset: Offset { x: 3.0, y: 5.0 },
                size: Size {
                    width: 97.0,
                    height: 71.0,
                },
            },
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
            scrolling_background_client_id: 0,
            scrolling_background_client_invalidation_reason: None,
            scrolling_background_client_is_cacheable: true,
            scrolling_background_client_is_just_created: std::cell::Cell::new(false),
        });
        let mut local = PaintTreeNode::default();
        local.SetOwnedFragmentForTesting(&local_fragment);
        local.paint_offset = Offset { x: 50.0, y: 40.0 };
        let contents = LocalBackgroundBorderRect(&local);
        assert_eq!((contents.x, contents.y), (37.0, 23.0));
        assert_eq!((contents.width, contents.height), (245.0, 189.0));
        let visual = ScrollingBackgroundVisualRect(&local);
        assert_eq!((visual.x, visual.y), (53.0, 45.0));
        assert_eq!((visual.width, visual.height), (97.0, 71.0));
        let content_clip = BackgroundBoxRectWithin(&local, BackgroundBox::kContentBox, contents);
        assert_eq!((content_clip.x, content_clip.y), (45.0, 35.0));
        assert_eq!((content_clip.width, content_clip.height), (227.0, 163.0));
    }

    #[test]
    fn canvas_background_images_use_document_element_positioning_geometry() {
        use crate::paint_context::PaintContext;
        use crate::{DisplayItemType, PaintRecording};

        let canvas_color = Color {
            red: 0.25,
            green: 0.5,
            blue: 0.75,
            alpha: 1.0,
        };
        let mut canvas_style = PaintStyle::default();
        canvas_style.background_color = canvas_color;
        let mut fragment = FragmentPaintData::default();
        fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: true,
            ..PaintResources::default()
        }));
        fragment.paint.canvas_background = Some(CanvasBackgroundPaintData {
            style: canvas_style,
            effective_zoom: 1.0,
            document_element_node_id: 0,
            previous_document_element_geometry: None,
            document_element_paint_offset: Offset { x: 25.0, y: 30.0 },
            document_element_size: Size {
                width: 300.0,
                height: 200.0,
            },
            document_element_border: Edges {
                top: 2.0,
                right: 3.0,
                bottom: 4.0,
                left: 5.0,
            },
            document_element_padding: Edges {
                top: 6.0,
                right: 7.0,
                bottom: 8.0,
                left: 9.0,
            },
            document_element_content_box: FragmentPaintRect::default(),
            document_element_scrollable_overflow: None,
            document_element_has_transform: false,
            background_fill_layer_count: 0,
            background_any_local_attachment_image: false,
            background_any_layer_uses_content_box: false,
        });
        let mut node = PaintTreeNode::default();
        node.paint_offset = Offset { x: 100.0, y: 200.0 };
        node.SetOwnedFragmentForTesting(&fragment);

        assert_eq!(
            CanvasBackgroundPositioningBoxRect(&node, BackgroundBox::kBorderBox),
            PaintRect {
                x: 25.0,
                y: 30.0,
                width: 300.0,
                height: 200.0,
            }
        );
        assert_eq!(
            CanvasBackgroundPositioningBoxRect(&node, BackgroundBox::kPaddingBox),
            PaintRect {
                x: 30.0,
                y: 32.0,
                width: 292.0,
                height: 194.0,
            }
        );
        assert_eq!(
            CanvasBackgroundPositioningBoxRect(&node, BackgroundBox::kContentBox),
            PaintRect {
                x: 39.0,
                y: 38.0,
                width: 276.0,
                height: 180.0,
            }
        );

        let mut recording = PaintRecording::default();
        {
            let context = RefCell::new(PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).PaintCanvasBackground(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(recording.first_painted);
        assert!(recording
            .items
            .iter()
            .any(|item| item.r#type == DisplayItemType::kDrawRect && item.color == canvas_color));

        let mut ignored = PaintRecording::default();
        {
            let _scope = crate::ignore_paint_timing_scope::IgnorePaintTimingScope::new();
            crate::ignore_paint_timing_scope::IgnorePaintTimingScope::IncrementIgnoreDepth();
            let context = RefCell::new(PaintContext::new(&mut ignored));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&node, &context).PaintCanvasBackground(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(!ignored.first_painted);
    }

    #[test]
    fn canvas_background_uses_document_element_pre_transform_state() {
        use crate::paint_property_tree::{PaintPropertyNodeLifecycle, TransformPaintPropertyNode};

        let mut canvas_style = PaintStyle::default();
        canvas_style.background_color = Color {
            red: 0.2,
            green: 0.4,
            blue: 0.6,
            alpha: 0.5,
        };
        let mut root_fragment = FragmentPaintData::default();
        root_fragment.node_id = 1;
        root_fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        root_fragment.paint.has_source = true;
        root_fragment.paint.establishes_paint_state = true;
        root_fragment.paint.is_layout_view = true;
        root_fragment.paint.background_transfers_to_view = true;
        root_fragment.paint.style.SetVisibleForTesting(true);
        root_fragment.paint.box_self_visual_overflow = Some(FragmentPaintRect {
            size: root_fragment.size,
            ..FragmentPaintRect::default()
        });
        root_fragment.paint.resources = Some(Arc::new(PaintResources {
            paints_opaque_base_background: true,
            ..PaintResources::default()
        }));
        root_fragment.paint.canvas_background = Some(CanvasBackgroundPaintData {
            style: canvas_style,
            effective_zoom: 1.0,
            document_element_node_id: 2,
            previous_document_element_geometry: None,
            document_element_paint_offset: Offset::default(),
            document_element_size: root_fragment.size,
            document_element_border: Edges::default(),
            document_element_padding: Edges::default(),
            document_element_content_box: FragmentPaintRect::default(),
            document_element_scrollable_overflow: None,
            document_element_has_transform: true,
            background_fill_layer_count: 0,
            background_any_local_attachment_image: false,
            background_any_layer_uses_content_box: false,
        });

        let mut root = PaintTreeNode::default();
        root.SetOwnedFragmentForTesting(&root_fragment);
        let view_contents_transform = root.contents_properties.nodes.transform.clone();
        let document_pre_transform = Arc::new(TransformPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 91,
            parent: Some(view_contents_transform.clone()),
            matrix: TranslationTransform(10.0, 20.0),
            ..(*view_contents_transform).clone()
        });
        let document_css_transform = Arc::new(TransformPaintPropertyNode {
            lifecycle: PaintPropertyNodeLifecycle::default(),
            id: 92,
            parent: Some(document_pre_transform.clone()),
            matrix: TranslationTransform(30.0, 40.0),
            ..(*view_contents_transform).clone()
        });
        let mut document_fragment = FragmentPaintData::default();
        document_fragment.node_id = 2;
        let mut document = PaintTreeNode::default();
        document.SetOwnedFragmentForTesting(&document_fragment);
        document.properties.nodes = root.contents_properties.nodes.clone();
        document.properties.nodes.transform = document_css_transform.clone();
        document.pre_transform = document_pre_transform.clone();
        root.children.push(Box::new(document));

        let background_state = CanvasBackgroundPaintState(&root);
        assert!(Arc::ptr_eq(
            &background_state.transform,
            &document_pre_transform,
        ));
        assert!(!Arc::ptr_eq(
            &background_state.transform,
            &document_css_transform,
        ));
        assert_eq!(
            CanvasBackgroundRectInState(&root, false, &background_state),
            Some(PaintRect {
                x: -10.0,
                y: -20.0,
                width: 100.0,
                height: 80.0,
            })
        );
        assert_eq!(
            CanvasBackgroundPositioningBoxRectInState(
                &root,
                BackgroundBox::kBorderBox,
                &background_state,
                PaintRect {
                    x: -10.0,
                    y: -20.0,
                    width: 100.0,
                    height: 80.0,
                },
            ),
            PaintRect {
                x: -10.0,
                y: -20.0,
                width: 100.0,
                height: 80.0,
            }
        );

        // Ignoring the document element's CSS transform makes this state the
        // same as LayoutView ContentsProperties, so ViewPainter folds the base
        // backdrop into DocumentBackground and omits RootBackdrop.
        root.children[0].pre_transform = view_contents_transform;
        root.is_root = true;
        let mut recording = crate::PaintRecording::default();
        {
            let context = RefCell::new(crate::paint_context::PaintContext::new(&mut recording));
            let paint_info = PaintInfo::new(&context, PaintPhase::kSelfBlockBackgroundOnly);
            BoxFragmentPainter::new(&root, &context).Paint(&paint_info);
            context.borrow_mut().Finish();
        }
        assert!(recording.display_items.iter().all(|item| {
            item.id.r#type != crate::display_item_id::DisplayItemIdType::kDocumentRootBackdrop
        }));
        let canvas_draw = recording
            .items
            .iter()
            .find(|item| {
                item.r#type == crate::DisplayItemType::kDrawRect
                    && item.phase == PaintPhase::kSelfBlockBackgroundOnly
            })
            .expect("combined canvas background drawing");
        assert_eq!(
            canvas_draw.color,
            Color {
                red: 0.6,
                green: 0.7,
                blue: 0.8,
                alpha: 1.0,
            }
        );
        assert!(canvas_draw.source_overwrite);
    }
}
