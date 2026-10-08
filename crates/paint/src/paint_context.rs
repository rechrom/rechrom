#![allow(non_snake_case)]

use layoutng_assembly::internal::paint_input::PaintBlendMode;

use crate::border_shape_utils::UniformCornerRadius;
use crate::display_item_id::{DisplayItemId, DisplayItemIdType};
use crate::paint_chunker::{DisplayItemClientInfo, PaintChunker};
use crate::paint_engine::{
    DisplayItem, DisplayItemType, PaintArtifact, PaintPhase, RasterEffectOutset,
    RecordedDisplayItem, RecordedDisplayItemKind,
};
use crate::paint_property_tree::PropertyTreeState;
use crate::pre_paint_tree_walk::{EnclosingRectInPropertySpace, PaintTreeNode};
use crate::PaintRect;
use layoutng_assembly::internal::layout_input::Offset;

// drawing_display_item.h:70-98 and drawing_display_item.cc:64-81,285-320.
// Only one fill rect/rrect op without a shader or pixel-moving paint flags
// qualifies. Images, text, strokes and multi-op records retain painter ink bounds.
fn TightenDrawingVisualRect(
    visual_rect: PaintRect,
    record: &[DisplayItem],
    delta: Offset,
) -> (PaintRect, bool) {
    if let [op] = record {
        if matches!(
            op.r#type,
            DisplayItemType::kDrawRect | DisplayItemType::kDrawRoundedRect
        ) && op.paint_shader.is_none()
            && op.filters.is_empty()
            && op.blend_mode == PaintBlendMode::kNormal
        {
            return (EnclosingRectInPropertySpace(op.rect, delta), true);
        }
        // SkCanvas image draws cannot write outside their destination/tile
        // rectangle unless an image filter, shadow or blur expands the op.
        // Treating ordinary DrawImage records as unknown made LayerTile fall
        // back to the whole viewport for each animated image layer.
        if matches!(
            op.r#type,
            DisplayItemType::kDrawImageRect | DisplayItemType::kDrawTiledImage
        ) && op.filters.is_empty()
            && !op.is_shadow
            && op.blur_radius == 0.0
        {
            return (EnclosingRectInPropertySpace(op.rect, delta), true);
        }
    }
    (EnclosingRectInPropertySpace(visual_rect, delta), false)
}

// drawing_display_item.cc:172-282. Keep this bounded and independent of image
// pixel contents. The standalone ABI has no certified shader/image opacity,
// so only solid fill rectangles supply a new proof. Unknown state is rejected.
pub fn CalculateRectKnownToBeOpaqueForRecord(
    visual_rect: PaintRect,
    record: &[DisplayItem],
) -> PaintRect {
    const OP_COUNT_LIMIT: usize = 8;
    // Official Blink stops at its prefix. This flat adapter also permits custom
    // state ops, so an uninspected tail cannot certify that later DstIn/filters
    // preserve the prefix's alpha. Reject long records without traversing them.
    if record.is_empty() || record.len() > OP_COUNT_LIMIT {
        return PaintRect::default();
    }
    let mut opaque = PaintRect::default();
    let mut clip = visual_rect.enclosed();
    for op in record {
        match op.r#type {
            DisplayItemType::kSave => continue,
            DisplayItemType::kClipRect => {
                clip = clip.intersection(op.rect.enclosed());
                continue;
            }
            // Keep the narrower clip as a conservative enclosure after restore.
            // Continue checking the bounded tail so DstIn cannot destroy a proof.
            DisplayItemType::kRestore => continue,
            // Source blending/filter states could remove a previous alpha
            // proof. Other unrecognized state ops conservatively reject too.
            kind if (kind as u8) < DisplayItemType::kDrawRect as u8 => return PaintRect::default(),
            _ => {}
        }
        if op.blend_mode != PaintBlendMode::kNormal
            || !op.filters.is_empty()
            || !op.mask_layers.is_empty()
            || op.r#type == DisplayItemType::kDrawMask
        {
            return PaintRect::default();
        }
        if op.paint_shader.is_some()
            || op.color.alpha != 1.0
            || op.opacity != 1.0
            || op.is_shadow
            || op.blur_radius != 0.0
        {
            continue;
        }
        let rect = match op.r#type {
            DisplayItemType::kDrawRect => op.rect.enclosed(),
            DisplayItemType::kDrawRoundedRect => {
                let r = op.corner_radii;
                if [
                    r.top_left.x,
                    r.top_left.y,
                    r.top_right.x,
                    r.top_right.y,
                    r.bottom_left.x,
                    r.bottom_left.y,
                    r.bottom_right.x,
                    r.bottom_right.y,
                ]
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0)
                {
                    continue;
                }
                // The contained rectangle excludes all four rounded corners.
                // Chromium uses .3, conservatively above (2 - sqrt(2)) / 2.
                let left = r.top_left.x.max(r.bottom_left.x) * 0.3;
                let top = r.top_left.y.max(r.top_right.y) * 0.3;
                let right = r.top_right.x.max(r.bottom_right.x) * 0.3;
                let bottom = r.bottom_left.y.max(r.bottom_right.y) * 0.3;
                PaintRect {
                    x: op.rect.x + left,
                    y: op.rect.y + top,
                    width: op.rect.width - left - right,
                    height: op.rect.height - top - bottom,
                }
                .enclosed()
            }
            _ => continue,
        };
        opaque = PaintRect::maximum_covered_rect(opaque, rect).intersection(clip);
    }
    opaque
}

