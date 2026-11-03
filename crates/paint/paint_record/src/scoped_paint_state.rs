//! Object and box-contents paint state shared by fragment painters and the
//! layer walk.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/scoped_paint_state.cc:130-164`.

use std::sync::Arc;

use layoutng_assembly::fragment_tree::FragmentPaintData;
use layoutng_assembly::internal::layout_input::{NodeKind, Offset};
use paint_model::geometry_mapper::SourceToDestinationRect;
use paint_model::geometry_mapper::TranslationTransform;
use paint_model::paint_property_tree::{PaintPropertyRole, TransformPaintPropertyNode};
use paint_model::{CullRect, PaintPhase, PaintRect, PaintResult};
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::display_item_id::DisplayItemIdType;
use crate::paint_context::ScopedPaintChunkProperties;
use crate::paint_info::PaintInfo;
use crate::{DisplayItem, DisplayItemType};

fn IsSvgLayoutChild(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::kSvgGroup
            | NodeKind::kSvgHiddenContainer
            | NodeKind::kSvgResourceClipper
            | NodeKind::kSvgResourceMasker
            | NodeKind::kSvgForeignObject
            | NodeKind::kSvgText
            | NodeKind::kSvgInline
            | NodeKind::kSvgTSpan
            | NodeKind::kSvgTextPath
            | NodeKind::kSvgShape
            | NodeKind::kSvgViewportContainer
    )
}

fn FragmentRect(rect: layoutng_assembly::fragment_tree::FragmentPaintRect) -> PaintRect {
    PaintRect {
        x: rect.offset.x,
        y: rect.offset.y,
        width: rect.size.width,
        height: rect.size.height,
    }
}

#[doc(hidden)]
pub fn CullRectInTransform(
    root_cull_rect: PaintRect,
    target: &Arc<TransformPaintPropertyNode>,
) -> PaintRect {
    let mut root = Arc::clone(target);
    while let Some(parent) = &root.parent {
        root = Arc::clone(parent);
    }
    SourceToDestinationRect(&root, target, root_cull_rect).unwrap_or_default()
}

fn LocalCullRect(
    root_cull_rect: Option<PaintRect>,
    target: &Arc<TransformPaintPropertyNode>,
) -> CullRect {
    root_cull_rect.map_or_else(CullRect::Infinite, |root_cull_rect| {
        CullRect::FromRect(CullRectInTransform(root_cull_rect, target))
    })
}

/// Source: `paint_layer_painter.cc:192-208`.
#[doc(hidden)]
pub fn ContentsVisualRect(node: &PaintTreeNode<'_>) -> Option<PaintRect> {
    let fragment = node.fragment.as_deref()?;
    let mut contents = FragmentRect(fragment.paint.paint_layer_contents_visual_rect?);
    if fragment.paint.svg_view_box.is_some() {
        contents = SourceToDestinationRect(
            &node.properties.nodes.transform,
            &node.contents_properties.nodes.transform,
            contents,
        )
        .map(PaintRect::enclosing)
        .unwrap_or_default();
    }
    Some(contents)
}

/// Source: `scoped_paint_state.cc:152-164`.
fn UpdatePreviousPaintResultForContents(node: &PaintTreeNode<'_>) {
    let Some(fragment) = node.fragment.as_deref() else {
        return;
    };
    if !fragment.paint.IsLayoutBox()
        || !fragment.paint.PaintLayerStateIsAuthoritative()
        || fragment.paint.paint_layer_client_id == 0
        || node.previous_paint_result.get() != PaintResult::kFullyPainted
    {
        return;
    }
    let (Some(root_cull_rect), Some(contents_visual_rect)) =
        (node.cull_rect, ContentsVisualRect(node))
    else {
        return;
    };
    let contents_cull_rect =
        CullRectInTransform(root_cull_rect, &node.contents_properties.nodes.transform);
    if !contents_cull_rect.contains(contents_visual_rect) {
        node.previous_paint_result
            .set(PaintResult::kMayBeClippedByCullRect);
    }
}

