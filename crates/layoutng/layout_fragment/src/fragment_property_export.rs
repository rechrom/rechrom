//! Chromium LayoutObject-to-fragment paint-property projection.
//!
//! This assembly-only owner keeps translated source types available while
//! assigning per-object projection and retained refresh self time to
//! `layout_fragment` in sampling profiles.

#![allow(non_snake_case)]

use super::*;
use crate::fragment_data::FragmentData;
use layoutng_style::style::style_transform::{
    ApplyIndependentTransformProperties, ApplyMotionPath, ApplyTransformOperations,
    ApplyTransformOrigin,
};

#[inline(never)]
pub(super) fn ExportPaintProperties(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const StyleComputedStyle,
    resources: &Arc<PaintResources>,
) -> PaintProperties {
    ExportProperties(
        object,
        std::ptr::null_mut(),
        sources,
        style_sources,
        effective_style,
        resources,
        LayoutUpdateKind::Paint,
        None,
        None,
        None,
    )
}

// Source: core/paint/paint_property_tree_builder.cc:464-470. Compositor
// element ids belong to the active FragmentData, not to its LayoutObject's
// first fragment. This distinction is observable for fragmented boxes: each
// FragmentData owns a different UniqueObjectId while all fragments share the
// same LayoutObject.
pub(super) fn PropertyFragmentUniqueId(
    object: *const LayoutObject,
    fragment_data: *mut FragmentData,
) -> u64 {
    let fragment_data = if fragment_data.is_null() {
        unsafe { &mut *object.cast_mut() }
            .GetMutableForPainting()
            .FirstFragment() as *mut FragmentData
    } else {
        fragment_data
    };
    let fragment_data = unsafe { &mut *fragment_data };
    fragment_data.EnsureId();
    fragment_data.UniqueId()
}

pub(super) fn ExportPrePaintInvalidation(
    object: *const LayoutObject,
) -> layout_fragment::fragment_tree::PrePaintInvalidation {
    if object.is_null() {
        return Default::default();
    }
    let object = unsafe { &*object };
    layout_fragment::fragment_tree::PrePaintInvalidation {
        blocking_wheel_event_handler_changed: object.GetPrePaintSubtreeWalkReasons().contains(crate::internal::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kBlockingWheelEventHandler),
        descendant_blocking_wheel_event_handler_changed: object.GetDescendantPrePaintSubtreeWalkReasons().contains(crate::internal::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReason::kBlockingWheelEventHandler),
        needs_paint_property_update: object.NeedsPaintPropertyUpdate(),
        descendant_needs_paint_property_update: object.DescendantNeedsPaintPropertyUpdate(),
        child_pre_paint_blocked_by_display_lock: object.ChildPrePaintBlockedByDisplayLock(),
        should_check_for_paint_invalidation: object.ShouldCheckForPaintInvalidation(),
        background_needs_full_paint_invalidation: object.BackgroundNeedsFullPaintInvalidation(),
        full_paint_invalidation_reason: object
            .ShouldDoFullPaintInvalidation()
            .then(|| object.PaintInvalidationReasonForPrePaint()),
        should_check_layout_for_paint_invalidation: object.ShouldCheckLayoutForPaintInvalidation(),
        descendant_should_check_layout_for_paint_invalidation: object
            .DescendantShouldCheckLayoutForPaintInvalidation(),
        subtree_should_check_for_paint_invalidation: object
            .SubtreeShouldCheckForPaintInvalidation(),
        subtree_should_do_full_paint_invalidation: object.SubtreeShouldDoFullPaintInvalidation(),
        should_delay_full_paint_invalidation: object.ShouldDelayFullPaintInvalidation(),
        previous_visibility_visible: object.PreviousVisibilityVisible(),
        visual_rect_respects_visibility: object.VisualRectRespectsVisibility(),
        should_invalidate_selection: object.ShouldInvalidateSelection(),
        can_be_selection_leaf: object.CanBeSelectionLeaf(),
    }
}

// Source: core/paint/text_painter.cc:743-764.  Keep this lookup in Layout,
// while the source ancestor tree and first-line style variant still exist.
fn ExportSvgTextDecorationStyle(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const StyleComputedStyle,
) -> Option<PaintStyle> {
    if object.is_null() || !unsafe { &*object }.IsSVGInlineText() {
        return None;
    }
    let standard_style = unsafe { &*object }.StyleRef() as *const StyleComputedStyle;
    let use_first_line = !effective_style.is_null() && effective_style != standard_style;
    let mut result = unsafe { &*object }.Parent();
    while !result.is_null() {
        let style = if use_first_line {
            unsafe { &*result }.FirstLineStyleRef()
        } else {
            unsafe { &*result }.StyleRef()
        };
        if style.GetTextDecorationLine() != TextDecorationLine::kNone {
            let source = sources.get(&(result as *const LayoutObject))?;
            let resources = style_sources
                .get(&(style as *const StyleComputedStyle))
                .map(|entry| unsafe { &*entry.resources })
                .unwrap_or_else(|| unsafe { &*source.node }.InputStyleResources());
            let exported =
                crate::internal::boundary::paint_style_export::ExportStyleWithInputResources(
                    style, resources,
                );
            return Some(exported);
        }
        result = unsafe { &*result }.Parent();
    }
    None
}

