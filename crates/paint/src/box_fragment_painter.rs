#![allow(non_snake_case)]

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use layoutng_assembly::fragment_tree::FragmentKind;
use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Display, Edges, ListStyleType, NodeKind, ObjectFit, Offset, PaintImage,
    PaintPathCommand, PaintPathVerb, Size, TextDecorationPaint, TextDecorationStyle,
    TransformMatrix, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    BackgroundAttachment, BackgroundBox, BackgroundImageLayer, BackgroundRepeat,
    BackgroundRepeatRule, PaintBlendMode, PaintCornerRadii, PaintStyleData, SvgPaintComponent,
};
use layoutng_style::style::appearance::AppearanceValue;

use crate::background_geometry::{ResolveBackgroundTile, ResolveBackgroundTileSize};
use crate::border_shape_utils::{
    ConicRoundedRectPath, ExpandCornerRadiiAxes, IncludedBorderEdges, ResolveCornerRadii,
    RoundedRectPath,
};
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::drawing_recorder::DrawingRecorder;
use crate::fieldset_painter::FieldsetPainter;
use crate::frame_set_painter::FrameSetPainter;
use crate::geometry_mapper::{
    MapRectToRoot, MapRectWithTransforms, MultiplyTransforms, TranslationTransform,
};
use crate::mathml_painter::MathMLPainter;
use crate::nine_piece_image_painter::NinePieceImagePainter;
use crate::paint_context::PaintContext;
use crate::paint_engine::{
    DisplayItem, DisplayItemType, PaintHitTestOpaqueness, PaintPhase,
    ResolveDocumentImageContainerKey,
};
use crate::paint_info::PaintInfo;
use crate::paint_property_tree::{
    ClipPaintPropertyNode, EffectPaintPropertyNode, PaintPropertyNodeLifecycle, PropertyTreeState,
};
use crate::paint_shader_resolver::ResolvePaintShader;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::scrollable_area_painter::ScrollableAreaPainter;
use crate::svg_shape_painter::SVGShapePainter;
use crate::table_painters::TablePainter;
use crate::theme_painter::ThemePainter;
use crate::PaintRect;

fn ProcessFlag(flag: &'static OnceLock<bool>, name: &str) -> bool {
    *flag.get_or_init(|| std::env::var_os(name).is_some())
}

static TRACE_STYLE_BACKGROUNDS: OnceLock<bool> = OnceLock::new();
static TRACE_STYLE_IMAGES: OnceLock<bool> = OnceLock::new();
static TRACE_TEXT_ORIGIN: OnceLock<bool> = OnceLock::new();

