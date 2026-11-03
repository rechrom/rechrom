//! Invalidation-owned retained LayoutObject admission and update transaction.

use super::*;
use crate::internal::layout_object::MarkingBehavior;

fn canonical_text_input(
    style: &StyleComputedStyle,
    resources: &crate::internal::layout_input::StyleResourceData,
) -> ComputedStyle {
    let mut input = ComputedStyle {
        style,
        ..Default::default()
    };
    let bindings = crate::style_projection::StyleResourceBindings::FromData(resources);
    bindings.ApplyTo(&mut input);
    input.paint.image_resource_id = resources.image_resource_id;
    input
}

fn compact_retained_style(
    style: *const StyleComputedStyle,
    resources: &crate::internal::layout_input::StyleResourceData,
) -> Arc<ComputedStyle> {
    if style.is_null() {
        return Arc::new(ComputedStyle::default());
    }
    Arc::new(canonical_text_input(unsafe { &*style }, resources))
}

impl LayoutObjectTreeStorage {
    #[inline(always)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn Create(
        &mut self,
        parent: *mut LayoutObject,
        id: u64,
        kind: NodeKind,
        style: &ComputedStyle,
        element: Option<ElementData>,
        debug_name: String,
        text: String,
        explicit_text_style: *const ComputedStyle,
        style_generated: bool,
        source_text_style: *const ComputedStyle,
        first_line_style: *const ComputedStyle,
        existing_key: Option<InputKey>,
    ) -> *mut LayoutObject {
        create_managed_object(
            self,
            parent,
            id,
            kind,
            style,
            element,
            debug_name,
            text,
            explicit_text_style,
            style_generated,
            source_text_style,
            first_line_style,
            existing_key,
        )
    }
}

