//! Targeted source node-input update transaction.
//!
//! The public LayoutObjectTree entry remains in the translated core API, while
//! validation, inherited-text expansion, and mutation are owned here so the
//! existing `layout_invalidation.NodeInput` interval and sampling stack agree.

use super::*;
use layout_style::style::computed_style_base::ComputedStyleBase;

/// One admitted restyle of a retained source LayoutObject.
enum NodeInputRestyle {
    /// cpp: core/dom/element.cc Element::RecalcOwnStyle and
    /// core/dom/text.cc Text::RecalcTextStyle -> LayoutObject::SetStyle
    /// (core/layout/layout_object.cc:2991-3030). The same LayoutObject, Node
    /// and attachment take the new ComputedStyle in place; `field_diff` is the
    /// one `FieldInvalidationDiff` of this style replacement and drives the
    /// layout, font-retention and paint decisions.
    InPlace {
        style: Arc<ComputedStyle>,
        resources: Arc<crate::internal::layout_input::StyleResourceData>,
        field_diff: u64,
    },
    /// Element metadata or attachment state changed together with the style:
    /// the general retained-object update (`create_managed_object`) owns it.
    Update {
        parent: *mut LayoutObject,
        input: NodeConstructionData,
    },
}

/// Whether a retained record can take a restyle through
/// `restyle_in_place`. The caller has proved its element metadata is
/// unchanged; image/video objects keep the general path because
/// ImageLoader/HTMLMediaElement state is refreshed there.
#[inline(always)]
fn in_place_restyle_admitted(
    storage: &LayoutObjectTreeStorage,
    record: &ManagedObject,
    new_style: *const StyleComputedStyle,
) -> bool {
    let object = unsafe { &*record.object };
    !new_style.is_null()
        && !record.effective_computed_style.is_null()
        && !record.input.style_generated
        && record.input.first_line_style.is_none()
        && !storage.reattach.contains(&record.input.id)
        && !object.WasDestroyedForTreeUpdate()
        && !object.IsLayoutImage()
        && !object.IsVideo()
}