// cpp: paint/box_fragment_painter.h:10-48
pub struct BoxFragmentPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> BoxFragmentPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/box_fragment_painter.h:12-13
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/box_fragment_painter.h:18-19
    // cpp: paint/box_fragment_painter.cc:595-598
    pub fn PaintFragment(node: &'n PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        let _chunk_properties = if paint_info.phase == PaintPhase::kMask {
            crate::paint_context::ScopedPaintChunkProperties::mask(paint_info.context, node)
        } else if node.is_paint_layer {
            crate::paint_context::ScopedPaintChunkProperties::layer(
                paint_info.context,
                node,
                DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase),
            )
        } else if node.applies_transform || node.applies_effect {
            crate::paint_context::ScopedPaintChunkProperties::new(
                paint_info.context,
                node,
                DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase),
            )
        } else {
            crate::paint_context::ScopedPaintChunkProperties::properties_only(
                paint_info.context,
                node,
            )
        };
        Self::new(node, paint_info.context).Paint(paint_info);
    }

    // cpp: paint/box_fragment_painter.h:22-23
    // cpp: paint/box_fragment_painter.cc:600-604
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

    // cpp: paint/box_fragment_painter.h:26
    // cpp: paint/box_fragment_painter.cc:606-611
    fn Paint(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let text_combine = fragment.paint.text_combine.is_some();
        if fragment.paint.svg_hidden_container
            || (!fragment.paint.has_source && !text_combine)
            || !fragment.paint.style.visible
        {
            return;
        }
        if text_combine {
            if !matches!(
                paint_info.phase,
                PaintPhase::kBlockBackground
                    | PaintPhase::kForcedColorsModeBackplate
                    | PaintPhase::kFloat
                    | PaintPhase::kSelfBlockBackgroundOnly
                    | PaintPhase::kDescendantBlockBackgroundsOnly
                    | PaintPhase::kSelfOutlineOnly
            ) {
                // TextCombinePainter paints parent decorations outside the
                // compressed glyph CTM, with emphasis between the two
                // decoration passes.
                // cpp: core/paint/text_combine_painter.cc:35-48
                self.PaintTextDecorations(paint_info.phase, false);
                self.PaintTextCombineEmphasis(paint_info.phase);
                self.PaintTextDecorations(paint_info.phase, true);
            }
            return;
        }
        self.PaintInternal(paint_info);
    }

    // cpp: paint/box_fragment_painter.h:27
    // cpp: paint/box_fragment_painter.cc:613-618
    fn PaintInternal(&self, paint_info: &PaintInfo<'c, 'o>) {
        self.PaintObject(paint_info);
    }

    // cpp: paint/box_fragment_painter.h:28
    // cpp: paint/box_fragment_painter.cc:620-712
    fn PaintObject(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let phase = paint_info.phase;
        if phase == PaintPhase::kMask {
            self.PaintMask();
            return;
        }
        if fragment.paint.source_kind == NodeKind::kFrameSet {
            return;
        }
        // SVGContainerPainter paints content/effects/outline, never CSS box
        // decoration. The LayoutBox fragment storage ABI does not make a
        // nested SVG viewport into a CSS border/background box.
        if IsBackgroundPhase(phase)
            && !matches!(
                fragment.paint.source_kind,
                NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
            )
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.establishes_paint_state
        {
            if self.node.is_root && fragment.paint.background_transfers_to_view {
                self.PaintCanvasBackground(phase);
            }
            let composites_fixed_background = UsesCompositedFixedBackground(self.node);
            if composites_fixed_background {
                self.PaintCompositedFixedBackground(phase);
            }
            // BoxFragmentPainter::PaintBoxDecorationBackgroundWithDecorationData
            // records all decoration commands together (official .cc:1539).
            // Column rules have a separate recorder (.cc:1623).
            let has_box_decoration = HasSupportedBoxDecoration(self.node);
            let decoration_visual_rect = DecorationVisualRect(self.node);
            let cached_box_decoration = has_box_decoration
                && DrawingRecorder::UseCachedDrawingIfPossible(
                    self.context,
                    self.node,
                    DisplayItemIdType::kBoxDecorationBackground,
                    decoration_visual_rect,
                );
            let background_recorder = (has_box_decoration && !cached_box_decoration).then(|| {
                DrawingRecorder::new(
                    self.context,
                    self.node,
                    DisplayItemIdType::kBoxDecorationBackground,
                    decoration_visual_rect,
                )
            });
            // Native theme ownership is a property of the control appearance,
            // independent of whether PaintController moved the previous
            // BoxDecorationBackground record into this artifact.
            let theme_owns_decoration =
                fragment.paint.form_control.as_ref().is_some_and(|control| {
                    !matches!(
                        control.appearance,
                        AppearanceValue::kNone | AppearanceValue::kAuto
                    )
                });
            if !cached_box_decoration {
                self.PaintBoxShadows(phase, false);
                let painted_by_theme = ThemePainter::new(self.node, self.context).Paint(paint_info);
                debug_assert_eq!(painted_by_theme, theme_owns_decoration);
            }
            if !cached_box_decoration && !theme_owns_decoration {
                let style = &*fragment.paint.style;
                let border_radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
                let has_background = !fragment.paint.background_transfers_to_view
                    && (style.background_color.alpha > 0.0 || !style.background_images.is_empty());
                let mut has_border = false;
                let border = IncludedBorderEdges(&fragment.paint);
                let widths = [border.top, border.right, border.bottom, border.left];
                for (edge, width) in widths.into_iter().enumerate() {
                    has_border |= width > 0.0
                        && fragment.paint.border_styles[edge] != BorderLineStyle::kNone
                        && style.border_colors[edge].alpha > 0.0;
                }
                let decoration_bleed_layer = border_radii.HasRadius()
                    && has_background
                    && has_border
                    && BackgroundBleedInset(self.node).is_none();
                if decoration_bleed_layer {
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
                            rect: self.node.PixelSnappedRect(&BackgroundBoxRect(
                                self.node,
                                BackgroundBox::kBorderBox,
                            )),
                            corner_radius: crate::border_shape_utils::UniformCornerRadius(
                                &border_radii,
                            ),
                            corner_radii: border_radii,
                            ..Default::default()
                        },
                        self.node,
                    );
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
                if matches!(
                    fragment.paint.display,
                    Display::kTableSection
                        | Display::kTableHeaderGroup
                        | Display::kTableFooterGroup
                        | Display::kTableRow
                ) {
                    self.PaintTablePartBackground(phase);
                } else if fragment.paint.background_paint_location.PaintsBorderBox()
                    && !composites_fixed_background
                {
                    self.PaintBackground(phase, decoration_bleed_layer);
                }
                self.PaintBorder(phase, decoration_bleed_layer);
                if decoration_bleed_layer {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kRestore,
                            phase,
                            node_id: fragment.node_id,
                            ..Default::default()
                        },
                        self.node,
                    );
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
            if !cached_box_decoration {
                self.PaintBoxShadows(phase, true);
            }
            // Official TablePainter::PaintBoxDecorationBackground (.cc:421)
            // paints all column backgrounds inside the table's surrounding
            // BoxDecorationBackground recorder, not a record per column/cell.
            if !cached_box_decoration && fragment.paint.table.is_some() {
                TablePainter::new(self.node, self.context).PaintColumnBackgrounds(paint_info);
            }
            drop(background_recorder);
            if !theme_owns_decoration && fragment.paint.background_paint_location.PaintsContents() {
                self.PaintContentsBackground(phase);
            }
            if !theme_owns_decoration {
                self.PaintColumnRules(phase);
            }
        }
        // Source: box_fragment_painter.cc:1447-1451 and box_painter.cc:37-122.
        // Record this after the non-scrolling background so background layer
        // squashing is unchanged. The current scoped properties are the local
        // border box; the payload points at the real contents ScrollTranslation.
        if IsBackgroundPhase(phase)
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.scroll_container.is_some()
        {
            let translation = self.node.contents_properties.nodes.transform.clone();
            if translation.scroll.is_some() {
                let style = &*fragment.paint.style;
                let opaqueness = if style.pointer_events_none {
                    PaintHitTestOpaqueness::Transparent
                } else if style.border_radius > 0.0
                    || style.border_radii.is_some()
                    || matches!(
                        fragment.paint.source_kind,
                        NodeKind::kSvgGroup | NodeKind::kSvgViewportContainer
                    )
                {
                    PaintHitTestOpaqueness::Mixed
                } else {
                    PaintHitTestOpaqueness::Opaque
                };
                self.context.borrow_mut().RecordScrollHitTestData(
                    self.node,
                    DecorationVisualRect(self.node),
                    opaqueness,
                    translation,
                    PaintRect::default(),
                );
            }
        }
        let _contents_properties =
            (phase == PaintPhase::kForeground && fragment.kind == FragmentKind::kBox).then(|| {
                crate::paint_context::ScopedPaintChunkProperties::contents(
                    self.context,
                    self.node,
                    phase,
                )
            });
        if phase == PaintPhase::kForeground
            && matches!(
                fragment.kind,
                FragmentKind::kText | FragmentKind::kGeneratedText
            )
        {
            self.PaintText(phase);
        }
        if phase == PaintPhase::kForeground && fragment.paint.mathml.is_some() {
            MathMLPainter::new(self.node, self.context).Paint(paint_info);
        }
        if phase == PaintPhase::kForeground && fragment.paint.svg_shape.is_some() {
            SVGShapePainter::new(self.node, self.context).Paint(paint_info);
        }
        if phase == PaintPhase::kForeground
            && fragment.paint.source_kind == NodeKind::kReplaced
            && fragment.paint.establishes_paint_state
        {
            self.PaintReplaced(phase);
        }
        if IsOutlinePhase(phase)
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.establishes_paint_state
        {
            self.PaintOutline(phase);
        }
    }

    // Blink box_fragment_painter.cc:1343-1370: one native client/phase
    // DrawingRecorder paints the actual mask-image source after the contents.
    fn PaintMask(&self) {
        if !self.node.applies_mask {
            return;
        }
        let fragment = self.node.fragment.as_deref().expect("mask fragment");
        let mut bounds = self.node.mask_layers.first().map(|layer| layer.clip_rect);
        for layer in self.node.mask_layers.iter().skip(1) {
            bounds.as_mut().unwrap().union(layer.clip_rect);
        }
        if let Some(mask_box) = fragment.paint.style.mask_box_image.as_ref() {
            let mask_box_bounds = NinePieceImagePainter::Bounds(self.node, mask_box);
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
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kMask),
            bounds,
        );
        // Chromium records only the mask source in the kMask paint phase. Its
        // EffectPaintPropertyNode supplies kDstIn when PaintArtifactCompositor
        // builds the cc mask layer; applying DstIn inside this source record
        // would first intersect the mask with a transparent destination and
        // publish an empty mask layer.
        if !self.node.mask_layers.is_empty() {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawMask,
                    phase: PaintPhase::kMask,
                    node_id: fragment.node_id,
                    rect: bounds,
                    mask_layers: self.node.mask_layers.clone(),
                    ..Default::default()
                },
                self.node,
            );
        }
        NinePieceImagePainter::PaintMaskBox(self.node, self.context, PaintPhase::kMask);
    }

    // cpp: paint/box_fragment_painter.h:29
    // cpp: paint/box_fragment_painter.cc:813-826
    fn PaintAfterChildren(&self, paint_info: &PaintInfo<'c, 'o>) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if !fragment.paint.has_source || !fragment.paint.style.visible {
            return;
        }
        if paint_info.phase == PaintPhase::kForeground {
            crate::caret_display_item_client::PaintCaret(self.node, self.context);
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
        if paint_info.phase == PaintPhase::kOverlayOverflowControls
            && fragment.paint.establishes_paint_state
            && fragment.paint.scrollbars.is_some()
        {
            ScrollableAreaPainter::new(self.node, self.context).PaintOverflowControls(paint_info);
        }
    }

    // cpp: paint/box_fragment_painter.h:40
    // cpp: paint/box_fragment_painter.cc:714-735
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
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: rule.color,
                    line_style: rule.line_style,
                    stroke_width: rule.stroke_width,
                    ..Default::default()
                },
                self.node,
            );
        }
    }

    // cpp: paint/box_fragment_painter.h:32
    // cpp: paint/box_fragment_painter.cc:737-811
    fn PaintTablePartBackground(&self, phase: PaintPhase) {
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
        let style = &*fragment.paint.style;
        if style.background_color.alpha <= 0.0 && style.background_images.is_empty() {
            return;
        }
        let has_blended_layer = style
            .background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        for cell in cells {
            let cell_fragment = cell
                .fragment
                .as_deref()
                .expect("paint tree node has a fragment");
            if !cell_fragment.paint.has_source || !cell_fragment.paint.style.visible {
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
            if style.background_color.alpha > 0.0 {
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: clip,
                        color: style.background_color,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            for layer in style.background_images.iter().rev() {
                self.PaintBackgroundImage(layer, phase, true, None);
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

    // cpp: paint/box_fragment_painter.h:36
    // cpp: paint/box_fragment_painter.cc:828-958
    fn PaintBoxShadows(&self, phase: PaintPhase, inset: bool) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &*fragment.paint.style;
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
            .then(|| crate::border_shape_utils::IncludedBorderEdges(&fragment.paint))
            .unwrap_or_default();
        let radii = ResolveCornerRadii(
            style,
            &fragment.paint.border_sides,
            outer_size,
            border_inset,
        );
        for shadow in style.box_shadows.iter().rev() {
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
                crate::border_shape_utils::ExpandCornerRadiiUniform(
                    radii,
                    shadow.spread,
                    Size {
                        width: (box_rect.width + 2.0 * shadow.spread).max(0.0),
                        height: (box_rect.height + 2.0 * shadow.spread).max(0.0),
                    },
                )
            };
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawBoxShadow,
                    phase,
                    node_id: fragment.node_id,
                    rect: box_rect,
                    color: shadow.color,
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&shadow_radii),
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
                    shadow_has_opaque_background: style.background_color.alpha >= 1.0,
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

    // cpp: paint/box_fragment_painter.h:30-31
    // cpp: paint/box_fragment_painter.cc:960-1131
    fn PaintBackground(&self, phase: PaintPhase, inside_decoration_bleed_layer: bool) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.background_transfers_to_view {
            return;
        }
        let style = &*fragment.paint.style;
        let has_blended_layer = style
            .background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let border_box = BackgroundBoxRect(self.node, BackgroundBox::kBorderBox);
        let border_radii = BackgroundRadii(self.node, BackgroundBox::kBorderBox);
        let needs_bleed_layer = border_radii.HasRadius()
            && BackgroundBleedInset(self.node).is_none()
            && !style.background_images.is_empty()
            && (style.background_color.alpha > 0.0 || style.background_images.len() > 1)
            && !BackgroundLayerOccludesFollowingLayers(&style.background_images[0]);
        let opens_outer_background_group =
            has_blended_layer || (needs_bleed_layer && !inside_decoration_bleed_layer);
        let has_outer_background_group =
            opens_outer_background_group || inside_decoration_bleed_layer;
        // Blink paints the color and the bottom FillLayer image in one source
        // layer before applying the text mask.  Keeping them together is
        // observable at antialiased mask edges: masking each separately and
        // then compositing would apply the coverage twice.
        let special_color_image_layer = IsSpecialBackgroundClip(style.background_clip)
            && style.background_color.alpha > 0.0
            && style.background_images.iter().any(|layer| {
                layer.paints_background_color && (layer.shader.is_some() || layer.resource_id != 0)
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
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(
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
        if style.background_color.alpha > 0.0
            && IsSpecialBackgroundClip(style.background_clip)
            && !special_color_image_layer
        {
            self.PaintSpecialClippedColor(style.background_color, style.background_clip, phase);
        } else if style.background_color.alpha > 0.0 {
            let bleed_inset = BackgroundBleedInset(self.node);
            let clips_background_stack_for_border =
                bleed_inset.is_some() && !style.background_images.is_empty();
            let shrink_for_border = style.background_clip == BackgroundBox::kBorderBox
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
                BackgroundRadii(self.node, style.background_clip)
            };
            let mut unsnapped_rect = self
                .node
                .PixelSnappedRect(&BackgroundBoxRect(self.node, style.background_clip));
            if shrink_for_border {
                let inset = bleed_inset.expect("shrink for border requires bleed inset");
                unsnapped_rect.x += inset.left;
                unsnapped_rect.y += inset.top;
                unsnapped_rect.width = (unsnapped_rect.width - inset.left - inset.right).max(0.0);
                unsnapped_rect.height = (unsnapped_rect.height - inset.top - inset.bottom).max(0.0);
            }
            let rect = unsnapped_rect;
            if ProcessFlag(&TRACE_STYLE_BACKGROUNDS, "BROWSER_TRACE_STYLE_BACKGROUNDS") {
                eprintln!(
                    "style-background-paint node={} rect={:.3},{:.3},{:.3},{:.3} color={:.6},{:.6},{:.6},{:.6} images={}",
                    fragment.node_id,
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    style.background_color.red,
                    style.background_color.green,
                    style.background_color.blue,
                    style.background_color.alpha,
                    style.background_images.len(),
                );
            }
            if IsVisible(self.node, &rect) {
                shares_bottom_layer_clip = !has_outer_background_group
                    && radii.HasRadius()
                    && !style.background_images.is_empty()
                    && style.background_images.last().unwrap().clip == style.background_clip;
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
                            corner_radius: crate::border_shape_utils::UniformCornerRadius(
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
                        color: style.background_color,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&radii),
                        corner_radii: radii,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
        let mut bottom_layer = true;
        for layer in style.background_images.iter().rev() {
            let outer_group_clips_layer =
                has_outer_background_group && layer.clip == BackgroundBox::kBorderBox;
            self.PaintBackgroundImage(
                layer,
                phase,
                outer_group_clips_layer || (shares_bottom_layer_clip && bottom_layer),
                (special_color_image_layer && layer.paints_background_color)
                    .then_some(style.background_color),
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
    fn PaintCompositedFixedBackground(&self, phase: PaintPhase) {
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
        let style = &*fragment.paint.style;
        let layer = style
            .background_images
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
        if style.background_color.alpha > 0.0 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: style.background_color,
                    ..Default::default()
                },
                self.node,
            );
        }
        self.PaintBackgroundImageSourceWithGeometry(
            layer,
            phase,
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

    /// ViewPainter::PaintRootElementGroup. Layout has already selected either
    /// the document element or first body background. CSS background-clip and
    /// border radii do not constrain the canvas; positioning still uses the
    /// root element box, while the paint area covers at least the viewport and
    /// (for scrolling layers) the complete scrollable contents.
    fn PaintCanvasBackground(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("root paint tree node has a fragment");
        let Some(canvas) = fragment.paint.canvas_background.as_ref() else {
            return;
        };
        let style = &*canvas.style;
        let has_fixed = style.background_images.iter().any(|layer| {
            (layer.shader.is_some() || layer.resource_id != 0)
                && EffectiveBackgroundAttachment(self.node, layer) == BackgroundAttachment::kFixed
        });
        let has_scrolling = style.background_images.iter().any(|layer| {
            (layer.shader.is_some() || layer.resource_id != 0)
                && EffectiveBackgroundAttachment(self.node, layer) != BackgroundAttachment::kFixed
        });
        if !(has_fixed && has_scrolling) {
            self.PaintCanvasBackgroundInOnePropertyState(
                phase,
                style,
                canvas.effective_zoom,
                has_fixed,
            );
            return;
        }

        // A mixed canvas cannot be represented by one PaintRecord: fixed
        // layers use the LayoutView border-box transform, while scroll/local
        // layers use ContentsProperties and its ScrollTranslation. Keep CSS
        // bottom-to-top order by assigning every layer its own stable record.
        let isolates_layers = style
            .background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let group_effect = isolates_layers.then(|| {
            let id = self.context.borrow_mut().AllocatePropertyId();
            Arc::new(CanvasBackgroundEffect(
                id,
                &self.node.properties.nodes,
                self.node.properties.nodes.clip.clone(),
                self.node.properties.nodes.effect.clone(),
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
        let scrolling_rect = CanvasBackgroundRect(self.node, true)
            .expect("root canvas background must have a scrolling paint rectangle");
        if style.background_color.alpha > 0.0 {
            let properties = if let Some(group_effect) = &group_effect {
                let mut state = self.node.contents_properties.nodes.clone();
                state.effect = group_effect.clone();
                crate::paint_context::ScopedPaintChunkProperties::state(self.context, state)
            } else {
                crate::paint_context::ScopedPaintChunkProperties::contents(
                    self.context,
                    self.node,
                    phase,
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
                        color: style.background_color,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            drop(_drawing);
            self.context.borrow_mut().EndEscapableGeometry(self.node);
            drop(properties);
        }
        for (layer_index, layer) in style.background_images.iter().enumerate().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let attachment = EffectiveBackgroundAttachment(self.node, layer);
            let fixed = attachment == BackgroundAttachment::kFixed;
            let rect = CanvasBackgroundRect(self.node, !fixed)
                .expect("root canvas background layer must have a paint rectangle");
            if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
                continue;
            }
            let base_state = if fixed {
                self.node.properties.nodes.clone()
            } else {
                self.node.contents_properties.nodes.clone()
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
                        self.node.properties.nodes.clip.clone(),
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
            } else if fixed {
                crate::paint_context::ScopedPaintChunkProperties::properties_only(
                    self.context,
                    self.node,
                )
            } else {
                crate::paint_context::ScopedPaintChunkProperties::contents(
                    self.context,
                    self.node,
                    phase,
                )
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
            let positioning = if fixed {
                FixedBackgroundPositioningArea(self.node)
                    .unwrap_or_else(|| BackgroundPositioningBoxRect(self.node, layer.origin))
            } else {
                BackgroundPositioningBoxRect(self.node, layer.origin)
            };
            let normalized_layer = layer_effect.as_ref().and_then(|_| {
                (layer.blend_mode != PaintBlendMode::kNormal).then(|| {
                    let mut layer = layer.clone();
                    layer.blend_mode = PaintBlendMode::kNormal;
                    layer
                })
            });
            self.PaintBackgroundImageSourceWithGeometry(
                normalized_layer.as_ref().unwrap_or(layer),
                phase,
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
    fn PaintContentsBackground(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.background_transfers_to_view {
            return;
        }
        let style = &*fragment.paint.style;
        let border_rect = ContentsBackgroundBorderRect(self.node);
        if border_rect.width <= 0.0
            || border_rect.height <= 0.0
            || !IsVisible(self.node, &border_rect)
        {
            return;
        }
        let _properties = crate::paint_context::ScopedPaintChunkProperties::contents(
            self.context,
            self.node,
            phase,
        );
        let _drawing = DrawingRecorder::with_client(
            self.context,
            self.node,
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: DisplayItemIdType::PaintPhaseToClipType(
                    PaintPhase::kSelfBlockBackgroundOnly,
                ),
                fragment: fragment.paint.display_item_fragment,
            },
            crate::paint_chunker::DisplayItemClientInfo::from_fragment(fragment),
            border_rect,
        );
        let isolates_layers = style
            .background_images
            .iter()
            .any(|layer| layer.blend_mode != PaintBlendMode::kNormal);
        let special_color_image_layer = IsSpecialBackgroundClip(style.background_clip)
            && style.background_color.alpha > 0.0
            && style.background_images.iter().any(|layer| {
                layer.paints_background_color && (layer.shader.is_some() || layer.resource_id != 0)
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
        if style.background_color.alpha > 0.0 && !special_color_image_layer {
            let rect = BackgroundBoxRectWithin(self.node, style.background_clip, border_rect);
            if IsSpecialBackgroundClip(style.background_clip) {
                self.BeginSpecialBackgroundClip(phase, PaintBlendMode::kNormal, border_rect);
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: style.background_color,
                    ..Default::default()
                },
                self.node,
            );
            if IsSpecialBackgroundClip(style.background_clip) {
                self.EndSpecialBackgroundClip(phase, style.background_clip);
            }
        }
        for layer in style.background_images.iter().rev() {
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
                            color: style.background_color,
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
                    phase,
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
                    phase,
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
    }

    fn PaintCanvasBackgroundInOnePropertyState(
        &self,
        phase: PaintPhase,
        style: &PaintStyleData,
        effective_zoom: f32,
        fixed: bool,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("root paint tree node has a fragment");
        let rect = CanvasBackgroundRect(self.node, !fixed)
            .expect("root canvas background must have a paint rectangle");
        if rect.width <= 0.0 || rect.height <= 0.0 || !IsVisible(self.node, &rect) {
            return;
        }
        let _properties = if fixed {
            crate::paint_context::ScopedPaintChunkProperties::properties_only(
                self.context,
                self.node,
            )
        } else {
            crate::paint_context::ScopedPaintChunkProperties::contents(
                self.context,
                self.node,
                phase,
            )
        };
        let item_type = if fixed {
            DisplayItemIdType::kFixedAttachmentBackground
        } else {
            DisplayItemIdType::kDocumentBackground
        };
        // ViewPainter records a scrolling canvas in ContentsProperties.
        // The layer's self-background is painted before its descendant clip,
        // so enter the same native overflow clip in the flat record wrapper.
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
        let isolates_layers = style
            .background_images
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
        if style.background_color.alpha > 0.0 {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: style.background_color,
                    ..Default::default()
                },
                self.node,
            );
        }
        for layer in style.background_images.iter().rev() {
            if layer.shader.is_none() && layer.resource_id == 0 {
                continue;
            }
            let positioning = if fixed {
                FixedBackgroundPositioningArea(self.node)
                    .unwrap_or_else(|| BackgroundPositioningBoxRect(self.node, layer.origin))
            } else {
                BackgroundPositioningBoxRect(self.node, layer.origin)
            };
            self.PaintBackgroundImageSourceWithGeometry(
                layer,
                phase,
                true,
                false,
                style,
                effective_zoom,
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
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&radii),
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
            // transparency.  PaintBorderContents' alpha-mask mode preserves
            // the used dotted/dashed/double/rounded border geometry while
            // forcing the mask source opaque.
            self.PaintBorderContents(PaintPhase::kTextClip, false, true);
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

    fn PaintSpecialClippedBackgroundImage(
        &self,
        layer: &BackgroundImageLayer,
        phase: PaintPhase,
        background_color: Option<Color>,
    ) {
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
        self.PaintBackgroundImageSource(&source, phase, true, false);
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
        {
            let mut context = self.context.borrow_mut();
            context.BeginGeometry(node);
            context.BeginCssClip(node);
            context.BeginEscapableGeometry(node, None);
        }
        if fragment.paint.has_source
            && fragment.paint.style.visible
            && matches!(
                fragment.kind,
                FragmentKind::kText | FragmentKind::kGeneratedText
            )
        {
            BoxFragmentPainter::new(node, self.context).PaintText(PaintPhase::kTextClip);
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

    // cpp: paint/box_fragment_painter.h:33-35
    // cpp: paint/box_fragment_painter.cc:1133-1285
    fn PaintBackgroundImage(
        &self,
        layer: &BackgroundImageLayer,
        phase: PaintPhase,
        already_clipped: bool,
        background_color: Option<Color>,
    ) {
        if ProcessFlag(&TRACE_STYLE_IMAGES, "BROWSER_TRACE_STYLE_IMAGES")
            && (layer.shader.is_some() || !layer.source_url.is_empty() || layer.resource_id != 0)
        {
            eprintln!(
                "style-image-paint source={} resource_id={} shader={}",
                layer.source_url,
                layer.resource_id,
                layer.shader.is_some()
            );
        }
        if layer.shader.is_none() && layer.resource_id == 0 {
            return;
        }
        if IsSpecialBackgroundClip(layer.clip) {
            self.PaintSpecialClippedBackgroundImage(layer, phase, background_color);
            return;
        }
        self.PaintBackgroundImageSource(layer, phase, already_clipped, true);
    }

    fn PaintBackgroundImageSource(
        &self,
        layer: &BackgroundImageLayer,
        phase: PaintPhase,
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
            phase,
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
        phase: PaintPhase,
        already_clipped: bool,
        apply_border_bleed_avoidance: bool,
        paint_style: &PaintStyleData,
        effective_zoom: f32,
        geometry_override: Option<(PaintRect, PaintRect)>,
        ignore_border_radius: bool,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let attachment = EffectiveBackgroundAttachment(self.node, layer);
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
                            .local_clip
                            .map(|rect| (rect, self.node.local_clip_radii)),
                    )
                }
            }
        };
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
        let tile_size =
            ResolveBackgroundTileSize(layer, &positioning, image, f64::from(effective_zoom));
        if tile_size.width == 0.0 || tile_size.height == 0.0 {
            return;
        }
        let resolved = ResolveBackgroundTile(layer, &positioning, tile_size);
        let tile = resolved.tile_rect;
        let repeat_x = resolved.repeat_x;
        let repeat_y = resolved.repeat_y;
        let bitmap_tile_scale = if let Some(image) = image {
            Offset {
                x: tile.width / image.width as f64,
                y: tile.height / image.height as f64,
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
                (!paint_style.color_scheme_flags_is_normal)
                    .then_some(u8::from(paint_style.used_dark_color_scheme)),
            )
        });
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
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&outer_radii),
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
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&radii),
                    corner_radii: radii,
                    antialias: radii.HasRadius(),
                    ..Default::default()
                },
                self.node,
            );
        }
        if let Some(shader) = layer.shader.as_ref() {
            let gradient_tile = self.node.PixelSnappedRect(&tile);
            let mut item = DisplayItem {
                r#type: if repeat_x || repeat_y {
                    DisplayItemType::kDrawTiledGradient
                } else {
                    DisplayItemType::kDrawGradientRect
                },
                phase,
                node_id: fragment.node_id,
                rect: if repeat_x || repeat_y {
                    self.node.PixelSnappedRect(&image_paint_rect)
                } else {
                    gradient_tile
                },
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
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawTiledImage,
                    phase,
                    node_id: fragment.node_id,
                    rect: clip,
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: image.width as f64,
                        height: image.height as f64,
                    },
                    tile_rect: tile,
                    resource_id: image.id,
                    image_interpolation_quality: paint_style.image_interpolation_quality,
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
            let image_origin = self.node.PixelSnappedOffset(Offset {
                x: tile.x,
                y: tile.y,
            });
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawImageRect,
                    phase,
                    node_id: fragment.node_id,
                    // BackgroundImageGeometry snaps the no-repeat origin before
                    // snapping the destination size; the phase retains its size.
                    rect: PaintRect {
                        x: image_origin.x,
                        y: image_origin.y,
                        width: tile.width.round(),
                        height: tile.height.round(),
                    },
                    source_rect: PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: image.width as f64,
                        height: image.height as f64,
                    },
                    resource_id: image.id,
                    image_interpolation_quality: paint_style.image_interpolation_quality,
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

    // cpp: paint/box_fragment_painter.h:37
    // cpp: paint/box_fragment_painter.cc:1287-1316
    fn PaintBorder(&self, phase: PaintPhase, inside_decoration_bleed_layer: bool) {
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
                && paint.style.border_colors[edge].alpha > 0.0;
        }
        if !has_paintable_border {
            return;
        }
        let info = PaintInfo::new(self.context, phase);
        let fieldset = FieldsetPainter::new(self.node, self.context);
        let clipped = fieldset.BeginBorderClip(&info);
        self.PaintBorderContents(phase, inside_decoration_bleed_layer, false);
        if clipped {
            fieldset.EndBorderClip(&info);
        }
    }

    // cpp: paint/box_fragment_painter.h:38-39
    // cpp: paint/box_fragment_painter.cc:1318-1785
    fn PaintBorderContents(
        &self,
        phase: PaintPhase,
        inside_decoration_bleed_layer: bool,
        alpha_mask: bool,
    ) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint;
        let opaque_mask_color = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let border_color = |edge: usize| {
            if alpha_mask {
                opaque_mask_color
            } else {
                style.style.border_colors[edge]
            }
        };
        let border_has_alpha =
            |edge: usize| alpha_mask || style.style.border_colors[edge].alpha > 0.0;
        let sides = &style.border_sides;
        let widths = [
            if sides.top { style.border.top } else { 0.0 },
            if sides.right { style.border.right } else { 0.0 },
            if sides.bottom {
                style.border.bottom
            } else {
                0.0
            },
            if sides.left { style.border.left } else { 0.0 },
        ];
        let decoration = self.node.PixelSnappedRect(&DecorationRect(self.node));
        let x = decoration.x;
        let y = decoration.y;
        let width = decoration.width;
        let height = decoration.height;
        let size = Size { width, height };
        let outer_radii = ResolveCornerRadii(&style.style, sides, size, Edges::default());
        let first_style = style.border_styles[0];
        let same_colors = alpha_mask
            || (style.style.border_colors[0] == style.style.border_colors[1]
                && style.style.border_colors[0] == style.style.border_colors[2]
                && style.style.border_colors[0] == style.style.border_colors[3]);
        let uniform_rounded_double = sides.HasAllSides()
            && outer_radii.HasRadius()
            && widths[0] >= 3.0
            && first_style == BorderLineStyle::kDouble
            && widths[1] == widths[0]
            && widths[2] == widths[0]
            && widths[3] == widths[0]
            && style.border_styles[1] == first_style
            && style.border_styles[2] == first_style
            && style.border_styles[3] == first_style
            && same_colors
            && border_has_alpha(0);
        if uniform_rounded_double {
            let band = widths[0] / 3.0;
            let inset_rect = |inset: f64| PaintRect {
                x: x + inset,
                y: y + inset,
                width: (width - 2.0 * inset).max(0.0),
                height: (height - 2.0 * inset).max(0.0),
            };
            let radii_at = |inset: f64| {
                ResolveCornerRadii(
                    &style.style,
                    sides,
                    size,
                    Edges {
                        top: inset,
                        right: inset,
                        bottom: inset,
                        left: inset,
                    },
                )
            };
            if IsVisible(
                self.node,
                &PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
            ) {
                for (outer_inset, inner_inset) in [(0.0, band), (2.0 * band, widths[0])] {
                    let band_outer_radii = radii_at(outer_inset);
                    let band_inner_radii = radii_at(inner_inset);
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawDoubleRoundedRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: inset_rect(outer_inset),
                            inner_rect: inset_rect(inner_inset),
                            color: border_color(0),
                            line_style: BorderLineStyle::kDouble,
                            corner_radius: crate::border_shape_utils::UniformCornerRadius(
                                &band_outer_radii,
                            ),
                            inner_corner_radius: crate::border_shape_utils::UniformCornerRadius(
                                &band_inner_radii,
                            ),
                            corner_radii: band_outer_radii,
                            inner_corner_radii: band_inner_radii,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
            }
            return;
        }
        let uniform_rounded_dash = sides.HasAllSides()
            && outer_radii.HasRadius()
            && widths[0] > 0.0
            && matches!(
                first_style,
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            )
            && widths[1] == widths[0]
            && widths[2] == widths[0]
            && widths[3] == widths[0]
            && style.border_styles[1] == first_style
            && style.border_styles[2] == first_style
            && style.border_styles[3] == first_style
            && same_colors
            && border_has_alpha(0);
        if uniform_rounded_dash {
            let half_width = widths[0] * 0.5;
            let centerline = PaintRect {
                x: x + half_width,
                y: y + half_width,
                width: (width - widths[0]).max(0.0),
                height: (height - widths[0]).max(0.0),
            };
            let center_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: half_width,
                    right: half_width,
                    bottom: half_width,
                    left: half_width,
                },
            );
            if IsVisible(
                self.node,
                &PaintRect {
                    x,
                    y,
                    width,
                    height,
                },
            ) {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kStrokePath,
                    phase,
                    node_id: fragment.node_id,
                    rect: PaintRect {
                        x,
                        y,
                        width,
                        height,
                    },
                    color: border_color(0),
                    line_style: first_style,
                    stroke_width: widths[0],
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&center_radii),
                    corner_radii: center_radii,
                    ..Default::default()
                };
                item.path = RoundedRectPath(&centerline, &center_radii);
                if first_style == BorderLineStyle::kDashed {
                    item.dash_intervals = vec![3.0 * widths[0], 3.0 * widths[0]];
                } else {
                    item.dash_intervals = vec![0.0, 2.0 * widths[0]];
                    item.round_cap = true;
                }
                self.context.borrow_mut().Append(item, self.node);
            }
            return;
        }
        let solid_rounded_ring = sides.HasAllSides()
            && outer_radii.HasRadius()
            && style
                .border_styles
                .iter()
                .all(|line| *line == BorderLineStyle::kSolid)
            && same_colors
            && border_has_alpha(0)
            && widths.iter().any(|width| *width > 0.0);
        if solid_rounded_ring {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inner = PaintRect {
                x: x + widths[3],
                y: y + widths[0],
                width: (width - widths[1] - widths[3]).max(0.0),
                height: (height - widths[0] - widths[2]).max(0.0),
            };
            let inner_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: widths[0],
                    right: widths[1],
                    bottom: widths[2],
                    left: widths[3],
                },
            );
            if IsVisible(self.node, &outer) {
                if inside_decoration_bleed_layer {
                    let mut item = DisplayItem {
                        r#type: DisplayItemType::kDrawPath,
                        phase,
                        node_id: fragment.node_id,
                        rect: inner,
                        color: border_color(0),
                        line_style: BorderLineStyle::kSolid,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&inner_radii),
                        corner_radii: inner_radii,
                        inverse_winding: true,
                        ..Default::default()
                    };
                    item.path = ConicRoundedRectPath(&inner, &inner_radii);
                    self.context.borrow_mut().Append(item, self.node);
                    return;
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawDoubleRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: outer,
                        inner_rect: inner,
                        color: border_color(0),
                        line_style: BorderLineStyle::kSolid,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&outer_radii),
                        inner_corner_radius: crate::border_shape_utils::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: outer_radii,
                        inner_corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            }
            return;
        }
        let mut rounded_solid_edges = outer_radii.HasRadius();
        for (edge, width) in widths.iter().enumerate() {
            if *width > 0.0 && style.border_styles[edge] != BorderLineStyle::kSolid {
                rounded_solid_edges = false;
            }
        }
        if rounded_solid_edges {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inner = PaintRect {
                x: x + widths[3],
                y: y + widths[0],
                width: (width - widths[1] - widths[3]).max(0.0),
                height: (height - widths[0] - widths[2]).max(0.0),
            };
            let inner_radii = ResolveCornerRadii(
                &style.style,
                sides,
                size,
                Edges {
                    top: widths[0],
                    right: widths[1],
                    bottom: widths[2],
                    left: widths[3],
                },
            );
            const ADJACENT_EDGES: [[usize; 2]; 4] = [[3, 1], [0, 2], [1, 3], [2, 0]];
            if IsVisible(self.node, &outer) {
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
                        rect: outer,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&outer_radii),
                        corner_radii: outer_radii,
                        ..Default::default()
                    },
                    self.node,
                );
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipOutRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: inner,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&inner_radii),
                        corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
                for (edge, width) in widths.iter().enumerate() {
                    if *width <= 0.0 || !border_has_alpha(edge) {
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
                    let corner_clips = BorderEdgeCornerClips(edge, &outer, &inner, &outer_radii);
                    let adjacent_renders = |adjacent: usize| {
                        widths[adjacent] > 0.0
                            && !matches!(
                                style.border_styles[adjacent],
                                BorderLineStyle::kNone | BorderLineStyle::kHidden
                            )
                            && border_has_alpha(adjacent)
                    };
                    let first_adjacent = ADJACENT_EDGES[edge][0];
                    let second_adjacent = ADJACENT_EDGES[edge][1];
                    let first_renders = adjacent_renders(first_adjacent);
                    let second_renders = adjacent_renders(second_adjacent);
                    let mut first_antialias =
                        !first_renders || border_color(edge) != border_color(first_adjacent);
                    let mut second_antialias =
                        !second_renders || border_color(edge) != border_color(second_adjacent);

                    // Chromium treats a missing adjacent edge as completed:
                    // there is no visible seam to antialias or miter against.
                    // Keep the real seam's AA mode here; CombinedBorderEdgeClip
                    // also extends the missing seam to the inner boundary.
                    match (first_renders, second_renders) {
                        (false, true) => first_antialias = second_antialias,
                        (true, false) => second_antialias = first_antialias,
                        _ => {}
                    }
                    if first_antialias == second_antialias {
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &CombinedBorderEdgeClip(
                                    edge,
                                    &outer,
                                    &inner,
                                    &corner_clips,
                                    first_renders,
                                    second_renders,
                                ),
                                first_antialias,
                            ),
                            self.node,
                        );
                    } else {
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &corner_clips[0],
                                first_antialias,
                            ),
                            self.node,
                        );
                        self.context.borrow_mut().Append(
                            BorderEdgeClip(
                                fragment.node_id,
                                phase,
                                &corner_clips[1],
                                second_antialias,
                            ),
                            self.node,
                        );
                    }
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: outer,
                            color: border_color(edge),
                            line_style: BorderLineStyle::kSolid,
                            ..Default::default()
                        },
                        self.node,
                    );
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
            return;
        }
        if outer_radii.HasRadius() {
            let outer = PaintRect {
                x,
                y,
                width,
                height,
            };
            let inset_rect = |inset: Edges| PaintRect {
                x: x + inset.left,
                y: y + inset.top,
                width: (width - inset.left - inset.right).max(0.0),
                height: (height - inset.top - inset.bottom).max(0.0),
            };
            let scaled_insets = |scale: f64| Edges {
                top: widths[0] * scale,
                right: widths[1] * scale,
                bottom: widths[2] * scale,
                left: widths[3] * scale,
            };
            let full_insets = scaled_insets(1.0);
            let inner = inset_rect(full_insets);
            let edge_polygons = BorderEdgePolygons(&outer, &inner);
            let half_insets = scaled_insets(0.5);
            let centerline = inset_rect(half_insets);
            let center_radii = ResolveCornerRadii(&style.style, sides, size, half_insets);
            let center_path = RoundedRectPath(&centerline, &center_radii);
            let append_ring = |outer_inset: Edges,
                               inner_inset: Edges,
                               color: Color,
                               line_style: BorderLineStyle| {
                let ring_outer_radii = ResolveCornerRadii(&style.style, sides, size, outer_inset);
                let ring_inner_radii = ResolveCornerRadii(&style.style, sides, size, inner_inset);
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawDoubleRoundedRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: inset_rect(outer_inset),
                        inner_rect: inset_rect(inner_inset),
                        color,
                        line_style,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(
                            &ring_outer_radii,
                        ),
                        inner_corner_radius: crate::border_shape_utils::UniformCornerRadius(
                            &ring_inner_radii,
                        ),
                        corner_radii: ring_outer_radii,
                        inner_corner_radii: ring_inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            };
            if IsVisible(self.node, &outer) {
                for (edge, width) in widths.iter().enumerate() {
                    if *width <= 0.0
                        || matches!(
                            style.border_styles[edge],
                            BorderLineStyle::kNone | BorderLineStyle::kHidden
                        )
                        || !border_has_alpha(edge)
                    {
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
                        BorderEdgeClip(fragment.node_id, phase, &edge_polygons[edge], false),
                        self.node,
                    );
                    let line_style = style.border_styles[edge];
                    let color = border_color(edge);
                    if matches!(
                        line_style,
                        BorderLineStyle::kDashed | BorderLineStyle::kDotted
                    ) {
                        let mut item = DisplayItem {
                            r#type: DisplayItemType::kStrokePath,
                            phase,
                            node_id: fragment.node_id,
                            rect: outer,
                            color,
                            line_style,
                            stroke_width: *width,
                            corner_radius: crate::border_shape_utils::UniformCornerRadius(
                                &center_radii,
                            ),
                            corner_radii: center_radii,
                            ..Default::default()
                        };
                        item.path = center_path.clone();
                        if line_style == BorderLineStyle::kDashed {
                            item.dash_intervals = vec![3.0 * width, 3.0 * width];
                        } else {
                            item.dash_intervals = vec![0.0, 2.0 * width];
                            item.round_cap = true;
                        }
                        self.context.borrow_mut().Append(item, self.node);
                    } else if line_style == BorderLineStyle::kDouble && *width >= 3.0 {
                        append_ring(
                            Edges::default(),
                            scaled_insets(1.0 / 3.0),
                            color,
                            line_style,
                        );
                        append_ring(scaled_insets(2.0 / 3.0), full_insets, color, line_style);
                    } else {
                        append_ring(
                            Edges::default(),
                            full_insets,
                            color,
                            BorderLineStyle::kSolid,
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
            return;
        }
        let rects = [
            PaintRect {
                x,
                y,
                width,
                height: widths[0],
            },
            PaintRect {
                x: x + width - widths[1],
                y,
                width: widths[1],
                height,
            },
            PaintRect {
                x,
                y: y + height - widths[2],
                width,
                height: widths[2],
            },
            PaintRect {
                x,
                y,
                width: widths[3],
                height,
            },
        ];
        let center_lines = [
            PaintRect {
                x,
                y: y + widths[0] / 2.0,
                width,
                height: 0.0,
            },
            PaintRect {
                x: x + width - widths[1] / 2.0,
                y,
                width: 0.0,
                height,
            },
            PaintRect {
                x,
                y: y + height - widths[2] / 2.0,
                width,
                height: 0.0,
            },
            PaintRect {
                x: x + widths[3] / 2.0,
                y,
                width: 0.0,
                height,
            },
        ];
        for (edge, edge_width) in widths.iter().enumerate() {
            if *edge_width <= 0.0
                || matches!(
                    style.border_styles[edge],
                    BorderLineStyle::kNone | BorderLineStyle::kHidden
                )
                || !border_has_alpha(edge)
            {
                continue;
            }
            if !IsVisible(self.node, &rects[edge]) {
                continue;
            }
            let line_style = style.border_styles[edge];
            if matches!(
                line_style,
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            ) {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kStrokeLine,
                    phase,
                    node_id: fragment.node_id,
                    rect: center_lines[edge],
                    color: border_color(edge),
                    line_style,
                    stroke_width: *edge_width,
                    ..Default::default()
                };
                if line_style == BorderLineStyle::kDashed {
                    item.dash_intervals = vec![3.0 * edge_width, 3.0 * edge_width];
                } else {
                    item.dash_intervals = vec![0.0, 2.0 * edge_width];
                    item.round_cap = true;
                }
                self.context.borrow_mut().Append(item, self.node);
                continue;
            }
            if line_style == BorderLineStyle::kDouble && *edge_width >= 3.0 {
                let band = edge_width / 3.0;
                let bands = match edge {
                    0 => [
                        PaintRect {
                            x,
                            y,
                            width,
                            height: band,
                        },
                        PaintRect {
                            x,
                            y: y + 2.0 * band,
                            width,
                            height: band,
                        },
                    ],
                    1 => [
                        PaintRect {
                            x: x + width - band,
                            y,
                            width: band,
                            height,
                        },
                        PaintRect {
                            x: x + width - 3.0 * band,
                            y,
                            width: band,
                            height,
                        },
                    ],
                    2 => [
                        PaintRect {
                            x,
                            y: y + height - band,
                            width,
                            height: band,
                        },
                        PaintRect {
                            x,
                            y: y + height - 3.0 * band,
                            width,
                            height: band,
                        },
                    ],
                    _ => [
                        PaintRect {
                            x,
                            y,
                            width: band,
                            height,
                        },
                        PaintRect {
                            x: x + 2.0 * band,
                            y,
                            width: band,
                            height,
                        },
                    ],
                };
                for band_rect in bands {
                    self.context.borrow_mut().Append(
                        DisplayItem {
                            r#type: DisplayItemType::kDrawRect,
                            phase,
                            node_id: fragment.node_id,
                            rect: band_rect,
                            color: border_color(edge),
                            line_style,
                            ..Default::default()
                        },
                        self.node,
                    );
                }
                continue;
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: rects[edge],
                    color: border_color(edge),
                    line_style,
                    stroke_width: *edge_width,
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(&outer_radii),
                    corner_radii: outer_radii,
                    ..Default::default()
                },
                self.node,
            );
        }
        let one_pixel_solid_border = sides.HasAllSides()
            && widths.iter().all(|width| *width == 1.0)
            && style
                .border_styles
                .iter()
                .all(|line| *line == BorderLineStyle::kSolid)
            && (alpha_mask
                || style
                    .style
                    .border_colors
                    .iter()
                    .all(|color| color.alpha >= 1.0));
        if one_pixel_solid_border {
            let channel = |first: f32, second: f32| {
                let first_byte = (first * 255.0).round() as i32;
                let second_byte = (second * 255.0).round() as i32;
                ((first_byte + second_byte) / 2) as f32 / 255.0
            };
            let average = |a: Color, b: Color| Color {
                red: channel(a.red, b.red),
                green: channel(a.green, b.green),
                blue: channel(a.blue, b.blue),
                alpha: 1.0,
            };
            let corners = [
                PaintRect {
                    x,
                    y,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x: x + width - 1.0,
                    y,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x: x + width - 1.0,
                    y: y + height - 1.0,
                    width: 1.0,
                    height: 1.0,
                },
                PaintRect {
                    x,
                    y: y + height - 1.0,
                    width: 1.0,
                    height: 1.0,
                },
            ];
            const ADJACENT: [[usize; 2]; 4] = [[0, 3], [0, 1], [2, 1], [2, 3]];
            for (corner, corner_rect) in corners.into_iter().enumerate() {
                let first = border_color(ADJACENT[corner][0]);
                let second = border_color(ADJACENT[corner][1]);
                if first == second {
                    continue;
                }
                self.context.borrow_mut().Append(
                    DisplayItem {
                        r#type: DisplayItemType::kDrawRect,
                        phase,
                        node_id: fragment.node_id,
                        rect: corner_rect,
                        color: average(first, second),
                        line_style: BorderLineStyle::kSolid,
                        antialias: false,
                        ..Default::default()
                    },
                    self.node,
                );
            }
        }
    }

    // cpp: paint/box_fragment_painter.h:41
    // cpp: paint/box_fragment_painter.cc:1787-1969
    fn PaintText(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &*fragment.paint.style;
        let text_clip = phase == PaintPhase::kTextClip;
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
        if fragment.paint.glyph_runs.is_empty() && fragment.paint.list_marker_symbol.is_none() {
            return;
        }
        // TextFragmentPainter records this text item, including its shadows,
        // symbol marker and decorations; glyph runs are paint-record ops.
        // text_fragment_painter.cc:429-444: native SelfInkOverflowRect is
        // moved by the physical fragment offset, then enclosed to pixels.
        let in_drawing = self.context.borrow().InDrawingRecorder();
        let drawing_type = DisplayItemIdType::PaintPhaseToDrawingType(phase);
        let visual_rect = (!in_drawing).then(|| {
            let mut visual_rect = FragmentVisualRect(self.node);
            // SkGlyphRunPainter's analytic/LCD coverage may touch the native
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
        if visual_rect.is_some_and(|visual_rect| {
            DrawingRecorder::UseCachedDrawingIfPossible(
                self.context,
                self.node,
                drawing_type,
                visual_rect,
            )
        }) {
            return;
        }
        let _drawing = visual_rect.map(|visual_rect| {
            DrawingRecorder::new(self.context, self.node, drawing_type, visual_rect)
        });
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
            if fragment.paint.glyph_runs.is_empty() {
                panic!("symbol list marker has no shaped run");
            }
            let ascent = fragment.paint.glyph_runs[0].baseline as i32;
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
                color: if text_clip { mask_color } else { style.color },
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
        let editing = (!text_clip)
            .then(|| self.context.borrow().caret)
            .flatten()
            .filter(|state| fragment.paint.text_control_host == Some(state.node_id));
        let range_rect = |range: (u32, u32)| {
            layoutng_assembly::caret::geometry::TextFragmentRangeRect(fragment, range.0, range.1)
                .map(|local| {
                    let text_origin = TextPaintOrigin(self.node);
                    let top = self.node.paint_offset.y
                        + fragment.paint.text_line_top_offset.unwrap_or(0.0);
                    PaintRect {
                        x: text_origin.x + local.offset.x,
                        y: self.node.paint_offset.y + local.offset.y + self.node.RoundedPaintY(top)
                            - top,
                        width: local.size.width,
                        height: local.size.height,
                    }
                })
        };
        let control_selection = editing
            .and_then(|state| state.selection)
            .and_then(range_rect);
        let document_selection = (!text_clip
            && fragment.paint.text_control_host.is_none()
            && !fragment.paint.style.user_select_none)
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
                (range_start < range_end).then_some((range_start, range_end))
            })
            .and_then(range_rect);
        let selection_rect = control_selection.or(document_selection);
        if let Some(rect) = selection_rect {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: Color {
                        red: 0.0,
                        green: 0.4,
                        blue: 1.0,
                        alpha: 1.0,
                    },
                    ..Default::default()
                },
                self.node,
            );
        }
        let mut glyph_foregrounds = Vec::new();
        for input_run in fragment.paint.glyph_runs.iter() {
            let make_item = |color: Color,
                             shadow_offset: Offset,
                             blur_radius: f64,
                             is_shadow: bool,
                             stroke_glyphs: bool| {
                let mut origin = TextPaintOrigin(self.node);
                let mut physical_box_offset = self.node.paint_offset;
                let mut text_line_top = 0.0;
                if let Some(text_line_top_offset) = fragment.paint.text_line_top_offset {
                    let line_top = self.node.paint_offset.y + text_line_top_offset;
                    text_line_top = line_top;
                    let snap_delta = self.node.RoundedPaintY(line_top) - line_top;
                    if ProcessFlag(&TRACE_TEXT_ORIGIN, "RECHROM_TRACE_TEXT_ORIGIN") {
                        eprintln!(
                            "text-origin node={} offset={:.4},{:.4} line_top={:.4} snap={:.4} baseline={:.4} origin={:.4},{:.4} font={:.4}",
                            fragment.node_id,
                            self.node.paint_offset.x,
                            self.node.paint_offset.y,
                            line_top,
                            snap_delta,
                            input_run.baseline,
                            origin.x,
                            origin.y + snap_delta + input_run.baseline,
                            input_run.font_size,
                        );
                    }
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
                    glyphs: Arc::new(input_run.glyphs.clone()),
                    ..Default::default()
                };
                if stroke_glyphs {
                    if svg.is_some() {
                        item.stroke_width = style.svg_stroke_width;
                        item.dash_intervals = style.svg_stroke_dash_array.clone();
                        item.dash_offset = style.svg_stroke_dash_offset;
                        item.svg_line_cap = style.svg_stroke_line_cap;
                        item.svg_line_join = style.svg_stroke_line_join;
                        item.miter_limit = style.svg_stroke_miter_limit;
                    } else {
                        item.stroke_width = style.text_stroke_width;
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
                // overflow still requires InlinePaintContext. Keep that proof
                // only for the ordinary untransformed fill path.
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
            for shadow in style.text_shadows.iter().rev() {
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
                        style.text_stroke_width
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
                        if component == SvgPaintComponent::kFill
                            && style.text_fill_color.alpha > 0.0
                        {
                            glyph_foregrounds.push(make_item(
                                style.text_fill_color,
                                Offset::default(),
                                0.0,
                                false,
                                false,
                            ));
                        } else if component == SvgPaintComponent::kStroke
                            && style.text_stroke_width > 0.0
                            && style.text_stroke_color.alpha > 0.0
                        {
                            glyph_foregrounds.push(make_item(
                                style.text_stroke_color,
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
                            && (style.svg_fill_server.is_some()
                                || style.svg_fill.is_some_and(|fill| fill.alpha > 0.0))
                        {
                            let mut item = make_item(
                                style.svg_fill.unwrap_or_default(),
                                Offset::default(),
                                0.0,
                                false,
                                false,
                            );
                            if let Some(server) = style.svg_fill_server.as_ref() {
                                item.paint_shader = Some(ResolvePaintShader(
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
                                ));
                            }
                            glyph_foregrounds.push(item);
                        } else if component == SvgPaintComponent::kStroke
                            && (style.svg_stroke_server.is_some()
                                || style.svg_stroke.is_some_and(|stroke| stroke.alpha > 0.0))
                            && style.svg_stroke_width > 0.0
                        {
                            let mut item = make_item(
                                style.svg_stroke.unwrap_or_default(),
                                Offset::default(),
                                0.0,
                                false,
                                true,
                            );
                            if let Some(server) = style.svg_stroke_server.as_ref() {
                                item.paint_shader = Some(ResolvePaintShader(
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
                                ));
                            }
                            glyph_foregrounds.push(item);
                        }
                    }
                }
            }
        }
        self.PaintTextDecorations(phase, false);
        for item in &glyph_foregrounds {
            self.context.borrow_mut().Append(item.clone(), self.node);
        }
        // Repaint selected glyphs under the selection clip, preserving ligatures
        // and fallback runs rather than splitting glyph ids at character offsets.
        if let Some(rect) = selection_rect {
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
                    rect,
                    ..Default::default()
                },
                self.node,
            );
            for mut item in glyph_foregrounds {
                item.color = Color {
                    red: 1.0,
                    green: 1.0,
                    blue: 1.0,
                    alpha: 1.0,
                };
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
        if let Some(mut rect) = editing
            .and_then(|state| state.composition)
            .and_then(range_rect)
        {
            rect.y += rect.height - 1.0;
            rect.height = 1.0;
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawRect,
                    phase,
                    node_id: fragment.node_id,
                    rect,
                    color: style.color,
                    ..Default::default()
                },
                self.node,
            );
        }
        self.PaintTextDecorations(phase, true);
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

    // cpp: paint/box_fragment_painter.h:42
    // cpp: paint/box_fragment_painter.cc:1971-2114
    fn PaintTextDecorations(&self, phase: PaintPhase, line_through: bool) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &*fragment.paint.style;
        let text_clip = phase == PaintPhase::kTextClip;
        let mask_color = Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
        let metrics = fragment
            .paint
            .glyph_runs
            .first()
            .map(|run| (run.font_size, run.ascent, run.baseline, run.writing_mode))
            .or_else(|| {
                fragment.paint.text_combine.as_ref().map(|combine| {
                    (
                        combine.font_size,
                        combine.text_top_offset,
                        combine.text_top_offset,
                        fragment.paint.writing_mode,
                    )
                })
            });
        let Some((font_size, ascent, baseline, writing_mode)) = metrics else {
            return;
        };
        let paint_decoration = |decoration: &TextDecorationPaint| {
            if if line_through {
                !decoration.line_through
            } else {
                !decoration.underline && !decoration.overline
            } {
                return;
            }
            let color = if text_clip {
                mask_color
            } else {
                decoration.color.unwrap_or(style.color)
            };
            // Blink resolves the initial `text-decoration-thickness: auto` to one
            // tenth of the used font size. Font underline metrics are used only by
            // the distinct `from-font` value, not by `auto`.
            let auto_thickness = font_size / 10.0;
            let used_thickness = |fallback: f64| {
                let resolved = decoration.thickness.unwrap_or(fallback);
                if !resolved.is_finite() {
                    panic!("text decoration geometry is invalid");
                }
                resolved.max(if fragment.paint.svg_text.is_some() {
                    0.0
                } else {
                    1.0
                })
            };
            let paint_line = |line: PaintRect, default_thickness: f64, skip_ink: bool| {
                let thickness = used_thickness(default_thickness);
                if !decoration.underline_offset.is_finite() {
                    panic!("text decoration geometry is invalid");
                }
                let make_item = |item_color: Color,
                                 shadow_offset: Offset,
                                 blur_radius: f64,
                                 is_shadow: bool| {
                    let mut item = DisplayItem {
                        r#type: if decoration.style == TextDecorationStyle::kWavy {
                            DisplayItemType::kStrokeWavyLine
                        } else {
                            DisplayItemType::kStrokeLine
                        },
                        phase,
                        node_id: fragment.node_id,
                        rect: line,
                        color: item_color,
                        decoration_style: decoration.style,
                        stroke_width: thickness,
                        shadow_offset,
                        blur_radius,
                        is_shadow,
                        is_text_decoration: true,
                        skip_ink,
                        ..Default::default()
                    };
                    if matches!(
                        decoration.style,
                        TextDecorationStyle::kDashed | TextDecorationStyle::kDotted
                    ) {
                        // DecorationLinePainter::DrawLineAsStroke snaps the
                        // center line and rounds StyledStrokeData thickness
                        // before creating its dash path effect.
                        let stroke_thickness = thickness.round().max(1.0);
                        let snapped_left = line.x.trunc();
                        let snapped_right = (line.x + line.width).trunc();
                        item.rect.x = snapped_left;
                        item.rect.width = (snapped_right - snapped_left).max(0.0);
                        item.rect.y = (line.y + (stroke_thickness / 2.0).max(0.5)).floor();
                        if stroke_thickness as i64 % 2 != 0 {
                            item.rect.y += 0.5;
                        }
                        if decoration.style == TextDecorationStyle::kDashed {
                            let dash_ratio = if stroke_thickness >= 3.0 { 2.0 } else { 3.0 };
                            let gap_ratio = if stroke_thickness >= 3.0 { 1.0 } else { 2.0 };
                            item.dash_intervals =
                                vec![dash_ratio * stroke_thickness, gap_ratio * stroke_thickness];
                        } else if stroke_thickness <= 3.0 {
                            item.dash_intervals = vec![stroke_thickness, stroke_thickness];
                        } else {
                            item.dash_intervals = vec![0.0, 2.0 * stroke_thickness - 0.01];
                            item.round_cap = true;
                            item.rect.x += stroke_thickness / 2.0;
                            item.rect.width = (item.rect.width - stroke_thickness).max(0.0);
                        }
                    }
                    item
                };
                for shadow in style.text_shadows.iter().rev() {
                    if text_clip || shadow.color.alpha <= 0.0 {
                        continue;
                    }
                    let item = make_item(shadow.color, shadow.offset, shadow.blur_radius, true);
                    let mut bounds = PaintRect {
                        x: item.rect.x + shadow.offset.x,
                        y: item.rect.y + shadow.offset.y,
                        width: item.rect.width,
                        height: item.rect.height,
                    };
                    let outset = 1.5 * shadow.blur_radius + thickness;
                    bounds.x -= outset;
                    bounds.y -= outset;
                    bounds.width += 2.0 * outset;
                    bounds.height += 2.0 * outset;
                    if IsVisible(self.node, &bounds) {
                        self.context.borrow_mut().Append(item, self.node);
                    }
                }
                let mut bounds = line;
                bounds.x -= thickness;
                bounds.y -= thickness;
                bounds.width += 2.0 * thickness;
                bounds.height += 2.0 * thickness;
                if IsVisible(self.node, &bounds) {
                    self.context
                        .borrow_mut()
                        .Append(make_item(color, Offset::default(), 0.0, false), self.node);
                }
            };
            let origin = TextPaintOrigin(self.node);
            let paint_size = TextPaintSize(self.node);
            let x = origin.x;
            let y = origin.y;
            let horizontal = writing_mode == WritingMode::kHorizontalTb;
            if line_through {
                if horizontal {
                    let thickness = used_thickness(auto_thickness);
                    paint_line(
                        PaintRect {
                            x,
                            y: y + 2.0 * ascent / 3.0 - thickness / 2.0,
                            width: paint_size.width,
                            height: 0.0,
                        },
                        thickness,
                        false,
                    );
                } else {
                    paint_line(
                        PaintRect {
                            x: x + paint_size.width / 2.0,
                            y,
                            width: 0.0,
                            height: paint_size.height,
                        },
                        auto_thickness,
                        false,
                    );
                }
                return;
            }
            if decoration.overline {
                if horizontal {
                    paint_line(
                        PaintRect {
                            x,
                            y: y + auto_thickness / 2.0,
                            width: paint_size.width,
                            height: 0.0,
                        },
                        auto_thickness,
                        decoration.skip_ink,
                    );
                } else {
                    let line_x = if writing_mode == WritingMode::kVerticalRl {
                        x + paint_size.width
                    } else {
                        x
                    };
                    paint_line(
                        PaintRect {
                            x: line_x,
                            y,
                            width: 0.0,
                            height: paint_size.height,
                        },
                        auto_thickness,
                        decoration.skip_ink,
                    );
                }
            }
            if decoration.underline {
                if horizontal {
                    let baseline_offset = if decoration.underline_offset_auto {
                        (used_thickness(auto_thickness) / 2.0).ceil().max(1.0)
                    } else {
                        decoration.underline_offset.round()
                    };
                    let wavy_offset = if decoration.style == TextDecorationStyle::kWavy {
                        used_thickness(auto_thickness) + 1.0
                    } else {
                        0.0
                    };
                    paint_line(
                        PaintRect {
                            x,
                            y: y + baseline + baseline_offset + wavy_offset,
                            width: paint_size.width,
                            height: 0.0,
                        },
                        auto_thickness,
                        decoration.skip_ink,
                    );
                } else {
                    let direction = if writing_mode == WritingMode::kVerticalRl {
                        -1.0
                    } else {
                        1.0
                    };
                    let line_x = (if direction < 0.0 {
                        x
                    } else {
                        x + paint_size.width
                    }) + direction * decoration.underline_offset;
                    paint_line(
                        PaintRect {
                            x: line_x,
                            y,
                            width: 0.0,
                            height: paint_size.height,
                        },
                        auto_thickness,
                        decoration.skip_ink,
                    );
                }
            }
        };
        if style.applied_text_decorations.is_empty() {
            paint_decoration(&style.text_decoration);
        } else {
            for decoration in &style.applied_text_decorations {
                paint_decoration(decoration);
            }
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
        } else {
            emphasis.color
        };
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
                glyphs: Arc::new(run.glyphs.clone()),
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

    // cpp: paint/box_fragment_painter.h:43
    // cpp: paint/box_fragment_painter.cc:2116-2245
    fn PaintReplaced(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint;
        let content_box = PaintRect {
            x: self.node.paint_offset.x + style.border.left + style.padding.left,
            y: self.node.paint_offset.y + style.border.top + style.padding.top,
            width: fragment.content_size.width,
            height: fragment.content_size.height,
        };
        let clips_to_content_box = style
            .replaced_content
            .as_ref()
            .map_or(true, |content| content.clips_to_content_box);
        if let Some(shader) = style.style.replaced_image_shader.as_ref() {
            let mut destination = content_box;
            if let Some(replaced_content) = style.replaced_content.as_ref() {
                destination = PaintRect {
                    x: self.node.paint_offset.x + replaced_content.offset.x,
                    y: self.node.paint_offset.y + replaced_content.offset.y,
                    width: replaced_content.size.width,
                    height: replaced_content.size.height,
                };
            }
            let destination = self.node.PixelSnappedRect(&destination);
            let visual_rect = if clips_to_content_box {
                destination.intersection(self.node.PixelSnappedRect(&content_box))
            } else {
                destination
            };
            if destination.width <= 0.0
                || destination.height <= 0.0
                || visual_rect.width <= 0.0
                || visual_rect.height <= 0.0
                || !IsVisible(self.node, &visual_rect)
            {
                return;
            }
            let _drawing = DrawingRecorder::new(
                self.context,
                self.node,
                DisplayItemIdType::PaintPhaseToDrawingType(phase),
                visual_rect,
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
        let Some(id) = style.style.image_resource_id else {
            return;
        };
        let image = FindImage(self.node, id);
        let image_orientation = if style.style.respect_image_orientation {
            image.orientation
        } else {
            image_resource::ImageOrientation::kDefault
        };
        let (source_width, source_height) =
            image_orientation.oriented_size(f64::from(image.width), f64::from(image.height));
        let mut destination = content_box;
        let mut source = PaintRect {
            x: 0.0,
            y: 0.0,
            width: source_width,
            height: source_height,
        };
        if destination.width <= 0.0 || destination.height <= 0.0 {
            return;
        }
        if !style.style.object_position.x.is_finite() || !style.style.object_position.y.is_finite()
        {
            panic!("object position must be finite");
        }
        // ImagePainter receives a zoomed content box. Match
        // LayoutReplaced::IntrinsicSize: resource pixels first lose their
        // image density and then acquire the element's effective zoom. This
        // fallback is observable while replaced-content geometry is pending,
        // so omitting zoom makes `object-fit: none/scale-down` jump sizes.
        let effective_zoom = f64::from(style.effective_zoom);
        if !effective_zoom.is_finite() || effective_zoom <= 0.0 {
            panic!("replaced image effective zoom must be finite and positive");
        }
        let intrinsic_width = source.width / image.resolution_scale * effective_zoom;
        let intrinsic_height = source.height / image.resolution_scale * effective_zoom;
        let mut fit = style.style.object_fit;
        if let Some(replaced_content) = style.replaced_content.as_ref() {
            destination = PaintRect {
                x: self.node.paint_offset.x + replaced_content.offset.x,
                y: self.node.paint_offset.y + replaced_content.offset.y,
                width: replaced_content.size.width,
                height: replaced_content.size.height,
            };
        } else if fit == ObjectFit::kScaleDown {
            let fits_without_scaling =
                intrinsic_width <= destination.width && intrinsic_height <= destination.height;
            fit = if fits_without_scaling {
                ObjectFit::kNone
            } else {
                ObjectFit::kContain
            };
        }
        if style.replaced_content.is_none() && matches!(fit, ObjectFit::kContain | ObjectFit::kNone)
        {
            let scale = if fit == ObjectFit::kNone {
                1.0
            } else {
                (destination.width / intrinsic_width).min(destination.height / intrinsic_height)
            };
            let fitted_width = intrinsic_width * scale;
            let fitted_height = intrinsic_height * scale;
            destination.x += (destination.width - fitted_width) * style.style.object_position.x
                + style.style.object_position_offset.x;
            destination.y += (destination.height - fitted_height) * style.style.object_position.y
                + style.style.object_position_offset.y;
            destination.width = fitted_width;
            destination.height = fitted_height;
        } else if style.replaced_content.is_none() && fit == ObjectFit::kCover {
            let scale =
                (destination.width / intrinsic_width).max(destination.height / intrinsic_height);
            // Blink ImagePainter::PaintIntoRect snaps the fitted image rectangle
            // before mapping the clipped content rectangle back to source pixels.
            // Cropping the floating source first changes both the sampling scale
            // and the selected mip level, especially for small object-fit images.
            let fitted_width = intrinsic_width * scale;
            let fitted_height = intrinsic_height * scale;
            destination.x += (destination.width - fitted_width) * style.style.object_position.x
                + style.style.object_position_offset.x;
            destination.y += (destination.height - fitted_height) * style.style.object_position.y
                + style.style.object_position_offset.y;
            destination.width = fitted_width;
            destination.height = fitted_height;
        }
        let document_image_container_key = ResolveDocumentImageContainerKey(
            image,
            Size {
                width: destination.width,
                height: destination.height,
            },
            f64::from(style.effective_zoom),
            fragment.paint.resources.as_deref(),
            (!style.style.color_scheme_flags_is_normal)
                .then_some(u8::from(style.style.used_dark_color_scheme)),
        );
        let needs_clipping = clips_to_content_box
            && (destination.x < content_box.x
                || destination.y < content_box.y
                || destination.x + destination.width > content_box.x + content_box.width
                || destination.y + destination.height > content_box.y + content_box.height);
        destination = self.node.PixelSnappedRect(&destination);
        if destination.width <= 0.0 || destination.height <= 0.0 {
            return;
        }
        // ImagePainter::PaintIntoRect records the replaced foreground as one
        // drawing, independently of the image sampling/cropping commands.
        // Official core/paint/image_painter.cc:198.
        let _drawing = DrawingRecorder::new(
            self.context,
            self.node,
            DisplayItemIdType::PaintPhaseToDrawingType(phase),
            destination,
        );
        if !needs_clipping {
            if !IsVisible(self.node, &destination) {
                return;
            }
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kDrawImageRect,
                    phase,
                    node_id: fragment.node_id,
                    rect: destination,
                    source_rect: source,
                    resource_id: id,
                    image_interpolation_quality: style.style.image_interpolation_quality,
                    image_orientation,
                    document_image_container_key: document_image_container_key.clone(),
                    ..Default::default()
                },
                self.node,
            );
            return;
        }
        let snapped_content = self.node.PixelSnappedRect(&content_box);
        let clipped_left = destination.x.max(snapped_content.x);
        let clipped_top = destination.y.max(snapped_content.y);
        let clipped_right =
            (destination.x + destination.width).min(snapped_content.x + snapped_content.width);
        let clipped_bottom =
            (destination.y + destination.height).min(snapped_content.y + snapped_content.height);
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
        if !IsVisible(self.node, &destination) {
            return;
        }
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                phase,
                node_id: fragment.node_id,
                rect: destination,
                source_rect: source,
                resource_id: id,
                image_interpolation_quality: style.style.image_interpolation_quality,
                image_orientation,
                document_image_container_key,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/box_fragment_painter.h:44
    // cpp: paint/box_fragment_painter.cc:2247-2492
    fn PaintOutline(&self, phase: PaintPhase) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &*fragment.paint.style;
        let auto_outline = fragment
            .paint
            .form_control
            .as_ref()
            .is_some_and(|control| control.auto_focus_ring);
        let has_outline = self.node.paints_outline
            && (auto_outline
                || (style.outline_width > 0.0
                    && style.outline_color.alpha > 0.0
                    && style.outline_style != BorderLineStyle::kNone));
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
        if !self.node.paints_outline
            || style.outline_width <= 0.0
            || style.outline_color.alpha <= 0.0
            || style.outline_style == BorderLineStyle::kNone
        {
            return;
        }
        if !style.outline_width.is_finite() || !style.outline_offset.is_finite() {
            panic!("outline geometry is invalid");
        }
        if self.node.outline_rects.len() > 1 {
            let expanded_rects = |additional_outset: f64| {
                let mut rects = Vec::with_capacity(self.node.outline_rects.len());
                for source in &self.node.outline_rects {
                    let horizontal_offset = style.outline_offset.max(-source.width / 2.0);
                    let vertical_offset = style.outline_offset.max(-source.height / 2.0);
                    rects.push(ExpandedOutlineRect(
                        source,
                        horizontal_offset + additional_outset,
                        vertical_offset + additional_outset,
                    ));
                }
                rects
            };
            let outer_rects = expanded_rects(style.outline_width);
            let bounds = UnionRects(&outer_rects);
            if !IsVisible(self.node, &bounds) {
                return;
            }
            let mut outline = DisplayItem {
                phase,
                node_id: fragment.node_id,
                rect: bounds,
                color: style.outline_color,
                line_style: style.outline_style,
                ..Default::default()
            };
            if matches!(
                style.outline_style,
                BorderLineStyle::kDashed | BorderLineStyle::kDotted
            ) {
                outline.r#type = DisplayItemType::kStrokePath;
                outline.stroke_width = style.outline_width;
                AppendUnionBoundary(
                    &expanded_rects(style.outline_width / 2.0),
                    &mut outline.path,
                );
                if style.outline_style == BorderLineStyle::kDashed {
                    outline.dash_intervals =
                        vec![3.0 * style.outline_width, 3.0 * style.outline_width];
                } else {
                    outline.dash_intervals = vec![0.0, 2.0 * style.outline_width];
                    outline.round_cap = true;
                }
            } else {
                outline.r#type = DisplayItemType::kDrawPath;
                outline.even_odd = true;
                AppendUnionBoundary(&outer_rects, &mut outline.path);
                if style.outline_style == BorderLineStyle::kDouble && style.outline_width > 2.0 {
                    let third = (style.outline_width / 3.0).round();
                    AppendUnionBoundary(
                        &expanded_rects(style.outline_width - third),
                        &mut outline.path,
                    );
                    AppendUnionBoundary(&expanded_rects(third), &mut outline.path);
                }
                AppendUnionBoundary(&expanded_rects(0.0), &mut outline.path);
            }
            if !outline.path.is_empty() {
                self.context.borrow_mut().Append(outline, self.node);
            }
            return;
        }
        let horizontal_offset = style.outline_offset.max(-fragment.size.width / 2.0);
        let vertical_offset = style.outline_offset.max(-fragment.size.height / 2.0);
        let center_outset_x = horizontal_offset + style.outline_width / 2.0;
        let center_outset_y = vertical_offset + style.outline_width / 2.0;
        let source_box = PaintRect {
            x: self.node.paint_offset.x,
            y: self.node.paint_offset.y,
            width: fragment.size.width,
            height: fragment.size.height,
        };
        let rect = ExpandedOutlineRect(&source_box, center_outset_x, center_outset_y);
        let outer_bounds = ExpandedOutlineRect(
            &source_box,
            horizontal_offset + style.outline_width,
            vertical_offset + style.outline_width,
        );
        if !IsVisible(self.node, &outer_bounds) {
            return;
        }
        let border_radii = ResolveCornerRadii(
            style,
            &fragment.paint.border_sides,
            fragment.size,
            Edges::default(),
        );
        if style.outline_style == BorderLineStyle::kDouble && style.outline_width > 2.0 {
            let third = (style.outline_width / 3.0).round();
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
                        color: style.outline_color,
                        line_style: BorderLineStyle::kDouble,
                        corner_radius: crate::border_shape_utils::UniformCornerRadius(&outer_radii),
                        inner_corner_radius: crate::border_shape_utils::UniformCornerRadius(
                            &inner_radii,
                        ),
                        corner_radii: outer_radii,
                        inner_corner_radii: inner_radii,
                        ..Default::default()
                    },
                    self.node,
                );
            };
            append_ring(style.outline_width, style.outline_width - third);
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
            color: style.outline_color,
            line_style: style.outline_style,
            stroke_width: style.outline_width,
            corner_radius: crate::border_shape_utils::UniformCornerRadius(&outline_radii),
            corner_radii: outline_radii,
            ..Default::default()
        };
        if style.outline_style == BorderLineStyle::kDashed {
            outline.dash_intervals = vec![3.0 * style.outline_width, 3.0 * style.outline_width];
        } else if style.outline_style == BorderLineStyle::kDotted {
            outline.dash_intervals = vec![0.0, 2.0 * style.outline_width];
            outline.round_cap = true;
        }
        if matches!(
            style.outline_style,
            BorderLineStyle::kDashed | BorderLineStyle::kDotted
        ) && border_radii.HasRadius()
        {
            let outer_outset_x = horizontal_offset + style.outline_width;
            let outer_outset_y = vertical_offset + style.outline_width;
            let half_used_width = (style.outline_width * 0.5) as i32;
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
                crate::border_shape_utils::UniformCornerRadius(&outline.corner_radii);
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
            outline.dash_fit_thickness = style.outline_width;
            outline.stroke_width = style.outline_width * 2.2;
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
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(
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
                    corner_radius: crate::border_shape_utils::UniformCornerRadius(
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
    if let Some(line_top_offset) = fragment.paint.text_line_top_offset {
        let line_top = node.paint_offset.y + line_top_offset;
        paint_offset.y += node.RoundedPaintY(line_top) - line_top;
    }
    if let Some(rect) = fragment.paint.self_ink_overflow {
        return node.EnclosingRect(PaintRect {
            x: paint_offset.x + rect.offset.x,
            y: paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        });
    }
    // Preserve real glyph overhangs even when a text decoration/marker branch
    // still needs the untranslated InlinePaintContext overflow calculation.
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
    for shadow in &fragment.paint.style.text_shadows {
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

fn HasSupportedBoxDecoration(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let style = &*fragment.paint.style;
    if (!fragment.paint.background_transfers_to_view
        && fragment.paint.background_paint_location.PaintsBorderBox()
        && !UsesCompositedFixedBackground(node)
        && (style.background_color.alpha > 0.0 || !style.background_images.is_empty()))
        || style
            .box_shadows
            .iter()
            .any(|shadow| shadow.color.alpha > 0.0)
        || fragment.paint.form_control.is_some()
        || fragment.paint.table.as_ref().is_some_and(|table| {
            table.columns.iter().any(|column| {
                column.style.background_color.alpha > 0.0
                    || !column.style.background_images.is_empty()
            })
        })
        || style.border_image.is_some()
    {
        return true;
    }
    let border = IncludedBorderEdges(&fragment.paint);
    [border.top, border.right, border.bottom, border.left]
        .into_iter()
        .enumerate()
        .any(|(edge, width)| {
            width > 0.0
                && fragment.paint.border_styles[edge] != BorderLineStyle::kNone
                && style.border_colors[edge].alpha > 0.0
        })
}

fn UsesCompositedFixedBackground(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    fragment.paint.can_composite_background_attachment_fixed
        && !fragment.paint.background_transfers_to_view
        && !node.fixed_background_acts_as_scroll
        && node.fixed_background_properties.is_some()
        && fragment.paint.background_paint_location.PaintsBorderBox()
        && fragment.paint.style.background_images.len() == 1
        && fragment.paint.style.background_images[0].attachment == BackgroundAttachment::kFixed
        && (fragment.paint.style.background_images[0].shader.is_some()
            || fragment.paint.style.background_images[0].resource_id != 0)
}

fn DecorationVisualRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    // BoxPainter::VisualRect:124-129; inline boxes use their item ink overflow.
    let native_rect = if fragment.paint.box_fragment_is_inline_box {
        fragment.paint.ink_overflow
    } else {
        fragment
            .paint
            .box_self_visual_overflow
            .or(fragment.paint.self_ink_overflow)
    };
    if let Some(rect) = native_rect {
        return node.EnclosingRect(PaintRect {
            x: node.paint_offset.x + rect.offset.x,
            y: node.paint_offset.y + rect.offset.y,
            width: rect.size.width,
            height: rect.size.height,
        });
    }
    // Full native box/outline overflow recalculation remains untranslated.
    // This fallback covers the finite supported decoration geometry only.
    let style = &*fragment.paint.style;
    let source = FragmentVisualRect(node);
    let outset = (style.outline_offset + style.outline_width).max(0.0);
    let mut left = source.x - outset;
    let mut top = source.y - outset;
    let mut right = source.x + source.width + outset;
    let mut bottom = source.y + source.height + outset;
    for shadow in &style.box_shadows {
        if shadow.inset {
            continue;
        }
        let extent = shadow.spread + (1.5 * shadow.blur_radius).ceil();
        left = left.min(source.x + shadow.offset.x - extent);
        top = top.min(source.y + shadow.offset.y - extent);
        right = right.max(source.x + source.width + shadow.offset.x + extent);
        bottom = bottom.max(source.y + source.height + shadow.offset.y + extent);
    }
    node.EnclosingRect(PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

// cpp: paint/box_fragment_painter.cc:26-29
fn IsBackgroundPhase(phase: PaintPhase) -> bool {
    phase == PaintPhase::kBlockBackground || phase == PaintPhase::kSelfBlockBackgroundOnly
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
        direct_compositing_reasons: Vec::new(),
    }
    .with_filter_presence()
    .with_inherited_view_transition()
}

fn CanvasBackgroundDisplayItemId(
    fragment: &layoutng_assembly::fragment_tree::FragmentNode,
    item_type: DisplayItemIdType,
    layer_ordinal: usize,
) -> DisplayItemId {
    // LayoutView never fragments. Use ScopedDisplayItemFragment's occurrence
    // field to give each independently transformed canvas layer a stable
    // semantic record identity while retaining the native LayoutView client.
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
    Some(node.PixelSnappedRect(&PaintRect {
        x: node.paint_offset.x,
        y: node.paint_offset.y,
        width: if scrolling {
            viewport
                .width
                .max(fragment.size.width)
                .max(fragment.paint.scroll_size.width)
        } else {
            viewport.width.max(fragment.size.width)
        },
        height: if scrolling {
            viewport
                .height
                .max(fragment.size.height)
                .max(fragment.paint.scroll_size.height)
        } else {
            viewport.height.max(fragment.size.height)
        },
    }))
}

// cpp: paint/box_fragment_painter.cc:31-34
fn IsOutlinePhase(phase: PaintPhase) -> bool {
    phase == PaintPhase::kOutline || phase == PaintPhase::kSelfOutlineOnly
}

// cpp: paint/box_fragment_painter.cc:36-39
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/box_fragment_painter.cc:41-46
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(self_cull_rect) = node.self_cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &self_cull_rect))
}

// cpp: paint/box_fragment_painter.cc:48-54
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

// cpp: paint/box_fragment_painter.cc:64-76
fn LineIntersection(a: Offset, b: Offset, c: Offset, d: Offset) -> Offset {
    let ab_x = a.x - b.x;
    let ab_y = a.y - b.y;
    let cd_x = c.x - d.x;
    let cd_y = c.y - d.y;
    let denominator = ab_x * cd_y - ab_y * cd_x;
    if denominator.abs() < 1e-12 {
        return b;
    }
    let determinant_ab = a.x * b.y - a.y * b.x;
    let determinant_cd = c.x * d.y - c.y * d.x;
    Offset {
        x: (determinant_ab * cd_x - ab_x * determinant_cd) / denominator,
        y: (determinant_ab * cd_y - ab_y * determinant_cd) / denominator,
    }
}

// cpp: paint/box_fragment_painter.cc:78-117
fn RoundedBorderSideClips(
    outer: &PaintRect,
    inner: &PaintRect,
    inner_radii: &PaintCornerRadii,
) -> [Vec<PaintPathCommand>; 4] {
    let outer_right = outer.x + outer.width;
    let outer_bottom = outer.y + outer.height;
    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let outer_corners = [
        Offset {
            x: outer.x,
            y: outer.y,
        },
        Offset {
            x: outer_right,
            y: outer.y,
        },
        Offset {
            x: outer_right,
            y: outer_bottom,
        },
        Offset {
            x: outer.x,
            y: outer_bottom,
        },
    ];
    let inner_corners = [
        Offset {
            x: inner.x,
            y: inner.y,
        },
        Offset {
            x: inner_right,
            y: inner.y,
        },
        Offset {
            x: inner_right,
            y: inner_bottom,
        },
        Offset {
            x: inner.x,
            y: inner_bottom,
        },
    ];
    let miter = [
        LineIntersection(
            outer_corners[0],
            inner_corners[0],
            Offset {
                x: inner.x + inner_radii.top_left.x,
                y: inner.y,
            },
            Offset {
                x: inner.x,
                y: inner.y + inner_radii.top_left.y,
            },
        ),
        LineIntersection(
            outer_corners[1],
            inner_corners[1],
            Offset {
                x: inner_right - inner_radii.top_right.x,
                y: inner.y,
            },
            Offset {
                x: inner_right,
                y: inner.y + inner_radii.top_right.y,
            },
        ),
        LineIntersection(
            outer_corners[2],
            inner_corners[2],
            Offset {
                x: inner_right,
                y: inner_bottom - inner_radii.bottom_right.y,
            },
            Offset {
                x: inner_right - inner_radii.bottom_right.x,
                y: inner_bottom,
            },
        ),
        LineIntersection(
            outer_corners[3],
            inner_corners[3],
            Offset {
                x: inner.x + inner_radii.bottom_left.x,
                y: inner_bottom,
            },
            Offset {
                x: inner.x,
                y: inner_bottom - inner_radii.bottom_left.y,
            },
        ),
    ];
    let path = |points: [Offset; 4]| {
        vec![
            PaintPathCommand {
                verb: PaintPathVerb::kMoveTo,
                point: points[0],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[1],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[2],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point: points[3],
                ..Default::default()
            },
            PaintPathCommand {
                verb: PaintPathVerb::kClose,
                ..Default::default()
            },
        ]
    };
    [
        path([outer_corners[0], miter[0], miter[1], outer_corners[1]]),
        path([outer_corners[1], miter[1], miter[2], outer_corners[2]]),
        path([outer_corners[2], miter[2], miter[3], outer_corners[3]]),
        path([outer_corners[3], miter[3], miter[0], outer_corners[0]]),
    ]
}

// cpp: paint/box_fragment_painter.cc:119-134
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

// cpp: paint/box_fragment_painter.cc:134
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

// cpp: paint/box_fragment_painter.cc:136-250
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

// cpp: paint/box_fragment_painter.cc:252-256
fn DecorationRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
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
        BackgroundBox::kContentBox => BackgroundBox::kContentBox,
        BackgroundBox::kBorderBox
        | BackgroundBox::kText
        | BackgroundBox::kBorderArea
        | BackgroundBox::kBorderAreaText => BackgroundBox::kBorderBox,
    }
}

