//! Chromium physical-fragment and inline-item paint population.
//!
//! This assembly-only fragment owner retains reciprocal source types while
//! placing the two recursive hot paths under `layout_fragment` symbols.

#![allow(non_snake_case)]

use super::*;
use crate::inline_cursor::InlineCursor;

fn RetainedResourcesAreCompatible(
    paint: &PaintPropertiesHandle,
    resources: &Arc<PaintResources>,
    owners: &mut FragmentOwners,
) -> bool {
    let Some(previous) = paint.resources.as_ref() else {
        return false;
    };
    if Arc::ptr_eq(previous, resources) {
        return true;
    }
    let key = Arc::as_ptr(previous) as usize;
    *owners
        .retained_resource_compatibility
        .entry(key)
        .or_insert_with(|| {
            previous.fonts == resources.fonts
                && previous.images == resources.images
                && previous.canvases == resources.canvases
                && previous.device_pixel_ratio == resources.device_pixel_ratio
                && previous.preferred_color_scheme == resources.preferred_color_scheme
                && previous.layout_zoom_factor == resources.layout_zoom_factor
                && previous.base_background_color == resources.base_background_color
                && previous.paints_opaque_base_background == resources.paints_opaque_base_background
                && previous.should_clear_document_background
                    == resources.should_clear_document_background
        })
}

fn RetainedPaintProperties(
    object: *const LayoutObject,
    purpose: LayoutUpdateKind,
    instance: u64,
    resources: &Arc<PaintResources>,
    paint_state: &crate::layout_engine::LayoutPaintState,
    owners: &mut FragmentOwners,
) -> Option<PrePaintInvalidation> {
    if purpose != LayoutUpdateKind::Paint {
        return None;
    }
    let invalidation = if object.is_null() {
        PrePaintInvalidation::default()
    } else {
        unsafe { &*object }
            .GetMutableForPainting()
            .EnsureIsReadyForPaintInvalidation();
        let invalidation = ExportPrePaintInvalidation(object);
        if invalidation.own() || invalidation.descendants() {
            return None;
        }
        invalidation
    };
    let paint = paint_state.ResidentPaintProperties(instance)?;
    RetainedResourcesAreCompatible(paint, resources, owners).then_some(invalidation)
}

