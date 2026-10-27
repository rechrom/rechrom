//! Targeted retained-paint input validation and native update.
//!
//! Included by `layout_object_builder` at the reciprocal assembly boundary.
//! The engine retains its translated entry points while the complete paint
//! admission and mutation paths receive fragment-qualified sampling symbols.

use super::*;

#[inline(never)]
pub(super) fn update_paint_style(
    tree: &mut LayoutObjectTree,
    id: u64,
    style: &NativeComputedStyle,
    clip_resource_version: crate::internal::paint_input::SvgClipResourceVersion,
) -> Option<PaintStyleBinding> {
    let storage = unsafe { &mut *tree.storage_ };
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    assert!(!storage.updating);
    // Validate before mutating native pointers. Resource-only changes use the
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
    let native_style = style as *const NativeComputedStyle;
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
        Arc::make_mut(&mut record.input.style).native_style = native_style;
        Arc::make_mut(&mut record.effective_style).native_style = native_style;
        record.input_style_roots = RetainInputStyles(&record.input, &record.effective_style);
        let node = unsafe { &mut *record.node };
        node.SetComputedStyle(native_style);
        unsafe { &mut *record.object }.SetStyle(native_style);
    }
    binding
}

#[inline(never)]
pub(super) fn paint_input_plan(
    tree: &LayoutObjectTree,
    id: u64,
    style: &ComputedStyle,
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
    // Propagate the source to inherited text owned by the same native style.
    let parents = records
        .iter()
        .map(|r| (r.object, r.effective_style.native_style))
        .collect::<Vec<_>>();
    let mut ids = std::collections::HashSet::from([id]);
    for &(parent, native) in &parents {
        for key in storage.ChildKeys(parent) {
            let child = &storage.managed[key];
            if child.input.kind == NodeKind::kText
                && !native.is_null()
                && child.effective_style.native_style == native
            {
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
        let old = unsafe { record.effective_style.native_style.as_ref() };
        let new = unsafe { style.native_style.as_ref() };
        let (Some(old), Some(new)) = (old, new) else {
            trace.set("missing_canonical_style", 1.0);
            trace.set("source_kind", record.input.kind as u8 as f64);
            return None;
        };
        let fields = ComputedStyleBase::FieldInvalidationDiff(old, new);
        let inline = DynamicTo::<LayoutInline>(record.object);
        if !inline.is_null()
            && crate::internal::boundary::layout_boundary::NativeStyleRequiresPaintBackedInlineFragment(old)
                != crate::internal::boundary::layout_boundary::NativeStyleRequiresPaintBackedInlineFragment(new)
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
        trace.set("native_needs_layout", object.NeedsLayout() as u8 as f64);
        trace.set(
            "first_line",
            unsafe { &*record.node }.InputHasFirstLineStyles() as u8 as f64,
        );
        let mut input = style.clone();
        if record.input.kind == NodeKind::kText && record.input.id != id {
            input.svg_clip_resources = record.effective_style.svg_clip_resources.clone();
        }
        let layout_equivalent = record.effective_style.LayoutEquivalent(&input);
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
            || record.effective_style.svg_clip_resources.snapshot.version
                != input.svg_clip_resources.snapshot.version
        {
            return None;
        }
        let candidate = if record.input.kind == NodeKind::kText {
            &None
        } else {
            element
        };
        if !GeometryMetadataEqual(candidate, &record.input.element) {
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
    style: &ComputedStyle,
    element: Option<ElementData>,
) -> Option<PaintInputUpdate> {
    let (ids, inline_ink_changes) = paint_input_plan(tree, id, style, &element)?;
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
        let native_style = PreservePaintOnlyFontBinding(style.native_style, previous);
        let mut projected = style.clone();
        projected.svg_clip_resources = record.effective_style.svg_clip_resources.clone();
        projected.paint.svg_filter_graphs = record.effective_style.paint.svg_filter_graphs.clone();
        let native_fields =
            layoutng_style::style::computed_style_base::ComputedStyleBase::FieldInvalidationDiff(
                previous,
                unsafe { &*native_style },
            );
        if native_fields != 0
            || record.effective_style.paint != projected.paint
            || control_paint_state_changed
        {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kStyle,
            );
        }
        let projected = Arc::new(projected);
        if !record.input.style.native_style.is_null() {
            record.input.style = Arc::clone(&projected);
        }
        record.input.element = if record.input.kind == NodeKind::kText {
            None
        } else {
            element.clone()
        };
        record.effective_style = projected;
        record.input_style_roots = RetainInputStyles(&record.input, &record.effective_style);
        let node = unsafe { &mut *record.node };
        let sealed = node.InputTreeIsSealed();
        node.UpdateInputData(&record.input);
        if sealed {
            node.seal_input_tree();
        }
        node.ClearInputFontBinding();
        node.SetComputedStyle(native_style);
        unsafe { &mut *record.object }.SetStyle(native_style);
    }
    Some(PaintInputUpdate {
        sources: ids.into_iter().collect(),
        inline_ink_changes,
    })
}
