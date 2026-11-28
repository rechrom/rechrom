//! Fragment-owned retained paint boundary schema.
//!
//! The plain schema values live in `paint_input_types::paint_input` so Paint,
//! Raster and Style name them without depending on Layout; they are
//! re-exported here at their established path. `PaintStyle` stays in Layout:
//! it is a view over the canonical ComputedStyle whose accessors read Layout's
//! paint-style projections.

#![allow(non_camel_case_types)]

pub use paint_input_types::paint_input::*;
use std::sync::{Arc, OnceLock};

use super::super::layout_input::{BorderLineStyle, PaintShadow, PaintStyle, TextDecorationPaint};
use super::super::layout_input_types::Color;
use crate::internal::caret_rect::CaretShape;

// base::ValuesEquivalent compares shared immutable groups by identity first.
// DerefMut below detaches before writing, so a changed group still compares all
// of its values; this is not a source-style-pointer geometry shortcut.
impl PartialEq for PaintStyle {
    fn eq(&self, other: &Self) -> bool {
        if self.SourcePtr() == other.SourcePtr() {
            // Two unresolved projections of the same inputs resolve to the
            // same values; comparing them must not force either resolution.
            if let (None, None, Some(a), Some(b)) =
                (self.data.get(), other.data.get(), &self.pending, &other.pending)
            {
                if a.SameInputs(b) {
                    return true;
                }
            }
            if std::sync::Arc::ptr_eq(self.DataArc(), other.DataArc()) {
                return true;
            }
        }
        let source_equal = match (self.Source(), other.Source()) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        };
        source_equal && self.Data() == other.Data()
    }
}

impl Default for PaintStyle {
    fn default() -> Self {
        static DEFAULT_DATA: OnceLock<Arc<PaintStyleData>> = OnceLock::new();
        Self {
            source_address: 0,
            data: OnceLock::from(Arc::clone(
                DEFAULT_DATA.get_or_init(|| Arc::new(PaintStyleData::default())),
            )),
            pending: None,
        }
    }
}

impl PaintStyle {
    pub(crate) fn FromData(data: PaintStyleData) -> Self {
        Self {
            source_address: 0,
            data: OnceLock::from(Arc::new(data)),
            pending: None,
        }
    }

    pub(crate) fn FromSource(
        source: &layout_style::style::computed_style::ComputedStyle,
        data: PaintStyleData,
    ) -> Self {
        let pointer = source as *const _;
        Self {
            source_address: foundation::IsManagedLayoutAddress(pointer)
                .then_some(pointer as usize)
                .unwrap_or(0),
            data: OnceLock::from(Arc::new(data)),
            pending: None,
        }
    }

    /// `FromSource` sharing an already resolved (memoized) projection.
    pub(crate) fn FromSourceData(
        source: &layout_style::style::computed_style::ComputedStyle,
        data: Arc<PaintStyleData>,
    ) -> Self {
        let pointer = source as *const _;
        Self {
            source_address: foundation::IsManagedLayoutAddress(pointer)
                .then_some(pointer as usize)
                .unwrap_or(0),
            data: OnceLock::from(data),
            pending: None,
        }
    }

    /// The shared projected values (resolving a pending projection).
    pub(crate) fn SharedData(&self) -> Arc<PaintStyleData> {
        Arc::clone(self.DataArc())
    }

    /// A projection whose values are resolved on first read from `pending`
    /// (fragment-local Paint data population records only these inputs).
    /// The source identity is the one ExportStyle records for the same
    /// ComputedStyle, so Source() and IsExportedFrom() never resolve it.
    pub(crate) fn FromPending(pending: crate::internal::layout_input::PendingPaintStyle) -> Self {
        let pointer =
            pending.style as *const layout_style::style::computed_style::ComputedStyle;
        Self {
            source_address: foundation::IsManagedLayoutAddress(pointer)
                .then_some(pointer as usize)
                .unwrap_or(0),
            data: OnceLock::new(),
            pending: Some(pending),
        }
    }

    /// Share `previous`'s already resolved values when this is an unresolved
    /// projection of exactly the same inputs (PendingPaintStyle::SameInputs:
    /// ComputedStyle, LayoutObject and the retained StyleResourceData Arc).
    /// The projection is a pure function of those inputs except for SVG
    /// clip-path / mask resource viewports, which are resolved against
    /// another object's layout (ResolveFragmentPaintStyle); styles that
    /// reference them always re-resolve. A projection written through
    /// DerefMut has no pending inputs and is never shared.
    pub(crate) fn AdoptResolvedProjection(&mut self, previous: &PaintStyle) {
        if self.data.get().is_some() {
            return;
        }
        let (Some(pending), Some(previous_pending), Some(data)) =
            (&self.pending, &previous.pending, previous.data.get())
        else {
            return;
        };
        if !pending.SameInputs(previous_pending) {
            return;
        }
        let Some(style) = self.Source() else {
            return;
        };
        if style.ClipPath().is_some() || style.MaskLayers().AnyLayerHasImage() {
            return;
        }
        let data = Arc::clone(data);
        let _ = self.data.set(data);
    }

    /// False only for a pending fragment projection none of whose projected
    /// values has been read yet.
    pub fn IsResolved(&self) -> bool {
        self.data.get().is_some()
    }