// Chromium: core/paint/box_fragment_painter.cc:102-127.  Line truncation
// marks the FragmentItem in the inline formatting context, while a
// self-painting atomic inline is later entered through its stored physical
// fragment.  Export the effective IsHiddenForPaint result once at the source
// Layout-to-Paint boundary so the paint packages do not have to recreate an
// InlineCursor from flattened data.
fn IsHiddenForPaintIncludingInlineItem(fragment: &PhysicalFragment) -> bool {
    if fragment.IsHiddenForPaint() {
        return true;
    }
    if !fragment.IsAtomicInline() || !fragment.HasSelfPaintingLayer() {
        return false;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() || !unsafe { &*object }.IsInLayoutNGInlineFormattingContext() {
        return false;
    }
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(unsafe { &*object });
    cursor.IsNotNull() && cursor.Current().IsHiddenForPaint()
}

// Chromium grows and shrinks a LayoutObject's FragmentData list while
// PrePaint walks its physical fragments (core/paint/pre_paint_tree_walk.cc,
// GetOrCreateFragmentData). This publication walk is Rechrom's equivalent
// source-fragment traversal, so establish the same one-entry-per-physical-
// fragment invariant before asking a fragment for its resident data.
fn SynchronizeBoxFragmentData(fragment: &PhysicalFragment) {
    if !fragment.IsBox() {
        return;
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    let Some(layout_box) = (unsafe { layout_box.as_ref() }) else {
        return;
    };
    let physical_count = layout_box.PhysicalFragmentCount() as u32;
    debug_assert_ne!(physical_count, 0);

    let owner = unsafe { &*object };
    let old_count = owner.GetMutableForPainting().FragmentList().size();
    if old_count == physical_count {
        return;
    }

    let mut mutable = owner.GetMutableForPainting();
    mutable.FragmentCountChanged();
    if old_count < physical_count {
        while mutable.FragmentList().size() < physical_count {
            mutable.FragmentList().AppendNewFragment();
        }
        // Chromium forces paint-property construction for newly added entries.
        mutable.SetOnlyThisNeedsPaintPropertyUpdate();
    } else {
        mutable.FragmentList().Shrink(physical_count);
    }
}

// Source: PrePaintTreeWalk::GetOrCreateFragmentData and
// AccompaniedFragmentIterator::Advance (one entry per inline container, not
// per line box). Reuse the existing publication traversal; do not perform a
// second same-object-item walk to discover the final count.
fn SynchronizeInlineFragmentData(item: &FragmentItem, container: u64, owners: &mut FragmentOwners) {
    let object = item.GetLayoutObject();
    let Some(owner) = (unsafe { object.as_ref() }) else {
        return;
    };
    if !owner.IsLayoutInline() {
        return;
    }
    let entry = owners
        .inline_fragment_data
        .entry(object)
        .or_insert((container, 1));
    if entry.0 != container {
        entry.0 = container;
        entry.1 += 1;
    }
    let count = entry.1;
    let mut mutable = owner.GetMutableForPainting();
    let previous_count = mutable.FragmentList().size();
    if previous_count < count || (item.IsLastForNode() && previous_count > count) {
        mutable.FragmentCountChanged();
        while mutable.FragmentList().size() < count {
            mutable.FragmentList().AppendNewFragment();
        }
        if item.IsLastForNode() && mutable.FragmentList().size() > count {
            mutable.FragmentList().Shrink(count);
        }
        mutable.SetOnlyThisNeedsPaintPropertyUpdate();
    }
}

fn PopulateInlineItemLocalPaintData(
    item: &FragmentItem,
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    parent_offset: PhysicalOffset,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    container_paint_offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    container_fragment_instance_id: u64,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    let object = item.GetLayoutObject();
    if purpose == LayoutUpdateKind::Paint {
        SynchronizeInlineFragmentData(item, container_fragment_instance_id, paint_state_owners);
    }
    let item_index =
        unsafe { (item as *const FragmentItem).offset_from(fragment_items.Items().as_ptr()) };
    assert!(item_index >= 0);
    let (identity_kind, identity_start, identity_end) = match item.Type() {
        ItemType::kText => (0, item.StartOffset(), item.EndOffset()),
        ItemType::kGeneratedText => (1, item.StartOffset(), item.EndOffset()),
        ItemType::kLine => (2, 0, 0),
        ItemType::kBox => (3, 0, 0),
        ItemType::kInvalid => unreachable!("invalid inline item"),
    };
    let uses_first_line_style = item.UsesFirstLineStyle();
    let effective_style = if uses_first_line_style {
        if item.Type() == ItemType::kLine && !object.is_null() {
            unsafe { &*object }.FirstLineStyleRef() as *const StyleComputedStyle
        } else {
            item.Style() as *const StyleComputedStyle
        }
    } else {
        std::ptr::null()
    };
    let display_item_client_id = if object.is_null() {
        0
    } else {
        unsafe { &*(object as *const DisplayItemClient) }.Id() as u64
    };
    let fragment_instance_id = paint_state_owners.fragment_instance(
        FragmentIdentityKey::InlineItem {
            container: container_fragment_instance_id,
            client: display_item_client_id,
            fragment: item.FragmentId(),
            kind: identity_kind,
            start: identity_start,
            end: identity_end,
            index: item_index as u32,
        },
        next_fragment_instance_id,
    );
    let retained_invalidation = RetainedPaintProperties(
        object,
        purpose,
        fragment_instance_id,
        resources,
        paint_state,
        paint_state_owners,
    );
    let store_started = browser_tracing::enabled().then(std::time::Instant::now);
    let (output, slot_retained) =
        paint_state.BindInline(fragment_items, item_index as usize, fragment_instance_id);
    if let Some(started) = store_started {
        paint_state_owners.population_stats.sidecar_store_ns = paint_state_owners
            .population_stats
            .sidecar_store_ns
            .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    let output = unsafe { &mut *output };
    if let Some(invalidation) = retained_invalidation {
        assert!(
            slot_retained,
            "retained paint properties have no resident slot"
        );
        output.prepare_for_publication();
        output.paint.pre_paint_invalidation = invalidation;
        output.paint.resources = Some(Arc::clone(resources));
    } else {
        let export_started = browser_tracing::enabled().then(std::time::Instant::now);
        let properties = ExportProperties(
            object,
            std::ptr::null_mut(),
            sources,
            style_sources,
            effective_style,
            resources,
            purpose,
            Some(&mut paint_state_owners.exported_styles),
            Some(&mut paint_state_owners.exported_selection_styles),
            Some(&mut paint_state_owners.debug_names),
        );
        if let Some(started) = export_started {
            paint_state_owners.population_stats.property_export_ns = paint_state_owners
                .population_stats
                .property_export_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        *output = FragmentPaintData::new(properties);
    }
    paint_state_owners.population_stats.inline_items += 1;
    output.fragment_instance_id = fragment_instance_id;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    output.node_id = InputId(object);
    // cpp: core/paint/box_fragment_painter.cc:2025-2026,2071-2072
    // cpp: core/paint/inline_box_fragment_painter.cc:63-64
    output.paint.display_item_fragment = item.FragmentId();
    output.paint.is_line_break = item.IsText() && item.IsLineBreak();
    output.paint.hidden =
        item.IsHiddenForPaint() || output.paint.svg_viewport.is_some_and(|v| v.disabled);
    let rect = *item.RectInContainerFragment();
    output.offset = Offset {
        x: Number(rect.offset.left - parent_offset.left),
        y: Number(rect.offset.top - parent_offset.top),
    };
    output.size = Size {
        width: Number(rect.size.width),
        height: Number(rect.size.height),
    };
    if purpose == LayoutUpdateKind::Paint && item.IsText() && !item.IsSvgText() {
        // Source: core/paint/box_fragment_painter.cc:2492-2510 and
        // fragment_item.cc:1090-1109. Preserve LayoutTextCombine's compressed
        // bounds when present; otherwise use FragmentItem's pixel-snapped
        // ordinary point-test rectangle. Export it relative to this resident
        // item because interaction already owns the accumulated parent offset.
        let text_combine = if object.is_null() {
            std::ptr::null_mut()
        } else {
            DynamicTo::<LayoutTextCombine>(unsafe { &*object }.Parent())
        };
        let mut bounds = if text_combine.is_null() {
            item.ComputeTextBoundsRectForHitTest(PhysicalOffset::default(), false)
        } else {
            unsafe { &*text_combine }
                .ComputeTextBoundsRectForHitTest(item, &PhysicalOffset::default())
        };
        bounds.offset -= rect.offset;
        output.paint.text_hit_test_bounds = Some(FragmentPaintRect {
            offset: Offset {
                x: Number(bounds.offset.left),
                y: Number(bounds.offset.top),
            },
            size: Size {
                width: Number(bounds.size.width),
                height: Number(bounds.size.height),
            },
        });
    }
    {
        output.content_size = output.size;
    }
    if purpose == LayoutUpdateKind::Paint && item.Type() == ItemType::kBox && !object.is_null() {
        let style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNinePieceImages(style, output.size, resources, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportInlineInkOverflow(item, &mut output.paint);
    }
    {
        ExportSvgTextPaintData(item, rect, &mut output.paint);
    }
    match item.Type() {
        ItemType::kText | ItemType::kGeneratedText => {
            output.kind = if item.Type() == ItemType::kText {
                FragmentKind::kText
            } else {
                FragmentKind::kGeneratedText
            };
            if let Some(line_offset) = containing_line_offset {
                output.paint.text_line_top_offset = Some(Number(line_offset.top - rect.offset.top));
                // Source: core/paint/text_fragment_painter.cc:69-103.
                // Preserve the exact inputs to ComputePhysicalBoxGeometry;
                // flattened text-item offsets no longer identify the source
                // containing fragment after PaintOffsetTranslation.
                let line_top = container_paint_offset.top + line_offset.top;
                output.paint.text_line_snap_delta =
                    Some(Number(LayoutUnit::from_signed(line_top.Round()) - line_top));
            }
            output.paint.text_line_height = containing_line_height.map(Number);
            output.text_start = Some(item.StartOffset());
            output.text_end = Some(item.EndOffset());
            let text = item.TextPaintInfo(fragment_items);
            output.text_paint_timing_non_whitespace = !text
                .text
                .SubstringContainsOnlyWhitespaceOrEmpty(text.from, text.to);
            output.paint.is_ellipsis = item.IsEllipsis();
            output.paint.text_fit_scale = if item.IsSvgText() {
                1.0
            } else {
                f64::from(item.GetTextFitScale())
            };
            if purpose == LayoutUpdateKind::Paint {
                ExportListMarkerSymbol(item, &mut output.paint);
                ExportTextCombineInlineOffset(item, &mut output.paint);
            }
        }
        ItemType::kLine => {
            output.kind = FragmentKind::kLine;
            output.node_id = 0;
            if uses_first_line_style && !object.is_null() {
                let line_style = unsafe { &*effective_style };
                let normal_style = unsafe { &*object }.StyleRef();
                if !std::ptr::eq(line_style, normal_style) {
                    let line_box = item.LineBoxFragment();
                    debug_assert!(!line_box.is_null());
                    let line_box = unsafe { &*line_box };
                    let line_metrics = line_box.Metrics();
                    let text_metrics = line_style.GetFontHeightForDefaultBaseline();
                    output.paint.first_line_decoration_rect =
                        Some(if line_style.IsHorizontalWritingMode() {
                            FragmentPaintRect {
                                offset: Offset {
                                    x: 0.0,
                                    y: Number(line_metrics.ascent - text_metrics.ascent),
                                },
                                size: Size {
                                    width: output.size.width,
                                    height: Number(text_metrics.LineHeight()),
                                },
                            }
                        } else {
                            FragmentPaintRect {
                                offset: Offset {
                                    x: output.size.width
                                        - Number(line_metrics.ascent)
                                        - Number(text_metrics.descent),
                                    y: 0.0,
                                },
                                size: Size {
                                    width: Number(text_metrics.LineHeight()),
                                    height: output.size.height,
                                },
                            }
                        });
                }
            }
        }
        ItemType::kBox => {
            output.kind = FragmentKind::kBox;
            let box_fragment = item.BoxFragment();
            if !box_fragment.is_null() {
                let box_fragment = unsafe { &*box_fragment };
                {
                    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    let outline_style = if effective_style.is_null() {
                        box_fragment.Style()
                    } else {
                        unsafe { &*effective_style }
                    };
                    ExportSelfOutlineGeometry(box_fragment, outline_style, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint && output.paint.scroll_container.is_some() {
                    output.paint_cull_scroll_offset = Some(output.paint.scroll_offset);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportOverflowClipMargin(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportScrollbarPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportTablePaintData(box_fragment, sources, &mut output.paint);
                }
                if box_fragment.IsTableCell() {
                    output.paint.table_cell_column =
                        Some(box_fragment.TableCellColumnIndex() as usize);
                }
                output.paint.painted_atomically = box_fragment.IsPaintedAtomically();
                output.paint.has_collapsed_borders = box_fragment.HasCollapsedBorders();
                let owner_fragment = item.PostLayoutBoxFragment();
                let owner_fragment = if owner_fragment.is_null() {
                    box_fragment as *const PhysicalBoxFragment
                } else {
                    owner_fragment
                };
                output.paint.establishes_paint_state =
                    paint_state_owners.insert(owner_fragment, output.fragment_instance_id);
                paint_state_owners
                    .instances
                    .entry(box_fragment as *const PhysicalBoxFragment)
                    .or_insert(output.fragment_instance_id);
                {
                    output.content_size = ExportContentSize(box_fragment);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportMathMLPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportFrameSetPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportFieldsetPaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportColumnRulePaintData(box_fragment, &mut output.paint);
                }
                if purpose == LayoutUpdateKind::Paint {
                    ExportGapRulePaintData(box_fragment, &mut output.paint);
                }
                if item.DescendantsCount() <= 1 {
                    let nested_items = box_fragment.Items();
                    for child in box_fragment.Children() {
                        if !child.is_present() || (!nested_items.is_null() && child.IsLineBox()) {
                            continue;
                        }
                        PopulatePhysicalFragmentPaintData(
                            child,
                            child.Offset(),
                            sources,
                            style_sources,
                            resources,
                            purpose,
                            paint_state,
                            paint_state_owners,
                            fragmentainer_instance_id,
                            item.FragmentId(),
                            next_fragment_instance_id,
                            next_fragmentainer_instance_id,
                        );
                    }
                    if !nested_items.is_null() {
                        let nested_paint_offset =
                            unsafe { &*box_fragment.GetFragmentData() }.PaintOffset();
                        let span = unsafe { &*nested_items }.Items();
                        let mut index = 0;
                        while index < span.len() {
                            index = PopulateInlineItemPaintData(
                                span,
                                unsafe { &*nested_items },
                                index,
                                span.len(),
                                PhysicalOffset::default(),
                                None,
                                None,
                                nested_paint_offset,
                                sources,
                                style_sources,
                                resources,
                                purpose,
                                paint_state,
                                paint_state_owners,
                                output.fragment_instance_id,
                                fragmentainer_instance_id,
                                next_fragment_instance_id,
                                next_fragmentainer_instance_id,
                            );
                        }
                    }
                }
                PopulateSVGChildPaintData(
                    object,
                    sources,
                    style_sources,
                    resources,
                    purpose,
                    paint_state,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    item.FragmentId(),
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
            }
        }
        ItemType::kInvalid => unreachable!("invalid inline item"),
    }
}

#[inline(never)]
fn PopulateInlineItemPaintData(
    items: &[FragmentItem],
    fragment_items: &layout_fragment::fragment_items::FragmentItems,
    index: usize,
    limit: usize,
    parent_offset: PhysicalOffset,
    containing_line_offset: Option<PhysicalOffset>,
    containing_line_height: Option<LayoutUnit>,
    container_paint_offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    container_fragment_instance_id: u64,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> usize {
    assert!(index < limit);
    let item = &items[index];
    let subtree_size = 1_usize.max(item.DescendantsCount() as usize);
    let subtree_end = limit.min(index + subtree_size);
    let line_offset = containing_line_offset
        .or_else(|| (item.Type() == ItemType::kLine).then(|| *item.OffsetInContainerFragment()));
    let line_height = containing_line_height
        .or_else(|| (item.Type() == ItemType::kLine).then(|| item.Size().height));
    PopulateInlineItemLocalPaintData(
        item,
        fragment_items,
        parent_offset,
        line_offset,
        line_height,
        container_paint_offset,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state,
        paint_state_owners,
        container_fragment_instance_id,
        fragmentainer_instance_id,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    let mut child_index = index + 1;
    while child_index < subtree_end {
        child_index = PopulateInlineItemPaintData(
            items,
            fragment_items,
            child_index,
            subtree_end,
            *item.OffsetInContainerFragment(),
            line_offset,
            line_height,
            container_paint_offset,
            sources,
            style_sources,
            resources,
            purpose,
            paint_state,
            paint_state_owners,
            container_fragment_instance_id,
            fragmentainer_instance_id,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    subtree_end
}

#[inline(never)]
pub(super) fn PopulatePhysicalFragmentPaintData(
    fragment: &PhysicalFragment,
    offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    purpose: LayoutUpdateKind,
    paint_state: &crate::layout_engine::LayoutPaintState,
    paint_state_owners: &mut FragmentOwners,
    mut fragmentainer_instance_id: u64,
    mut display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    SynchronizeBoxFragmentData(fragment);
    let object = fragment.GetLayoutObject();
    let kind = if fragment.IsBox() {
        FragmentKind::kBox
    } else {
        FragmentKind::kLine
    };
    let effective_style = if fragment.UsesFirstLineStyle() {
        fragment.Style() as *const StyleComputedStyle
    } else {
        std::ptr::null()
    };
    let fragment_data = if fragment.IsBox() {
        fragment.GetFragmentData().cast_mut()
    } else {
        std::ptr::null_mut()
    };
    let property_fragment_unique_id = if object.is_null() {
        0
    } else {
        PropertyFragmentUniqueId(object, fragment_data)
    };
    let display_item_client_id = if object.is_null() {
        0
    } else {
        unsafe { &*(object as *const DisplayItemClient) }.Id() as u64
    };
    let fragment_instance_id = paint_state_owners.fragment_instance(
        if property_fragment_unique_id != 0 {
            FragmentIdentityKey::FragmentData(property_fragment_unique_id)
        } else {
            FragmentIdentityKey::PhysicalFallback {
                client: display_item_client_id,
                fragment: display_item_fragment,
                kind: if fragment.IsBox() { 0 } else { 1 },
                preorder: paint_state_owners.population_stats.physical_fragments as u32,
            }
        },
        next_fragment_instance_id,
    );
    let retained_invalidation = RetainedPaintProperties(
        object,
        purpose,
        fragment_instance_id,
        resources,
        paint_state,
        paint_state_owners,
    );
    let store_started = browser_tracing::enabled().then(std::time::Instant::now);
    let (output, slot_retained) = paint_state.BindPhysical(fragment, fragment_instance_id);
    if let Some(started) = store_started {
        paint_state_owners.population_stats.sidecar_store_ns = paint_state_owners
            .population_stats
            .sidecar_store_ns
            .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
    }
    let output = unsafe { &mut *output };
    if let Some(invalidation) = retained_invalidation {
        assert!(
            slot_retained,
            "retained paint properties have no resident slot"
        );
        output.prepare_for_publication();
        output.paint.pre_paint_invalidation = invalidation;
        output.paint.resources = Some(Arc::clone(resources));
    } else {
        let export_started = browser_tracing::enabled().then(std::time::Instant::now);
        let properties = ExportProperties(
            object,
            fragment_data,
            sources,
            style_sources,
            effective_style,
            resources,
            purpose,
            Some(&mut paint_state_owners.exported_styles),
            Some(&mut paint_state_owners.exported_selection_styles),
            Some(&mut paint_state_owners.debug_names),
        );
        if let Some(started) = export_started {
            paint_state_owners.population_stats.property_export_ns = paint_state_owners
                .population_stats
                .property_export_ns
                .saturating_add(started.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        }
        *output = FragmentPaintData::new(properties);
    }
    output.fragment_instance_id = fragment_instance_id;
    paint_state_owners.population_stats.physical_fragments += 1;
    output.node_id = InputId(object);
    output.paint.display_item_fragment = display_item_fragment;
    output.paint.hidden = IsHiddenForPaintIncludingInlineItem(fragment)
        || output.paint.svg_viewport.is_some_and(|v| v.disabled);
    output.kind = kind;
    output.offset = Offset {
        x: Number(offset.left),
        y: Number(offset.top),
    };
    output.size = Size {
        width: Number(fragment.Size().width),
        height: Number(fragment.Size().height),
    };
    {
        output.content_size = output.size;
    }
    {
        output.fragmentainer_instance_id = fragmentainer_instance_id;
    }
    if !fragment.IsBox() {
        return;
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    output.paint.is_page_container =
        box_fragment.GetBoxType() == crate::physical_fragment::BoxType::kPageContainer;
    if output.paint.is_page_container {
        let page_border_link = box_fragment
            .Children()
            .iter()
            .find(|child| child.GetBoxType() == crate::physical_fragment::BoxType::kPageBorderBox)
            .expect("page container must contain its page border box");
        let page_border = page_border_link.get();
        assert!(!page_border.is_null());
        let page_border = unsafe { &*page_border.cast::<PhysicalBoxFragment>() };
        let border = page_border.Borders();
        let padding = page_border.Padding();
        output.paint.page_container_background_geometry =
            Some(crate::fragment_tree::PageContainerBackgroundPaintData {
                positioning_box_offset: Offset {
                    x: Number(page_border_link.offset.left),
                    y: Number(page_border_link.offset.top),
                },
                border: Edges {
                    top: Number(border.top),
                    right: Number(border.right),
                    bottom: Number(border.bottom),
                    left: Number(border.left),
                },
                padding: Edges {
                    top: Number(padding.top),
                    right: Number(padding.right),
                    bottom: Number(padding.bottom),
                    left: Number(padding.left),
                },
                ..crate::fragment_tree::PageContainerBackgroundPaintData::default()
            });
    }
    output.paint.is_page_border_box =
        box_fragment.GetBoxType() == crate::physical_fragment::BoxType::kPageBorderBox;
    if output.paint.is_page_border_box {
        let page_area = box_fragment
            .Children()
            .first()
            .expect("page border box must contain its page area")
            .get();
        assert!(!page_area.is_null());
        let page_area = unsafe { &*page_area.cast::<PhysicalBoxFragment>() };
        debug_assert_eq!(
            page_area.GetBoxType(),
            crate::physical_fragment::BoxType::kPageArea
        );
        let offset = page_area.OffsetFromRootFragmentationContext();
        output.paint.page_document_background_geometry =
            Some(crate::fragment_tree::PageBackgroundPaintData {
                positioning_area_offset: Offset {
                    x: Number(offset.left),
                    y: Number(offset.top),
                },
                ..crate::fragment_tree::PageBackgroundPaintData::default()
            });
    }
    output.paint.is_paginated_root = box_fragment.IsPaginatedRoot();
    output.paint.is_inline_formatting_context = box_fragment.IsInlineFormattingContext();
    output.paint.is_first_for_node = box_fragment.IsFirstForNode();
    output.paint.can_paint_multiple_fragments =
        crate::internal::fragmentation_utils::CanPaintMultipleFragmentsForFragment(box_fragment);
    if purpose == LayoutUpdateKind::Paint && !object.is_null() {
        let style = if effective_style.is_null() {
            unsafe { &*object }.StyleRef()
        } else {
            unsafe { &*effective_style }
        };
        ExportNinePieceImages(style, output.size, resources, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportBoxInkOverflow(box_fragment, &mut output.paint);
    }
    if box_fragment.IsFragmentainerBox() {
        fragmentainer_instance_id = *next_fragmentainer_instance_id;
        *next_fragmentainer_instance_id += 1;
        // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
        let token = box_fragment.GetBreakToken();
        display_item_fragment = if token.is_null() {
            0
        } else {
            unsafe { &*token }.SequenceNumber() + 1
        };
    } else if !object.is_null() && unsafe { &*object }.HasLayer() {
        // PaintLayerPainter scopes each physical fragment using its index in
        // the owning LayoutBox, independently of exported preorder numbering.
        // cpp: core/paint/paint_layer_painter.cc:776-778
        let mut fragments =
            crate::fragment_data_iterator::AccompaniedFragmentIterator::new(unsafe { &*object });
        let mut index = 0;
        while let Some(physical) = fragments.GetPhysicalBoxFragment() {
            if std::ptr::eq(physical, box_fragment) {
                display_item_fragment = index;
                break;
            }
            index += 1;
            fragments.Advance();
        }
    }
    output.paint.display_item_fragment = display_item_fragment;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    {
        ExportBoxPaintGeometry(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        let outline_style = if effective_style.is_null() {
            box_fragment.Style()
        } else {
            unsafe { &*effective_style }
        };
        ExportSelfOutlineGeometry(box_fragment, outline_style, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint && output.paint.scroll_container.is_some() {
        output.paint_cull_scroll_offset = Some(output.paint.scroll_offset);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportOverflowClipMargin(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportScrollbarPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportTablePaintData(box_fragment, sources, &mut output.paint);
    }
    if box_fragment.IsTableCell() {
        output.paint.table_cell_column = Some(box_fragment.TableCellColumnIndex() as usize);
    }
    output.paint.painted_atomically = fragment.IsPaintedAtomically();
    output.paint.has_collapsed_borders = fragment.HasCollapsedBorders();
    output.paint.establishes_paint_state = paint_state_owners.insert(
        box_fragment as *const PhysicalBoxFragment,
        output.fragment_instance_id,
    );
    {
        output.content_size = ExportContentSize(box_fragment);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportMathMLPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportFrameSetPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportFieldsetPaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportColumnRulePaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportGapRulePaintData(box_fragment, &mut output.paint);
    }
    if purpose == LayoutUpdateKind::Paint {
        ExportCollapsedTablePaintData(box_fragment, style_sources, &mut output.paint);
    }
    let items = box_fragment.Items();
    for child in box_fragment.Children() {
        if !child.is_present() || (!items.is_null() && child.IsLineBox()) {
            continue;
        }
        PopulatePhysicalFragmentPaintData(
            child,
            child.Offset(),
            sources,
            style_sources,
            resources,
            purpose,
            paint_state,
            paint_state_owners,
            fragmentainer_instance_id,
            display_item_fragment,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    if !items.is_null() {
        let container_paint_offset = if fragment_data.is_null() {
            PhysicalOffset::default()
        } else {
            unsafe { &*fragment_data }.PaintOffset()
        };
        let span = unsafe { &*items }.Items();
        let mut index = 0;
        while index < span.len() {
            index = PopulateInlineItemPaintData(
                span,
                unsafe { &*items },
                index,
                span.len(),
                PhysicalOffset::default(),
                None,
                None,
                container_paint_offset,
                sources,
                style_sources,
                resources,
                purpose,
                paint_state,
                paint_state_owners,
                output.fragment_instance_id,
                fragmentainer_instance_id,
                next_fragment_instance_id,
                next_fragmentainer_instance_id,
            );
        }
    }
    PopulateSVGChildPaintData(
        object,
        sources,
        style_sources,
        resources,
        purpose,
        paint_state,
        paint_state_owners,
        fragmentainer_instance_id,
        display_item_fragment,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
}
