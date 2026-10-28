//! Native LayoutObject-to-fragment paint-property projection.
//!
//! This assembly-only owner keeps translated native types available while
//! assigning per-object projection and retained refresh self time to
//! `layout_fragment` in sampling profiles.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:735-830
#[inline(never)]
pub(super) fn ExportPaintProperties(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const NativeComputedStyle,
    resources: &Arc<PaintResources>,
) -> PaintProperties {
    ExportProperties(
        object,
        sources,
        style_sources,
        effective_style,
        resources,
        LayoutUpdateKind::Paint,
        None,
        None,
    )
}

fn ExportPrePaintInvalidation(
    object: *const LayoutObject,
) -> layout_fragment::fragment_tree::PrePaintInvalidation {
    if object.is_null() {
        return Default::default();
    }
    let object = unsafe { &*object };
    layout_fragment::fragment_tree::PrePaintInvalidation {
        needs_paint_property_update: object.NeedsPaintPropertyUpdate(),
        descendant_needs_paint_property_update: object.DescendantNeedsPaintPropertyUpdate(),
        should_check_for_paint_invalidation: object.ShouldCheckForPaintInvalidation(),
        should_check_layout_for_paint_invalidation: object.ShouldCheckLayoutForPaintInvalidation(),
        descendant_should_check_layout_for_paint_invalidation: object
            .DescendantShouldCheckLayoutForPaintInvalidation(),
        subtree_should_check_for_paint_invalidation: object
            .SubtreeShouldCheckForPaintInvalidation(),
    }
}