// This is deliberately a free function: inherent method symbols omit the
// source module, while this frame keeps per-node update cost package-qualified.
#[inline(never)]
#[allow(clippy::too_many_arguments)]
fn create_managed_object(
    storage: &mut LayoutObjectTreeStorage,
    parent: *mut LayoutObject,
    id: u64,
    kind: NodeKind,
    style: &ComputedStyle,
    element: Option<ElementData>,
    debug_name: String,
    text: String,
    explicit_text_style: *const ComputedStyle,
    style_generated: bool,
    source_text_style: *const ComputedStyle,
    first_line_style: *const ComputedStyle,
    existing_key: Option<InputKey>,
) -> *mut LayoutObject {
    // CompleteUpdate installs these scopes once for the complete
    // attachment transaction. Compatible NodeInput updates are local and
    // retain standalone scopes here.
    let _heap_scope = (!storage.updating).then(LayoutHeapScope::new);
    if existing_key.is_none() {
        storage.input_index.take();
    }
    let inherited_text_input =
        (kind == NodeKind::kText && explicit_text_style.is_null()).then(|| {
            assert!(!parent.is_null(), "LayoutText cannot be the layout root");
            let parent_node = unsafe { &*parent }.GetNode();
            canonical_text_input(
                unsafe { &*parent }.StyleRef(),
                unsafe { &*parent_node }.InputStyleResources(),
            )
        });
    let effective = if kind == NodeKind::kText {
        assert!(!parent.is_null(), "LayoutText cannot be the layout root");
        if !explicit_text_style.is_null() {
            unsafe { &*explicit_text_style }
        } else {
            inherited_text_input.as_ref().unwrap()
        }
    } else {
        style
    };
    let source = if kind == NodeKind::kText && !source_text_style.is_null() {
        unsafe { &*source_text_style }
    } else {
        effective
    };
    let first_line = if first_line_style.is_null() {
        None
    } else {
        Some(unsafe { &*first_line_style })
    };
    let targeted_style_update = existing_key.is_some();
    let place = existing_key.is_none();
    let key = existing_key.unwrap_or_else(|| {
        let base = (
            id,
            kind as i32,
            style_generated,
            Arc::<str>::from(debug_name),
        );
        let occurrence = storage.occurrences.entry(base.clone()).or_default();
        let key = InputKey(base.0, base.1, base.2, base.3, *occurrence);
        *occurrence += 1;
        key
    });
    let mut previous = storage.pending.remove(&key);
    let reattach = storage.reattach.contains(&id);
    // Compare borrowed inputs before making owned descriptors: unchanged nodes
    // need neither strings nor construction resolver scopes.
    if let Some(old) = previous.as_ref() {
        if !reattach
            && !unsafe { &*old.object }.WasDestroyedForTreeUpdate()
            && old.parent == parent
            && old.input.id == id
            && old.input.kind == kind
            && old.input.style_generated == style_generated
            && old.input.style.as_ref() == source
            && old.input.text == text
            && old.input.element == element
            && old.input.debug_name == key.3.as_ref()
            && old.input.first_line_style.as_deref() == first_line
            && old.effective_style.as_ref() == effective
        {
            let object = old.object;
            let node = old.node;
            if place {
                storage.Place(parent, object, node);
            }
            storage.stats.reused += 1;
            if storage.partial_index.is_some() {
                storage.attached_keys.push(key.clone());
            }
            storage.managed.insert(key, previous.take().unwrap());
            return object;
        }
    }
    let fonts: *mut dyn FontResolver = &mut *storage.fonts;
    // SAFETY: `fonts` is boxed in this Storage and outlives the scope;
    // the scope only installs its pointer in the thread-local slot.
    let _font_scope =
        (!storage.updating).then(|| unsafe { FontResolverScope::new_from_raw(fonts) });
    let _factory_scope =
        (!storage.updating).then(|| LayoutObjectFactoryScope::new(&storage.factories));
    let effective_style = Arc::new(effective.clone());
    let source_style = if std::ptr::eq(source, effective) {
        Arc::clone(&effective_style)
    } else {
        Arc::new(source.clone())
    };
    let input_style = if std::ptr::eq(style, effective) {
        Arc::clone(&effective_style)
    } else if std::ptr::eq(style, source) {
        Arc::clone(&source_style)
    } else {
        Arc::new(style.clone())
    };
    let input_resources =
        Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input_style));
    let source_resources =
        Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(&source_style));
    let mut input = NodeConstructionData {
        id,
        kind,
        style_generated,
        computed_style: input_style.style,
        style: input_style,
        resources: input_resources,
        element,
        debug_name: key.3.to_string(),
        text,
        first_line_computed_style: std::ptr::null(),
        first_line_style: None,
        first_line_resources: None,
    };
    let descriptor = NodeConstructionData {
        id,
        kind,
        style_generated,
        computed_style: source_style.style,
        style: source_style,
        resources: source_resources,
        element: input.element.clone(),
        debug_name: input.debug_name.clone(),
        text: input.text.clone(),
        first_line_computed_style: first_line.map_or(std::ptr::null(), |style| style.style),
        first_line_style: first_line.cloned().map(Arc::new),
        first_line_resources: first_line.map(|style| {
            Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(style))
        }),
    };
    if let Some(old) = &previous {
        let compatible = if targeted_style_update {
            LayoutObjectTreeStorage::StyleUpdateCompatible(
                old,
                parent,
                &descriptor,
                &effective_style,
            )
        } else {
            LayoutObjectTreeStorage::Compatible(old, parent, &descriptor, &effective_style)
        };
        if reattach || unsafe { &*old.object }.WasDestroyedForTreeUpdate() || !compatible {
            if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
                unsafe { &mut *old.object }.Destroy();
                unsafe { &mut *old.node }.DetachForTreeUpdate();
            }
            if old.object == storage.root.Get() {
                storage.root = Persistent::from_ptr(std::ptr::null_mut());
            }
            storage.stats.removed += 1;
            // Keep the old allocation rooted until the transaction's
            // destruction phase has completed, including unwind cleanup.
            storage.retired.push((old.object, old.node));
            previous = None;
        }
    }
    let natural_size_change_is_paint_only = previous.as_ref().is_some_and(|old| {
        LayoutImageNaturalSizeChangeIsPaintOnly(
            old.object,
            &descriptor.element,
            &old.input.element,
            descriptor.computed_style,
        )
    });
    let natural_size_changed = previous.as_ref().is_some_and(|old| {
        LayoutImageNaturalSizeChanged(old.object, &descriptor.element, &old.input.element)
    });
    let same_non_style_inputs = previous.as_ref().is_some_and(|old| {
        NonStyleInputsEqual(&descriptor, &old.input, natural_size_change_is_paint_only)
    });
    let control_paint_state_changed = previous
        .as_ref()
        .is_some_and(|old| ControlPaintStateChanged(&descriptor.element, &old.input.element));
    let image_paint_state_changed = previous
        .as_ref()
        .is_some_and(|old| ImagePaintStateChanged(&descriptor.element, &old.input.element));
    let image_notification = if previous.is_some() {
        natural_size_changed || image_paint_state_changed
    } else {
        descriptor
            .element
            .as_ref()
            .is_some_and(|element| element.image_resource_is_loaded || element.image_resource_error)
    };
    let paint_only = previous.as_ref().is_some_and(|old| {
        same_non_style_inputs
            && old.input.style.LayoutEquivalent(&descriptor.style)
            && old.effective_style.LayoutEquivalent(&effective_style)
    });
    let context = layout_algo_block_fragmentation_context::fragmentation_context(parent);
    let mut prepare_style = |data: &NodeConstructionData,
                             parent_style: *const StyleComputedStyle,
                             parent_kind: NodeKind| {
        // SAFETY: Create runs on one thread, and the TLS scope is restored
        // before the boxed resolver can be moved or destroyed.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            PrepareStyle(
                data,
                unsafe { &mut *fonts },
                context,
                parent_style,
                parent_kind,
            )
        }));
        match result {
            Ok(style) => style,
            Err(error) => {
                if let Some(invalid) = error.downcast_ref::<InvalidLayoutInput>() {
                    let mut detail = String::new();
                    if invalid.to_string() == "Invalid layout input: width" {
                        if let Some(extra) = data.style.extended.as_ref() {
                            let value = |number: Option<f64>| {
                                number.map_or_else(
                                    || "none".to_string(),
                                    |number| format!("{number:.6}"),
                                )
                            };
                            detail = format!(
                                " [px={}, percent={}, calculated={}, sizing={}]",
                                value(data.style.width),
                                value(extra.width_percent),
                                extra.width_calculated as u8,
                                extra.width_sizing as u32,
                            );
                        }
                    }
                    panic!("Invalid style for {}: {invalid}{detail}", data.debug_name);
                }
                std::panic::resume_unwind(error)
            }
        }
    };
    let parent_kind = if parent.is_null() {
        NodeKind::kBox
    } else {
        let parent_node = unsafe { &*parent }.GetNode();
        if parent_node.is_null() {
            NodeKind::kBox
        } else {
            unsafe { &*parent_node }.InputKind()
        }
    };
    // A paint-only style change retains geometry, not the old style object.
    // Blink's LayoutObject::SetStyle installs the new ComputedStyle even
    // when StyleDifference does not require layout; Paint then observes
    // updated StyleImage/effect/color data through StyleRef().  Keeping the
    // previous pointer here made the Rust projection current while the
    // resident LayoutObject and fragment-local paint data stayed one revision behind.
    // PrepareStyle returns StyleEngine's canonical source pointer
    // when one is present, so this does not translate the projection back
    // into a second style.
    let layout_style = if paint_only {
        if kind == NodeKind::kText {
            input.style = if !explicit_text_style.is_null() {
                Arc::new(unsafe { &*explicit_text_style }.clone())
            } else {
                let parent_node = unsafe { &*parent }.GetNode();
                Arc::new(canonical_text_input(
                    unsafe { &*parent }.StyleRef(),
                    unsafe { &*parent_node }.InputStyleResources(),
                ))
            };
            input.resources = Arc::new(
                crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input.style),
            );
            input.computed_style = input.style.style;
        }
        prepare_style(
            &input,
            if parent.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*parent }.StyleRef()
            },
            parent_kind,
        )
    } else if kind == NodeKind::kText {
        if parent.is_null() {
            panic!("LayoutText cannot be the layout root");
        }
        if !explicit_text_style.is_null() {
            input.style = Arc::new(unsafe { &*explicit_text_style }.clone());
            input.resources = Arc::new(
                crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input.style),
            );
            input.computed_style = input.style.style;
            prepare_style(&input, unsafe { &*parent }.StyleRef(), parent_kind)
        } else {
            let parent_node = unsafe { &*parent }.GetNode();
            input.style = Arc::new(canonical_text_input(
                unsafe { &*parent }.StyleRef(),
                unsafe { &*parent_node }.InputStyleResources(),
            ));
            input.resources = Arc::new(
                crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input.style),
            );
            input.computed_style = unsafe { &*parent }.StyleRef();
            unsafe { &*parent }.StyleRef() as *const StyleComputedStyle
        }
    } else {
        let parent_style = if parent.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*parent }.StyleRef() as *const StyleComputedStyle
        };
        prepare_style(&input, parent_style, parent_kind)
    };
    let layout_style = if paint_only {
        let old = previous
            .as_ref()
            .expect("paint-only update has a previous object");
        PreservePaintOnlyFontBinding(layout_style, unsafe { &*old.object }.StyleRef())
    } else {
        layout_style
    };
    let mut layout_first_line_style: *const StyleComputedStyle = std::ptr::null();
    if !first_line_style.is_null() {
        if kind == NodeKind::kText {
            panic!("first-line style belongs to the text's containing element");
        }
        input.first_line_style = Some(Arc::new(unsafe { &*first_line_style }.clone()));
        input.first_line_resources = input.first_line_style.as_ref().map(|style| {
            Arc::new(crate::internal::layout_input::StyleResourceData::FromLegacyInput(style))
        });
        input.first_line_computed_style = input
            .first_line_style
            .as_ref()
            .map_or(std::ptr::null(), |style| style.style);
        let mut first_line_input = input.clone();
        first_line_input.style = Arc::new(unsafe { &*first_line_style }.clone());
        first_line_input.resources = Arc::new(
            crate::internal::layout_input::StyleResourceData::FromLegacyInput(
                &first_line_input.style,
            ),
        );
        first_line_input.computed_style = first_line_input.style.style;
        let mut parent_first_line_style = std::ptr::null();
        if !parent.is_null() {
            let parent_node = unsafe { &*parent }.GetNode();
            parent_first_line_style = unsafe { &*parent_node }.InputFirstLineStyle();
            if parent_first_line_style.is_null() {
                parent_first_line_style = unsafe { &*parent }.StyleRef();
            }
        }
        layout_first_line_style =
            prepare_style(&first_line_input, parent_first_line_style, parent_kind);
    }
    if kind == NodeKind::kText && !source_text_style.is_null() {
        input.style = Arc::new(unsafe { &*source_text_style }.clone());
        input.resources = Arc::new(
            crate::internal::layout_input::StyleResourceData::FromLegacyInput(&input.style),
        );
        input.computed_style = input.style.style;
    }
    let old_font_dirty = previous
        .as_ref()
        .is_some_and(|old| unsafe { &*old.node }.NeedsInputFontBinding());
    let metadata_node: *mut Node = if let Some(old) = &previous {
        if kind == NodeKind::kText {
            unsafe { &mut *old.node }.UpdateInputData(&input);
            // Style/resource updates still refresh live metadata, but an
            // unchanged text payload already owns the same source string.
            if old.input.text != input.text {
                unsafe { &mut *old.node.cast::<TextNode>() }
                    .SetDataForTreeUpdate(BlinkString::FromUtf8(input.text.as_bytes()));
            }
        } else {
            unsafe {
                &mut *old
                    .node
                    .cast::<crate::internal::layout_node_metadata::Element>()
            }
            .UpdateElementInput(&input);
        }
        unsafe { &mut *old.node }.SetComputedStyle(layout_style);
        old.node
    } else if kind == NodeKind::kText {
        let text = BlinkString::FromUtf8(input.text.as_bytes());
        if text.IsNull() {
            panic!("LayoutText must contain valid UTF-8");
        }
        MakeGarbageCollected(
            TextNode::new(&input, layout_style, text).unwrap_or_else(|error| panic!("{error:?}")),
        ) as *mut Node
    } else {
        CreateElementMetadata(&input, layout_style, &storage.factories)
            .unwrap_or_else(|error| panic!("{error:?}")) as *mut Node
    };
    if paint_only && !old_font_dirty {
        unsafe { &mut *metadata_node }.ClearInputFontBinding();
    }
    unsafe { &mut *metadata_node }.SetInputFirstLineStyle(layout_first_line_style);
    if previous.is_none() && !parent.is_null() {
        let parent_node = unsafe { &*parent }.GetNode().cast::<ContainerNode>();
        unsafe { &mut *parent_node }
            .AppendChild(unsafe { &mut *metadata_node })
            .unwrap_or_else(|error| panic!("{error:?}"));
    }
    NodeMetadataRelations::AttachIncrementalRelations(unsafe { &mut *metadata_node })
        .unwrap_or_else(|error| panic!("{error:?}"));
    let object = if let Some(old) = &previous {
        storage.stats.updated += 1;
        if kind == NodeKind::kText && old.input.text != input.text {
            unsafe { &mut *DynamicTo::<LayoutText>(old.object) }
                .ForceSetText(BlinkString::FromUtf8(input.text.as_bytes()));
        }
        // Colors, source control flags and scroll offsets affect paint.
        // Retain geometry while updating metadata; other inputs remain conservative.
        if natural_size_changed && !natural_size_change_is_paint_only {
            unsafe { &mut *old.object }
                .SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
                    &raw const crate::internal::layout_invalidation_reason::kSizeChanged,
                );
        } else if !same_non_style_inputs
            || !old.effective_style.LayoutEquivalent(&effective_style)
            || !old.input.style.LayoutEquivalent(&descriptor.style)
        {
            unsafe { &mut *old.object }.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                &raw const crate::internal::layout_invalidation_reason::kStyleChange,
            );
        }
        old.object
    } else {
        storage.stats.created += 1;
        CreateLayoutObject(
            unsafe { &mut *metadata_node },
            unsafe { &*layout_style },
            &storage.factories,
        )
    };
    unsafe { &mut *object }.SetStyle(layout_style);
    if unsafe { &*object }.IsLayoutImage() {
        let ratio = descriptor
            .element
            .as_ref()
            .map_or(1.0, |element| element.image_device_pixel_ratio);
        let resource_error = descriptor
            .element
            .as_ref()
            .is_some_and(|element| element.image_resource_error);
        let update = storage
            .factories
            .replaced_image_state_updater
            .expect("LayoutImage state updater is not installed");
        update(
            unsafe { &mut *object.cast::<crate::internal::layout_replaced::LayoutReplaced>() },
            ratio,
            resource_error,
        );
    }
    if natural_size_changed {
        unsafe { &mut *object }.SetNeedsPaintPropertyUpdate();
    }
    if natural_size_change_is_paint_only {
        let object = unsafe { &mut *object };
        object.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkContainerChain);
        object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kImage,
        );
    }
    if image_paint_state_changed {
        let object = unsafe { &mut *object };
        object.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kImage,
        );
        // cpp: core/layout/layout_image.cc:288-290. Managed input refresh is
        // the defer-capable resource notification boundary in this assembly.
        if object.IsLayoutImage()
            && input
                .element
                .as_ref()
                .is_some_and(|element| element.image_resource_maybe_animated)
        {
            object.SetShouldDelayFullPaintInvalidation();
        }
    } else if control_paint_state_changed {
        unsafe { &mut *object }.SetShouldDoFullPaintInvalidationWithoutLayoutChange(
            foundation::PaintInvalidationReason::kStyle,
        );
    }
    if let Some(element) = input.element.as_ref() {
        unsafe { &mut *metadata_node }.UpdateInputScrollOffset(element.scroll_offset);
    }
    if kind != NodeKind::kText {
        let establishes_fixed_container = unsafe { input.computed_style.as_ref() }.map_or_else(
            || {
                let paint = &input.style.paint;
                (!paint.filters.is_empty()
                    || !paint.backdrop_filters.is_empty()
                    || paint.backdrop_filter_has_reference)
                    || (paint.transform.is_some() && unsafe { &*object }.IsBox())
            },
            |style| {
                style.HasFilter()
                    || style.HasBackdropFilter()
                    || (style.HasTransform() && unsafe { &*object }.IsBox())
            },
        );
        if establishes_fixed_container {
            unsafe { &mut *object }.SetCanContainFixedPositionObjects(true);
            unsafe { &mut *object }.SetCanContainAbsolutePositionObjects(true);
        }
    }
    unsafe { &mut *metadata_node }.SetLayoutObject(object);
    if previous.is_none() {
        storage
            .metadata
            .insert(metadata_node, Persistent::from_ptr(metadata_node));
        storage.objects.insert(object, Persistent::from_ptr(object));
    }
    let effective_resources =
        crate::internal::layout_input::StyleResourceData::FromLegacyInput(&effective_style);
    let effective_computed_style = if effective_style.style.is_null() {
        layout_style
    } else {
        effective_style.style
    };
    let mut retained_input = descriptor;
    retained_input.style = compact_retained_style(
        retained_input.computed_style,
        retained_input.resources.as_ref(),
    );
    retained_input.first_line_style =
        retained_input
            .first_line_resources
            .as_ref()
            .map(|resources| {
                compact_retained_style(retained_input.first_line_computed_style, resources.as_ref())
            });
    let retained_effective_style =
        compact_retained_style(effective_computed_style, &effective_resources);
    let record = ManagedObject {
        object,
        node: metadata_node,
        parent,
        input_style_roots: RetainInputStyles(&retained_input, effective_computed_style),
        input: retained_input,
        effective_style: retained_effective_style,
        effective_computed_style,
    };
    let record = if let Some(mut previous) = previous {
        *previous = record;
        previous
    } else {
        Box::new(record)
    };
    if storage.partial_index.is_some() {
        storage.attached_keys.push(key.clone());
    }
    storage.managed.insert(key, record);
    if place {
        storage.Place(parent, object, metadata_node);
    }
    if image_notification && unsafe { &*object }.IsLayoutImage() {
        unsafe { &*object.cast::<crate::internal::layout_replaced::LayoutInputReplaced>() }
            .IncrementVisuallyNonEmptyPixelCountIfNeeded();
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
        if !layout_inline.is_null() && RequiresPaintBackedInlineFragment(&input) {
            unsafe { &mut *layout_inline }.SetIsInLayoutNGInlineFormattingContext(true);
            unsafe { &mut *layout_inline }.SetShouldCreateBoxFragmentDefault();
        }
    }
    object
}