// cpp: paint/paint_context.h:8-26
pub struct PaintContext<'o> {
    output: &'o mut PaintArtifact,
    pub(crate) caret: Option<crate::paint_engine::CaretPosition>,
    pub(crate) caret_local_rect: Option<layoutng_assembly::caret::geometry::LocalCaretRect>,
    chunker: PaintChunker,
    drawing_recorder: Option<(
        DisplayItemId,
        DisplayItemClientInfo,
        PropertyTreeState,
        PaintRect,
        usize,
        RecordedDisplayItemKind,
        Option<std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>>,
        RasterEffectOutset,
        bool,
        Offset,
    )>,
    next_property_id: u64,
}

impl<'o> PaintContext<'o> {
    // cpp: paint/paint_context.h:10
    pub fn new(output: &'o mut PaintArtifact) -> Self {
        Self {
            output,
            caret: None,
            caret_local_rect: None,
            chunker: PaintChunker::default(),
            drawing_recorder: None,
            next_property_id: 1,
        }
    }

    pub(crate) fn WithCaret(
        output: &'o mut PaintArtifact,
        caret: Option<crate::paint_engine::CaretPosition>,
    ) -> Self {
        Self {
            output,
            caret,
            caret_local_rect: None,
            chunker: PaintChunker::default(),
            drawing_recorder: None,
            next_property_id: 1,
        }
    }
    pub(crate) fn SetCaretGeometry(&mut self, geometry: crate::paint_engine::CaretGeometry) {
        self.output.caret = Some(geometry);
    }

