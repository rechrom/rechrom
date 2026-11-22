//! Chromium tree preparation owned by the invalidation package.
//!
//! This file is included at the reciprocal assembly boundary.  Keep the
//! coarse transaction and its distinct invalidation phases here so traces and
//! sampling profiles attribute preparation costs to `layout_invalidation`
//! instead of the generic source boundary.

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:2648-2709
fn rebind_font(
    style: &StyleComputedStyle,
    parent: *const LayoutObject,
) -> *const StyleComputedStyle {
    let mut builder = ComputedStyleBuilder::from_style(style);
    let description = style.GetFontDescription();
    let mut resolved_font_size = f64::from(style.ComputedFontSize());
    if style.MathDepth() > 0 && !parent.is_null() {
        let parent_style = unsafe { &*parent }.StyleRef();
        let parent_font = parent_style.GetFont();
        if !parent_font.is_null() {
            let parent_depth = parent_style.MathDepth() as i32;
            let mut scale = 1.0;
            let math_depth = style.MathDepth() as i32;
            if math_depth > parent_depth {
                let mut script_scale = 0.71;
                let mut script_script_scale = script_scale * script_scale;
                let data = unsafe { &*parent_font }.PrimaryFont();
                if !data.is_null() {
                    let face = unsafe { &*data }.PlatformData().GetHarfBuzzFace();
                    script_scale = OpenTypeMathSupport::MathConstant(
                        face,
                        MathConstants::kScriptPercentScaleDown,
                    )
                    .map(|value| value as f64)
                    .unwrap_or(script_scale);
                    if !(script_scale > 0.0) {
                        script_scale = 0.71;
                    }
                    script_script_scale = OpenTypeMathSupport::MathConstant(
                        face,
                        MathConstants::kScriptScriptPercentScaleDown,
                    )
                    .map(|value| value as f64)
                    .unwrap_or(script_script_scale);
                    if !(script_script_scale > 0.0) {
                        script_script_scale = script_scale * script_scale;
                    }
                }
                let scale_for_depth = |depth: i32| {
                    if depth <= 0 {
                        1.0
                    } else if depth == 1 {
                        script_scale
                    } else {
                        script_script_scale * 0.71_f64.powi(depth - 2)
                    }
                };
                scale = scale_for_depth(math_depth) / scale_for_depth(parent_depth);
            }
            resolved_font_size = parent_style.ComputedFontSize() as f64 * scale;
        }
    }
    let mut families = Vec::new();
    let mut family: *const font_engine::FontFamily = description.Family();
    while !family.is_null() && !unsafe { &*family }.FamilyName().IsNull() {
        let current = unsafe { &*family };
        families.push(current.FamilyName().Utf8());
        family = current.Next();
    }
    let language = description.LocaleOrDefault().Ascii();
    let mut request = FontRequest::new_auto(
        resolved_font_size,
        f64::from(description.LetterSpacing()),
        f64::from(description.WordSpacing()),
        PublicWritingMode(builder.GetWritingMode()),
        &language,
        &families,
        description.Weight().RawValue() as f64 / 4.0,
        description.Style().RawValue() >= 14 * 4,
        builder.ComputeFontOrientation(),
    )
    .WithBaseDescription(description);
    request.specified_size = resolved_font_size / f64::from(style.EffectiveZoom());
    let font = unsafe { &mut *CurrentFontResolver() }.Resolve(&request);
    builder.SetFont(Member::from_ptr(font));
    builder.TakeStyle()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2607-2647
// cpp: layoutng/internal/boundary/layout_boundary.cc:2710-2752
#[inline(never)]
fn rebind_targeted_font(object: *mut LayoutObject) -> bool {
    let current = unsafe { &mut *object };
    let node = current.GetNode();
    debug_assert!(!node.is_null());
    let previous_font = unsafe { &*node }.InputBoundFont();
    if unsafe { &*node }.IsTextNode() {
        let mut logical_parent = unsafe { &*node }.parentNode() as *mut Node;
        while !logical_parent.is_null() && unsafe { &*logical_parent }.GetLayoutObject().is_null() {
            logical_parent = unsafe { &*logical_parent }.parentNode() as *mut Node;
        }
        if !logical_parent.is_null() {
            let layout_parent = unsafe { &mut *unsafe { &*logical_parent }.GetLayoutObject() };
            let style = if layout_parent.IsInitialLetterBox() {
                InitialLetterTextStyle(layout_parent)
            } else {
                layout_parent.StyleRef() as *const StyleComputedStyle
            };
            unsafe { &mut *node }.SetComputedStyle(style);
            let actual_parent = current.Parent();
            let object_style =
                if !actual_parent.is_null() && unsafe { &*actual_parent }.IsLayoutTextCombine() {
                    unsafe { &*actual_parent }.StyleRef() as *const StyleComputedStyle
                } else {
                    style
                };
            current.SetStyleWithChanges(object_style, ApplyStyleChanges::kNo);
            let text = unsafe { &mut *To::<LayoutText>(object) };
            text.InvalidateInlineItems();
            text.SetNeedsCollectInlines();
            text.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
                &raw const kFontsChanged,
            );
        }
    } else {
        let style = rebind_font(current.StyleRef(), current.Parent());
        unsafe { &mut *node }.SetComputedStyle(style);
        current.SetStyleWithChanges(style, ApplyStyleChanges::kNo);
        let first_line_style = unsafe { &*node }.InputFirstLineStyle();
        if !first_line_style.is_null() {
            let rebound = rebind_font(unsafe { &*first_line_style }, current.Parent());
            unsafe { &mut *node }.SetInputFirstLineStyle(rebound);
        }
        current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
    }
    let resolved_font = current.StyleRef().GetFont() as *mut font_engine::fonts::font::Font;
    unsafe { &mut *node }.SetInputBoundFont(resolved_font);
    unsafe { &mut *node }.ClearInputFontBinding();
    resolved_font != previous_font
}