/// Source: `scoped_paint_state.h:18-94` and
/// `scoped_paint_state.cc:16-48,70-80,93-121`.
///
/// PaintTreeNode is Rechrom's resident FragmentData/ObjectPaintProperties
/// owner. The scope therefore borrows that node instead of reconstructing an
/// object-property decision from style or display values.
pub struct ScopedPaintState<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    input_paint_info: PaintInfo<'c, 'o>,
    paint_offset: Offset,
    _chunk_properties: Option<ScopedPaintChunkProperties<'c, 'o>>,
    paint_offset_translation_as_drawing: bool,
}

impl<'n, 'f, 'c, 'o> ScopedPaintState<'n, 'f, 'c, 'o> {
    pub fn new(node: &'n PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) -> Self {
        let mut adjusted_paint_info = *paint_info;
        adjusted_paint_info.SetCullRect(LocalCullRect(
            node.self_cull_rect,
            &node.properties.nodes.transform,
        ));
        let Some(fragment) = node.fragment.as_deref() else {
            return Self {
                node,
                input_paint_info: adjusted_paint_info,
                paint_offset: Offset {
                    x: f64::MAX / 4.0,
                    y: f64::MAX / 4.0,
                },
                _chunk_properties: None,
                paint_offset_translation_as_drawing: false,
            };
        };

        // PaintLayerPainter already installed LocalBorderBoxProperties for a
        // self-painting layer, and overlay controls are painted in that same
        // layer-owned state.
        let layer_adjusted = paint_info.phase == PaintPhase::kOverlayOverflowControls
            || (node.is_paint_layer && node.is_self_painting_layer);
        let svg_child = IsSvgLayoutChild(fragment.paint.source_kind);
        let paint_offset_translation_as_drawing = !layer_adjusted
            && !svg_child
            && node.paint_offset_translation.is_some()
            && paint_info.context.borrow().InDrawingRecorder();
        if paint_offset_translation_as_drawing {
            let translation = node.paint_offset_translation.as_ref().unwrap().matrix;
            paint_info.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSave,
                    phase: paint_info.phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                node,
            );
            paint_info.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kConcat,
                    phase: paint_info.phase,
                    node_id: fragment.node_id,
                    transform: TranslationTransform(translation.values[12], translation.values[13]),
                    ..Default::default()
                },
                node,
            );
        }
        if !layer_adjusted && !svg_child && !paint_offset_translation_as_drawing {
            // ObjectPaintProperties has no ElementCanvasTransform slot in the
            // current typed boundary. The other individual-transform assertions
            // remain runtime checks, matching ScopedPaintState's source contract.
            debug_assert!(!node.HasOwnPropertyRole(PaintPropertyRole::Translate));
            debug_assert!(!node.HasOwnPropertyRole(PaintPropertyRole::Rotate));
            debug_assert!(!node.HasOwnPropertyRole(PaintPropertyRole::Scale));
            debug_assert!(!node.HasOwnPropertyRole(PaintPropertyRole::Offset));
        }
        let chunk_properties = if paint_info.phase == PaintPhase::kMask {
            // PaintLayerPainter's source path installs MaskProperties before
            // entering the object painter. Rechrom retains that source slot on
            // the node and installs it at this package boundary.
            Some(ScopedPaintChunkProperties::mask(paint_info.context, node))
        } else if layer_adjusted || svg_child || paint_offset_translation_as_drawing {
            None
        } else if fragment.paint.is_page_border_box {
            // The detached @page border box paints document decorations in
            // document space, not its fitted page-container space.
            Some(ScopedPaintChunkProperties::new(
                paint_info.context,
                node,
                DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase),
            ))
        } else if node.applies_transform || node.applies_effect {
            Some(ScopedPaintChunkProperties::new(
                paint_info.context,
                node,
                DisplayItemIdType::PaintPhaseToDrawingType(paint_info.phase),
            ))
        } else {
            None
        };

        Self {
            node,
            input_paint_info: adjusted_paint_info,
            paint_offset: node.paint_offset,
            _chunk_properties: chunk_properties,
            paint_offset_translation_as_drawing,
        }
    }

    fn unadjusted(
        node: &'n PaintTreeNode<'f>,
        paint_info: &PaintInfo<'c, 'o>,
        paint_offset: Offset,
    ) -> Self {
        let mut adjusted_paint_info = *paint_info;
        adjusted_paint_info.SetCullRect(LocalCullRect(
            node.self_cull_rect,
            &node.properties.nodes.transform,
        ));
        Self {
            node,
            input_paint_info: adjusted_paint_info,
            paint_offset,
            _chunk_properties: None,
            paint_offset_translation_as_drawing: false,
        }
    }

    pub fn GetPaintInfo(&self) -> &PaintInfo<'c, 'o> {
        &self.input_paint_info
    }

    pub fn MutablePaintInfo(&mut self) -> &mut PaintInfo<'c, 'o> {
        &mut self.input_paint_info
    }

    pub fn PaintOffset(&self) -> Offset {
        self.paint_offset
    }

    pub fn FragmentToPaint(&self) -> Option<&FragmentPaintData> {
        self.node.fragment.as_deref()
    }

    pub fn LocalRectIntersectsCullRect(&self, local_rect: PaintRect) -> bool {
        self.input_paint_info
            .IntersectsCullRect(local_rect, self.paint_offset)
    }
}

