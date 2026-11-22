//! Targeted retained-paint input validation and source update.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! The engine retains its translated entry points while the complete paint
//! admission and mutation paths receive fragment-qualified sampling symbols.

use super::*;
use crate::internal::layout_object::MarkingBehavior;

/// Resource completion does not change a CSS image's URL, so
/// FieldInvalidationDiff can legitimately report no computed-style field
/// change while the immutable StyleFetchedImage binding changes from the
/// unpublished id to a decoded PaintImage id. Keep that resource transition
/// in the retained-paint invalidation proof instead of reusing the drawing
/// recorded while the image was absent.
fn StyleImageResourceBindings(style: &StyleComputedStyle) -> Vec<(String, u64)> {
    use layout_style::style::{
        content_data::ImageContentData, fill_layer::FillLayer, style_fetched_image::FetchedImage,
        style_image::StyleImage,
    };

    fn append(image: *mut StyleImage, output: &mut Vec<(String, u64)>) {
        let Some(image) = (unsafe { image.as_ref() }) else {
            return;
        };
        let image = layout_style::style::style_mask_source_image::MaskSourceImage(image)
            .and_then(|mask| mask.FetchedImage())
            .unwrap_or(image);
        if let Some(image) = FetchedImage(image) {
            output.push((image.SourceURL().to_owned(), image.ResourceId()));
        }
    }
    fn layers(mut layer: *const FillLayer, output: &mut Vec<(String, u64)>) {
        while let Some(current) = unsafe { layer.as_ref() } {
            append(current.GetImage(), output);
            layer = current.Next();
        }
    }

    let mut output = Vec::new();
    layers(style.BackgroundLayers(), &mut output);
    layers(style.MaskLayers(), &mut output);
    append(style.BorderImageSource(), &mut output);
    append(style.MaskBoxImageSource(), &mut output);
    if let Some(reflection) = unsafe { style.BoxReflect().as_ref() } {
        append(reflection.Mask().GetImage(), &mut output);
    }
    append(style.ListStyleImage().Get(), &mut output);
    if let Some(shape) = unsafe { style.ShapeOutside().as_ref() } {
        append(shape.GetImage(), &mut output);
    }
    if let Some(cursors) = unsafe { style.Cursors().as_ref() } {
        for cursor in cursors.iter() {
            append(cursor.GetImage(), &mut output);
        }
    }
    let mut content = style.GetContentData();
    while let Some(pointer) = content {
        let data = unsafe { &*pointer };
        if data.IsImage() {
            append(
                unsafe { &*(pointer as *const ImageContentData) }.GetImage(),
                &mut output,
            );
        }
        content = data.Next();
    }
    output
}

fn SvgClipResourceBindingChangedForStyles(
    old_style: &StyleComputedStyle,
    new_style: &StyleComputedStyle,
    old: &crate::internal::paint_input::SvgClipResourceBinding,
    new: &crate::internal::paint_input::SvgClipResourceBinding,
) -> bool {
    old != new
        && (StyleReferencesSvgResource(old_style) || StyleReferencesSvgResource(new_style))
}

fn StyleResourceBindingsChangedForStyles(
    old_style: &StyleComputedStyle,
    new_style: &StyleComputedStyle,
    old: &crate::internal::layout_input::StyleResourceData,
    new: &crate::internal::layout_input::StyleResourceData,
) -> bool {
    old.image_resource_id != new.image_resource_id
        || old.svg_filter_graphs != new.svg_filter_graphs
        || SvgClipResourceBindingChangedForStyles(
            old_style,
            new_style,
            &old.svg_clip_resources,
            &new.svg_clip_resources,
        )
        || old.svg_mask_resource != new.svg_mask_resource
        || old.svg_pattern_content_target != new.svg_pattern_content_target
        || old.svg_fill_server != new.svg_fill_server
        || old.svg_stroke_server != new.svg_stroke_server
        || old.svg_markers != new.svg_markers
        || old.replaced_image_shader != new.replaced_image_shader
}

pub(super) fn StyleImageResourceBindingsChanged(
    old: &StyleComputedStyle,
    new: &StyleComputedStyle,
) -> bool {
    StyleImageResourceBindings(old) != StyleImageResourceBindings(new)
}

