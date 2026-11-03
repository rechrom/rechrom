//! Targeted retained-paint input validation and source update.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! The engine retains its translated entry points while the complete paint
//! admission and mutation paths receive fragment-qualified sampling symbols.

use super::*;
use crate::internal::layout_object::MarkingBehavior;

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
    use layoutng_style::style::computed_style_base::{ComputedStyleBase, FieldDifference as F};
    let mut trace = layout_fragment_boundary::trace::span(
        layout_fragment_boundary::trace::TracePhase::ValidatePaintInput,
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
        if !inline.is_null()
            && crate::internal::boundary::layout_boundary::ComputedStyleRequiresPaintBackedInlineFragment(old)
                != crate::internal::boundary::layout_boundary::ComputedStyleRequiresPaintBackedInlineFragment(new)
        {
            return None;
        }
        let geometry = (F::kLayout
            | F::kReshape
            | F::kHasTransform
            | F::kTransformData
            | F::kTransformOther
            | F::kFilterData
            | F::kMask
            | F::kStroke
            | F::kBorderShape)
            .bits();
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
        if fields & geometry != 0
            || other_ink
            || object.NeedsLayout()
            || object.IsSVG()
            || (record.input.style_generated
                && !(record.input.kind == NodeKind::kText && record.input.id != id))
            || record.input.first_line_style.is_some()
            || unsafe { &*record.node }.InputHasFirstLineStyles()
            || !layout_equivalent
            || record.input.resources.svg_clip_resources.snapshot.version
                != resources.svg_clip_resources.snapshot.version
        {
            return None;
        }
        let candidate = if record.input.kind == NodeKind::kText {
            &None
        } else {
            element
        };
        if !GeometryMetadataEqual(candidate, &record.input.element)
            && !LayoutImageNaturalSizeChangeIsPaintOnly(
                record.object,
                candidate,
                &record.input.element,
                style,
            )
        {
            trace.set("metadata_requires_export", 1.0);
            return None;
        }
        if text_ink {
            inline_ink_changes.push(record.input.id);
        }
    }
    drop(records);
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
            layoutng_style::style::computed_style_base::ComputedStyleBase::FieldInvalidationDiff(
                previous,
                unsafe { &*layout_style },
            );
        if natural_size_change_is_paint_only {
            let object = unsafe { &mut *record.object };
            object.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
            object.SetNeedsPaintPropertyUpdate();
            object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        } else if image_paint_state_changed {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        } else if fields != 0
            || record.input.resources.as_ref() != &new_resources
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