impl Drop for ScopedPaintState<'_, '_, '_, '_> {
    fn drop(&mut self) {
        if !self.paint_offset_translation_as_drawing {
            return;
        }
        debug_assert!(self.input_paint_info.context.borrow().InDrawingRecorder());
        let node_id = self
            .node
            .fragment
            .as_deref()
            .map_or(0, |fragment| fragment.node_id);
        self.input_paint_info.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: self.input_paint_info.phase,
                node_id,
                ..Default::default()
            },
            self.node,
        );
    }
}

/// Source: `scoped_paint_state.h:98-116` and
/// `scoped_paint_state.cc:130-164`.
///
/// The resident tree already owns the adjusted paint offset and cull state.
/// This scope installs ContentsProperties and performs the source layer-result
/// update while retaining paint_record's existing package boundary.
pub struct ScopedBoxContentsPaintState<'n, 'f, 'c, 'o> {
    state: ScopedPaintState<'n, 'f, 'c, 'o>,
    _chunk_properties: Option<ScopedPaintChunkProperties<'c, 'o>>,
}

impl<'n, 'f, 'c, 'o> ScopedBoxContentsPaintState<'n, 'f, 'c, 'o> {
    pub fn from_state(
        input: &ScopedPaintState<'n, 'f, 'c, 'o>,
        node: &'n PaintTreeNode<'f>,
    ) -> Self {
        Self::with_offset(input.GetPaintInfo(), input.PaintOffset(), node)
    }

    pub fn new(paint_info: &PaintInfo<'c, 'o>, node: &'n PaintTreeNode<'f>) -> Self {
        Self::with_offset(paint_info, node.paint_offset, node)
    }

    fn with_offset(
        paint_info: &PaintInfo<'c, 'o>,
        paint_offset: Offset,
        node: &'n PaintTreeNode<'f>,
    ) -> Self {
        debug_assert!(!matches!(
            paint_info.phase,
            PaintPhase::kSelfOutlineOnly | PaintPhase::kMask
        ));
        let mut state = ScopedPaintState::unadjusted(node, paint_info, paint_offset);
        let Some(fragment) = node.fragment.as_deref() else {
            return Self {
                state,
                _chunk_properties: None,
            };
        };
        if !fragment.paint.IsLayoutBox() {
            return Self {
                state,
                _chunk_properties: None,
            };
        }
        if fragment.paint.scroll_container.is_some()
            && !Arc::ptr_eq(
                &node.properties.nodes.transform,
                &node.contents_properties.nodes.transform,
            )
        {
            let scroll_origin = fragment
                .paint
                .scroll_container
                .as_ref()
                .unwrap()
                .scroll_origin;
            state.paint_offset.x += scroll_origin.x;
            state.paint_offset.y += scroll_origin.y;
        }
        state.input_paint_info.SetCullRect(LocalCullRect(
            node.cull_rect,
            &node.contents_properties.nodes.transform,
        ));
        let chunk_properties = Some(ScopedPaintChunkProperties::contents(
            paint_info.context,
            node,
            paint_info.phase,
        ));
        UpdatePreviousPaintResultForContents(node);
        Self {
            state,
            _chunk_properties: chunk_properties,
        }
    }