    fn DataArc(&self) -> &Arc<PaintStyleData> {
        self.data.get_or_init(|| {
            let pending = self
                .pending
                .as_ref()
                .expect("an unresolved PaintStyle carries its pending projection");
            crate::internal::boundary::layout_boundary::ResolvePendingPaintStyle(pending)
        })
    }

    fn Data(&self) -> &PaintStyleData {
        self.DataArc()
    }

    fn DataMut(&mut self) -> &mut PaintStyleData {
        self.DataArc();
        // A written projection is no longer a pure function of its inputs.
        self.pending = None;
        Arc::make_mut(self.data.get_mut().expect("resolved above"))
    }

    /// A projection-free view of a live, managed ComputedStyle: every accessor
    /// that reads `Source()` reads `style` itself, the way Chromium's hit
    /// testing and painters read the fragment's `Style()`. Accessors that still
    /// read projected data see `PaintStyleData::default()`. None for an
    /// unmanaged style, whose values exist only in an exported projection.
    pub fn LiveView(style: &layout_style::style::computed_style::ComputedStyle) -> Option<Self> {
        let pointer = style as *const layout_style::style::computed_style::ComputedStyle;
        foundation::IsManagedLayoutAddress(pointer).then(|| Self {
            source_address: pointer as usize,
            ..Self::default()
        })
    }

    /// Whether this projection was exported from exactly `source`. A retained
    /// projection whose owner has since received a different ComputedStyle
    /// must be re-exported; its old source address may already be reused.
    pub fn IsExportedFrom(
        &self,
        source: *const layout_style::style::computed_style::ComputedStyle,
    ) -> bool {
        self.source_address == source as usize
    }

    pub fn Source(&self) -> Option<&layout_style::style::computed_style::ComputedStyle> {
        unsafe { self.SourcePtr().as_ref() }
    }

    /// Inside a `ResidentPaintSourceScope` (the paint tree walk) a non-zero
    /// source is the managed ComputedStyle of a fragment in the leased
    /// resident result (every constructor records only a managed address,
    /// and PrePaint re-exports a projection whose owner changed style), so
    /// it is read directly, as Chromium's painters read
    /// `PhysicalFragment::Style()` (core/layout/physical_fragment.h). Outside
    /// the walk the heap membership query still guards retained payloads.
    #[inline]
    fn SourcePtr(&self) -> *mut layout_style::style::computed_style::ComputedStyle {
        let pointer =
            self.source_address as *mut layout_style::style::computed_style::ComputedStyle;
        if pointer.is_null() {
            return std::ptr::null_mut();
        }
        if crate::fragment_tree::ResidentPaintSourceScope::IsActive() {
            debug_assert!(foundation::IsManagedLayoutAddress(pointer));
            return pointer;
        }
        if !foundation::IsManagedLayoutAddress(pointer) {
            std::ptr::null_mut()
        } else {
            pointer
        }
    }

    // Border radii and SVG paint values are read from the source
    // ComputedStyle, as Chromium's painters read them; detached projections
    // (no live source) keep the exported PaintStyleData values.
    pub fn border_radii(&self) -> Option<PaintCornerRadii> {
        self.Source().map_or_else(|| self.Data().border_radii, |style| {
            crate::internal::boundary::paint_style_export::BorderRadiiParts(style).0
        })
    }

    pub fn border_radii_percentages(&self) -> PaintCornerRadii {
        self.Source().map_or_else(|| self.Data().border_radii_percentages, |style| {
            crate::internal::boundary::paint_style_export::BorderRadiiParts(style).1
        })
    }

    pub fn border_corner_curvatures(&self) -> PaintCornerCurvatures {
        self.Source().map_or_else(|| self.Data().border_corner_curvatures, |style| {
            crate::internal::boundary::paint_style_export::BorderRadiiParts(style).2
        })
    }

    pub fn svg_fill(&self) -> Option<Color> {
        self.Source().map_or_else(|| self.Data().svg_fill, |style| {
            crate::internal::boundary::paint_style_export::SvgFill(style)
        })
    }

    pub fn svg_stroke(&self) -> Option<Color> {
        self.Source().map_or_else(|| self.Data().svg_stroke, |style| {
            crate::internal::boundary::paint_style_export::SvgStroke(style)
        })
    }

    pub fn svg_fill_reference(&self) -> Option<String> {
        self.Source().map_or_else(
            || self.Data().svg_fill_reference.clone(),
            crate::internal::boundary::paint_style_export::SvgFillReference,
        )
    }

    pub fn svg_stroke_reference(&self) -> Option<String> {
        self.Source().map_or_else(
            || self.Data().svg_stroke_reference.clone(),
            crate::internal::boundary::paint_style_export::SvgStrokeReference,
        )
    }