#[inline(never)]
pub(super) fn update_node_input(
    tree: &mut LayoutObjectTree,
    id: u64,
    style: ComputedStyle,
    element: Option<ElementData>,
) -> Option<NodeInputUpdate> {
    let _heap = LayoutHeapScope::new();
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let keys = storage.InputKeys(id).to_vec();
    if keys.is_empty() {
        return None;
    }
    // Sub-phase attribution of this targeted SetStyle transaction: input
    // admission (validation, no mutation), then in-place application.
    let admission_trace =
        browser_tracing::span("layout", "layout_invalidation.NodeInput.Admission");
    // cpp: core/layout/layout_object.cc:2991-2995 (LayoutObject::SetStyle).
    // Element::RecalcOwnStyle hands every LayoutObject of the element the
    // same ComputedStyle, and Text::RecalcTextStyle gives its LayoutText the
    // parent's style; neither copies it. The boundary's envelope becomes this
    // restyle's one owner (moved, not cloned) and its resource projection is
    // built once and shared with every retained input.
    let new_style: *const StyleComputedStyle = style.style;
    let shared_style = Arc::new(style);
    let shared_resources =
        Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(&shared_style));
    let mut inputs: Vec<(InputKey, NodeInputRestyle, NodeInputObjectUpdate)> = Vec::new();
    let mut relations_changed = false;
    for key in &keys {
        let record = &storage.managed[key];
        if record.input.style_generated
            || record.input.first_line_style.is_some()
            || unsafe { &*record.object }.IsSVG()
            || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
            || unsafe { &*record.node }.InputHasFirstLineStyles()
            || record
                .input
                .element
                .as_ref()
                .is_some_and(|e| e.form_control_type.is_some())
        {
            return None;
        }
        // Resource bindings remain with their owner. A resource completion may
        // refresh this exact input; SVG graph ownership remains resident.
        // Only a retained graph that differs from the restyle's needs an
        // envelope of its own.
        let retained_filter_graphs = &record.effective_style.paint.svg_filter_graphs;
        let shared_filter_graphs = &shared_style.paint.svg_filter_graphs;
        let (record_style, record_resources) = if Arc::ptr_eq(
            retained_filter_graphs,
            shared_filter_graphs,
        ) || (retained_filter_graphs.is_empty() && shared_filter_graphs.is_empty())
        {
            (Arc::clone(&shared_style), Arc::clone(&shared_resources))
        } else {
            let mut own_style = (*shared_style).clone();
            own_style.paint.svg_filter_graphs = Arc::clone(retained_filter_graphs);
            let own_style = Arc::new(own_style);
            let own_resources = Arc::new(
                crate::internal::layout_input::StyleResourceData::FromLegacyInput(&own_style),
            );
            (own_style, own_resources)
        };
        let candidate: &Option<ElementData> = if record.input.kind == NodeKind::kText {
            &None
        } else {
            &element
        };
        let element_unchanged = record.input.element == *candidate;
        let restyle = if element_unchanged && in_place_restyle_admitted(storage, record, new_style)
        {
            // The retained input already carries this element metadata, so
            // it is the compatibility input; nothing is copied for it.
            if !LayoutObjectTreeStorage::StyleUpdateCompatible(
                record,
                record.parent,
                &record.input,
                &record_style,
            ) {
                return None;
            }
            let field_diff = ComputedStyleBase::FieldInvalidationDiff(
                unsafe { &*record.effective_computed_style },
                unsafe { &*new_style },
            );
            NodeInputRestyle::InPlace {
                style: record_style,
                resources: record_resources,
                field_diff,
            }
        } else {
            // The retained descriptor's scalar and text fields carry over; its
            // style envelope, resources and element data are replaced (first-
            // line inputs were declined above).
            let input = NodeConstructionData {
                id: record.input.id,
                kind: record.input.kind,
                style_generated: record.input.style_generated,
                computed_style: new_style,
                style: record_style,
                resources: record_resources,
                text: record.input.text.clone(),
                element: candidate.clone(),
                // CreateWithStyleOwner keeps the existing InputKey, whose name
                // is the retained debug name; Create reads the parameter only
                // to mint a new key, so copying the string here is dead work.
                debug_name: String::new(),
                first_line_computed_style: record.input.first_line_computed_style,
                first_line_style: None,
                first_line_resources: None,
            };
            if !LayoutObjectTreeStorage::StyleUpdateCompatible(
                record,
                record.parent,
                &input,
                &input.style,
            ) {
                return None;
            }
            NodeInputRestyle::Update {
                parent: record.parent,
                input,
            }
        };
        let image_resource_binding_changed = unsafe {
            match (
                record.effective_computed_style.as_ref(),
                new_style.as_ref(),
            ) {
                (Some(old), Some(new)) => {
                    layout_fragment_paint_input::StyleImageResourceBindingsChanged(old, new)
                }
                _ => false,
            }
        };
        if image_resource_binding_changed {
            unsafe { &mut *record.object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
                foundation::PaintInvalidationReason::kImage,
            );
        }
        relations_changed |= !element_unchanged
            || record
                .input
                .element
                .as_ref()
                .is_some_and(|data| data.document_role != DocumentRole::kNone)
            || candidate
                .as_ref()
                .is_some_and(|data| data.document_role != DocumentRole::kNone);
        inputs.push((
            key.clone(),
            restyle,
            NodeInputObjectUpdate {
                object: record.object,
            },
        ));
    }
    // Source: core/dom/text.cc Text::RecalcTextStyle. A resident LayoutText is
    // restyled from its style parent even when StyleForText returned a distinct
    // inherited-equal ComputedStyle allocation. Use tree ownership, not address
    // identity between the old snapshots.
    let parents = keys
        .iter()
        .map(|key| storage.managed[key].object)
        .collect::<Vec<_>>();
    for parent in parents {
        for key in storage.ChildKeys(parent) {
            let record = &storage.managed[key];
            if record.input.kind == NodeKind::kText && !keys.contains(key) {
                if record.input.style_generated || record.input.first_line_style.is_some() {
                    return None;
                }
                let restyle = if record.input.element.is_none()
                    && in_place_restyle_admitted(storage, record, new_style)
                    && LayoutObjectTreeStorage::StyleUpdateCompatible(
                        record,
                        record.parent,
                        &record.input,
                        &shared_style,
                    ) {
                    NodeInputRestyle::InPlace {
                        style: Arc::clone(&shared_style),
                        resources: Arc::clone(&shared_resources),
                        field_diff: ComputedStyleBase::FieldInvalidationDiff(
                            unsafe { &*record.effective_computed_style },
                            unsafe { &*new_style },
                        ),
                    }
                } else {
                    let mut input = record.input.clone();
                    input.style = Arc::clone(&shared_style);
                    input.computed_style = new_style;
                    input.resources = Arc::clone(&shared_resources);
                    NodeInputRestyle::Update {
                        parent: record.parent,
                        input,
                    }
                };
                inputs.push((
                    key.clone(),
                    restyle,
                    NodeInputObjectUpdate {
                        object: record.object,
                    },
                ));
            }
        }
    }
    drop(admission_trace);
    let _apply_trace = browser_tracing::span("layout", "layout_invalidation.NodeInput.Apply");
    // Input comparisons and validation finish before any mutation. No sibling
    // topology or partial document transaction is exposed here.
    let mut objects = Vec::with_capacity(inputs.len());
    for (key, restyle, object) in inputs {
        objects.push(object);
        match restyle {
            NodeInputRestyle::InPlace {
                style,
                resources,
                field_diff,
            } => restyle_in_place(storage, &key, style, resources, field_diff),
            NodeInputRestyle::Update { parent, input } => {
                let record = storage.managed.remove(&key).unwrap();
                unsafe { &mut *record.node }.UnsealForTreeUpdate();
                storage.pending.insert(key.clone(), record);
                let text_style = if input.kind == NodeKind::kText {
                    Arc::as_ptr(&input.style)
                } else {
                    std::ptr::null()
                };
                storage.CreateWithStyleOwner(
                    parent,
                    input.id,
                    input.kind,
                    &input.style,
                    input.element,
                    input.debug_name,
                    input.text,
                    text_style,
                    key,
                    (&input.style, &input.resources),
                );
            }
        }
    }
    if !relations_changed {
        for change in &objects {
            let node = unsafe { &*change.object }.GetNode();
            if !node.is_null() {
                unsafe { &mut *node }.seal_input_tree();
            }
        }
    }
    Some(NodeInputUpdate {
        objects,
        relations_changed,
    })
}