    // cpp: paint/paint_context.h:22
    // cpp: paint/paint_context.cc:60-80
    pub fn Append(&mut self, mut item: DisplayItem, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        item.fragment_instance_id = fragment.fragment_instance_id;
        // Blink records save/restore/clip inside PaintRecords, not as drawing
        // DisplayItems. They must neither consume a pending scope ID nor create
        // a fallback identity. The existing flat replay ABI retains the ops.
        if self.drawing_recorder.is_some()
            || (item.r#type as u8) < (DisplayItemType::kDrawRect as u8)
        {
            std::sync::Arc::make_mut(&mut self.output.items).push(item);
            return;
        }
        // Blink's fallback ID is a semantic DisplayItem ID, not a Skia opcode.
        // The standalone stream remains flat; scopes supply the outer ID.
        let id = DisplayItemId {
            client_id: fragment.paint.display_item_client_id,
            r#type: DisplayItemIdType::PaintPhaseToDrawingType(item.phase),
            fragment: fragment.paint.display_item_fragment,
        };
        let client = DisplayItemClientInfo::from_fragment(fragment);
        let (visual_rect, visual_rect_is_accurate) = TightenDrawingVisualRect(
            item.rect,
            std::slice::from_ref(&item),
            node.paint_snap_offset,
        );
        let raster_effect_outset = fragment.paint.display_item_raster_effect_outset;
        let rect_known_to_be_opaque =
            CalculateRectKnownToBeOpaqueForRecord(visual_rect, std::slice::from_ref(&item));
        self.chunker.increment_display_item(
            &mut self.output.chunks,
            id,
            client,
            visual_rect,
            true,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
        );
        self.output.display_items.push(RecordedDisplayItem {
            kind: RecordedDisplayItemKind::Drawing,
            scroll_translation: None,
            id,
            visual_rect,
            visual_rect_is_accurate,
            draws_content: true,
            raster_effect_outset,
            record_begin: self.output.items.len(),
            record_end: self.output.items.len() + 1,
        });
        std::sync::Arc::make_mut(&mut self.output.items).push(item);
    }

    pub(crate) fn InDrawingRecorder(&self) -> bool {
        self.drawing_recorder.is_some()
    }

    pub(crate) fn BeginDrawingRecorder(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
        visual_rect: PaintRect,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        self.BeginDrawingRecorderWithClient(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: item_type,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            visual_rect,
            node.paint_snap_offset,
        );
        if let Some(recorder) = &mut self.drawing_recorder {
            recorder.7 = fragment.paint.display_item_raster_effect_outset;
            recorder.8 = fragment.paint.self_ink_overflow.is_some()
                || fragment.paint.box_self_visual_overflow.is_some();
        }
    }

    pub(crate) fn BeginDrawingRecorderWithClient(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        delta: Offset,
    ) {
        assert!(!self.InDrawingRecorder(), "DrawingRecorders cannot nest");
        self.drawing_recorder = Some((
            id,
            client,
            self.chunk_properties(),
            visual_rect,
            self.output.items.len(),
            RecordedDisplayItemKind::Drawing,
            None,
            RasterEffectOutset::kNone,
            true,
            delta,
        ));
    }

    pub(crate) fn BeginScrollbarDisplayItem(
        &mut self,
        id: DisplayItemId,
        client: DisplayItemClientInfo,
        visual_rect: PaintRect,
        scroll_translation: std::sync::Arc<crate::paint_property_tree::TransformPaintPropertyNode>,
        delta: Offset,
    ) {
        assert!(
            !self.InDrawingRecorder(),
            "ScrollbarDisplayItem is an outer item"
        );
        self.drawing_recorder = Some((
            id,
            client,
            self.chunk_properties(),
            visual_rect,
            self.output.items.len(),
            RecordedDisplayItemKind::Scrollbar,
            Some(scroll_translation),
            RasterEffectOutset::kNone,
            true,
            delta,
        ));
    }

    pub(crate) fn EndDrawingRecorder(&mut self) {
        let (
            id,
            client,
            properties,
            visual_rect,
            record_begin,
            kind,
            scroll_translation,
            raster_effect_outset,
            visual_rect_is_accurate,
            delta,
        ) = self
            .drawing_recorder
            .take()
            .expect("DrawingRecorder must be active");
        let record_end = self.output.items.len();
        let (visual_rect, tightened) = if kind == RecordedDisplayItemKind::Drawing {
            TightenDrawingVisualRect(
                visual_rect,
                &self.output.items[record_begin..record_end],
                delta,
            )
        } else {
            (EnclosingRectInPropertySpace(visual_rect, delta), false)
        };
        let visual_rect_is_accurate = visual_rect_is_accurate || tightened;
        // DrawingDisplayItem uses !record.empty(), including state ops.
        // ScrollbarDisplayItem is independently drawable, without a PaintRecord.
        let draws_content =
            kind == RecordedDisplayItemKind::Scrollbar || record_begin != record_end;
        self.chunker.update_properties(properties);
        let rect_known_to_be_opaque = if kind == RecordedDisplayItemKind::Drawing {
            CalculateRectKnownToBeOpaqueForRecord(
                visual_rect,
                &self.output.items[record_begin..record_end],
            )
        } else {
            PaintRect::default()
        };
        self.chunker.increment_display_item(
            &mut self.output.chunks,
            id,
            client,
            visual_rect,
            draws_content,
            raster_effect_outset,
            visual_rect_is_accurate,
            rect_known_to_be_opaque,
        );
        self.output.display_items.push(RecordedDisplayItem {
            kind,
            scroll_translation,
            id,
            visual_rect,
            visual_rect_is_accurate,
            draws_content,
            raster_effect_outset,
            record_begin,
            record_end,
        });
    }

    pub(crate) fn SetNextPropertyId(&mut self, next: u64) {
        self.next_property_id = next;
    }

    pub(crate) fn chunk_properties(&self) -> PropertyTreeState {
        self.chunker.current_properties().clone()
    }

    pub(crate) fn update_chunk_properties(&mut self, properties: PropertyTreeState) {
        self.chunker.update_properties(properties);
    }

    pub(crate) fn update_chunk_id_and_properties(
        &mut self,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        self.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: item_type,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            node.properties.nodes.clone(),
        );
    }