    pub fn svg_fill_current_color(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_fill_current_color,
            crate::internal::boundary::paint_style_export::SvgFillCurrentColor,
        )
    }

    pub fn svg_stroke_current_color(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_stroke_current_color,
            crate::internal::boundary::paint_style_export::SvgStrokeCurrentColor,
        )
    }

    pub fn svg_stop_color(&self) -> Color {
        self.Source().map_or_else(
            || self.Data().svg_stop_color,
            crate::internal::boundary::paint_style_export::SvgStopColor,
        )
    }

    pub fn svg_stop_current_color(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_stop_current_color,
            crate::internal::boundary::paint_style_export::SvgStopCurrentColor,
        )
    }

    pub fn svg_stop_opacity(&self) -> f32 {
        self.Source().map_or_else(
            || self.Data().svg_stop_opacity,
            crate::internal::boundary::paint_style_export::SvgStopOpacity,
        )
    }

    pub fn svg_stroke_width(&self) -> f64 {
        self.Source().map_or_else(
            || self.Data().svg_stroke_width,
            crate::internal::boundary::paint_style_export::SvgStrokeWidth,
        )
    }

    pub fn svg_stroke_dash_offset(&self) -> f64 {
        self.Source().map_or_else(
            || self.Data().svg_stroke_dash_offset,
            crate::internal::boundary::paint_style_export::SvgStrokeDashOffset,
        )
    }

    pub fn svg_stroke_dash_array(&self) -> Vec<f64> {
        self.Source().map_or_else(
            || self.Data().svg_stroke_dash_array.clone(),
            crate::internal::boundary::paint_style_export::SvgStrokeDashArray,
        )
    }

    pub fn svg_stroke_line_cap(&self) -> SvgStrokeLineCap {
        self.Source().map_or_else(
            || self.Data().svg_stroke_line_cap,
            crate::internal::boundary::paint_style_export::SvgStrokeLineCapValue,
        )
    }

    pub fn svg_stroke_line_join(&self) -> SvgStrokeLineJoin {
        self.Source().map_or_else(
            || self.Data().svg_stroke_line_join,
            crate::internal::boundary::paint_style_export::SvgStrokeLineJoinValue,
        )
    }

    pub fn svg_stroke_miter_limit(&self) -> f64 {
        self.Source().map_or_else(
            || self.Data().svg_stroke_miter_limit,
            crate::internal::boundary::paint_style_export::SvgStrokeMiterLimit,
        )
    }

    pub fn svg_fill_even_odd(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_fill_even_odd,
            crate::internal::boundary::paint_style_export::SvgFillEvenOdd,
        )
    }

    pub fn svg_clip_even_odd(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_clip_even_odd,
            crate::internal::boundary::paint_style_export::SvgClipEvenOdd,
        )
    }

    pub fn svg_non_scaling_stroke(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_non_scaling_stroke,
            crate::internal::boundary::paint_style_export::SvgNonScalingStroke,
        )
    }

    pub fn svg_shape_antialias(&self) -> bool {
        self.Source().map_or_else(
            || self.Data().svg_shape_antialias,
            crate::internal::boundary::paint_style_export::SvgShapeAntialias,
        )
    }

    pub fn svg_paint_order(&self) -> [SvgPaintComponent; 3] {
        self.Source().map_or_else(
            || self.Data().svg_paint_order,
            crate::internal::boundary::paint_style_export::SvgPaintOrder,
        )
    }

    pub fn color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().color, |style| {
            crate::internal::boundary::paint_style_export::StyleColor(style, style.Color())
        })
    }

    pub fn text_fill_color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().text_fill_color, |style| {
            crate::internal::boundary::paint_style_export::StyleColor(style, style.TextFillColor())
        })
    }

    pub fn text_stroke_color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().text_stroke_color, |style| {
            crate::internal::boundary::paint_style_export::StyleColor(
                style,
                style.TextStrokeColor(),
            )
        })
    }

    pub fn text_stroke_width(&self) -> f64 {
        self.Source().map_or_else(|| self.Data().text_stroke_width, |style| {
            f64::from(style.TextStrokeWidth())
        })
    }

    pub fn print_color_adjust_economy(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().print_color_adjust_economy, |style| {
                style.PrintColorAdjust() == foundation::EPrintColorAdjust::kEconomy
            })
    }

    pub fn caret_shape(&self) -> CaretShape {
        self.Source()
            .map_or_else(|| self.Data().caret_shape, |style| match style.CaretShape() {
                foundation::ECaretShape::kAuto | foundation::ECaretShape::kBar => CaretShape::kBar,
                foundation::ECaretShape::kBlock => CaretShape::kBlock,
                foundation::ECaretShape::kUnderscore => CaretShape::kUnderscore,
            })
    }
    pub fn caret_color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().caret_color, |style| {
            crate::internal::boundary::paint_style_export::ColorValue(style.ResolvedCaretColor())
        })
    }

    pub fn image_interpolation_quality(&self) -> foundation::InterpolationQuality {
        self.Source()
            .map_or_else(|| self.Data().image_interpolation_quality, |style| {
                style.GetInterpolationQuality()
            })
    }

    pub fn dynamic_range_limit(&self) -> foundation::DynamicRangeLimit {
        self.Source()
            .map_or_else(|| self.Data().dynamic_range_limit, |style| {
                *style.GetDynamicRangeLimit()
            })
    }

    pub fn color_scheme_flags_is_normal(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().color_scheme_flags_is_normal, |style| {
                style.ColorSchemeFlagsIsNormal()
            })
    }

    pub fn used_dark_color_scheme(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().used_dark_color_scheme, |style| {
                matches!(
                    style.UsedColorScheme(),
                    layout_style::style::color_scheme::mojom::blink::ColorScheme::kDark
                )
            })
    }

    pub fn force_dark(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().force_dark, |style| style.ForceDark())
    }

    /// Projections are exported with the pause clear; only PrePaint sets it
    /// (through `SetAutoDarkModePaused` or DerefMut, which resolve first), so
    /// an unresolved pending projection reads false without resolving.
    pub fn auto_dark_mode_paused(&self) -> bool {
        self.data
            .get()
            .is_some_and(|data| data.auto_dark_mode_paused)
    }

    /// PrePaint's inherited GraphicsContext::ScopedAutoDarkModeState. Leaves
    /// an unresolved projection pending when the pause stays clear.
    pub fn SetAutoDarkModePaused(&mut self, paused: bool) {
        if self.auto_dark_mode_paused() != paused {
            self.DataMut().auto_dark_mode_paused = paused;
        }
    }

    pub fn background_color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().background_color, |style| {
            crate::internal::boundary::paint_style_export::StyleColor(
                style,
                style.BackgroundColor(),
            )
        })
    }

    pub fn background_clip(&self) -> BackgroundBox {
        self.Source().map_or_else(|| self.Data().background_clip, |style| {
            crate::internal::boundary::paint_style_export::BottomBackgroundClip(style)
        })
    }

    pub fn border_colors(&self) -> [Color; 4] {
        self.Source().map_or_else(|| self.Data().border_colors, |style| {
            [
                crate::internal::boundary::paint_style_export::StyleColor(
                    style,
                    style.BorderTopColor(),
                ),
                crate::internal::boundary::paint_style_export::StyleColor(
                    style,
                    style.BorderRightColor(),
                ),
                crate::internal::boundary::paint_style_export::StyleColor(
                    style,
                    style.BorderBottomColor(),
                ),
                crate::internal::boundary::paint_style_export::StyleColor(
                    style,
                    style.BorderLeftColor(),
                ),
            ]
        })
    }

    pub fn outline_color(&self) -> Color {
        self.Source().map_or_else(|| self.Data().outline_color, |style| {
            crate::internal::boundary::paint_style_export::StyleColor(style, style.OutlineColor())
        })
    }

    pub fn outline_width(&self) -> f64 {
        self.Source().map_or_else(|| self.Data().outline_width, |style| {
            if style.OutlineStyle() == foundation::EBorderStyle::kNone {
                0.0
            } else {
                *style.OutlineWidth() as f64
            }
        })
    }

    pub fn outline_offset(&self) -> f64 {
        self.Source().map_or_else(|| self.Data().outline_offset, |style| {
            *style.OutlineOffset() as f64
        })
    }

    pub fn outline_style(&self) -> BorderLineStyle {
        self.Source().map_or_else(|| self.Data().outline_style, |style| {
            crate::internal::boundary::paint_style_export::BorderStyle(style.OutlineStyle())
        })
    }

    pub fn border_shape_unsupported(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().border_shape_unsupported, |style| {
                style.HasBorderShape()
            })
    }

    pub fn box_shadow_count(&self) -> usize {
        self.Source().map_or_else(|| self.Data().box_shadows.len(), |style| {
            let list = style.BoxShadow();
            if list.is_null() {
                0
            } else {
                unsafe { &*list }.Shadows().len()
            }
        })
    }

    pub fn box_shadow(&self, index: usize) -> PaintShadow {
        self.Source().map_or_else(
            || self.Data().box_shadows[index].clone(),
            |style| {
                let list = style.BoxShadow();
                let shadow = &unsafe { &*list }.Shadows()[index];
                crate::internal::boundary::paint_style_export::ShadowValue(style, shadow)
            },
        )
    }

    pub fn text_shadow_count(&self) -> usize {
        self.Source().map_or_else(|| self.Data().text_shadows.len(), |style| {
            let list = style.TextShadow();
            if list.is_null() {
                0
            } else {
                unsafe { &*list }.Shadows().len()
            }
        })
    }

    pub fn text_shadow(&self, index: usize) -> PaintShadow {
        self.Source().map_or_else(
            || self.Data().text_shadows[index].clone(),
            |style| {
                let list = style.TextShadow();
                let shadow = &unsafe { &*list }.Shadows()[index];
                crate::internal::boundary::paint_style_export::ShadowValue(style, shadow)
            },
        )
    }

    pub fn applied_text_decoration_count(&self) -> usize {
        self.Source()
            .map_or_else(|| self.Data().applied_text_decorations.len(), |style| {
                let decorations = style.AppliedTextDecorationData();
                if decorations.is_null() {
                    0
                } else {
                    unsafe { &*decorations }.len()
                }
            })
    }

    pub fn applied_text_decoration(&self, index: usize) -> TextDecorationPaint {
        self.Source().map_or_else(
            || self.Data().applied_text_decorations[index].clone(),
            |style| {
                let decorations = style.AppliedTextDecorationData();
                let decoration = &unsafe { &*decorations }[index];
                crate::internal::boundary::layout_boundary::ExportAppliedTextDecoration(
                    decoration,
                    style.TextDecorationSkipInk(),
                )
            },
        )
    }

    pub fn opacity(&self) -> f32 {
        self.Source()
            .map_or_else(|| self.Data().opacity, |style| style.Opacity())
    }

    pub fn visible(&self) -> bool {
        self.Source().map_or_else(|| self.Data().visible, |style| {
            style.Visibility() == foundation::EVisibility::kVisible
        })
    }

    /// Whether border-image paints: its NinePieceImage has an image (the only
    /// case ExportNinePieceImageForFragment produces a value).
    pub fn has_border_image(&self) -> bool {
        self.Source().map_or_else(|| self.Data().border_image.is_some(), |style| {
            !style.BorderImage().GetImage().is_null()
        })
    }

    /// Whether -webkit-mask-box-image paints (see has_border_image).
    pub fn has_mask_box_image(&self) -> bool {
        self.Source().map_or_else(|| self.Data().mask_box_image.is_some(), |style| {
            !style.MaskBoxImage().GetImage().is_null()
        })
    }

    /// Whether a box reflection paints a mask image (see has_border_image).
    pub fn has_box_reflection_mask_image(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().box_reflection_mask_image.is_some(), |style| {
                unsafe { style.BoxReflect().as_ref() }.is_some_and(|reflection| {
                    reflection.Mask().HasImage() && !reflection.Mask().GetImage().is_null()
                })
            })
    }

    /// The style facts PrePaint's resource-usage classification needs, from
    /// one source resolution: whether any background layer has a fixed
    /// attachment image (ComputedStyle::HasFixedAttachmentBackgroundImage),
    /// and whether this style paints any image or filter resource.
    ///
    /// A live source is read in place, as Chromium's PrePaint reads
    /// `object.StyleRef()` (paint_invalidator.cc, box_paint_invalidator.cc);
    /// that path never resolves a pending projection.
    pub fn ResourceUsageFacts(&self, nine_piece_images: bool) -> (bool, bool) {
        match self.Source() {
            Some(style) => (
                style.HasFixedAttachmentBackgroundImage(),
                self.HasImageResourceId()
                    // Box fragments paint nine-piece images from the style
                    // (border-image, mask-box-image, reflection mask).
                    || (nine_piece_images
                        && (!style.BorderImage().GetImage().is_null()
                            || !style.MaskBoxImage().GetImage().is_null()
                            || unsafe { style.BoxReflect().as_ref() }.is_some_and(
                                |reflection| {
                                    reflection.Mask().HasImage()
                                        && !reflection.Mask().GetImage().is_null()
                                },
                            )))
                    || style.HasBackgroundImage()
                    || style.MaskLayers().AnyLayerHasImage()
                    || style.HasFilter()
                    || style.HasBoxReflect()
                    || style.HasBackdropFilter(),
            ),
            None => (
                self.Data()
                    .background_images
                    .iter()
                    .any(|layer| layer.attachment == BackgroundAttachment::kFixed),
                self.Data().image_resource_id.is_some()
                    || self.Data().border_image.is_some()
                    || self.Data().mask_box_image.is_some()
                    || self.Data().box_reflection_mask_image.is_some()
                    || !self.Data().background_images.is_empty()
                    || !self.Data().mask_images.is_empty()
                    || !self.Data().filters.is_empty()
                    || !self.Data().backdrop_filters.is_empty(),
            ),
        }
    }

    /// Whether the projection carries an image resource id, without resolving
    /// a pending projection: ExportForPaint copies it verbatim from the
    /// paired StyleResourceData (ExportStyleWithInputResources), and the
    /// memoized projection of the same pair carries the same value.
    fn HasImageResourceId(&self) -> bool {
        match (self.data.get(), &self.pending) {
            (Some(data), _) => data.image_resource_id.is_some(),
            (None, Some(pending)) => pending.resources.image_resource_id.is_some(),
            (None, None) => self.Data().image_resource_id.is_some(),
        }
    }

    /// visible(), outline_style(), outline_width() and outline_color().alpha
    /// from a single source resolution, for PrePaint's per-node record.
    pub fn PhaseStyleValues(&self) -> (bool, BorderLineStyle, f64, f32) {
        match self.Source() {
            Some(style) => {
                let outline = style.OutlineStyle();
                (
                    style.Visibility() == foundation::EVisibility::kVisible,
                    crate::internal::boundary::paint_style_export::BorderStyle(outline),
                    if outline == foundation::EBorderStyle::kNone {
                        0.0
                    } else {
                        *style.OutlineWidth() as f64
                    },
                    crate::internal::boundary::paint_style_export::StyleColor(
                        style,
                        style.OutlineColor(),
                    )
                    .alpha,
                )
            }
            None => (
                self.Data().visible,
                self.Data().outline_style,
                self.Data().outline_width,
                self.Data().outline_color.alpha,
            ),
        }
    }

    pub fn forced_color_adjust_auto(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().forced_color_adjust_auto, |style| {
                style.ForcedColorAdjust() == foundation::EForcedColorAdjust::kAuto
            })
    }

    pub fn forced_background_color(&self) -> Color {
        self.Source()
            .map_or_else(|| self.Data().forced_background_color, |style| {
                crate::internal::boundary::paint_style_export::StyleColor(
                    style,
                    style.InternalForcedBackgroundColor(),
                )
            })
    }

    pub fn pointer_events_none(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().pointer_events_none, |style| {
                style.PointerEvents() == foundation::EPointerEvents::kNone
            })
    }

    pub fn pointer_events_bounding_box(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().pointer_events_bounding_box, |style| {
                style.UsedPointerEvents() == foundation::EPointerEvents::kBoundingBox
            })
    }

    pub fn user_select_none(&self) -> bool {
        self.Source().map_or_else(|| self.Data().user_select_none, |style| {
            style.UsedUserSelect() == foundation::EUserSelect::kNone
        })
    }

    pub fn cursor(&self) -> ECursor {
        self.Source()
            .map_or_else(|| self.Data().cursor, |style| style.Cursor())
    }

    pub fn z_index(&self) -> Option<i32> {
        self.Source().map_or_else(|| self.Data().z_index, |style| {
            (!style.HasAutoZIndex()).then(|| style.ZIndex())
        })
    }

    pub fn blend_mode(&self) -> PaintBlendMode {
        self.Source().map_or_else(|| self.Data().blend_mode, |style| {
            crate::internal::boundary::paint_style_export::BlendValue(style.GetBlendMode())
        })
    }

    fn BindFilterResources(&self, filters: &mut [PaintFilterOperation]) {
        let mut references = 0usize;
        let mut bound = 0usize;
        for filter in filters {
            if filter.r#type != PaintFilterType::kReference {
                continue;
            }
            references += 1;
            let Some(id) = filter
                .reference_url
                .as_deref()
                .and_then(|url| url.strip_prefix('#'))
            else {
                continue;
            };
            if let Some(graph) = self
                .Data()
                .svg_filter_graphs
                .iter()
                .find(|graph| graph.id == id)
            {
                let mut graph = graph.clone();
                graph.fill_paint = self.svg_fill();
                graph.stroke_paint = self.svg_stroke();
                filter.reference_error = graph.unsupported.clone();
                filter.reference_graph = Some(Arc::new(graph));
                bound += 1;
            }
        }
        if browser_tracing::enabled() && references != 0 {
            browser_tracing::instant(
                "layout_fragment",
                "PaintStyle.BindFilterResources",
                &[
                    ("references", references as f64),
                    ("graphs", self.Data().svg_filter_graphs.len() as f64),
                    ("bound", bound as f64),
                ],
            );
        }
    }

    pub fn filters(&self) -> Vec<PaintFilterOperation> {
        let Some(style) = self.Source() else {
            return self.Data().filters.clone();
        };
        let mut filters =
            crate::internal::boundary::paint_style_export::Filters(style, style.Filter());
        crate::internal::boundary::paint_style_export::AppendBoxReflection(style, &mut filters);
        if let Some(reflection) = filters
            .iter_mut()
            .find(|filter| filter.r#type == PaintFilterType::kBoxReflect)
        {
            reflection.reflection_mask_image = self.Data().box_reflection_mask_image.clone();
        }
        self.BindFilterResources(&mut filters);
        filters
    }

    pub fn backdrop_filters(&self) -> Vec<PaintFilterOperation> {
        let Some(style) = self.Source() else {
            return self.Data().backdrop_filters.clone();
        };
        let mut filters =
            crate::internal::boundary::paint_style_export::Filters(style, style.BackdropFilter());
        self.BindFilterResources(&mut filters);
        filters
    }

    pub fn has_filters(&self) -> bool {
        self.Source()
            .map_or(!self.Data().filters.is_empty(), |style| {
                style.HasFilter() || style.HasBoxReflect()
            })
    }

    pub fn has_reference_filter_or_reflection(&self) -> bool {
        self.Source().map_or_else(
            || {
                self.Data().filters.iter().any(|operation| {
                    matches!(
                        operation.r#type,
                        PaintFilterType::kReference | PaintFilterType::kBoxReflect
                    )
                })
            },
            |style| style.HasReferenceFilter() || style.HasBoxReflect(),
        )
    }

    pub fn filter_moves_pixels(&self) -> bool {
        self.Source().map_or_else(
            || {
                self.Data().filters.iter().any(|operation| {
                    matches!(
                        operation.r#type,
                        PaintFilterType::kBlur
                            | PaintFilterType::kDropShadow
                            | PaintFilterType::kReference
                            | PaintFilterType::kBoxReflect
                    )
                })
            },
            |style| {
                use layout_style::style::filter_operation::OperationType;
                style.HasBoxReflect()
                    || style.Filter().Operations().iter().any(|member| {
                        let Some(operation) = (unsafe { member.Get().as_ref() }) else {
                            return true;
                        };
                        matches!(
                            operation.GetType(),
                            OperationType::kBlur
                                | OperationType::kDropShadow
                                | OperationType::kReference
                        )
                    })
            },
        )
    }

    pub fn has_backdrop_filters(&self) -> bool {
        self.Source()
            .map_or(!self.Data().backdrop_filters.is_empty(), |style| {
                style.HasBackdropFilter()
            })
    }

    pub fn backdrop_filter_has_reference(&self) -> bool {
        self.backdrop_filters().iter().any(|filter| {
            filter.r#type == PaintFilterType::kReference && filter.reference_graph.is_none()
        })
    }

    pub fn background_images(&self) -> Vec<BackgroundImageLayer> {
        self.Source().map_or_else(
            || self.Data().background_images.clone(),
            crate::internal::boundary::paint_style_export::Backgrounds,
        )
    }

    pub fn has_background_images(&self) -> bool {
        self.Source()
            .map_or(!self.Data().background_images.is_empty(), |style| {
                style.HasBackgroundImage()
            })
    }

    pub fn mask_images(&self) -> Vec<PaintMaskLayer> {
        let Some(style) = self.Source() else {
            return self.Data().mask_images.clone();
        };
        let mut layers = crate::internal::boundary::paint_style_export::Masks(style);
        for layer in &mut layers {
            let Some(reference) = layer.svg_reference.as_mut() else {
                continue;
            };
            let Some(binding) = self
                .Data()
                .svg_mask_bindings
                .iter()
                .find(|binding| binding.id == reference.id && binding.scope == reference.scope)
            else {
                layer.image_state = PaintMaskImageState::kInvalid;
                continue;
            };
            *reference = binding.clone();
            if let Some(graph) = reference.graph.as_ref() {
                layer.source_error = graph.unsupported.clone();
                if layer.match_source {
                    layer.mode = graph.mode;
                }
            } else {
                layer.image_state = PaintMaskImageState::kInvalid;
            }
        }
        layers
    }

    pub fn has_mask_images(&self) -> bool {
        self.Source()
            .map_or(!self.Data().mask_images.is_empty(), |style| {
                style.MaskLayers().AnyLayerHasImage()
            })
    }

    pub fn has_transform_payload(&self) -> bool {
        self.has_translate()
            || self.has_rotate()
            || self.has_scale()
            || self.has_transform()
            || self.has_offset_transform()
    }

    /// The resolved motion-path matrix exists only for a style with an
    /// offset (ApplyResolvedOffsetTransform), so a live style without one
    /// answers without resolving the projection.
    fn has_offset_transform(&self) -> bool {
        self.Source()
            .is_none_or(|style| style.HasOffset())
            && self.Data().offset_transform.is_some()
    }

    pub fn has_translate(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().translate.is_some(), |style| {
                !style.Translate().is_null()
            })
    }

    pub fn has_rotate(&self) -> bool {
        self.Source().map_or_else(|| self.Data().rotate.is_some(), |style| {
            !style.Rotate().is_null()
        })
    }

    pub fn has_scale(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().scale.is_some(), |style| !style.Scale().is_null())
    }

    pub fn has_transform(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().transform.is_some(), |style| {
                style.HasTransformOperations()
            })
    }

    pub fn translate(&self) -> Option<PaintTransform> {
        self.Source().map_or_else(
            || self.Data().translate.clone(),
            crate::internal::boundary::paint_style_export::Translate,
        )
    }

    pub fn rotate(&self) -> Option<PaintTransform> {
        self.Source().map_or_else(
            || self.Data().rotate.clone(),
            crate::internal::boundary::paint_style_export::Rotate,
        )
    }

    pub fn scale(&self) -> Option<PaintTransform> {
        self.Source().map_or_else(
            || self.Data().scale.clone(),
            crate::internal::boundary::paint_style_export::Scale,
        )
    }

    pub fn transform(&self) -> Option<PaintTransform> {
        self.Source().map_or_else(
            || self.Data().transform.clone(),
            crate::internal::boundary::paint_style_export::Transform,
        )
    }

    pub fn transform_origin(&self) -> Option<PaintTransformOrigin> {
        self.Source().map_or_else(|| self.Data().transform_origin, |style| {
            (self.has_translate()
                || self.has_rotate()
                || self.has_scale()
                || self.has_transform()
                || style.HasOffset())
            .then(|| crate::internal::boundary::paint_style_export::TransformOrigin(style))
        })
    }

    pub fn perspective_origin(&self) -> PaintTransformOrigin {
        self.Source().map_or_else(|| self.Data().perspective_origin, |style| {
            crate::internal::boundary::paint_style_export::PerspectiveOrigin(style)
        })
    }

    pub fn transform_reference_box(&self) -> PaintTransformReferenceBox {
        self.Source()
            .map_or_else(|| self.Data().transform_reference_box, |style| {
                use foundation::ETransformBox;
                use layout_style::style::style_transform::TransformBoxContext;
                match style.UsedTransformBox(TransformBoxContext::kLayoutBox) {
                    ETransformBox::kContentBox => PaintTransformReferenceBox::kContentBox,
                    ETransformBox::kBorderBox => PaintTransformReferenceBox::kBorderBox,
                    ETransformBox::kFillBox
                    | ETransformBox::kStrokeBox
                    | ETransformBox::kViewBox => {
                        panic!(
                            "layout transform-box must be normalized to content-box or border-box"
                        )
                    }
                }
            })
    }

    pub fn perspective(&self) -> Option<f64> {
        self.Source().map_or_else(|| self.Data().perspective, |style| {
            style
                .HasPerspective()
                .then(|| f64::from(style.UsedPerspective()))
        })
    }

    pub fn preserves_3d(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().preserves_3d, |style| style.Preserves3D())
    }

    pub fn has_3d_transform_operation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_3d_transform_operation, |style| {
                style.Has3DTransformOperation()
            })
    }

    pub fn has_nontrivial_3d_transform(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_nontrivial_3d_transform, |style| {
                style.Transform().HasNonPerspective3DOperation()
                    && style.Transform().HasNonTrivial3DComponent()
            })
    }

    pub fn has_trivial_3d_transform(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_trivial_3d_transform, |style| {
                style.Transform().HasNonPerspective3DOperation()
                    && !style.Transform().HasNonTrivial3DComponent()
            })
    }

    pub fn has_3d_translate(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_3d_translate, |style| unsafe {
                let value = style.Translate();
                !value.is_null() && (*value).Z() != 0.0
            })
    }

    pub fn has_3d_rotate(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_3d_rotate, |style| unsafe {
                let value = style.Rotate();
                !value.is_null() && ((*value).X() != 0.0 || (*value).Y() != 0.0)
            })
    }

    pub fn has_3d_scale(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().has_3d_scale, |style| unsafe {
                let value = style.Scale();
                !value.is_null() && (*value).Z() != 1.0
            })
    }

    pub fn isolate_blending(&self) -> bool {
        self.Source().map_or_else(|| self.Data().isolate_blending, |style| {
            style.Isolation() == foundation::EIsolation::kIsolate
        })
    }

    pub fn respect_image_orientation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().respect_image_orientation, |style| {
                style.ImageOrientation()
                    == foundation::RespectImageOrientationEnum::kRespectImageOrientation
            })
    }

    pub fn will_change_scroll_position(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().will_change_scroll_position, |style| {
                style.HasWillChangeScrollPosition()
            })
    }

    pub fn subtree_will_change_contents(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().subtree_will_change_contents, |style| {
                style.SubtreeWillChangeContents()
            })
    }

    fn has_will_change(&self, fallback: bool, property: foundation::CSSPropertyID) -> bool {
        self.Source()
            .map_or(fallback, |style| style.HasWillChangeProperty(property))
    }

    pub fn will_change_transform(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().will_change_transform, |style| {
                style.HasWillChangeAnyTransformProperty()
            })
    }

    pub fn will_change_transform_property(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().will_change_transform_property, |style| {
                style.HasWillChangeProperty(foundation::CSSPropertyID::kTransform)
            })
    }

    pub fn will_change_opacity(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().will_change_opacity, |style| {
                style.HasWillChangeProperty(foundation::CSSPropertyID::kOpacity)
            })
    }

    pub fn will_change_filter(&self) -> bool {
        self.Source().map_or_else(|| self.Data().will_change_filter, |style| {
            style.HasWillChangeProperty(foundation::CSSPropertyID::kFilter)
        })
    }

    pub fn will_change_backdrop_filter(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_backdrop_filter,
            foundation::CSSPropertyID::kBackdropFilter,
        )
    }

    pub fn will_change_clip_path(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_clip_path,
            foundation::CSSPropertyID::kClipPath,
        )
    }

    pub fn will_change_mask(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_mask,
            foundation::CSSPropertyID::kMaskImage,
        )
    }

    pub fn will_change_mix_blend_mode(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_mix_blend_mode,
            foundation::CSSPropertyID::kMixBlendMode,
        )
    }

    pub fn will_change_perspective(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_perspective,
            foundation::CSSPropertyID::kPerspective,
        )
    }

    pub fn will_change_transform_style(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_transform_style,
            foundation::CSSPropertyID::kTransformStyle,
        )
    }

    pub fn will_change_scale(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_scale,
            foundation::CSSPropertyID::kScale,
        )
    }

    pub fn will_change_rotate(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_rotate,
            foundation::CSSPropertyID::kRotate,
        )
    }

    pub fn will_change_translate(&self) -> bool {
        self.has_will_change(
            self.Data().will_change_translate,
            foundation::CSSPropertyID::kTranslate,
        )
    }

    pub fn will_change_other(&self) -> bool {
        self.Source().map_or_else(|| self.Data().will_change_other, |style| {
            [
                foundation::CSSPropertyID::kOffsetPath,
                foundation::CSSPropertyID::kOffsetPosition,
                foundation::CSSPropertyID::kTop,
                foundation::CSSPropertyID::kLeft,
                foundation::CSSPropertyID::kBottom,
                foundation::CSSPropertyID::kRight,
            ]
            .into_iter()
            .any(|property| style.HasWillChangeProperty(property))
        })
    }

    pub fn backface_visibility_hidden(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().backface_visibility_hidden, |style| {
                style.BackfaceVisibility() == foundation::EBackfaceVisibility::kHidden
            })
    }

    pub fn active_transform_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_transform_animation, |style| {
                style.HasCurrentTransformAnimation()
            })
    }

    pub fn active_scale_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_scale_animation, |style| {
                style.HasCurrentScaleAnimation()
            })
    }

    pub fn active_rotate_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_rotate_animation, |style| {
                style.HasCurrentRotateAnimation()
            })
    }

    pub fn active_translate_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_translate_animation, |style| {
                style.HasCurrentTranslateAnimation()
            })
    }

    pub fn active_opacity_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_opacity_animation, |style| {
                // The animation bit is almost always clear; consult the base
                // !important set only for an animating style.
                if !style.HasCurrentOpacityAnimation() {
                    return false;
                }
                let base_data = style.BaseData();
                let important = if base_data.is_null() {
                    std::ptr::null()
                } else {
                    unsafe { &*base_data }.GetBaseImportantSet()
                };
                important.is_null()
                    || !unsafe { &*important }.Has(foundation::CSSPropertyID::kOpacity)
            })
    }

    pub fn active_filter_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_filter_animation, |style| {
                style.HasCurrentFilterAnimation()
            })
    }

    pub fn active_backdrop_filter_animation(&self) -> bool {
        self.Source()
            .map_or_else(|| self.Data().active_backdrop_filter_animation, |style| {
                style.HasCurrentBackdropFilterAnimation()
            })
    }

    pub fn SetVisibleForTesting(&mut self, value: bool) {
        debug_assert!(self.Source().is_none());
        self.DataMut().visible = value;
    }

    pub fn SetOpacityForTesting(&mut self, value: f32) {
        debug_assert!(self.Source().is_none());
        self.DataMut().opacity = value;
    }

    pub fn SetPointerEventsNoneForTesting(&mut self, value: bool) {
        debug_assert!(self.Source().is_none());
        self.DataMut().pointer_events_none = value;
    }
}

impl std::ops::Deref for PaintStyle {
    type Target = PaintStyleData;
    fn deref(&self) -> &Self::Target {
        self.Data()
    }
}

impl std::ops::DerefMut for PaintStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.DataMut()
    }
}