    pub fn GetPaintInfo(&self) -> &PaintInfo<'c, 'o> {
        self.state.GetPaintInfo()
    }

    pub fn MutablePaintInfo(&mut self) -> &mut PaintInfo<'c, 'o> {
        self.state.MutablePaintInfo()
    }

    pub fn PaintOffset(&self) -> Offset {
        self.state.PaintOffset()
    }

    pub fn FragmentToPaint(&self) -> Option<&FragmentPaintData> {
        self.state.FragmentToPaint()
    }

    pub fn LocalRectIntersectsCullRect(&self, local_rect: PaintRect) -> bool {
        self.state.LocalRectIntersectsCullRect(local_rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_assembly::fragment_tree::{
        FragmentPaintData, FragmentPaintRect, ScrollContainerPaintData,
    };
    use layoutng_assembly::internal::layout_input::{Offset, Size};
    use paint_model::paint_property_tree::TransformPaintPropertyNode;
    use std::cell::RefCell;

    use crate::paint_context::PaintContext;

    fn layer_with_contents(contents_width: f64, cull_width: f64) -> PaintTreeNode<'static> {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.is_layout_box = true;
        fragment.paint.paint_layer_state_is_authoritative = true;
        fragment.paint.paint_layer_client_id = 7;
        fragment.paint.paint_layer_contents_visual_rect = Some(FragmentPaintRect {
            offset: Offset::default(),
            size: Size {
                width: contents_width,
                height: 10.0,
            },
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.cull_rect = Some(PaintRect {
            width: cull_width,
            height: 10.0,
            ..Default::default()
        });
        node.previous_paint_result.set(PaintResult::kFullyPainted);
        node
    }

    #[test]
    fn contents_scope_marks_layer_result_when_contents_cull_is_incomplete() {
        let clipped = layer_with_contents(20.0, 10.0);
        let mut output = crate::PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        {
            let info = PaintInfo::new(&context, PaintPhase::kForeground);
            let _scope = ScopedBoxContentsPaintState::new(&info, &clipped);
        }
        assert_eq!(
            clipped.previous_paint_result.get(),
            PaintResult::kMayBeClippedByCullRect
        );

        let contained = layer_with_contents(10.0, 20.0);
        {
            let info = PaintInfo::new(&context, PaintPhase::kForeground);
            let _scope = ScopedBoxContentsPaintState::new(&info, &contained);
        }
        assert_eq!(
            contained.previous_paint_result.get(),
            PaintResult::kFullyPainted
        );
    }

    #[test]
    fn object_scope_skips_self_painting_layers_and_svg_children() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.applies_transform = true;
        let parent = node.properties.nodes.transform.clone();
        node.properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
            id: 17,
            parent: Some(parent),
            ..(*node.properties.nodes.transform).clone()
        });

        let mut output = crate::PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        let input = context.borrow().chunk_properties();
        let info = PaintInfo::new(&context, PaintPhase::kForeground);
        {
            let _state = ScopedPaintState::new(&node, &info);
            assert!(Arc::ptr_eq(
                &context.borrow().chunk_properties().transform,
                &node.properties.nodes.transform
            ));
        }
        assert!(context.borrow().chunk_properties().same_nodes(&input));

        node.is_paint_layer = true;
        node.is_self_painting_layer = true;
        {
            let _state = ScopedPaintState::new(&node, &info);
            assert!(context.borrow().chunk_properties().same_nodes(&input));
        }

        node.is_paint_layer = false;
        node.is_self_painting_layer = false;
        node.fragment.as_mut().unwrap().paint.source_kind = NodeKind::kSvgShape;
        {
            let _state = ScopedPaintState::new(&node, &info);
            assert!(context.borrow().chunk_properties().same_nodes(&input));
        }

        node.fragment.as_mut().unwrap().paint.source_kind = NodeKind::kBox;
        node.fragment.as_mut().unwrap().paint.is_page_border_box = true;
        node.applies_transform = false;
        {
            let _state = ScopedPaintState::new(&node, &info);
            assert!(Arc::ptr_eq(
                &context.borrow().chunk_properties().transform,
                &node.properties.nodes.transform
            ));
        }
        assert!(context.borrow().chunk_properties().same_nodes(&input));
    }