    // cpp: paint/paint_context.h:12
    // cpp: paint/paint_context.cc:82-86
    pub fn BeginNode(&mut self, node: &PaintTreeNode<'_>) {
        self.BeginGeometry(node);
        self.BeginEffects(node);
        self.BeginEscapableGeometry(node, None);
    }

    // cpp: paint/paint_context.h:16
    // cpp: paint/paint_context.cc:88-130
    pub fn BeginGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_transform {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            for transform in &node.local_transforms {
                let mut item = DisplayItem {
                    r#type: DisplayItemType::kConcat,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                };
                item.transform = *transform;
                self.Append(item, node);
            }
        }
        if node.applies_root_clip {
            if let Some(root_clip) = node.root_clip {
                self.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kSave,
                        phase: PaintPhase::kForeground,
                        node_id: fragment.node_id,
                        ..Default::default()
                    },
                    node,
                );
                self.Append(
                    DisplayItem {
                        r#type: DisplayItemType::kClipRect,
                        phase: PaintPhase::kForeground,
                        node_id: fragment.node_id,
                        rect: root_clip,
                        // paint_chunks_to_cc_layer.cc:589-595: property clips
                        // use AA for both plain and rounded rectangles.
                        antialias: true,
                        ..Default::default()
                    },
                    node,
                );
            }
        }
        if node.applies_clip_path {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            let mut item = DisplayItem {
                r#type: DisplayItemType::kClipPath,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                antialias: true,
                ..Default::default()
            };
            item.path = node.local_clip_path.clone();
            item.even_odd = node.clip_path_even_odd;
            self.Append(item, node);
        }
    }

    // cpp: paint/paint_context.h:18-20
    // cpp: paint/paint_context.cc:132-169
    pub fn BeginEscapableGeometry(
        &mut self,
        node: &PaintTreeNode<'_>,
        clip_override: Option<PaintRect>,
    ) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_overflow_clip
            || node.local_clip.is_none()
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
        let local_clip = node.local_clip.expect("overflow clip checked above");
        let clip = clip_override.unwrap_or(local_clip);
        let mut radii = node.local_clip_radii;
        if clip_override.is_some() && local_clip.width > 0.0 && local_clip.height > 0.0 {
            let scale_x = clip.width / local_clip.width;
            let scale_y = clip.height / local_clip.height;
            radii.top_left.x *= scale_x.abs();
            radii.top_left.y *= scale_y.abs();
            radii.top_right.x *= scale_x.abs();
            radii.top_right.y *= scale_y.abs();
            radii.bottom_right.x *= scale_x.abs();
            radii.bottom_right.y *= scale_y.abs();
            radii.bottom_left.x *= scale_x.abs();
            radii.bottom_left.y *= scale_y.abs();
        }
        let has_radius = radii.HasRadius();
        self.Append(
            DisplayItem {
                r#type: if has_radius {
                    DisplayItemType::kClipRoundedRect
                } else {
                    DisplayItemType::kClipRect
                },
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                rect: clip,
                corner_radius: UniformCornerRadius(&radii),
                corner_radii: radii,
                antialias: true,
                ..Default::default()
            },
            node,
        );
    }

    // cpp: paint/paint_context.h:14
    // cpp: paint/paint_context.cc:171-223
    pub fn BeginEffects(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        let style = &*fragment.paint.style;
        if node.applies_compositing_layer {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerBlend,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    blend_mode: style.blend_mode,
                    opacity: if node.applies_opacity {
                        style.opacity
                    } else {
                        1.0
                    },
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_opacity && !node.applies_blend {
            // SkCanvas.h saveLayer bounds is optional. Chromium fills it from
            // all chunk drawable bounds after effect conversion (not this
            // owner's box). Until that union is available, omit the hint: a
            // scrolling child/OOF can extend beyond the owner, and SkCanvas's
            // current bounded device would otherwise discard real pixels.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerAlpha,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    opacity: style.opacity,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_mask {
            // Isolate the content Effect. Its mask is a later, independently
            // recorded source drawing under the child DstIn Mask effect.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayer,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            // Match the content-only MaskClip before background recording,
            // independently of the later overflow/scroll geometry scopes.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kClipRect,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    rect: node.ResolvedMaskClipRect().expect("resolved MaskClip"),
                    antialias: true,
                    ..Default::default()
                },
                node,
            );
        }
        self.BeginCssClip(node);
        if node.applies_filter {
            let mut item = DisplayItem {
                r#type: DisplayItemType::kSaveLayerFilter,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            };
            item.filters = style.filters.clone();
            self.Append(item, node);
        }
    }

    // cpp: paint/paint_context.h:13
    // cpp: paint/paint_context.cc:225-229
    pub fn EndNode(&mut self, node: &PaintTreeNode<'_>) {
        self.EndEscapableGeometry(node);
        self.EndEffects(node);
        self.EndGeometry(node);
    }

    // cpp: paint/paint_context.h:15
    // cpp: paint/paint_context.cc:231-265
    pub(crate) fn EndFilter(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_filter {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    pub(crate) fn EndMaskClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if fragment.paint.has_source && fragment.paint.establishes_paint_state && node.applies_mask
        {
            // The mask source uses its original ancestor clip, not MaskClip.
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    pub fn EndEffects(&mut self, node: &PaintTreeNode<'_>) {
        self.EndFilter(node);
        self.EndCssClip(node);
        self.EndMaskClip(node);
        self.EndEffectsAfterFilter(node);
    }

    fn BeginCssClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_css_clip
        {
            return;
        }
        let rect = node.local_css_clip.expect("CSS clip checked above");
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                rect,
                antialias: true,
                ..Default::default()
            },
            node,
        );
    }

    pub(crate) fn EndCssClip(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_css_clip
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
    }

    pub(crate) fn EndEffectsAfterFilter(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_mask {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_opacity && !node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_blend {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_compositing_layer {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    // cpp: paint/paint_context.h:17
    // cpp: paint/paint_context.cc:267-289
    pub fn EndGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source || !fragment.paint.establishes_paint_state {
            return;
        }
        if node.applies_clip_path {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_root_clip {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
        if node.applies_transform {
            self.Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: PaintPhase::kForeground,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
        }
    }

    // cpp: paint/paint_context.h:21
    // cpp: paint/paint_context.cc:291-301
    pub fn EndEscapableGeometry(&mut self, node: &PaintTreeNode<'_>) {
        let fragment = node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment");
        if !fragment.paint.has_source
            || !fragment.paint.establishes_paint_state
            || !node.applies_overflow_clip
            || node.local_clip.is_none()
        {
            return;
        }
        self.Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: PaintPhase::kForeground,
                node_id: fragment.node_id,
                ..Default::default()
            },
            node,
        );
    }
}

/// Translated scope behavior from Blink scoped_paint_chunk_properties.h.
/// Restoring properties deliberately does not restore the previous chunk ID.
pub(crate) struct ScopedPaintChunkProperties<'c, 'o> {
    context: &'c std::cell::RefCell<PaintContext<'o>>,
    previous: PropertyTreeState,
}

impl<'c, 'o> ScopedPaintChunkProperties<'c, 'o> {
    pub(crate) fn caret(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        caret: crate::paint_engine::CaretPosition,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        use crate::paint_property_tree::EffectPaintPropertyNode;
        let fragment = node.fragment.as_deref().expect("caret fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let mut properties = previous.clone();
        // BoxFragmentPainter::PaintCaretsIfNeeded applies OverflowClip while
        // retaining the border-box transform (not ScrollTranslation).
        properties.clip = node.contents_properties.nodes.clip.clone();
        let id = borrowed.next_property_id;
        borrowed.next_property_id += 1;
        // FrameCaret::PaintCaret + CaretEffectNodeState, frame_caret.cc:86,282.
        properties.effect = std::sync::Arc::new(EffectPaintPropertyNode {
            lifecycle: Default::default(),
            id,
            parent: Some(previous.effect.clone()),
            local_transform_space: previous.transform.clone(),
            output_clip: None,
            opacity: if caret.visible { 1.0 } else { 0.001 },
            blend_mode: layoutng_assembly::internal::paint_input::PaintBlendMode::kNormal,
            filters: Vec::new(),
            isolates_blending: false,
            has_mask: false,
            is_mask: false,
            direct_compositing_reasons: vec!["ActiveOpacityAnimation"],
        });
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: caret.display_item_client_id,
                r#type: DisplayItemIdType::kCaret,
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo {
                is_cacheable: caret.display_item_client_is_cacheable,
                is_just_created: caret.display_item_client_is_just_created,
            },
            properties,
        );
        Self { context, previous }
    }

    pub(crate) fn layer(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("paint layer fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        if fragment.paint.paint_layer_client_id != 0 {
            borrowed.chunker.update_id_and_properties(
                DisplayItemId {
                    client_id: fragment.paint.paint_layer_client_id,
                    r#type: item_type,
                    fragment: fragment.paint.display_item_fragment,
                },
                DisplayItemClientInfo {
                    is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
                    is_just_created: fragment.paint.paint_layer_client_is_just_created,
                },
                node.properties.nodes.clone(),
            );
        } else {
            borrowed.update_chunk_properties(node.properties.nodes.clone());
        }
        Self { context, previous }
    }

    pub(crate) fn mask(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("mask fragment");
        let properties = node
            .mask_properties
            .as_ref()
            .expect("Mask paint properties")
            .clone();
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        let layer_client = fragment.paint.paint_layer_client_id;
        let (client_id, client) = if layer_client != 0 {
            (
                layer_client,
                DisplayItemClientInfo {
                    is_cacheable: fragment.paint.paint_layer_client_is_cacheable,
                    is_just_created: fragment.paint.paint_layer_client_is_just_created,
                },
            )
        } else {
            // Standalone owners without a native PaintLayer use their real
            // drawing client. The recorder supplies the same legal fallback.
            (
                fragment.paint.display_item_client_id,
                DisplayItemClientInfo::from_fragment(fragment),
            )
        };
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id,
                r#type: DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kMask),
                fragment: fragment.paint.display_item_fragment,
            },
            client,
            properties,
        );
        Self { context, previous }
    }

    pub(crate) fn contents(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        phase: PaintPhase,
    ) -> Self {
        let fragment = node.fragment.as_deref().expect("contents fragment");
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.chunker.update_id_and_properties(
            DisplayItemId {
                client_id: fragment.paint.display_item_client_id,
                r#type: DisplayItemIdType::PaintPhaseToClipType(phase),
                fragment: fragment.paint.display_item_fragment,
            },
            DisplayItemClientInfo::from_fragment(fragment),
            node.contents_properties.nodes.clone(),
        );
        Self { context, previous }
    }

    pub(crate) fn properties_only(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_properties(node.properties.nodes.clone());
        Self { context, previous }
    }

    pub(crate) fn new(
        context: &'c std::cell::RefCell<PaintContext<'o>>,
        node: &PaintTreeNode<'_>,
        item_type: DisplayItemIdType,
    ) -> Self {
        let mut borrowed = context.borrow_mut();
        let previous = borrowed.chunk_properties();
        borrowed.update_chunk_id_and_properties(node, item_type);
        Self { context, previous }
    }
}