#[inline(never)]
pub(super) fn update_paint_style(
    tree: &mut LayoutObjectTree,
    id: u64,
    style: &StyleComputedStyle,
    clip_resource_version: crate::internal::paint_input::SvgClipResourceVersion,
) -> Option<PaintStyleBinding> {
    let storage = unsafe { &mut *tree.storage_ };
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    assert!(!storage.updating);
    // Validate before mutating source pointers. Resource-only changes use the
    // ordinary tree/export lifecycle to refresh every client.
    if storage
        .InputKeys(id)
        .iter()
        .map(|key| &storage.managed[key])
        .any(|record| {
            record.input.style.svg_clip_resources.snapshot.version != clip_resource_version
                && unsafe { record.effective_computed_style.as_ref() }
                    .is_none_or(StyleReferencesSvgResource)
        })
    {
        return None;
    }
    let style = style as *const StyleComputedStyle;
    let mut binding = None;
    for key in storage.InputKeys(id).to_vec() {
        let record = storage.managed.get_mut(&key).unwrap();
        if record.input.style_generated || record.input.kind == NodeKind::kText {
            continue;
        }
        let current = PaintStyleBinding {
            image_resource_id: record.input.style.paint.image_resource_id,
            svg_clip_resources: record.input.style.svg_clip_resources.clone(),
        };
        if let Some(previous) = binding.as_ref() {
            debug_assert_eq!(previous, &current);
        } else {
            binding = Some(current);
        }
        Arc::make_mut(&mut record.input.style).style = style;
        Arc::make_mut(&mut record.effective_style).style = style;
        record.input.computed_style = style;
        record.effective_computed_style = style;
        record.input_style_roots = RetainInputStyles(&record.input, style);
        let node = unsafe { &mut *record.node };
        node.SetComputedStyle(style);
        unsafe { &mut *record.object }.SetStyle(style);
    }
    binding
}