/// LayoutObject::SetStyle for a retained object whose node, kind, parent and
/// element metadata are unchanged (core/layout/layout_object.cc:2991-3030,
/// StyleDidChange at :3120-3290). Nothing is recreated: the retained input
/// takes the restyle's style owner, the Node and LayoutObject take the new
/// ComputedStyle, and the one field diff computed at admission decides
///   - needs layout (ComputedStyle::VisualInvalidationDiff layout stages, via
///     the projected `LayoutEquivalent` owner),
///   - whether the resolved Font is retained (FontBuilder keeps an unchanged
///     font description's Font, core/css/resolver/font_builder.cc), and
///   - SetStyle's own paint-invalidation / paint-property classification.
/// Identity, fragments and FragmentData sidecars stay resident.
#[inline(never)]
fn restyle_in_place(
    storage: &mut LayoutObjectTreeStorage,
    key: &InputKey,
    style: Arc<ComputedStyle>,
    resources: Arc<crate::internal::layout_input::StyleResourceData>,
    field_diff: u64,
) {
    let fonts: *mut dyn FontResolver = &mut *storage.fonts;
    // SAFETY: `fonts` is boxed in this Storage and outlives the scope; the
    // scope only installs its pointer in the thread-local slot (as
    // create_managed_object does for a standalone update).
    let _font_scope = unsafe { FontResolverScope::new_from_raw(fonts) };
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    storage.stats.updated += 1;
    if storage.partial_index.is_some() {
        storage.attached_keys.push(key.clone());
    }
    let record = storage
        .managed
        .get_mut(key)
        .expect("admitted NodeInput key is resident");
    let object = record.object;
    let node = record.node;
    let parent = record.parent;
    let kind = record.input.kind;
    let new_style = style.style;
    let old_style = record.effective_computed_style;
    unsafe { &mut *node }.UnsealForTreeUpdate();

    // cpp: core/layout/layout_object.cc:3018-3020. One classification of this
    // replacement: the projected layout comparison reuses the admission's
    // FieldInvalidationDiff when it describes the same canonical pair.
    let layout_equivalent_to = |old: &ComputedStyle| {
        if old.style == old_style {
            old.LayoutEquivalentWithFieldDiff(&style, field_diff)
        } else {
            old.LayoutEquivalent(&style)
        }
    };
    let layout_equivalent = layout_equivalent_to(&*record.effective_style)
        && (Arc::ptr_eq(&record.input.style, &record.effective_style)
            || layout_equivalent_to(&*record.input.style));
    let previous: *const StyleComputedStyle = unsafe { &*object }.StyleRef();
    // A paint-only or geometry-only change keeps the element's font request:
    // retain its resolved platform Font instead of re-resolving it and
    // dirtying layout for kFontsChanged.
    let font_retained = layout_equivalent
        || (kind != NodeKind::kText
            && !unsafe { &*previous }.GetFont().is_null()
            && unsafe { record.input.computed_style.as_ref() }
                .is_some_and(|old| FontRequestInputsEqual(old, unsafe { &*new_style })));
    let layout_style = if font_retained {
        PreservePaintOnlyFontBinding(new_style, unsafe { &*previous })
    } else {
        new_style
    };

    // The retained input shares the restyle's canonical envelope owner; no
    // compacted copy or second resource projection is built.
    record.input.computed_style = new_style;
    record.input.style = Arc::clone(&style);
    record.input.resources = resources;
    record.effective_style = style;
    record.effective_computed_style = new_style;
    let roots = [
        record.input.computed_style,
        record.effective_computed_style,
        record.input.first_line_computed_style,
    ];
    RefreshInputStyleRoots(&mut record.input_style_roots, roots);

    let metadata_trace =
        browser_tracing::span("layout", "layout_invalidation.NodeInput.Metadata");
    let node_ref = unsafe { &mut *node };
    let scroll_offset = if font_retained {
        // Identity, element metadata, relations and the font binding are
        // unchanged; only the resource edges are replaced.
        node_ref.UpdatePaintInputData(&record.input);
        None
    } else {
        // A new font request: refresh the node input as the general update
        // does, which marks the font binding dirty for RebindFonts.
        if kind == NodeKind::kText {
            node_ref.UpdateInputData(&record.input);
        } else {
            unsafe { &mut *node.cast::<crate::internal::layout_node_metadata::Element>() }
                .UpdateElementInput(&record.input);
        }
        NodeMetadataRelations::AttachIncrementalRelations(node_ref)
            .unwrap_or_else(|error| panic!("{error:?}"));
        record
            .input
            .element
            .as_ref()
            .map(|element| element.scroll_offset)
    };
    node_ref.SetComputedStyle(layout_style);
    drop(metadata_trace);
    let paint_backed_inline = !DynamicTo::<LayoutInline>(object).is_null()
        && RequiresPaintBackedInlineFragment(&record.input);

    if !layout_equivalent {
        unsafe { &mut *object }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
            &raw const crate::internal::layout_invalidation_reason::kStyleChange,
        );
    }
    let set_style_trace =
        browser_tracing::span("layout", "layout_invalidation.NodeInput.SetStyle");
    if font_retained {
        // The resident pair differs from the canonical pair only by the
        // retained (identical) Font binding, so the admission diff is this
        // replacement's diff.
        unsafe { &mut *object }.SetStyleWithFieldDiff(layout_style, field_diff);
    } else {
        unsafe { &mut *object }.SetStyle(layout_style);
    }
    drop(set_style_trace);
    if let Some(offset) = scroll_offset {
        unsafe { &mut *node }.UpdateInputScrollOffset(offset);
    }
    if kind != NodeKind::kText {
        let computed = unsafe { &*new_style };
        if computed.HasFilter()
            || computed.HasBackdropFilter()
            || (computed.HasTransform() && unsafe { &*object }.IsBox())
        {
            unsafe { &mut *object }.SetCanContainFixedPositionObjects(true);
            unsafe { &mut *object }.SetCanContainAbsolutePositionObjects(true);
        }
    }
    if !parent.is_null() {
        let layout_inline = DynamicTo::<LayoutInline>(object);
        if !layout_inline.is_null()
            && RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled()
            && unsafe { &*layout_inline }.IsInlineRubyText()
        {
            let view = unsafe { &*layout_inline }.View();
            unsafe { &mut *view }.SetContainsAnnotations();
        }
        if !layout_inline.is_null() && paint_backed_inline {
            unsafe { &mut *layout_inline }.SetIsInLayoutNGInlineFormattingContext(true);
            unsafe { &mut *layout_inline }.SetShouldCreateBoxFragmentDefault();
        }
    }
}

/// Source HTMLMediaElement::UpdateLayoutObject scalar admission. Keep the
/// retained input descriptor in sync with its original metadata node, so the
/// next ordinary tree reconciliation cannot rediscover this same update as an
/// unrelated geometry change. This uses the existing per-node attachment index.
pub(super) fn update_video_layout_state(
    tree: &mut LayoutObjectTree,
    id: u64,
    state: super::super::layout_input::VideoLayoutState,
) -> Option<*mut LayoutObject> {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let keys = storage.InputKeys(id).to_vec();
    let mut result = None;
    for key in keys {
        let record = storage
            .managed
            .get_mut(&key)
            .expect("native attachment key");
        if record.input.style_generated
            || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
            || !unsafe { &*record.object }.IsVideo()
        {
            continue;
        }
        assert!(
            result.is_none(),
            "one canonical video LayoutObject per element"
        );
        let input = record.input.element.as_mut().expect("video element input");
        input.video = Some(state);
        input.media_is_fullscreen = state.is_fullscreen;
        unsafe { &mut *record.node }.SetVideoLayoutState(state);
        result = Some(record.object);
    }
    result
}