    #[test]
    fn object_scope_records_paint_offset_translation_inside_drawing() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.display_item_client_id = 9;
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        let parent = node.properties.nodes.transform.clone();
        let translation = Arc::new(TransformPaintPropertyNode {
            id: 18,
            parent: Some(parent),
            matrix: TranslationTransform(4.0, 6.0),
            ..(*node.properties.nodes.transform).clone()
        });
        node.paint_offset_translation = Some(translation.clone());
        node.properties.nodes.transform = translation;
        node.applies_transform = true;

        let mut output = crate::PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        context.borrow_mut().BeginDrawingRecorder(
            &node,
            DisplayItemIdType::kBoxDecorationBackground,
            PaintRect::default(),
        );
        let info = PaintInfo::new(&context, PaintPhase::kForeground);
        {
            let _state = ScopedPaintState::new(&node, &info);
        }
        context.borrow_mut().EndDrawingRecorder();
        drop(context);

        assert_eq!(output.items.len(), 3);
        assert_eq!(output.items[0].r#type, DisplayItemType::kSave);
        assert_eq!(output.items[1].r#type, DisplayItemType::kConcat);
        assert_eq!(output.items[1].transform.values[12], 4.0);
        assert_eq!(output.items[1].transform.values[13], 6.0);
        assert_eq!(output.items[2].r#type, DisplayItemType::kRestore);
    }

    #[test]
    fn contents_scope_uses_scroll_origin_and_contents_cull() {
        let mut fragment = FragmentPaintData::default();
        fragment.paint.has_source = true;
        fragment.paint.is_layout_box = true;
        fragment.paint.scroll_container = Some(ScrollContainerPaintData {
            compositor_element_id: Default::default(),
            scrollable_axes: [true, true],
            scroll_origin: Offset { x: 7.0, y: 11.0 },
            scroll_position: Offset::default(),
            background_needs_repaint_on_scroll: None,
            prefers_non_composited_scrolling: None,
            scroll_snap_type: None,
            container_rect: Default::default(),
            scrolling_background_visual_rect: Default::default(),
            minimum_scroll_offset: Offset::default(),
            maximum_scroll_offset: Offset::default(),
            user_scrollable_horizontal: true,
            user_scrollable_vertical: true,
            scrolling_background_client_id: 0,
            scrolling_background_client_invalidation_reason: None,
            scrolling_background_client_is_cacheable: false,
            scrolling_background_client_is_just_created: Default::default(),
        });
        let mut node = PaintTreeNode::default();
        node.SetOwnedFragmentForTesting(&fragment);
        node.paint_offset = Offset { x: 3.0, y: 5.0 };
        let parent = node.contents_properties.nodes.transform.clone();
        node.contents_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
            id: 19,
            parent: Some(parent),
            ..(*node.contents_properties.nodes.transform).clone()
        });
        node.self_cull_rect = Some(PaintRect {
            width: 5.0,
            height: 5.0,
            ..Default::default()
        });
        node.cull_rect = Some(PaintRect {
            x: 10.0,
            y: 15.0,
            width: 20.0,
            height: 20.0,
        });

        let mut output = crate::PaintRecording::default();
        let context = RefCell::new(PaintContext::new(&mut output));
        let info = PaintInfo::new(&context, PaintPhase::kForeground);
        let state = ScopedBoxContentsPaintState::new(&info, &node);
        assert_eq!(state.PaintOffset(), Offset { x: 10.0, y: 16.0 });
        assert!(state.LocalRectIntersectsCullRect(PaintRect {
            width: 1.0,
            height: 1.0,
            ..Default::default()
        }));
        assert!(!state.LocalRectIntersectsCullRect(PaintRect {
            x: 100.0,
            y: 100.0,
            width: 1.0,
            height: 1.0,
        }));
    }
}