#[inline(never)]
pub(super) fn ExportProperties(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const NativeComputedStyle,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    mut exported_styles: Option<&mut ExportedStyleCache>,
    mut debug_names: Option<&mut DebugNameCache>,
) -> PaintProperties {
    let mut output = PaintProperties::default();
    output.resources = Some(Arc::clone(resources));
    if object.is_null() {
        return output;
    }
    output.svg_hidden_container = unsafe { &*object }.IsSVGHiddenContainer();
    output.is_layout_view = unsafe { &*object }.IsLayoutView();
    if purpose == LayoutUpdateKind::Paint {
        let object_ref = unsafe { &*object };
        output.pre_paint_invalidation = ExportPrePaintInvalidation(object);
        // Blink's PrePaintTreeWalk runs ObjectPaintInvalidator before paint
        // records are produced.  The boundary owns that transition here: a
        // LayoutObject's pending full-invalidation reason must invalidate its
        // DisplayItemClient before the immutable fragment snapshot captures
        // the client state.
        // cpp: core/paint/object_paint_invalidator.cc:56-121
        // cpp: core/layout/layout_object.cc:2514-2520
        let client = unsafe { &*(object as *const DisplayItemClient) };
        if object_ref.ShouldDoFullPaintInvalidation() {
            // LayoutObject's default virtual implementation invalidates this
            // base DisplayItemClient. Derived auxiliary clients are exported
            // separately by their owning paint data.
            client.Invalidate(object_ref.PaintInvalidationReasonForPrePaint());
        }
        // LayoutObject's base-first representation preserves the native
        // DisplayItemClient subobject, including anonymous layout clients.
        // cpp: core/layout/layout_object.h:280
        // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
        // Source: paint_property_tree_builder.cc:464-470. Allocate on the
        // actual FragmentData owner before publishing immutable Paint input.
        unsafe { &mut *object.cast_mut() }
            .GetMutableForPainting()
            .EnsureId();
        output.property_fragment_unique_id = unsafe { &*object }.UniqueId();
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
            .map(|entry| unsafe { &*entry.node }.InputStyle().paint.image_resource_id)
            .unwrap_or(None);
        output.style = if purpose == LayoutUpdateKind::Paint {
            crate::internal::boundary::native_paint_style::ExportStyle(
                parent_style,
                parent_image_resource_id,
            )
        } else {
            crate::internal::boundary::native_paint_style::ExportGeometryStyle(parent_style)
        };
        if purpose == LayoutUpdateKind::Paint {
            output.style.applied_text_decorations = ExportAppliedTextDecorations(parent_style);
        }
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
                primary_font.GetFontMetrics().Ascent() as f32
                    - primary_font.InternalLeading() / 2.0,
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
        if !layout_box.is_null() {
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
                    value => panic!("invalid native background paint location {value}"),
                };
            // Preserve the LayoutBox virtual override rather than calling only
            // the DisplayItemClient base default (layout_box.cc:4099-4108).
            output.display_item_raster_effect_outset =
                unsafe { &*layout_box }.VisualRectOutsetForRasterEffects();
            if unsafe { &*layout_box }.HasCSSClip() {
                output.css_clip = Some(ExportNativeInkRect(unsafe { &*layout_box }.CSSClipRect()));
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
        // A native object with no PaintLayer is a known result, not permission for
        // Paint to reconstruct a layer from flattened style (notably SVG children).
        output.paint_layer_state_is_native = true;
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
                output.paint_layer_is_self_painting = layer.IsSelfPaintingLayer();
                // These decisions belong to native Layout/PaintLayer setup. In
                // particular, style adjustment makes static flex/grid items with
                // z-index stacking contexts, and containment/perspective can form
                // contexts without an equivalent field in the public projection.
                let layout_object = unsafe { &*object };
                output.paint_layer_is_stacking_context = layout_object.IsStackingContext();
                output.paint_layer_stacking_level = layout_object.StyleRef().EffectiveZIndex();
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
    let mut paint_style = source.InputStyle();
    if !effective_style.is_null() {
        if let Some(&effective_source) = style_sources.get(&effective_style) {
            paint_style = unsafe { &*effective_source };
        }
    }
    let projected_style_identity = paint_style as *const ComputedStyle as usize;
    let zoomed_paint_style = crate::internal::css_zoom::ZoomedStyle(paint_style);
    let paint_style = zoomed_paint_style.as_ref();
    output.has_source = true;
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
    output.effective_zoom = paint_style
        .extended
        .as_ref()
        .map_or(1.0, |e| e.effective_zoom);
    let native_paint_style = if effective_style.is_null() {
        unsafe { &*object }.StyleRef()
    } else {
        unsafe { &*effective_style }
    };
    let exported_style_key = (
        native_paint_style as *const NativeComputedStyle as usize,
        projected_style_identity,
    );
    let cached_style = exported_styles
        .as_deref()
        .and_then(|styles| styles.get(&exported_style_key))
        .cloned();
    let used_cached_style = cached_style.is_some();
    if purpose == LayoutUpdateKind::Layout {
        output.style =
            crate::internal::boundary::native_paint_style::ExportGeometryStyle(native_paint_style);
    } else if let Some(style) = cached_style {
        output.style = style;
    } else {
        output.style = crate::internal::boundary::native_paint_style::ExportStyleWithProjectedSvg(
            native_paint_style,
            paint_style.paint.image_resource_id,
            &paint_style.paint,
            &paint_style.svg_clip_resources,
        );
        if let Some(target) = output
            .style
            .clip_path
            .as_ref()
            .and_then(|clip| clip.reference_target_id)
        {
            // Reuse the native SVG viewport query at the existing Layout boundary.
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
                // Connected ID targets without an attached native resource are
                // unavailable (for example a display:none ancestor).
                output.style.clip_path = None;
            }
        }
        for layer in &mut output.style.mask_images {
            let Some(reference) = layer.svg_reference.as_mut() else {
                continue;
            };
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
                layer.image_state = crate::internal::paint_input::PaintMaskImageState::kInvalid;
            }
        }
        output.style.applied_text_decorations = ExportAppliedTextDecorations(native_paint_style);
        // Layout-dependent transforms are resolved below per object. Every
        // other style payload is immutable and can be shared by all physical
        // and inline fragments that reference the same native/projected pair.
        if !native_paint_style.HasOffset() {
            if let Some(styles) = exported_styles.as_deref_mut() {
                styles.insert(exported_style_key, output.style.clone());
            }
        }
    }
    if native_paint_style.HasOffset() && !layout_box.is_null() {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            crate::internal::boundary::native_paint_style::SetResolvedLayoutTransform(
                &mut output.style,
                unsafe { &*box_model }.CurrentTransformForLayout(),
            );
        }
    }
    if purpose == LayoutUpdateKind::Paint && !used_cached_style && native_paint_style.HasOffset() {
        output.style.applied_text_decorations = ExportAppliedTextDecorations(native_paint_style);
    }
    output.margin = paint_style.margin;
    output.border = paint_style.border;
    output.border_styles = paint_style.border_styles;
    output.padding = paint_style.padding;
    output.display = paint_style.display;
    output.position = paint_style.position;
    output.fixed_to_view =
        !layout_box.is_null() && unsafe { &*layout_box }.IsFixedToView(std::ptr::null());
    output.floating = paint_style.floating;
    ExportStickyPaintProperties(object, &mut output);
    output.writing_mode = paint_style.writing_mode;
    output.direction = paint_style.direction;
    if let Some(extended) = &paint_style.extended {
        output.overflow_x = extended.overflow_x;
        output.overflow_y = extended.overflow_y;
    }
    let export_overscroll = |behavior| match behavior {
        EOverscrollBehavior::kAuto => crate::internal::layout_input::OverscrollBehavior::kAuto,
        EOverscrollBehavior::kContain => {
            crate::internal::layout_input::OverscrollBehavior::kContain
        }
        EOverscrollBehavior::kNone => crate::internal::layout_input::OverscrollBehavior::kNone,
        EOverscrollBehavior::kChain => crate::internal::layout_input::OverscrollBehavior::kChain,
    };
    output.overscroll_behavior_x = export_overscroll(native_paint_style.OverscrollBehaviorX());
    output.overscroll_behavior_y = export_overscroll(native_paint_style.OverscrollBehaviorY());
    // Source: core/paint/paint_property_tree_builder.cc:3634-3635.
    output.prevent_scroll_axis_locking =
        native_paint_style.ScrollAxisLock() == foundation::EScrollAxisLock::kNone;
    let element_data = if source.IsElementNode() {
        unsafe { &*(source as *const Node as *const Element) }
            .InputElementData()
            .as_ref()
    } else {
        None
    };
    if let Some(element_data) = element_data {
        output.scroll_offset = element_data.scroll_offset;
        if let Some(svg_shape) = &element_data.svg_shape {
            output.svg_shape = Some(Arc::new(svg_shape.clone()));
        }
        output.svg_view_box = element_data.svg_view_box;
    }
    if !object.is_null() && unsafe { &*object }.IsSVG() {
        let algorithms = crate::internal::layout_pass_scope::LayoutPassScope::Algorithms();
        if let Some(query) =
            unsafe { algorithms.as_ref() }.and_then(|a| a.svg_support.paint_geometry)
        {
            (output.svg_local_transform, output.svg_viewport) = query(unsafe { &*object });
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
    if element_data.is_some_and(|data| data.text_control_inner_editor) {
        output.text_control_inner_editor = true;
        let style = unsafe { &*object }.StyleRef();
        let font = style.GetFontHeightForDefaultBaseline();
        let font_height = Number(font.LineHeight());
        let line_height = style.ComputedLineHeight() as f64;
        let editor = unsafe { &*layout_box };
        let size = editor.StitchedSize();
        let border = output.border;
        let padding = output.padding;
        let left = border.left + padding.left;
        let right = (Number(size.width) - border.right - padding.right - 1.0).max(left);
        let align = paint_style
            .extended
            .as_ref()
            .map_or(crate::internal::layout_input::TextAlign::kStart, |e| {
                e.text_align
            });
        use crate::internal::layout_input::TextAlign;
        let rtl = output.direction == TextDirection::kRtl;
        let x = match align {
            TextAlign::kCenter | TextAlign::kWebkitCenter => (left + right) * 0.5,
            TextAlign::kRight | TextAlign::kWebkitRight => right,
            TextAlign::kEnd if !rtl => right,
            TextAlign::kStart | TextAlign::kJustify if rtl => right,
            _ => left,
        };
        output.text_control_empty_caret = Some(FragmentPaintRect {
            offset: Offset {
                x,
                y: border.top + padding.top + (line_height - font_height) * 0.5,
            },
            size: Size {
                width: 1.0,
                height: font_height,
            },
        });
    }
    let appearance = paint_style
        .extended
        .as_ref()
        .map_or(AppearanceValue::kNone, |extended| {
            extended.effective_appearance
        });
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
                && native_paint_style.HasOutline()
                && native_paint_style.OutlineStyleIsAuto();
            control.autofilled = element_data.autofill_state != AutofillState::kNotFilled;
            control.value_ratio = element_data
                .control_value_ratio
                .or(element_data.range_value_ratio);
        }
        if let Some(accent_color) = paint_style.paint.accent_color {
            control.accent_color = accent_color;
        }
        output.form_control = Some(Arc::new(control));
    }
    // Blink can resolve an insertion position before focus style has changed.
    // Keep text-control editing geometry on every native fragment owner; appearance
    // and focus only decide whether FormControlPainter has anything to draw.
    if source.InputKind() == NodeKind::kFormControl {
        let style = unsafe { &*object }.StyleRef();
        let height = style.GetFontHeightForDefaultBaseline();
        output.text_control_caret_metrics = Some(crate::fragment_tree::TextControlCaretMetrics {
            font_height: Number(height.LineHeight()),
            ascent: Number(height.ascent),
            line_height: style.ComputedLineHeight() as f64,
        });
    }
    output
}