#[inline(never)]
fn rebind_fonts_from_constraint_space(root: &mut LayoutObject, only_dirty: bool) {
    let mut rebound = std::collections::HashSet::new();
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &mut *object };
        // A new font environment invalidates the old shaping generation, even
        // when CSS and text are unchanged. Previous InlineItems refer to their
        // LayoutObject's current style, so rebinding first would make an old
        // fallback result appear reusable with the newly loaded face.
        // Blink invalidates these results through FontSelector's version.
        if !only_dirty {
            let flow = DynamicTo::<LayoutBlockFlow>(object);
            if !flow.is_null() {
                unsafe { &mut *flow }.ClearInlineNodeData();
            }
        }
        let node = current.GetNode();
        // A parent's resolved font can change even when the child's source
        // style is unchanged (for example relative MathML font sizes).
        let inherited_font_changed = rebound.contains(&current.Parent());
        if only_dirty
            && !node.is_null()
            && !unsafe { &*node }.NeedsInputFontBinding()
            && !inherited_font_changed
        {
            object = current.NextInPreOrder(root_ptr);
            continue;
        }
        let previous_font = if node.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*node }.InputBoundFont()
        };
        if node.is_null() {
            let parent = current.Parent();
            if !parent.is_null() {
                let style = if current.IsLayoutTextCombine() {
                    rebind_font(current.StyleRef(), parent)
                } else {
                    let font = unsafe { &*parent }.StyleRef().GetFont();
                    if font.is_null() || (only_dirty && current.StyleRef().GetFont() == font) {
                        std::ptr::null()
                    } else {
                        let mut builder = ComputedStyleBuilder::from_style(current.StyleRef());
                        builder.SetFont(Member::from_ptr(font));
                        builder.TakeStyle()
                    }
                };
                if !style.is_null() {
                    current.SetStyleWithChanges(style, ApplyStyleChanges::kNo);
                    current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
                    rebound.insert(object);
                }
            }
        } else if unsafe { &*node }.IsTextNode() {
            let mut logical_parent = unsafe { &*node }.parentNode() as *mut Node;
            while !logical_parent.is_null()
                && unsafe { &*logical_parent }.GetLayoutObject().is_null()
            {
                logical_parent = unsafe { &*logical_parent }.parentNode() as *mut Node;
            }
            if !logical_parent.is_null() {
                let layout_parent = unsafe { &mut *unsafe { &*logical_parent }.GetLayoutObject() };
                let style = if layout_parent.IsInitialLetterBox() {
                    InitialLetterTextStyle(layout_parent)
                } else {
                    layout_parent.StyleRef() as *const StyleComputedStyle
                };
                unsafe { &mut *node }.SetComputedStyle(style);
                let actual_parent = current.Parent();
                let object_style = if !actual_parent.is_null()
                    && unsafe { &*actual_parent }.IsLayoutTextCombine()
                {
                    unsafe { &*actual_parent }.StyleRef() as *const StyleComputedStyle
                } else {
                    style
                };
                current.SetStyleWithChanges(object_style, ApplyStyleChanges::kNo);
                let text = To::<LayoutText>(object);
                let text = unsafe { &mut *text };
                text.InvalidateInlineItems();
                text.SetNeedsCollectInlines();
                // Font completion also changes glyph paint even when the
                // text and its geometry are unchanged (LayoutText font updates).
                text.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
                    &raw const kFontsChanged,
                );
            }
        } else {
            let style = rebind_font(current.StyleRef(), current.Parent());
            unsafe { &mut *node }.SetComputedStyle(style);
            current.SetStyleWithChanges(style, ApplyStyleChanges::kNo);
            let first_line_style = unsafe { &*node }.InputFirstLineStyle();
            if !first_line_style.is_null() {
                let rebound = rebind_font(unsafe { &*first_line_style }, current.Parent());
                unsafe { &mut *node }.SetInputFirstLineStyle(rebound);
            }
            current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
        }
        if !node.is_null() {
            let resolved_font = current.StyleRef().GetFont() as *mut font_engine::fonts::font::Font;
            // A dirty source node can change width/height while retaining its
            // exact resolved font request. Propagate typography only when the
            // actual binding changes; the node's own layout/style dirtiness
            // and text invalidation remain intact.
            if resolved_font != previous_font {
                rebound.insert(object);
            }
            unsafe { &mut *node }.SetInputBoundFont(resolved_font);
            unsafe { &mut *node }.ClearInputFontBinding();
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &mut *object };
        if current.GetNode().is_null() {
            let owner = current.InputOwnerForLayout();
            let owner_object = owner.GetLayoutObject();
            if !owner_object.is_null() {
                let font = unsafe { &*owner_object }.StyleRef().GetFont();
                if !font.is_null() && (!only_dirty || current.StyleRef().GetFont() != font) {
                    let mut builder = ComputedStyleBuilder::from_style(current.StyleRef());
                    builder.SetFont(Member::from_ptr(font));
                    current.SetStyleWithChanges(builder.TakeStyle(), ApplyStyleChanges::kNo);
                    current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
                }
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &*object };
        let text = DynamicTo::<LayoutText>(object);
        if !text.is_null() && (!only_dirty || rebound.contains(&object)) {
            unsafe { &mut *text }.TransformAndSecureOriginalText();
        }
        object = current.NextInPreOrder(root_ptr);
    }
}