#[inline(never)]
pub(super) fn ExportProperties(
    object: *const LayoutObject,
    fragment_data: *mut FragmentData,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const StyleComputedStyle,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    mut exported_styles: Option<&mut ExportedStyleCache>,
    mut exported_selection_styles: Option<&mut ExportedSelectionStyleCache>,
    mut debug_names: Option<&mut DebugNameCache>,
) -> PaintProperties {
    let mut output = PaintProperties::default();
    output.resources = Some(Arc::clone(resources));
    if object.is_null() {
        return output;
    }
    output.BindSourceObject(object);
    let object_ref = unsafe { &*object };
    output.is_printing = object_ref.IsPrintingForLayout();
    output.should_print_backgrounds = object_ref.ShouldPrintBackgroundsForLayout();
    output.visible_to_hit_testing = object_ref.VisibleToHitTesting();
    if purpose == LayoutUpdateKind::Paint {
        output.is_selected = object_ref.IsSelected();
    }
    output.effective_allowed_touch_action = object_ref.EffectiveAllowedTouchAction();
    output.inside_blocking_wheel_event_handler = object_ref.InsideBlockingWheelEventHandler();
    output.background_is_known_to_be_obscured = object_ref.BackgroundIsKnownToBeObscured();
    let element = DynamicTo::<Element>(object_ref.GetNode());
    if let Some(element) = unsafe { element.as_ref() } {
        output.region_capture_crop_id = element.InputRegionCaptureCropId();
        output.tracked_element_sub_rects = element.InputTrackedElementSubRects().to_vec();
    }
    output.is_svg_child = object_ref.IsSVGChild();
    output.svg_hidden_container = object_ref.IsSVGHiddenContainer();
    output.svg_descendant_may_have_transform_related_operations =
        object_ref.SVGDescendantMayHaveTransformRelatedOperations();
    if purpose == LayoutUpdateKind::Paint {
        // Source: paint_invalidator.cc:222. LayoutBox uses this point to
        // promote a delayed full invalidation once it intersects the visible
        // viewport, before Paint snapshots the invalidation facts below.
        unsafe { &*object }
            .GetMutableForPainting()
            .EnsureIsReadyForPaintInvalidation();
        output.pre_paint_invalidation = ExportPrePaintInvalidation(object);
        let client = unsafe { &*(object as *const DisplayItemClient) };
        // LayoutObject's base-first representation preserves the source
        // DisplayItemClient subobject, including anonymous layout clients.
        // cpp: core/layout/layout_object.h:280
        // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
        output.property_fragment_unique_id = PropertyFragmentUniqueId(object, fragment_data);
        output.display_item_client_id = client.Id() as u64;
        output.display_item_client_owner_node_id = unsafe { &*object }.OwnerNodeIdDefault();
        output.display_item_client_invalidation_reason = Some(client.GetPaintInvalidationReason());
        output.display_item_client_debug_name = if let Some(cache) = debug_names.as_deref_mut() {
            Arc::clone(
                cache
                    .entry(object)
                    .or_insert_with(|| Arc::from(unsafe { &*object }.DebugName().Utf8())),
            )
        } else {
            Arc::from(unsafe { &*object }.DebugName().Utf8())
        };
        output.display_item_client_is_cacheable = client.IsCacheable();
        output
            .display_item_client_is_just_created
            .set(client.IsJustCreated());
        let layout_text = DynamicTo::<LayoutText>(object);
        let selection_client = if layout_text.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*layout_text }.GetSelectionDisplayItemClient()
        };
        if let Some(selection_client) = unsafe { selection_client.as_ref() } {
            output.selection_client_id = selection_client.Id() as u64;
            output.selection_client_invalidation_reason =
                Some(selection_client.GetPaintInvalidationReason());
            output.selection_client_is_cacheable = selection_client.IsCacheable();
            output
                .selection_client_is_just_created
                .set(selection_client.IsJustCreated());
        }
        output.display_item_raster_effect_outset = client.VisualRectOutsetForRasterEffects();
    }
    let text_combine = DynamicTo::<LayoutTextCombine>(object);
    if !text_combine.is_null() {
        let text_combine = unsafe { &*text_combine };
        // TextCombinePainter paints decorations and emphasis from the parent
        // inline's style after restoring the compressed-text transform.  The
        // anonymous LayoutTextCombine has no source Node, so freeze that
        // parent-owned paint state into this wrapper's typed payload here.
        // cpp: core/paint/text_combine_painter.cc:35-48
        let parent = text_combine.Parent();
        let parent_style = unsafe { &*parent }.StyleRef();
        let parent_image_resource_id = sources
            .get(&(parent as *const LayoutObject))
            .map(|entry| {
                unsafe { &*entry.node }
                    .InputStyleResources()
                    .image_resource_id
            })
            .unwrap_or(None);
        output.style = if purpose == LayoutUpdateKind::Paint {
            crate::internal::boundary::paint_style_export::ExportStyle(
                parent_style,
                parent_image_resource_id,
            )
        } else {
            crate::internal::boundary::paint_style_export::ExportGeometryStyle(parent_style)
        };
        output.writing_mode = PublicWritingMode(parent_style.GetWritingMode());
        let scale_x = text_combine.ScaleXForInline().map(f64::from);
        let inline_spacing =
            scale_x.map_or(0.0, |_| f64::from(text_combine.ComputeInlineSpacing()));
        let wrapper_font = unsafe { &*text_combine.StyleRef().GetFont() };
        let primary_font = wrapper_font.PrimaryFont();
        let text_top_offset = if primary_font.is_null() {
            0.0
        } else {
            let primary_font = unsafe { &*primary_font };
            Number(LayoutUnit::from_f32(
                primary_font.GetFontMetrics().FloatAscent() - primary_font.InternalLeading() / 2.0,
            ))
        };
        output.text_combine = Some(layout_fragment::fragment_tree::TextCombinePaintData {
            scale_x,
            inline_spacing,
            text_top_offset,
            synthetic_oblique: text_combine.UsingSyntheticOblique(),
            font_size: unsafe { &*parent_style.GetFont() }
                .GetFontDescription()
                .ComputedSize() as f64,
            emphasis: (purpose == LayoutUpdateKind::Paint)
                .then(|| ExportTextCombineEmphasis(parent_style))
                .flatten(),
        });
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    if purpose == LayoutUpdateKind::Paint {
        // Sources: core/paint/css_mask_painter.cc:44-98 and
        // svg_background_paint_context.cc:29-33. Ordinary CSS boxes,
        // including SVGForeignObject, use one object-wide maximum mask region
        // for ObjectPaintProperties::MaskClip. The no-clip query below unions
        // the foreignObject's self-painting descendants at its PaintLayer,
        // matching the specialized SVG context while keeping this geometry
        // distinct from the per-layer clip boxes resolved in Paint.
        let object_ref = unsafe { &*object };
        let style = object_ref.StyleRef();
        if style.HasMask()
            && object_ref.IsBoxModelObject()
            && (!object_ref.IsSVGChild() || object_ref.IsSVGForeignObject())
        {
            let no_clip = style.MaskLayers().LayersClipMax() == EFillBox::kNoClip;
            let mut bounds = if object_ref.IsBox() {
                let box_ = unsafe { &*To::<LayoutBox>(object) };
                if no_clip {
                    let layer = box_.Layer();
                    assert!(!layer.is_null());
                    unsafe { &*layer }.LocalBoundingBoxIncludingSelfPaintingDescendants()
                } else {
                    box_.PhysicalBorderBoxRect()
                }
            } else {
                let inline = unsafe { &*To::<LayoutInline>(object) };
                if no_clip {
                    let layer = inline.Layer();
                    assert!(!layer.is_null());
                    unsafe { &*layer }.LocalBoundingBoxIncludingSelfPaintingDescendants()
                } else {
                    inline.PhysicalLinesBoundingBox()
                }
            };
            if style.HasMaskBoxImageOutsets() {
                bounds.Expand(&style.MaskBoxImageOutsets());
            }
            output.css_mask_bounding_box = Some(ExportInkRect(bounds));
        }
        if !layout_box.is_null() {
            let box_ = unsafe { &*layout_box };
            let previous_size = box_.PreviousSize();
            let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(object);
            output.previous_box_geometry =
                Some(layout_fragment::fragment_tree::PreviousBoxGeometry {
                    size: Size {
                        width: Number(previous_size.width),
                        height: Number(previous_size.height),
                    },
                    content_box: ExportInkRect(box_.PreviousPhysicalContentBoxRect()),
                    self_visual_overflow: ExportInkRect(box_.PreviousSelfVisualOverflowRect()),
                    scrollable_overflow: ExportInkRect(box_.PreviousScrollableOverflowRect()),
                    replaced_content: (!replaced.is_null()).then(|| {
                        ExportInkRect(
                            unsafe { &*replaced }
                                .ReplacedContentRectFrom(&box_.PreviousPhysicalContentBoxRect()),
                        )
                    }),
                });
            output.background_should_always_be_clipped =
                unsafe { &*layout_box }.BackgroundShouldAlwaysBeClipped();
            output.scrollable_overflow = Some(ExportInkRect(
                unsafe { &*layout_box }.ScrollableOverflowRect(),
            ));
            output.can_composite_background_attachment_fixed =
                unsafe { &*object }.CanCompositeBackgroundAttachmentFixed();
            output.background_paint_location =
                match unsafe { &*layout_box }.GetBackgroundPaintLocation() {
                    crate::internal::layout_object::kBackgroundPaintInBorderBoxSpace => {
                        BackgroundPaintLocation::BorderBox
                    }
                    crate::internal::layout_object::kBackgroundPaintInContentsSpace => {
                        BackgroundPaintLocation::Contents
                    }
                    crate::internal::layout_object::kBackgroundPaintInBothSpaces => {
                        BackgroundPaintLocation::Both
                    }
                    value => panic!("invalid source background paint location {value}"),
                };
            // Preserve the LayoutBox virtual override rather than calling only
            // the DisplayItemClient base default (layout_box.cc:4099-4108).
            output.display_item_raster_effect_outset =
                unsafe { &*layout_box }.VisualRectOutsetForRasterEffects();
            if unsafe { &*layout_box }.HasCSSClip() {
                output.css_clip = Some(ExportInkRect(unsafe { &*layout_box }.CSSClipRect()));
            }
            let layout_view = DynamicTo::<LayoutView>(object);
            if !layout_view.is_null() {
                output.previous_view_background_rect = Some(ExportInkRect(
                    unsafe { &*layout_view }.PreviousBackgroundRect(),
                ));
                output.view_background_rect =
                    Some(ExportInkRect(unsafe { &*layout_view }.BackgroundRect()));
            }
        } else if unsafe { &*object }.IsSVGShape() {
            // layout_svg_shape.cc:664-674: hairline raster expansion is metadata,
            // not an inflation of the stored VisualRect or chunk bounds.
            let style = unsafe { &*object }.StyleRef();
            if style.HasVisibleStroke() {
                output.display_item_raster_effect_outset =
                    if style.CapStyle() == foundation::LineCap::kButtCap {
                        layout_fragment::fragment_tree::RasterEffectOutset::kHalfPixel
                    } else {
                        layout_fragment::fragment_tree::RasterEffectOutset::kWholePixel
                    };
            }
        }
        // A source object with no PaintLayer is a known result, not permission for
        // Paint to reconstruct a layer from flattened style (notably SVG children).
        // PaintLayerPainter uses the layer's own client, distinct from this box.
        // cpp: core/paint/paint_layer_painter.cc:452-454,484-486,730-732
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            let layer = unsafe { &*box_model }.Layer();
            if !layer.is_null() {
                let layer = unsafe { &*layer };
                output.paint_layer_client_id = layer.Id();
                output.paint_layer_client_invalidation_reason =
                    Some(layer.DisplayItemClient().GetPaintInvalidationReason());
                output.paint_layer_client_is_cacheable = layer.IsCacheable();
                output
                    .paint_layer_client_is_just_created
                    .set(layer.IsJustCreated());
                // Source: core/paint/paint_layer_painter.cc:304-318,361-368,
                // 612-623. Export PaintLayer's already-maintained aggregates;
                // Paint must not approximate them from flattened fragments.
                // Source: core/paint/paint_layer_painter.cc:192-219. Export
                // the LayoutObject-local overflow. PaintLayerPainter applies
                // the current PrePaint FragmentData paint offset, matching the
                // source timing; the Layout snapshot precedes that lifecycle.
                let layout_object = layer.GetLayoutObject();
                let mut visual_rect = if layout_object.IsBox() {
                    let layout_box = DynamicTo::<LayoutBox>(layout_object);
                    debug_assert!(!layout_box.is_null());
                    unsafe { &*layout_box }.SelfVisualOverflowRect()
                } else {
                    layout_object.VisualOverflowRect()
                };
                visual_rect = layout_object.ApplyFiltersToRect(&visual_rect);
                let visual_rect = foundation::gfx::ToEnclosingRect(gfx::RectF::from(visual_rect));
                output.paint_layer_first_fragment_visual_rect = Some(FragmentPaintRect {
                    offset: Offset {
                        x: visual_rect.x() as f64,
                        y: visual_rect.y() as f64,
                    },
                    size: Size {
                        width: visual_rect.width() as f64,
                        height: visual_rect.height() as f64,
                    },
                });
                let layout_box = DynamicTo::<LayoutBox>(layout_object);
                if !layout_box.is_null() {
                    let mut contents_visual_rect =
                        unsafe { &*layout_box }.ContentsVisualOverflowRect();
                    output.paint_layer_contents_visual_rect =
                        Some(ExportInkRect(contents_visual_rect));
                }
                // Source: core/paint/paint_layer.cc:2044-2047. The retained
                // fragment boundary is immutable, so this snapshots the value
                // that PaintLayerResourceInfo caches in Chromium. PrePaint's
                // revision comparison provides the same change detection.
                if layer.HasFilterThatMovesPixels() {
                    output.paint_layer_filter_reference_box =
                        Some(super::layout_fragment_ink_overflow_export::ExportInkRect(
                            layer.LocalBoundingBoxIncludingSelfPaintingDescendants(),
                        ));
                    output.paint_layer_filter_viewport =
                        layer.ComputeFilterViewport().map(|viewport| Size {
                            width: f64::from(viewport.width()),
                            height: f64::from(viewport.height()),
                        });
                }
                // These decisions belong to source Layout/PaintLayer setup. In
                // particular, style adjustment makes static flex/grid items with
                // z-index stacking contexts, and containment/perspective can form
                // contexts without an equivalent field in the public projection.
                let layout_object = unsafe { &*object };
                output.paint_layer_is_in_top_or_view_transition_layer =
                    layout_object.IsInTopOrViewTransitionLayer();
                let layout_parent = layout_object.Parent();
                output.paint_layer_is_direct_layout_view_child =
                    !layout_parent.is_null() && unsafe { &*layout_parent }.IsLayoutView();
                output.paint_layer_can_contain_absolute_position_objects =
                    layout_object.CanContainAbsolutePositionObjects();
                output.paint_layer_can_contain_fixed_position_objects =
                    layout_object.CanContainFixedPositionObjects();
                let scrollable_area = unsafe { &*box_model }.GetScrollableArea();
                output.paint_layer_has_overlay_overflow_controls = !scrollable_area.is_null()
                    && unsafe { &*scrollable_area }.HasOverlayOverflowControls();

                // Source: core/paint/paint_layer_stacking_node.cc:187-254.
                // Export the exact comparison projection used by
                // ForAllChildrenSortedByOrder().
                let parent = layout_object.Parent();
                let order_ancestor = if !parent.is_null()
                    && (unsafe { &*parent }.IsFlexibleBox()
                        || unsafe { &*parent }.IsLayoutGridOrGridLanes())
                {
                    object
                } else if !parent.is_null() {
                    let grandparent = unsafe { &*parent }.Parent();
                    if !grandparent.is_null()
                        && (unsafe { &*grandparent }.IsFlexibleBox()
                            || unsafe { &*grandparent }.IsLayoutGridOrGridLanes())
                    {
                        parent
                    } else {
                        std::ptr::null_mut()
                    }
                } else {
                    std::ptr::null_mut()
                };
                if !order_ancestor.is_null() {
                    let order_parent = unsafe { &*order_ancestor }.Parent();
                    if !order_parent.is_null() {
                        // Chromium compares the shared flex/grid parent by
                        // LayoutObject identity (paint_layer_stacking_node.cc:
                        // 218-236).  The retained projection carries that
                        // identity as FragmentData's UniqueObjectId instead,
                        // so allocate the id on this actual owner before
                        // reading it.  The parent may not have reached its own
                        // paint export yet (for example through a display:
                        // contents wrapper), and UniqueId() deliberately
                        // assumes EnsureId() has already run.
                        unsafe { &mut *order_parent }
                            .GetMutableForPainting()
                            .EnsureId();
                        output.paint_layer_order_parent_unique_id =
                            Some(unsafe { &*order_parent }.UniqueId());
                        output.paint_layer_order = if order_ancestor == object
                            && unsafe { &*order_ancestor }.IsOutOfFlowPositioned()
                        {
                            0
                        } else {
                            unsafe { &*order_ancestor }.StyleRef().Order()
                        };
                    }
                }
            }
        }
    }
    let Some(source_entry) = sources.get(&object) else {
        return output;
    };
    let source = unsafe { &*source_entry.node };
    if source.IsElementNode() {
        let element = unsafe {
            &*(source as *const Node as *const crate::internal::layout_node_metadata::Element)
        };
        if let Some(data) = element.InputElementData() {
            output.scroll_offset = data.scroll_offset;
        }
    }
    let mut style_resources = source.InputStyleResources();
    if !effective_style.is_null() {
        if let Some(effective_source) = style_sources.get(&effective_style) {
            style_resources = unsafe { &*effective_source.resources };
        }
    }
    let projected_style_identity =
        style_resources as *const crate::internal::layout_input::StyleResourceData as usize;
    output.logical_tree_order = source_entry.logical_tree_order;
    let mut parent = unsafe { &*object }.Parent();
    while !parent.is_null() {
        if let Some(parent_source) = sources.get(&(parent as *const LayoutObject)) {
            output.logical_parent_node_id = Some(unsafe { &*parent_source.node }.InputId());
            break;
        }
        parent = unsafe { &*parent }.Parent();
    }
    output.source_kind = source.InputKind();
    // LayoutImage owns this virtual identity. Do not reconstruct it from DOM
    // fields here: generated ImageContentData has no HTML-image flag, while
    // other LayoutReplaced subclasses (for example canvas) are not images.
    // cpp: core/layout/layout_image.h:135-138
    output.replaced_is_image = unsafe { &*object }.IsLayoutImage();
    if purpose == LayoutUpdateKind::Paint {
        let replaced = DynamicTo::<crate::internal::layout_replaced::LayoutReplaced>(object);
        if !replaced.is_null() {
            let replaced = unsafe { &*replaced };
            output.replaced_selection_state = replaced.GetSelectionState();
            let selection_rect = replaced.LocalSelectionVisualRect();
            if !selection_rect.IsEmpty() {
                output.replaced_local_selection_visual_rect = Some(ExportInkRect(selection_rect));
            }
        }
    }
    let layout_style = if effective_style.is_null() {
        unsafe { &*object }.StyleRef()
    } else {
        unsafe { &*effective_style }
    };
    let selection_key = layout_style as *const StyleComputedStyle as usize;
    let style_facts = if purpose == LayoutUpdateKind::Paint {
        exported_selection_styles
            .as_deref()
            .and_then(|styles| styles.get(&selection_key))
            .map(|entry| entry.2)
            .unwrap_or_else(|| crate::style_projection::ReadFragmentStyleFacts(layout_style))
    } else {
        crate::style_projection::ReadFragmentStyleFacts(layout_style)
    };
    output.effective_zoom = layout_style.EffectiveZoom();
    if purpose == LayoutUpdateKind::Paint {
        output.background_fill_layer_count =
            crate::internal::boundary::paint_style_export::BackgroundFillLayerCount(layout_style);
        output.has_background = layout_style.HasBackground();
        let background_layers = layout_style.BackgroundLayers();
        output.background_any_local_attachment_image =
            background_layers.AnyLayerHasLocalAttachmentImage();
        output.background_any_default_attachment_image =
            background_layers.AnyLayerHasDefaultAttachmentImage();
        output.background_any_layer_uses_content_box = background_layers.AnyLayerUsesContentBox();
    }
    let exported_style_key = (
        layout_style as *const StyleComputedStyle as usize,
        projected_style_identity,
    );
    let cached_style = exported_styles
        .as_deref()
        .and_then(|styles| styles.get(&exported_style_key))
        .cloned();
    if purpose == LayoutUpdateKind::Layout {
        output.style =
            crate::internal::boundary::paint_style_export::ExportGeometryStyle(layout_style);
    } else if let Some(style) = cached_style {
        output.style = style;
    } else {
        output.style = crate::internal::boundary::paint_style_export::ExportStyleWithInputResources(
            layout_style,
            style_resources,
        );
        if let Some(target) = output
            .style
            .clip_path
            .as_ref()
            .and_then(|clip| clip.reference_target_id)
        {
            // Reuse the source SVG viewport query at the existing Layout boundary.
            // The resource can live under a different nested SVG from its client.
            let resource = sources.iter().find_map(|(&object, entry)| {
                (unsafe { &*entry.node }.InputId() == target
                    && unsafe { &*entry.node }.InputKind() == NodeKind::kSvgResourceClipper)
                    .then_some(object)
            });
            let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
            let resolve = unsafe { algorithms.as_ref() }.and_then(|a| a.svg_support.viewport_size);
            if let (Some(resource), Some(resolve)) = (resource, resolve) {
                let size = resolve(unsafe { &*resource });
                output.style.clip_path.as_mut().unwrap().reference_viewport = Some(Size {
                    width: f64::from(size.width()),
                    height: f64::from(size.height()),
                });
            } else {
                // Connected ID targets without an attached source resource are
                // unavailable (for example a display:none ancestor).
                output.style.clip_path = None;
            }
        }
        for reference in &mut output.style.svg_mask_bindings {
            let Some(target) = reference.target_id else {
                continue;
            };
            let resource = sources.iter().find_map(|(&object, entry)| {
                (unsafe { &*entry.node }.InputId() == target
                    && unsafe { &*entry.node }.InputKind() == NodeKind::kSvgResourceMasker)
                    .then_some(object)
            });
            let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
            let resolve = unsafe { algorithms.as_ref() }.and_then(|a| a.svg_support.viewport_size);
            if let (Some(resource), Some(resolve)) = (resource, resolve) {
                let size = resolve(unsafe { &*resource });
                reference.viewport = Some(Size {
                    width: f64::from(size.width()),
                    height: f64::from(size.height()),
                });
            } else {
                reference.graph = None;
            }
        }
        // Layout-dependent transforms are resolved below per object. Every
        // other style payload is immutable and can be shared by all physical
        // and inline fragments that reference the same source/projected pair.
        if !layout_style.HasOffset() {
            if let Some(styles) = exported_styles.as_deref_mut() {
                styles.insert(exported_style_key, output.style.clone());
            }
        }
    }
    if layout_style.HasOffset() && !layout_box.is_null() {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            let layout_box = unsafe { &*layout_box };
            let reference_box =
                crate::internal::transform_utils::ComputeReferenceBoxForLayoutBox(layout_box);
            let mut offset = foundation::gfx::Transform::default();
            layout_style.ApplyTransformPhysicalRect(
                &mut offset,
                Some(layout_box),
                &reference_box,
                ApplyTransformOperations::kExcludeTransformOperations,
                ApplyTransformOrigin::kExcludeTransformOrigin,
                ApplyMotionPath::kIncludeMotionPath,
                ApplyIndependentTransformProperties::kExcludeIndependentTransformProperties,
            );
            crate::internal::boundary::paint_style_export::SetResolvedOffsetTransform(
                &mut output.style,
                offset,
            );
        }
    }
    if purpose == LayoutUpdateKind::Paint {
        let cached_selection = exported_selection_styles
            .as_deref()
            .and_then(|styles| styles.get(&selection_key))
            .cloned();
        let (selection_style, selection_text_decoration, _) =
            cached_selection.unwrap_or_else(|| {
                let exported = (
                    ExportSelectionPaintStyle(layout_style),
                    ExportSelectionTextDecoration(layout_style),
                    style_facts,
                );
                if let Some(styles) = exported_selection_styles.as_deref_mut() {
                    styles.insert(selection_key, exported.clone());
                }
                exported
            });
        output.selection_style = selection_style;
        output.selection_text_decoration = selection_text_decoration;
        output.svg_text_decoration_style =
            ExportSvgTextDecorationStyle(object, sources, style_sources, effective_style);
    }
    output.margin = style_facts.margin;
    output.border = style_facts.border;
    output.border_styles = style_facts.border_styles;
    output.padding = style_facts.padding;
    output.display = style_facts.display;
    output.position = style_facts.position;
    output.fixed_to_view =
        !layout_box.is_null() && unsafe { &*layout_box }.IsFixedToView(std::ptr::null());
    output.needs_anchor_position_scroll_adjustment =
        !layout_box.is_null() && unsafe { &*layout_box }.NeedsAnchorPositionScrollAdjustment();
    output.floating = style_facts.floating;
    ExportStickyPaintProperties(object, &mut output);
    output.writing_mode = style_facts.writing_mode;
    output.direction = style_facts.direction;
    output.overflow_x = style_facts.overflow_x;
    output.overflow_y = style_facts.overflow_y;
    let export_overscroll = |behavior| match behavior {
        EOverscrollBehavior::kAuto => crate::internal::layout_input::OverscrollBehavior::kAuto,
        EOverscrollBehavior::kContain => {
            crate::internal::layout_input::OverscrollBehavior::kContain
        }
        EOverscrollBehavior::kNone => crate::internal::layout_input::OverscrollBehavior::kNone,
        EOverscrollBehavior::kChain => crate::internal::layout_input::OverscrollBehavior::kChain,
    };
    output.overscroll_behavior_x = export_overscroll(layout_style.OverscrollBehaviorX());
    output.overscroll_behavior_y = export_overscroll(layout_style.OverscrollBehaviorY());
    // Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    output.prevent_scroll_axis_locking =
        layout_style.ScrollAxisLock() == foundation::EScrollAxisLock::kNone;
    let element_data = if source.IsElementNode() {
        unsafe { &*(source as *const Node as *const Element) }
            .InputElementData()
            .as_ref()
    } else {
        None
    };
    if let Some(element_data) = element_data {
        output.scroll_offset = element_data.scroll_offset;
        output.image_decoding_mode = element_data.image_decoding_mode;
        output.video_frame_resource_id = element_data.video_frame_resource_id;
        output.focused_image_map_area = element_data.focused_image_map_area.clone();
        if !object.is_null() && unsafe { &*object }.IsSVGShape() {
            let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
            output.svg_shape = unsafe { algorithms.as_ref() }
                .and_then(|a| a.svg_support.retained_shape)
                .and_then(|query| query(unsafe { &*object }))
                .cloned();
        }
        output.svg_view_box = element_data.svg_view_box;
        output.svg_view_box_is_empty = element_data
            .svg_view_box
            .is_some_and(|view_box| view_box.width <= 0.0 || view_box.height <= 0.0);
    }
    if !object.is_null() && unsafe { &*object }.IsSVG() {
        let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
        if let Some(query) =
            unsafe { algorithms.as_ref() }.and_then(|a| a.svg_support.paint_geometry)
        {
            let (transform, viewport, visual_rect) = query(unsafe { &*object });
            output.svg_local_transform = transform;
            output.svg_viewport = viewport;
            output.svg_visual_rect = visual_rect.map(|rect| FragmentPaintRect {
                offset: Offset {
                    x: rect.x() as f64,
                    y: rect.y() as f64,
                },
                size: Size {
                    width: rect.width() as f64,
                    height: rect.height() as f64,
                },
            });
        }
    }
    // SVGResources::ReferenceBoxForEffects(kViewBox) uses (0,0) and the
    // nearest viewport's resolved dimensions. Stop at a nested viewport even
    // when it has no viewBox; Paint does not discover Layout/DOM ancestors.
    if !object.is_null() && unsafe { &*object }.IsSVGChild() {
        let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
        if let Some(resolve) =
            unsafe { algorithms.as_ref() }.and_then(|a| a.svg_support.viewport_size)
        {
            let size = resolve(unsafe { &*object });
            output.svg_view_box = Some(crate::internal::layout_input::SvgViewBoxData {
                width: size.width() as f64,
                height: size.height() as f64,
                ..Default::default()
            });
        }
    }
    if purpose == LayoutUpdateKind::Layout {
        return output;
    }
    // Preserve the real editor identity rather than reconstructing its synthetic id.
    let mut ancestor = source as *const Node;
    while !ancestor.is_null() {
        let node = unsafe { &*ancestor };
        if node.InputKind() == NodeKind::kFormControl {
            output.text_control_host = Some(node.InputId());
            break;
        }
        ancestor = node.parentNode().cast();
    }
    let appearance = layout_style.EffectiveAppearance();
    if appearance != AppearanceValue::kNone
        || element_data.is_some_and(|element_data| element_data.control_focused)
    {
        let mut control = FormControlPaintData {
            appearance,
            ..FormControlPaintData::default()
        };
        if let Some(element_data) = element_data {
            control.r#type = element_data.form_control_type;
            control.checked = element_data.control_checked;
            control.indeterminate = element_data.control_indeterminate;
            control.is_progress = element_data.control_is_progress;
            control.disabled = element_data.control_disabled;
            control.read_only = element_data.control_read_only;
            control.hovered = element_data.control_hovered;
            control.active = element_data.control_active;
            control.focused = element_data.control_focused;
            // OutlinePainter uses the computed outline, including the UA
            // :focus-visible rule; focus alone must not override outline:none
            // or add an auto ring alongside an author's non-auto outline.
            control.auto_focus_ring = element_data.control_focused
                && appearance == AppearanceValue::kNone
                && layout_style.HasOutline()
                && layout_style.OutlineStyleIsAuto();
            control.autofilled = element_data.autofill_state != AutofillState::kNotFilled;
            control.value_ratio = element_data
                .control_value_ratio
                .or(element_data.range_value_ratio);
        }
        if !layout_style.AccentColor().IsAutoColor() {
            control.accent_color = crate::internal::boundary::paint_style_export::StyleColor(
                layout_style,
                layout_style.AccentColor().ToStyleColor(),
            );
        }
        output.form_control = Some(Arc::new(control));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn property_fragment_unique_id_uses_selected_fragment_data() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut first = FragmentData::default();
        let mut second = FragmentData::default();
        let first_id = PropertyFragmentUniqueId(std::ptr::null(), &mut first);
        let second_id = PropertyFragmentUniqueId(std::ptr::null(), &mut second);

        assert_ne!(first_id, second_id);
        assert_eq!(
            first_id,
            PropertyFragmentUniqueId(std::ptr::null(), &mut first)
        );
        assert_eq!(
            second_id,
            PropertyFragmentUniqueId(std::ptr::null(), &mut second)
        );
    }
}

/// Selection commit updates the same resident PrePaint sidecars after LayoutClean.
pub(crate) fn ExportSelectionPrePaintInvalidation(
    object: *const LayoutObject,
) -> layout_fragment::fragment_tree::PrePaintInvalidation {
    ExportPrePaintInvalidation(object)
}