#[inline(never)]
pub(super) fn paint_input_plan(
    tree: &LayoutObjectTree,
    id: u64,
    style: &StyleComputedStyle,
    resources: &crate::style_projection::StyleResourceBindings<'_>,
    element: &Option<ElementData>,
) -> Option<(std::collections::HashSet<u64>, Vec<u64>)> {
    use layout_style::style::computed_style_base::{ComputedStyleBase, FieldDifference as F};
    let mut trace = layout_fragment::trace::span(
        layout_fragment::trace::TracePhase::ValidatePaintInput,
    );
    trace.set("node_id", id as f64);
    let storage = unsafe { &*tree.storage_ };
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    assert!(!storage.updating);
    let records = storage
        .InputKeys(id)
        .iter()
        .map(|key| &storage.managed[key])
        .collect::<Vec<_>>();
    if records.is_empty() {
        // Hidden, inactive flat-tree and collapsible text sources can retain
        // computed style without any attached layout object.
        trace.set("no_layout_object", 1.0);
        return Some((Default::default(), Vec::new()));
    }
    // Source: core/dom/text.cc Text::RecalcTextStyle. A resident LayoutText is
    // restyled from its style parent even when StyleForText returned a distinct
    // inherited-equal ComputedStyle allocation. Use that topology, not address
    // identity between the old snapshots.
    let parents = records.iter().map(|r| r.object).collect::<Vec<_>>();
    let mut ids = std::collections::HashSet::from([id]);
    for &parent in &parents {
        for key in storage.ChildKeys(parent) {
            let child = &storage.managed[key];
            if child.input.kind == NodeKind::kText {
                ids.insert(child.input.id);
            }
        }
    }
    drop(records);
    let records = ids
        .iter()
        .flat_map(|&id| storage.InputKeys(id))
        .map(|key| &storage.managed[key])
        .collect::<Vec<_>>();
    let mut inline_ink_changes = Vec::new();
    for record in &records {
        let object = unsafe { &*record.object };
        let old = unsafe { record.effective_computed_style.as_ref() };
        let Some(old) = old else {
            trace.set("missing_canonical_style", 1.0);
            trace.set("source_kind", record.input.kind as u8 as f64);
            return None;
        };
        let new = style;
        let fields = ComputedStyleBase::FieldInvalidationDiff(old, new);
        let inline = DynamicTo::<LayoutInline>(record.object);
        let inline_topology_changed = !inline.is_null()
            && crate::internal::boundary::layout_boundary::ComputedStyleRequiresPaintBackedInlineFragment(old)
                != crate::internal::boundary::layout_boundary::ComputedStyleRequiresPaintBackedInlineFragment(new);
        trace.set(
            "inline_topology_changed",
            inline_topology_changed as u8 as f64,
        );
        if inline_topology_changed {
            return None;
        }
        // Source: ComputedStyle::VisualInvalidationDiff (computed_style.cc
        // transform branch) and LayoutBox::StyleDidChange. A transform value
        // change is a paint-property update: it never requests layout unless
        // TransformsChangeMayRequireLayout(), which SetStyle marks on the
        // source object and RefreshTargetedPaintInputs then rejects. Toggling
        // HasTransform() changes PaintLayer requirements and the stacking
        // context, which Chromium answers with a full layout; keep those, and
        // transform-origin/box/style/perspective changes, on the layout path.
        let transform_fields =
            (F::kHasTransform | F::kTransformData | F::kTransformProperty).bits();
        let transform_topology_changed = fields & transform_fields != 0
            && (old.HasTransform() != new.HasTransform()
                || old.HasTransformRelatedProperty() != new.HasTransformRelatedProperty()
                || old.IsStackingContextWithoutContainment()
                    != new.IsStackingContextWithoutContainment());
        let geometry = (F::kLayout
            | F::kReshape
            | F::kTransformOther
            | F::kFilterData
            | F::kMask
            | F::kStroke
            | F::kBorderShape)
            .bits()
            | if transform_topology_changed {
                transform_fields
            } else {
                0
            };
        trace.set("field_difference", fields as f64);
        trace.set("needs_layout", object.NeedsLayout() as u8 as f64);
        trace.set(
            "first_line",
            unsafe { &*record.node }.InputHasFirstLineStyles() as u8 as f64,
        );
        let layout_equivalent = fields & geometry == 0;
        trace.set("layout_equivalent", layout_equivalent as u8 as f64);
        let text_ink = old.DiffNeedsRecomputeVisualOverflow(new, fields)
            && fields & F::kTextDecoration.bits() != 0;
        let other_ink =
            old.DiffNeedsRecomputeVisualOverflow(new, fields & !F::kTextDecoration.bits());
        let style_generated_rejects = record.input.style_generated
            && !(record.input.kind == NodeKind::kText && record.input.id != id);
        let input_has_first_line = record.input.first_line_style.is_some();
        let node_has_first_line = unsafe { &*record.node }.InputHasFirstLineStyles();
        let resource_version_changed = SvgClipResourceBindingChangedForStyles(
            old,
            new,
            &record.input.resources.svg_clip_resources,
            resources.svg_clip_resources,
        );
        // The trace recorder deliberately bounds per-span fields. Keep every
        // admission cause in one stable bitset so a rejected fast path remains
        // diagnosable without relinking for each individual predicate.
        let rejection_bits = u16::from(fields & geometry != 0)
            | (u16::from(other_ink) << 1)
            | (u16::from(object.NeedsLayout()) << 2)
            | (u16::from(object.IsSVG()) << 3)
            | (u16::from(style_generated_rejects) << 4)
            | (u16::from(input_has_first_line) << 5)
            | (u16::from(node_has_first_line) << 6)
            | (u16::from(!layout_equivalent) << 7)
            | (u16::from(resource_version_changed) << 8);
        trace.set("rejection_bits", rejection_bits as f64);
        if rejection_bits != 0 {
            return None;
        }
        let candidate = if record.input.kind == NodeKind::kText {
            &None
        } else {
            element
        };
        let metadata_equal = GeometryMetadataEqual(candidate, &record.input.element);
        let natural_size_change_is_paint_only = LayoutImageNaturalSizeChangeIsPaintOnly(
                record.object,
                candidate,
                &record.input.element,
                style,
            );
        trace.set("metadata_equal", metadata_equal as u8 as f64);
        trace.set(
            "natural_size_paint_only",
            natural_size_change_is_paint_only as u8 as f64,
        );
        if !metadata_equal && !natural_size_change_is_paint_only {
            trace.set("metadata_requires_export", 1.0);
            return None;
        }
        if text_ink {
            inline_ink_changes.push(record.input.id);
        }
    }
    drop(records);
    trace.set("admitted", 1.0);
    Some((ids, inline_ink_changes))
}