#[inline(never)]
pub(super) fn prepare_layout_object_tree(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    layout_space: &LayoutConstraintSpace,
    reuse_prepared_fonts: bool,
    relations_dirty: bool,
    shape_image_inputs_dirty: bool,
    document_image_container_dirty: bool,
    targeted_font_objects: Option<&[*mut LayoutObject]>,
) {
    if !root.Parent().is_null() {
        panic!("Layout input must reference the tree root");
    }
    let root_node = root.GetNode();
    if root_node.is_null() {
        panic!("Layout root requires standalone metadata");
    }

    if relations_dirty {
        let _phase =
            layout_invalidation::trace_span(layout_invalidation::TracePhase::FinalizeRelations);
        NodeMetadataRelations::FinalizeTreeRelations(unsafe { &mut *root_node })
            .unwrap_or_else(|error| panic!("{error:?}"));
    }

    if space.printing {
        let view = DynamicTo::<LayoutView>(root as *mut LayoutObject);
        if view.is_null() {
            panic!("Print layout root must be a LayoutView");
        }
        unsafe { &mut *view }.SetInitialContainingBlockSizeForPrinting(ToPhysicalSize(
            layout_space.AvailableSize(),
            layout_space.GetWritingMode(),
        ));
    }
    {
        let _phase = layout_invalidation::trace_span(layout_invalidation::TracePhase::RebindFonts);
        if reuse_prepared_fonts {
            if let Some(objects) = targeted_font_objects {
                let font_changed = objects.iter().copied().any(rebind_targeted_font);
                if font_changed {
                    // A changed inherited binding can affect descendants and
                    // anonymous owners outside the targeted style update. The
                    // rare font-changing path takes the complete Chromium
                    // invalidation walk; ordinary geometry styles stay local.
                    rebind_fonts_from_constraint_space(root, false);
                }
            } else {
                rebind_fonts_from_constraint_space(root, true);
            }
        } else {
            rebind_fonts_from_constraint_space(root, false);
        }
    }

    let root_ptr: *mut LayoutObject = root;
    let viewport_defining_element = unsafe { &*root_node }.InputViewportDefiningElement();
    let has_first_line_styles = if relations_dirty {
        let mut has_first_line_styles = false;
        let mut object = root_ptr;
        while !object.is_null() {
            let current = unsafe { &*object };
            let node = current.GetNode();
            if !node.is_null() {
                has_first_line_styles |= !unsafe { &*node }.InputFirstLineStyle().is_null();
            }
            object = current.NextInPreOrder(root_ptr);
        }
        has_first_line_styles
    } else {
        unsafe { &*root_node }.InputHasFirstLineStyles()
    };
    if relations_dirty || shape_image_inputs_dirty || document_image_container_dirty {
        let _phase =
            layout_invalidation::trace_span(layout_invalidation::TracePhase::RefreshImageInputs);
        let mut object = root_ptr;
        while !object.is_null() {
            let current = unsafe { &*object };
            if relations_dirty || shape_image_inputs_dirty {
                let box_ptr = DynamicTo::<LayoutBox>(object);
                if !box_ptr.is_null() {
                    let shape = ShapeOutsideInfo::Info(unsafe { &*box_ptr });
                    if !shape.is_null() {
                        unsafe { &mut *shape }.MarkShapeAsDirty();
                        // ShapeOutsideInfo observes decoded image content.
                        // Chromium invalidates it when that resource changes;
                        // an unrelated viewport constraint update retains it.
                        unsafe { &mut *box_ptr }
                            .SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kStyleChange);
                    }
                }
            }
            let node = current.GetNode();
            let next = current.NextInPreOrder(root_ptr);
            if (relations_dirty || document_image_container_dirty) && current.IsSVGRoot() {
                let size = space
                    .document_image_container
                    .filter(|container| {
                        !node.is_null() && unsafe { &*node }.InputId() == container.root_node_id
                    })
                    .map_or(PhysicalSize::default(), |container| {
                        PhysicalSize::new(
                            LayoutUnit::from_f64(container.size.width),
                            LayoutUnit::from_f64(container.size.height),
                        )
                    });
                let algorithms = LayoutPassScope::Algorithms();
                let set_container = unsafe { &*algorithms }
                    .svg_support
                    .set_root_container_size
                    .expect("SVG layout module has a root container setter");
                set_container(unsafe { &mut *object }, size);
                let scale = space
                    .document_image_container
                    .filter(|container| {
                        !node.is_null() && unsafe { &*node }.InputId() == container.root_node_id
                    })
                    .map_or([1.0, 1.0], |container| container.scale);
                unsafe { &*algorithms }
                    .svg_support
                    .set_root_container_scale
                    .expect("SVG layout module has a root container-scale setter")(
                    unsafe { &mut *object },
                    foundation::gfx::Vector2dF::new(scale[0], scale[1]),
                );
            }
            object = next;
        }
    }
    unsafe { &mut *root_node }.PrepareStandaloneLayout(
        space,
        viewport_defining_element,
        has_first_line_styles,
    );
    // PaintLayer::StyleDidChange normally sees live Document/Frame settings.
    // This extracted boundary installs their immutable equivalents only above,
    // so replay any scrollable-area style lifecycle deferred during object
    // construction before layout starts.
    {
        let mut object = root_ptr;
        while !object.is_null() {
            let current = unsafe { &*object };
            let box_ = DynamicTo::<LayoutBox>(object);
            if !box_.is_null() {
                let area = unsafe { &*box_ }.GetScrollableArea();
                if !area.is_null() {
                    unsafe { &mut *area }.RunPendingUpdateAfterStyleChange();
                }
            }
            object = current.NextInPreOrder(root_ptr);
        }
    }
    if relations_dirty {
        let _phase = layout_invalidation::trace_span(
            layout_invalidation::TracePhase::PrepareInlineFragments,
        );
        let mut object = root_ptr;
        while !object.is_null() {
            let current = unsafe { &*object };
            let layout_inline = DynamicTo::<LayoutInline>(object);
            let node = current.GetNode();
            if !layout_inline.is_null() && !node.is_null() {
                let style = current.StyleRef();
                let first_line_style = current.FirstLineStyleRef();
                let needs_fragment = ComputedStyleRequiresPaintBackedInlineFragment(style)
                    || (!std::ptr::eq(style, first_line_style)
                        && ComputedStyleRequiresPaintBackedInlineFragment(first_line_style));
                if needs_fragment {
                    let layout_inline = unsafe { &mut *layout_inline };
                    layout_inline.SetIsInLayoutNGInlineFormattingContext(true);
                    layout_inline.SetShouldCreateBoxFragmentDefault();
                }
            }
            object = current.NextInPreOrder(root_ptr);
        }
    }
}