// cpp: paint/box_fragment_painter.cc:258-277
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
        BackgroundAttachment::kFixed if node.fixed_background_acts_as_scroll => {
            BackgroundAttachment::kScroll
        }
        // A local attachment only establishes scrolling geometry on an actual
        // native scroll owner. Without one, Chromium paints it in border-box
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

fn LocalBackgroundBorderRect(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let mut rect = ContentsBackgroundBorderRect(node);
    // Border-box painting has no ScrollTranslation property. Offset the
    // enlarged scrolling background manually for the native fallback path.
    rect.x -= fragment.paint.scroll_offset.x;
    rect.y -= fragment.paint.scroll_offset.y;
    rect
}

// cpp: paint/box_fragment_painter.cc:279-304
fn BackgroundPositioningBoxRect(node: &PaintTreeNode<'_>, box_type: BackgroundBox) -> PaintRect {
    let paint = &node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .paint;
    let box_type = UsedBackgroundGeometryBox(box_type);
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

// cpp: paint/box_fragment_painter.cc:306-318
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

// cpp: paint/box_fragment_painter.cc:320-327
fn BackgroundRadii(node: &PaintTreeNode<'_>, box_type: BackgroundBox) -> PaintCornerRadii {
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

// cpp: paint/box_fragment_painter.cc:329-353
fn BackgroundBleedInset(node: &PaintTreeNode<'_>) -> Option<Edges> {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let style = &*fragment.paint.style;
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
            || style.border_colors[edge].alpha < 1.0
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

// cpp: paint/box_fragment_painter.cc:355-378
fn BackgroundLayerOccludesFollowingLayers(layer: &BackgroundImageLayer) -> bool {
    // A text/border-area mask never proves coverage of the whole background
    // painting area, even when its generated image is intrinsically opaque.
    if IsSpecialBackgroundClip(layer.clip) {
        return false;
    }
    let Some(shader) = layer.shader.as_deref() else {
        return false;
    };
    if layer.blend_mode != PaintBlendMode::kNormal || shader.stops.is_empty() {
        return false;
    }
    let repeat_x = layer.repeat_rule_x.unwrap_or(
        if matches!(
            layer.repeat,
            BackgroundRepeat::kRepeat | BackgroundRepeat::kRepeatX
        ) {
            BackgroundRepeatRule::kRepeat
        } else {
            BackgroundRepeatRule::kNoRepeat
        },
    );
    let repeat_y = layer.repeat_rule_y.unwrap_or(
        if matches!(
            layer.repeat,
            BackgroundRepeat::kRepeat | BackgroundRepeat::kRepeatY
        ) {
            BackgroundRepeatRule::kRepeat
        } else {
            BackgroundRepeatRule::kNoRepeat
        },
    );
    matches!(
        repeat_x,
        BackgroundRepeatRule::kRepeat | BackgroundRepeatRule::kRound
    ) && matches!(
        repeat_y,
        BackgroundRepeatRule::kRepeat | BackgroundRepeatRule::kRound
    ) && shader.stops.iter().all(|stop| stop.color.alpha >= 1.0)
}

// cpp: paint/box_fragment_painter.cc:380-397
fn BorderEdgePolygons(outer: &PaintRect, inner: &PaintRect) -> [[Offset; 4]; 4] {
    let right = outer.x + outer.width;
    let bottom = outer.y + outer.height;
    let inner_right = inner.x + inner.width;
    let inner_bottom = inner.y + inner.height;
    let point = |x, y| Offset { x, y };
    [
        [
            point(outer.x, outer.y),
            point(right, outer.y),
            point(inner_right, inner.y),
            point(inner.x, inner.y),
        ],
        [
            point(right, outer.y),
            point(right, bottom),
            point(inner_right, inner_bottom),
            point(inner_right, inner.y),
        ],
        [
            point(right, bottom),
            point(outer.x, bottom),
            point(inner.x, inner_bottom),
            point(inner_right, inner_bottom),
        ],
        [
            point(outer.x, bottom),
            point(outer.x, outer.y),
            point(inner.x, inner.y),
            point(inner.x, inner_bottom),
        ],
    ]
}

// cpp: paint/box_fragment_painter.cc:399-412
fn BorderEdgeClip(
    node_id: u64,
    phase: PaintPhase,
    polygon: &[Offset; 4],
    antialias: bool,
) -> DisplayItem {
    let mut clip = DisplayItem {
        r#type: DisplayItemType::kClipPath,
        phase,
        node_id,
        antialias,
        ..Default::default()
    };
    clip.path = [
        PaintPathVerb::kMoveTo,
        PaintPathVerb::kLineTo,
        PaintPathVerb::kLineTo,
        PaintPathVerb::kLineTo,
    ]
    .into_iter()
    .zip(*polygon)
    .map(|(verb, point)| PaintPathCommand {
        verb,
        point,
        ..Default::default()
    })
    .collect();
    clip
}

// cpp: paint/box_fragment_painter.cc:414-487
fn BorderEdgeCornerClips(
    edge: usize,
    outer: &PaintRect,
    inner: &PaintRect,
    radii: &PaintCornerRadii,
) -> [[Offset; 4]; 2] {
    let l = outer.x;
    let t = outer.y;
    let r = outer.x + outer.width;
    let b = outer.y + outer.height;
    let il = inner.x;
    let it = inner.y;
    let ir = inner.x + inner.width;
    let ib = inner.y + inner.height;
    let top = inner.y - outer.y;
    let right = r - ir;
    let bottom = b - ib;
    let left = inner.x - outer.x;
    const E: f64 = 0.1;
    let point = |x, y| Offset { x, y };
    match edge {
        0 => {
            let ly = t + (radii.top_left.y + top) * 0.5;
            let ry = t + (radii.top_right.y + top) * 0.5;
            let lx = if top > 0.0 {
                l + left * (ly - t) / top
            } else {
                il
            };
            let rx = if top > 0.0 {
                r - right * (ry - t) / top
            } else {
                ir
            };
            [
                [
                    point(l - E, t),
                    point(lx - E, ly),
                    point(r, ly),
                    point(r, t),
                ],
                [
                    point(l, t),
                    point(l, ry),
                    point(rx + E, ry),
                    point(r + E, t),
                ],
            ]
        }
        1 => {
            let tx = r - (radii.top_right.x + right) * 0.5;
            let bx = r - (radii.bottom_right.x + right) * 0.5;
            let ty = (if right > 0.0 {
                t + top * (r - tx) / right
            } else {
                it
            }) - E;
            let by = (if right > 0.0 {
                b - bottom * (r - bx) / right
            } else {
                ib
            }) + E;
            [
                [point(r, t - E), point(tx, ty), point(tx, b), point(r, b)],
                [point(r, t), point(bx, t), point(bx, by), point(r, b + E)],
            ]
        }
        2 => {
            let ry = b - (radii.bottom_right.y + bottom) * 0.5;
            let ly = b - (radii.bottom_left.y + bottom) * 0.5;
            let rx = if bottom > 0.0 {
                r - right * (b - ry) / bottom
            } else {
                ir
            };
            let lx = if bottom > 0.0 {
                l + left * (b - ly) / bottom
            } else {
                il
            };
            [
                [
                    point(r + E, b),
                    point(rx + E, ry),
                    point(l, ry),
                    point(l, b),
                ],
                [point(r, b), point(r, ly), point(lx, ly), point(l - E, b)],
            ]
        }
        _ => {
            let bx = l
                + (radii.bottom_left.x * left * left / (left * left + bottom * bottom).max(1e-9))
                    .max((radii.bottom_left.x + left) * 0.5);
            let tx = l
                + (radii.top_left.x * left * left / (left * left + top * top).max(1e-9))
                    .max((radii.top_left.x + left) * 0.5);
            let by = (if left > 0.0 {
                b - bottom * (bx - l) / left
            } else {
                ib
            }) + E;
            let ty = (if left > 0.0 {
                t + top * (tx - l) / left
            } else {
                it
            }) - E;
            [
                [point(l, b + E), point(bx, by), point(bx, t), point(l, t)],
                [point(l, b), point(tx, b), point(tx, ty), point(l, t - E)],
            ]
        }
    }
}

// cpp: paint/box_fragment_painter.cc:489-520
fn CombinedBorderEdgeClip(
    edge: usize,
    outer: &PaintRect,
    inner: &PaintRect,
    clips: &[[Offset; 4]; 2],
    first_adjacent_renders: bool,
    second_adjacent_renders: bool,
) -> [Offset; 4] {
    let l = outer.x;
    let t = outer.y;
    let r = outer.x + outer.width;
    let b = outer.y + outer.height;
    let il = inner.x;
    let it = inner.y;
    let ir = inner.x + inner.width;
    let ib = inner.y + inner.height;
    const E: f64 = 0.1;
    let point = |x, y| Offset { x, y };
    match edge {
        0 => [
            point(l, t),
            if first_adjacent_renders {
                point(clips[0][1].x + E, clips[0][1].y)
            } else {
                point(l, it)
            },
            if second_adjacent_renders {
                point(clips[1][2].x - E, clips[1][2].y)
            } else {
                point(r, it)
            },
            point(r, t),
        ],
        1 => [
            point(r, t),
            if first_adjacent_renders {
                point(clips[0][1].x, clips[0][1].y + E)
            } else {
                point(ir, t)
            },
            if second_adjacent_renders {
                point(clips[1][2].x, clips[1][2].y - E)
            } else {
                point(ir, b)
            },
            point(r, b),
        ],
        2 => [
            point(r, b),
            if first_adjacent_renders {
                point(clips[0][1].x - E, clips[0][1].y)
            } else {
                point(r, ib)
            },
            if second_adjacent_renders {
                point(clips[1][2].x + E, clips[1][2].y)
            } else {
                point(l, ib)
            },
            point(l, b),
        ],
        _ => [
            point(l, b),
            if first_adjacent_renders {
                point(clips[0][1].x, clips[0][1].y - E)
            } else {
                point(il, b)
            },
            if second_adjacent_renders {
                point(clips[1][2].x, clips[1][2].y + E)
            } else {
                point(il, t)
            },
            point(l, t),
        ],
    }
}

// cpp: paint/box_fragment_painter.cc:522-541
fn FindImage<'a>(node: &'a PaintTreeNode<'_>, id: u64) -> &'a PaintImage {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let Some(resources) = fragment.paint.resources.as_ref() else {
        panic!("image paint has no PaintResources");
    };
    let images = &resources.images;
    let Some(found) = images.iter().find(|image| image.id == id) else {
        panic!("paint image resource id is missing");
    };
    if found.width == 0
        || found.height == 0
        || !found.resolution_scale.is_finite()
        || found.resolution_scale <= 0.0
        || found
            .BitmapPixels()
            .is_some_and(|pixels| pixels.len() != found.width as usize * found.height as usize * 4)
    {
        panic!("paint image must contain dimensions, scale and width*height RGBA bytes");
    }
    found
}

// cpp: paint/box_fragment_painter.cc:543-557
fn TextWritingModeTransform(
    node: &PaintTreeNode<'_>,
    writing_mode: WritingMode,
    physical_box_offset: Offset,
) -> TransformMatrix {
    let mut transform = TransformMatrix::default();
    if writing_mode == WritingMode::kHorizontalTb {
        return transform;
    }
    let x = physical_box_offset.x;
    let y = physical_box_offset.y;
    let block_size = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment")
        .size
        .width;
    transform.values = [
        0.0,
        1.0,
        0.0,
        0.0,
        -1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        x + y + block_size,
        y - x,
        0.0,
        1.0,
    ];
    transform
}

// cpp: paint/box_fragment_painter.cc:559-576
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

// cpp: paint/box_fragment_painter.cc:578-585
fn TextPaintOrigin(node: &PaintTreeNode<'_>) -> Offset {
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

// cpp: paint/box_fragment_painter.cc:587-591
fn TextPaintSize(node: &PaintTreeNode<'_>) -> Size {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    if let Some(svg) = fragment.paint.svg_text.as_ref() {
        return svg.untransformed_size;
    }
    fragment.size
}

#[cfg(test)]
mod background_attachment_tests {
    use std::sync::Arc;

    use layoutng_assembly::fragment_tree::{
        FragmentNode, FragmentPaintRect, PaintResources, ScrollContainerPaintData,
    };
    use layoutng_assembly::internal::layout_input::{Offset, Size, ViewportGeometry};
    use layoutng_assembly::internal::layout_input_types::IntSize;
    use layoutng_assembly::internal::paint_input::BackgroundBox;

    use crate::geometry_mapper::TranslationTransform;

    use super::{
        BackgroundBoxRectWithin, FixedBackgroundPositioningArea, LocalBackgroundBorderRect,
        PaintTreeNode,
    };

    #[test]
    fn fixed_and_local_backgrounds_use_viewport_zoom_and_scroll_geometry() {
        let mut fixed_fragment = FragmentNode::default();
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
        fixed.SetFragment(&fixed_fragment);
        fixed.scroll_offset = Offset { x: 7.0, y: 11.0 };
        fixed.transforms.push(TranslationTransform(5.0, 9.0));
        let viewport = FixedBackgroundPositioningArea(&fixed).unwrap();
        assert_eq!((viewport.x, viewport.y), (2.0, 2.0));
        assert_eq!((viewport.width, viewport.height), (480.0, 300.0));

        let mut local_fragment = FragmentNode::default();
        local_fragment.size = Size {
            width: 100.0,
            height: 80.0,
        };
        local_fragment.paint.scroll_offset = Offset { x: 13.0, y: 17.0 };
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
            background_needs_repaint_on_scroll: None,
            prefers_non_composited_scrolling: None,
            scroll_snap_type_is_none: None,
            container_rect: FragmentPaintRect::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
        });
        let mut local = PaintTreeNode::default();
        local.SetFragment(&local_fragment);
        local.paint_offset = Offset { x: 50.0, y: 40.0 };
        let contents = LocalBackgroundBorderRect(&local);
        assert_eq!((contents.x, contents.y), (37.0, 23.0));
        assert_eq!((contents.width, contents.height), (245.0, 189.0));
        let content_clip = BackgroundBoxRectWithin(&local, BackgroundBox::kContentBox, contents);
        assert_eq!((content_clip.x, content_clip.y), (45.0, 35.0));
        assert_eq!((content_clip.width, content_clip.height), (227.0, 163.0));
    }
}