#[inline(never)]
pub(super) fn update_paint_input(
    tree: &mut LayoutObjectTree,
    id: u64,
    style: &StyleComputedStyle,
    resources: &crate::style_projection::StyleResourceBindings<'_>,
    element: Option<ElementData>,
) -> Option<PaintInputUpdate> {
    let (ids, inline_ink_changes) = paint_input_plan(tree, id, style, resources, &element)?;
    let storage = unsafe { &mut *tree.storage_ };
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    let keys = ids
        .iter()
        .flat_map(|&id| storage.InputKeys(id).iter().cloned())
        .collect::<Vec<_>>();
    for key in keys {
        let record = storage.managed.get_mut(&key).unwrap();
        let previous = unsafe { &*record.object }.StyleRef();
        let control_paint_state_changed = ControlPaintStateChanged(&element, &record.input.element);
        let image_paint_state_changed = ImagePaintStateChanged(&element, &record.input.element);
        let element_capture_state_changed =
            ElementCaptureStateChanged(&element, &record.input.element);
        let natural_size_change_is_paint_only = LayoutImageNaturalSizeChangeIsPaintOnly(
            record.object,
            &element,
            &record.input.element,
            style,
        );
        let layout_style = PreservePaintOnlyFontBinding(style, previous);
        let image_resource_id = record.input.resources.image_resource_id;
        let new_resources = resources.ToData(image_resource_id);
        let fields =
            layout_style::style::computed_style_base::ComputedStyleBase::FieldInvalidationDiff(
                previous,
                unsafe { &*layout_style },
            );
        let image_resource_binding_changed =
            StyleImageResourceBindingsChanged(previous, unsafe { &*layout_style });
        if natural_size_change_is_paint_only {
            let object = unsafe { &mut *record.object };
            object.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
            object.SetNeedsPaintPropertyUpdate();
            object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        } else if image_paint_state_changed || image_resource_binding_changed {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        } else if element_capture_state_changed {
            // Region-capture and tracked-element metadata both require a full
            // paint invalidation so their new chunk payload is republished.
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kStyle,
            );
        } else if fields != 0
            || StyleResourceBindingsChangedForStyles(
                previous,
                unsafe { &*layout_style },
                record.input.resources.as_ref(),
                &new_resources,
            )
            || control_paint_state_changed
        {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kStyle,
            );
        }
        record.input.resources = Arc::new(new_resources);
        record.input.computed_style = style;
        {
            let compatibility = Arc::make_mut(&mut record.input.style);
            compatibility.style = style;
            resources.ApplyTo(compatibility);
            compatibility.paint.image_resource_id = image_resource_id;
        }
        record.input.element = if record.input.kind == NodeKind::kText {
            None
        } else {
            element.clone()
        };
        {
            let compatibility = Arc::make_mut(&mut record.effective_style);
            compatibility.style = style;
            resources.ApplyTo(compatibility);
            compatibility.paint.image_resource_id = image_resource_id;
        }
        record.effective_computed_style = style;
        record.input_style_roots =
            RetainInputStyles(&record.input, record.effective_computed_style);
        let node = unsafe { &mut *record.node };
        let sealed = node.InputTreeIsSealed();
        node.UpdateInputData(&record.input);
        if sealed {
            node.seal_input_tree();
        }
        node.ClearInputFontBinding();
        node.SetComputedStyle(layout_style);
        unsafe { &mut *record.object }.SetStyle(layout_style);
    }
    Some(PaintInputUpdate {
        sources: ids.into_iter().collect(),
        inline_ink_changes,
    })
}