impl Drop for ScopedPaintChunkProperties<'_, '_> {
    fn drop(&mut self) {
        self.context
            .borrow_mut()
            .update_chunk_properties(self.previous.clone());
    }
}

#[cfg(test)]
mod opaque_record_tests {
    use super::*;
    use layoutng_assembly::internal::layout_input_types::Color;
    use layoutng_assembly::internal::paint_input::{
        PaintCornerRadii, PaintCornerRadius, PaintShader,
    };

    #[test]
    fn bounded_opaque_record_proof_preserves_clips_rounding_and_uncovered_gaps() {
        let visual = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let solid = DisplayItem {
            rect: visual,
            color: Color {
                alpha: 1.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let clipped = vec![
            DisplayItem {
                r#type: DisplayItemType::kSave,
                ..Default::default()
            },
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                rect: PaintRect {
                    x: 10.2,
                    y: 15.2,
                    width: 30.6,
                    height: 40.6,
                },
                ..Default::default()
            },
            solid.clone(),
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                ..Default::default()
            },
        ];
        let expected = PaintRect {
            x: 11.0,
            y: 16.0,
            width: 29.0,
            height: 39.0,
        };
        assert_eq!(
            CalculateRectKnownToBeOpaqueForRecord(visual, &clipped),
            expected
        );
        let radius = PaintCornerRadius { x: 10.0, y: 10.0 };
        let rounded = DisplayItem {
            r#type: DisplayItemType::kDrawRoundedRect,
            corner_radii: PaintCornerRadii {
                top_left: radius,
                top_right: radius,
                bottom_left: radius,
                bottom_right: radius,
            },
            ..solid.clone()
        };
        assert_eq!(
            CalculateRectKnownToBeOpaqueForRecord(visual, &[rounded]),
            PaintRect {
                x: 3.0,
                y: 3.0,
                width: 94.0,
                height: 94.0
            }
        );
        for unknown in [
            DisplayItem {
                color: Color {
                    alpha: 0.5,
                    ..Default::default()
                },
                ..solid.clone()
            },
            DisplayItem {
                paint_shader: Some(PaintShader::default()),
                ..solid.clone()
            },
            DisplayItem {
                r#type: DisplayItemType::kDrawImageRect,
                ..solid.clone()
            },
            DisplayItem {
                blend_mode: PaintBlendMode::kMultiply,
                ..solid.clone()
            },
        ] {
            assert!(CalculateRectKnownToBeOpaqueForRecord(visual, &[unknown]).is_empty());
        }
        let mut destroying = clipped;
        destroying.push(DisplayItem {
            r#type: DisplayItemType::kSaveLayerDstIn,
            ..Default::default()
        });
        assert!(CalculateRectKnownToBeOpaqueForRecord(visual, &destroying).is_empty());
        assert!(!CalculateRectKnownToBeOpaqueForRecord(visual, &vec![solid.clone(); 8]).is_empty());
        assert!(CalculateRectKnownToBeOpaqueForRecord(visual, &vec![solid.clone(); 9]).is_empty());
        let left = PaintRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        let adjacent = PaintRect { x: 10.0, ..left };
        let separate = PaintRect { x: 20.0, ..left };
        assert_eq!(
            PaintRect::maximum_covered_rect(left, adjacent),
            PaintRect {
                width: 20.0,
                ..left
            }
        );
        assert_eq!(PaintRect::maximum_covered_rect(left, separate), left);
        let client = DisplayItemClientInfo {
            is_cacheable: true,
            is_just_created: false,
        };
        let mut chunker = PaintChunker::default();
        let mut chunks = Vec::new();
        for opaque in [left, adjacent, separate] {
            chunker.increment_display_item(
                &mut chunks,
                DisplayItemId::default(),
                client,
                visual,
                true,
                RasterEffectOutset::kNone,
                true,
                opaque,
            );
        }
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0].rect_known_to_be_opaque,
            PaintRect {
                width: 30.0,
                ..left
            }
        );
    }
}
