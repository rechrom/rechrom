//! Fragment paint-property tree construction.
//!
//! Source: Chromium revision 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! `third_party/blink/renderer/core/paint/paint_property_tree_builder.{h,cc}`.

use super::*;
use crate::compositing_reason_finder::CompositingReasonFinder;

// Source: core/paint/paint_property_tree_builder.cc:499-530. The Layout
// boundary materializes ScrollContainerPaintData from the source predicate,
// including root-scroller, non-zero-position and non-propagating cases.
fn NeedsScrollAndScrollTranslation(fragment: &FragmentPaintData) -> bool {
    fragment.paint.IsOverscrollContainer() || fragment.paint.scroll_container.is_some()
}

// Source: core/paint/paint_property_tree_builder.cc,
// NeedsEffectIgnoringClipPathAnd2DScale(). Isolation establishes the boundary
// for descendant blending, but does not by itself require an Effect/saveLayer.
fn NeedsCssBlendEffect(
    blend_mode: PaintBlendMode,
    is_paint_layer: bool,
    is_stacking_context: bool,
    has_non_isolated_blending_descendant: bool,
) -> bool {
    blend_mode != PaintBlendMode::kNormal
        || (is_paint_layer && is_stacking_context && has_non_isolated_blending_descendant)
}

pub(super) struct PaintPropertyTreeBuilder<'a> {
    walk: &'a mut PrePaintTreeWalk,
    previous_paint_result: paint_model::PaintResult,
}

impl<'a> PaintPropertyTreeBuilder<'a> {
    pub(super) fn new(walk: &'a mut PrePaintTreeWalk) -> Self {
        Self {
            walk,
            previous_paint_result: paint_model::PaintResult::kMayBeClippedByCullRect,
        }
    }

    pub(super) fn with_previous_paint_result(
        mut self,
        previous_paint_result: paint_model::PaintResult,
    ) -> Self {
        self.previous_paint_result = previous_paint_result;
        self
    }

    // Source: core/paint/paint_property_tree_builder.{h,cc}.
    pub(super) fn BuildNode(
        &mut self,
        fragment: &FragmentPaintData,
        parent_offset: Offset,
        paint_space_origin: Offset,
        parent_clip: Option<PaintRect>,
        parent_cull_rect: Option<PaintRect>,
        transforms: Vec<TransformMatrix>,
        parent_opacity: f32,
        scroll_offset: Offset,
        inherited_fixed_background_acts_as_scroll: bool,
        parent_properties: PropertyTreeState,
        canvas_contents: Option<CanvasContentsContext>,
        inherited_rendering_context_id: u64,
        inherited_should_flatten_transform: bool,
        inherited_can_inherit_backface: bool,
        inherited_directly_composited_subpixel_delta: Offset,
        inherited_subtree_should_use_infinite_cull_rect: bool,
        old_paint_offset: Option<Offset>,
        old_had_paint_offset_translation: bool,
        old_self_cull_rect: Option<Option<PaintRect>>,
        old_contents_cull_rect: Option<Option<PaintRect>>,
        is_root: bool,
        is_flex_or_grid_item: bool,
    ) -> PaintTreeNode<'static> {
        self.walk.stats.computed_nodes += 1;
        fragment
            .paint
            .UpdateSourcePaintLayerDescendantDependentFlags();
        let own_key_start = self.walk.next_property_id;
        let inherited = InheritedContext {
            inside_blocking_wheel_event_handler: false,
            blocking_wheel_event_handler_changed: false,
            offset: parent_offset,
            paint_space: paint_space_origin,
            clip: parent_clip,
            cull: parent_cull_rect,
            transforms: transforms.clone(),
            opacity: parent_opacity,
            scroll: scroll_offset,
            fixed_background_acts_as_scroll: inherited_fixed_background_acts_as_scroll,
            properties: parent_properties.clone(),
            canvas_contents: canvas_contents.clone(),
            rendering_context_id: inherited_rendering_context_id,
            should_flatten_inherited_transform: inherited_should_flatten_transform,
            can_inherit_backface_visibility: inherited_can_inherit_backface,
            directly_composited_container_paint_offset_subpixel_delta:
                inherited_directly_composited_subpixel_delta,
            subtree_should_use_infinite_cull_rect: inherited_subtree_should_use_infinite_cull_rect,
            root: is_root,
            flex_or_grid_item: is_flex_or_grid_item,
        };
        let mut node = PaintTreeNode::default();
        node.SetFragment(fragment);
        node.is_root = is_root;
        let local_establishes_fixed_background_containing_block = !is_root
            && fragment.paint.establishes_paint_state
            && fragment.paint.HasSource()
            && (fragment.paint.style.has_transform_payload()
                || fragment.paint.style.will_change_transform());
        node.fixed_background_acts_as_scroll = inherited_fixed_background_acts_as_scroll
            || local_establishes_fixed_background_containing_block;
        let inherited_transform_count = transforms.len();
        let mut transforms = transforms;
        let geometry_property_delta =
            FlatToPropertySpaceOffset(&transforms, &parent_properties.nodes.transform)
                .unwrap_or_default();
        let mut geometry = ResolveFragmentGeometry(
            fragment,
            parent_offset,
            paint_space_origin,
            &mut transforms,
            geometry_property_delta,
            inherited.directly_composited_container_paint_offset_subpixel_delta,
        );
        if fragment.paint.HasSource()
            && HasSupportedDirectCompositingReasons(fragment, &fragment.paint.style)
        {
            geometry.directly_composited_container_paint_offset_subpixel_delta =
                if geometry.applies_paint_offset_translation && old_had_paint_offset_translation {
                    old_paint_offset.map_or_else(Offset::default, |old| Offset {
                        x: geometry.paint_offset.x - old.x,
                        y: geometry.paint_offset.y - old.y,
                    })
                } else {
                    Offset::default()
                };
        }
        let paint_space_origin = geometry.paint_space_origin;
        node.absolute_offset = geometry.absolute_offset;
        node.paint_offset = geometry.paint_offset;
        // cpp: FragmentPaintPropertyTreeBuilder::UpdateForObjectLocation.
        // An ancestor can move this object without dirtying its local Layout
        // geometry. Admit paint invalidation after resolving the inherited
        // offset, before the invalidator decides whether to reuse its drawing.
        if old_paint_offset.is_some_and(|old| old != node.paint_offset) {
            node.fragment
                .as_mut()
                .expect("property builder must retain its fragment")
                .paint
                .pre_paint_invalidation
                .should_check_for_paint_invalidation = true;
        }
        node.directly_composited_container_paint_offset_subpixel_delta =
            geometry.directly_composited_container_paint_offset_subpixel_delta;
        node.pixel_snap_scale = fragment
            .paint
            .resources
            .as_ref()
            .map_or(1.0, |resources| resources.ArtifactToDeviceScale())
            .max(f64::MIN_POSITIVE);
        node.clip = parent_clip;
        node.cull_rect = parent_cull_rect;
        node.self_cull_rect = parent_cull_rect;
        node.transforms = transforms;
        node.scroll_offset = scroll_offset;
        node.properties = parent_properties.clone();
        let inherited_tainted_subtree = parent_properties
            .nodes
            .effect
            .compositor
            .is_in_tainted_subtree;
        let is_in_drawable_canvas_subtree = parent_properties
            .nodes
            .effect
            .compositor
            .is_in_drawable_canvas_subtree
            || CompositingReasonFinder::DirectReasonsForPaintProperties(fragment)
                .Has(foundation::CompositingReason::kCanvasChild);
        node.applies_clip = is_root && parent_clip.is_some();
        node.applies_root_clip = node.applies_clip;
        if node.applies_root_clip {
            node.root_clip = parent_clip;
        }

        let style: Option<&PaintStyle> =
            fragment.paint.HasSource().then_some(&fragment.paint.style);
        let establishes = fragment.paint.establishes_paint_state;
        let authored_filters = style
            .filter(|_| establishes)
            .map(PaintStyle::filters)
            .unwrap_or_default();
        let authored_backdrop_filters = style
            .filter(|_| establishes)
            .map(PaintStyle::backdrop_filters)
            .unwrap_or_default();
        let backdrop_filter_has_reference = authored_backdrop_filters.iter().any(|filter| {
            filter.r#type == PaintFilterType::kReference && filter.reference_graph.is_none()
        });
        let authored_mask_images = style
            .filter(|_| establishes)
            .map(PaintStyle::mask_images)
            .unwrap_or_default();
        node.opacity = parent_opacity
            * if establishes {
                style.map_or(1.0, MaskAdjustedOpacity)
            } else {
                1.0
            };
        let position = if fragment.paint.HasSource() {
            fragment.paint.position
        } else {
            Position::kStatic
        };
        let layer = ClassifyFragmentPaintLayer(fragment, is_root, is_flex_or_grid_item);
        node.stacking_level = layer.stacking_level;
        node.is_paint_layer = layer.is_paint_layer;
        node.paint_layer_facts_are_authoritative = fragment.paint.PaintLayerStateIsAuthoritative();
        node.is_self_painting_layer = if fragment.paint.PaintLayerStateIsAuthoritative() {
            fragment.paint.PaintLayerIsSelfPainting()
        } else {
            layer.is_paint_layer
        };
        node.invisible_for_position_visibility =
            fragment.paint.PaintLayerInvisibleForPositionVisibility();
        node.has_ancestor_invisible_for_position_visibility = fragment
            .paint
            .PaintLayerHasAncestorInvisibleForPositionVisibility();
        node.has_visible_content = fragment.paint.PaintLayerHasVisibleContent();
        node.has_self_painting_layer_descendant =
            fragment.paint.PaintLayerHasSelfPaintingLayerDescendant();
        node.is_under_svg_hidden_container = fragment.paint.PaintLayerIsUnderSvgHiddenContainer();
        node.supports_subsequence_caching = fragment.paint.PaintLayerSupportsSubsequenceCaching();
        node.self_or_descendant_needs_repaint =
            fragment.paint.PaintLayerSelfOrDescendantNeedsRepaint();
        node.paint_layer_is_fragmented = fragment.paint.PaintLayerIsFragmented();
        node.known_to_clip_subtree_to_padding_box =
            fragment.paint.PaintLayerKnownToClipSubtreeToPaddingBox();
        node.child_paint_blocked_by_display_lock = fragment.paint.ChildPaintBlockedByDisplayLock();
        node.is_stacking_context = layer.is_stacking_context;
        node.is_stacked = layer.is_stacked;
        node.is_z_order_list_visible = fragment.paint.PaintLayerIsZOrderListVisible();
        node.is_in_top_or_view_transition_layer = fragment
            .paint
            .paint_layer_is_in_top_or_view_transition_layer;
        node.is_direct_layout_view_child = fragment.paint.paint_layer_is_direct_layout_view_child;
        node.can_contain_absolute_position_objects = fragment
            .paint
            .paint_layer_can_contain_absolute_position_objects;
        node.can_contain_fixed_position_objects = fragment
            .paint
            .paint_layer_can_contain_fixed_position_objects;
        node.has_overlay_overflow_controls =
            fragment.paint.paint_layer_has_overlay_overflow_controls;
        node.paint_layer_order_parent_unique_id = fragment.paint.paint_layer_order_parent_unique_id;
        node.paint_layer_order = fragment.paint.paint_layer_order;

        let mut has_active_transform_animation = false;
        let mut has_non_composited_transform = false;
        let mut pre_transform = None;
        let canvas_transform = fragment.paint.ElementCanvasTransformInput();
        if geometry.applies_transform || canvas_transform.is_some() {
            node.properties.transform_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            node.applies_transform = true;
            node.local_transforms
                .extend_from_slice(&node.transforms[inherited_transform_count..]);
            // Distinct source roles retain their identities when a preceding
            // sticky/viewport translation is added or removed. Assigning slots
            // by flat replay index would accidentally recycle another role.
            let mut source_roles = Vec::new();
            if geometry.applies_paint_offset_translation {
                source_roles.push(PaintPropertyRole::PaintOffsetTranslation);
            }
            if position == Position::kSticky
                && (fragment.paint.sticky_constraint.is_some()
                    || fragment.paint.sticky_offset.x != 0.0
                    || fragment.paint.sticky_offset.y != 0.0)
            {
                source_roles.push(PaintPropertyRole::StickyTranslation);
            }
            if fragment.paint.svg_local_transform.is_none() {
                if let Some(style) = style {
                    source_roles.extend(TransformPropertyRoles(fragment, style));
                }
            }
            if fragment.paint.svg_local_transform.is_some() {
                source_roles.push(PaintPropertyRole::SvgLocalTransform);
            }
            if fragment
                .paint
                .svg_shape
                .as_ref()
                .is_some_and(|shape| shape.local_transform.is_some())
            {
                // The source SVG affine matrix is a distinct existing replay
                // input. Mixed CSS+SVG transform lowering is still adapted;
                // it is not claimed to match LocalToSVGParentTransform fully.
                source_roles.push(PaintPropertyRole::SvgLocalTransform);
            }
            if fragment.paint.text_combine.is_some() {
                source_roles.push(PaintPropertyRole::TextCombineTransform);
            }
            debug_assert_eq!(source_roles.len(), node.local_transforms.len());
            let mut operations: Vec<_> = source_roles
                .into_iter()
                .zip(node.local_transforms.iter().copied())
                .collect();
            if let Some((_, matrix)) = canvas_transform {
                let index = operations
                    .iter()
                    .take_while(|(role, _)| {
                        matches!(
                            role,
                            PaintPropertyRole::PaintOffsetTranslation
                                | PaintPropertyRole::StickyTranslation
                        )
                    })
                    .count();
                operations.insert(index, (PaintPropertyRole::ElementCanvasTransform, matrix));
            }
            let mut flat_transform_context = node.transforms[..inherited_transform_count].to_vec();
            let mut current_rendering_context_id = inherited_rendering_context_id;
            for (index, (role, matrix)) in operations.iter().enumerate() {
                // UpdatePaintOffsetTranslation/UpdateTransform establish a
                // local paint space. Express this reset matrix in its actual
                // property-parent space, so P_parent*M_property
                // equals the existing F_parent*M_flat. Constant sticky/SVG
                // local/text-combine matrices and will-change identity slots
                // do not reset paint_offset and keep their scroll anchor.
                let resets_paint_space = *role == PaintPropertyRole::PaintOffsetTranslation;
                let property_matrix = if resets_paint_space {
                    FlatToPropertySpaceOffset(
                        &flat_transform_context,
                        &node.properties.nodes.transform,
                    )
                    .map(|delta| {
                        MultiplyTransforms(&TranslationTransform(delta.x, delta.y), matrix)
                    })
                    .unwrap_or(*matrix)
                } else {
                    *matrix
                };
                let id = if index == 0 {
                    node.properties.transform_id
                } else {
                    let id = self.walk.next_property_id;
                    self.walk.next_property_id += 1;
                    id
                };
                let sticky_constraint = (*role == PaintPropertyRole::StickyTranslation)
                    .then_some(fragment.paint.sticky_constraint)
                    .flatten();
                let mut parent_transform = node.properties.nodes.transform.clone();
                if *role == PaintPropertyRole::ElementCanvasTransform {
                    if let Some(native_canvas) = canvas_transform.and_then(|(canvas, _)| canvas) {
                        let canvas = canvas_contents.as_ref().filter(|context| context.layout_object == native_canvas)
                            .expect("canvas FirstFragment ContentsProperties precede its drawable descendants");
                        parent_transform = canvas.properties.nodes.transform.clone();
                        node.properties.nodes.clip = canvas.properties.nodes.clip.clone();
                        node.properties.clip_id = canvas.properties.clip_id;
                    }
                }
                let is_css_transform_property = matches!(
                    role,
                    PaintPropertyRole::Transform
                        | PaintPropertyRole::Offset
                        | PaintPropertyRole::Scale
                        | PaintPropertyRole::Rotate
                        | PaintPropertyRole::Translate
                );
                if is_css_transform_property && pre_transform.is_none() {
                    pre_transform = Some(parent_transform.clone());
                }
                let mut compositor = TransformCompositorState(fragment, *role);
                if let Some(sticky) = sticky_constraint {
                    compositor.sticky_constraint = BuildCompositorStickyConstraint(
                        sticky,
                        &parent_transform,
                        geometry.sticky_pixel_snap_offset,
                    );
                }
                let reasons = CompositingReasonFinder::TransformReasons(
                    fragment,
                    *role,
                    compositor.sticky_constraint.is_some(),
                );
                if matches!(
                    role,
                    PaintPropertyRole::Transform
                        | PaintPropertyRole::Offset
                        | PaintPropertyRole::Scale
                        | PaintPropertyRole::Rotate
                        | PaintPropertyRole::Translate
                ) {
                    has_non_composited_transform |= reasons.IsEmpty();
                    has_active_transform_animation |= [
                        foundation::CompositingReason::kActiveTransformAnimation,
                        foundation::CompositingReason::kActiveScaleAnimation,
                        foundation::CompositingReason::kActiveRotateAnimation,
                        foundation::CompositingReason::kActiveTranslateAnimation,
                    ]
                    .into_iter()
                    .any(|reason| reasons.Has(reason));
                }
                let is_css_transform = *role == PaintPropertyRole::Transform;
                let style_3d = style.filter(|_| is_css_transform);
                let establishes_3d_context = style_3d
                    .is_some_and(|style| style.preserves_3d() && current_rendering_context_id == 0);
                let rendering_context_id = if establishes_3d_context {
                    id
                } else {
                    current_rendering_context_id
                };
                let backface_visibility =
                    if style_3d.is_some_and(|style| style.backface_visibility_hidden()) {
                        PaintBackfaceVisibility::Hidden
                    } else if style_3d.is_some_and(|style| {
                        establishes_3d_context
                            || !inherited_can_inherit_backface
                            || style.has_3d_transform_operation()
                    }) {
                        PaintBackfaceVisibility::Visible
                    } else {
                        PaintBackfaceVisibility::Inherited
                    };
                self.walk.RegisterProperty(fragment, *role, id);
                let transform_node = Arc::new(TransformPaintPropertyNode {
                    compositor,
                    geometry_mapper_cache: Default::default(),
                    scroll_parent_scroll_translation: if *role
                        == PaintPropertyRole::PaintOffsetTranslation
                        && fragment.paint.fixed_to_view
                    {
                        self.walk
                            .root_fixed_position
                            .as_ref()
                            .and_then(|context| context.scroll_translation.clone())
                    } else {
                        None
                    },
                    lifecycle: Default::default(),
                    id,
                    parent: Some(parent_transform),
                    matrix: property_matrix,
                    origin: [0.0; 3],
                    flattens_inherited_transform: inherited_should_flatten_transform,
                    rendering_context_id,
                    backface_visibility,
                    scroll: None,
                    sticky_constraint,
                    direct_compositing_reasons: reasons,
                });
                if *role == PaintPropertyRole::PaintOffsetTranslation {
                    node.paint_offset_translation = Some(transform_node.clone());
                }
                node.properties.nodes.transform = transform_node;
                current_rendering_context_id = rendering_context_id;
                if *role != PaintPropertyRole::ElementCanvasTransform {
                    // This changes the element's property placement, never its
                    // child-record pixels (source deliberately omits OnUpdateTransform).
                    flat_transform_context.push(*matrix);
                }
            }
        }
        node.pre_transform =
            pre_transform.unwrap_or_else(|| node.properties.nodes.transform.clone());
        // Unsupported non-affine transforms are rejected by raster lowering;
        // only a proven coordinate translation changes these property payloads.
        let property_space_delta = if geometry.applies_transform {
            FlatToPropertySpaceOffset(&node.transforms, &node.properties.nodes.transform)
                .unwrap_or_default()
        } else {
            // Neither flat transforms nor the property transform changed;
            // reuse the already validated projection for ordinary fragments.
            geometry_property_delta
        };
        node.paint_snap_offset = property_space_delta;
        let mut has_filter_that_moves_pixels = false;
        let mut has_perspective = false;
        let mut isolates_descendant_blending = false;
        let mut primary_effect_reasons = Default::default();
        let mut filter_reasons = Default::default();
        if let Some(style) = style.filter(|_| establishes) {
            if !authored_filters.is_empty() {
                ValidateFilters(&authored_filters);
                has_filter_that_moves_pixels = HasFilterThatMovesPixels(&authored_filters);
            }
            if style.perspective().is_some()
                || node.transforms[inherited_transform_count..]
                    .iter()
                    .any(HasPerspective)
            {
                has_perspective = true;
            }
            node.applies_opacity = MaskAdjustedOpacity(style) != 1.0;
            // Source: paint_property_tree_builder.cc
            // NeedsEffectIgnoringClipPathAnd2DScale(). A CSS isolated group
            // needs an Effect only when a non-isolated descendant actually
            // blends. `isolation:isolate` alone merely establishes the
            // stacking-context boundary and must not introduce a saveLayer.
            isolates_descendant_blending = node.is_paint_layer
                && node.is_stacking_context
                && fragment
                    .paint
                    .PaintLayerHasNonIsolatedDescendantWithBlendMode();
            node.applies_blend = NeedsCssBlendEffect(
                style.blend_mode(),
                node.is_paint_layer,
                node.is_stacking_context,
                fragment
                    .paint
                    .PaintLayerHasNonIsolatedDescendantWithBlendMode(),
            );
            primary_effect_reasons = CompositingReasonFinder::EffectReasons(fragment);
            filter_reasons = CompositingReasonFinder::FilterReasons(fragment);
            node.applies_filter = !authored_filters.is_empty() || !filter_reasons.IsEmpty();
            if !authored_backdrop_filters.is_empty() {
                ValidateFilters(&authored_backdrop_filters);
            }
            // Source: paint_property_tree_builder.cc:4408-4414.
            // Printing does not establish a backdrop-filter effect even
            // though the computed style still retains the authored value.
            node.applies_backdrop_filter = !fragment.paint.IsPrinting()
                && (!authored_backdrop_filters.is_empty() || backdrop_filter_has_reference);
            if !authored_mask_images.is_empty() {
                node.mask_layers = ResolveMaskLayers(
                    fragment,
                    node.paint_offset,
                    property_space_delta,
                    node.pixel_snap_scale,
                    &authored_mask_images,
                );
            } else {
                node.mask_layers.clear();
            }
            node.applies_mask = !node.mask_layers.is_empty() || style.mask_box_image.is_some();
        }
        if establishes {
            if let Some(style) = style {
                if let Some(clip_path) = style.clip_path.as_ref() {
                    // Source: paint_property_tree_builder.cc:2835-2844.
                    // Ordinary SVG children already paint in local SVG
                    // coordinates and therefore do not add paint_offset to a
                    // clip-path. foreignObject is a CSS box and stays on the
                    // ordinary paint-offset path.
                    let svg_effect_child = IsSvgEffectChild(fragment.paint.source_kind);
                    node.clip_path_error = clip_path.reference_error.clone();
                    if svg_effect_child
                        && clip_path.reference_box == ClipPathReferenceBox::kStrokeBox
                    {
                        node.clip_path_error.get_or_insert_with(|| {
                            "exact SVG stroke-box geometry is not exported by Layout".into()
                        });
                    }
                    let clip_path_paint_offset = if svg_effect_child {
                        Offset::default()
                    } else {
                        node.paint_offset
                    };
                    let reference = ClipPathReferenceRect(
                        fragment,
                        clip_path_paint_offset,
                        clip_path.reference_box,
                    );
                    let path_bounds = if clip_path.geometry_box_only {
                        match GeometryBoxClipPathRadii(fragment, clip_path.reference_box) {
                            Ok(radii) => node
                                .local_clip_path
                                .extend(ConicRoundedRectPath(&reference, &radii)),
                            Err(error) => {
                                node.clip_path_error.get_or_insert_with(|| error.into());
                            }
                        }
                        reference
                    } else if clip_path.compiled_shape {
                        // Source: ClipPathClipper::GetPathWithObjectZoom and
                        // StyleShape::GetPath. The actual compiled owner is
                        // borrowed after the resident reference box is known.
                        match ResolveComputedShapeClipPath(
                            style,
                            reference,
                            svg_effect_child,
                            &mut node.local_clip_path,
                        ) {
                            Ok((bounds, even_odd)) => {
                                node.clip_path_even_odd = even_odd;
                                bounds
                            }
                            Err(error) => {
                                node.clip_path_error.get_or_insert_with(|| error.into());
                                reference
                            }
                        }
                    } else if let Some(graph) = clip_path.reference_graph.as_ref() {
                        if node.clip_path_error.is_none() {
                            node.clip_path_error = graph.unsupported.clone()
                        }
                        node.clip_path_even_odd =
                            graph.children.first().is_some_and(|child| child.even_odd);
                        ResolveReferenceClipPath(
                            graph,
                            reference,
                            clip_path_paint_offset,
                            f64::from(fragment.paint.effective_zoom),
                            svg_effect_child,
                            clip_path.reference_viewport,
                            fragment
                                .paint
                                .resources
                                .as_ref()
                                .map_or(&[], |resources| resources.fonts.as_slice()),
                            &mut node.clip_path_error,
                            &mut node.local_clip_path,
                            &mut node.local_clip_path_children,
                        )
                    } else if clip_path.reference_url.is_some() {
                        PaintRect {
                            x: reference.x,
                            y: reference.y,
                            width: 0.0,
                            height: 0.0,
                        }
                    } else if clip_path.reference_error.is_some()
                        && clip_path.inset.is_none()
                        && clip_path.circle.is_none()
                        && clip_path.ellipse.is_none()
                        && clip_path.commands.is_empty()
                        && clip_path.polygon_rounding_radius.is_none()
                    {
                        // An unavailable document resource does not become a
                        // fabricated empty/basic-shape path.
                        reference
                    } else {
                        node.clip_path_even_odd = clip_path.even_odd;
                        ResolveClipPath(
                            clip_path,
                            Offset {
                                x: reference.x,
                                y: reference.y,
                            },
                            Size {
                                width: reference.width,
                                height: reference.height,
                            },
                            if svg_effect_child {
                                1.0 / f64::from(fragment.paint.effective_zoom)
                                    .max(f64::MIN_POSITIVE)
                            } else {
                                1.0
                            },
                            &mut node.local_clip_path,
                        )
                    };
                    if let Some(mapped) = MapRectToRoot(path_bounds, &node.transforms) {
                        node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
                    }
                    node.properties.clip_id = self.walk.next_property_id;
                    self.walk.next_property_id += 1;
                    self.walk.RegisterProperty(
                        fragment,
                        PaintPropertyRole::ClipPathClip,
                        node.properties.clip_id,
                    );
                    node.properties.nodes.clip = Arc::new(
                        ClipPaintPropertyNode {
                            geometry_mapper_cache: Default::default(),
                            layout_clip_rects: Default::default(),
                            lifecycle: Default::default(),
                            id: node.properties.clip_id,
                            parent: Some(node.properties.nodes.clip.clone()),
                            local_transform_space: node.properties.nodes.transform.clone(),
                            rect: Some(PropertySpaceRect(path_bounds, property_space_delta)),
                            radii: PaintCornerRadii::default(),
                            clip_path: Some(Arc::new(
                                paint_model::paint_clip_path::PaintClipPath::from_authored(
                                    &node.local_clip_path,
                                    &node.local_clip_path_children,
                                    node.clip_path_even_odd,
                                    property_space_delta,
                                ),
                            )),
                            clip_path_error: node.clip_path_error.clone(),
                            pixel_moving_filter: None,
                        }
                        .with_layout_clips(),
                    );
                    node.applies_clip = true;
                    node.applies_clip_path = true;
                }
            }
        }

        // UpdateEffect/UpdateMask use the ancestor output clip; MaskClip
        // restricts the contents only, never the mask source itself.
        let mask_output_clip = node.properties.nodes.clip.clone();
        let has_primary_effect = node.applies_opacity
            || node.applies_blend
            || node.applies_backdrop_filter
            || node.applies_mask
            || !primary_effect_reasons.IsEmpty();
        node.applies_effect = has_primary_effect || node.applies_filter;
        if has_primary_effect {
            let style = style.expect("effect must have source style");
            let backdrop_filter_clip = if node.applies_backdrop_filter {
                let id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk
                    .RegisterProperty(fragment, PaintPropertyRole::BackdropFilterClip, id);
                Some(Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(
                            node.PixelSnappedRect(&PaintRect {
                                x: node.paint_offset.x,
                                y: node.paint_offset.y,
                                width: fragment.size.width,
                                height: fragment.size.height,
                            }),
                            property_space_delta,
                        )),
                        radii: ResolveCornerRadii(
                            style,
                            &fragment.paint.border_sides,
                            fragment.size,
                            Edges::default(),
                        ),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                ))
            } else {
                None
            };
            node.properties.effect_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk.RegisterProperty(
                fragment,
                PaintPropertyRole::Effect,
                node.properties.effect_id,
            );
            // Source: paint_layer.cc:2618-2636. Blink's backdrop effect owns
            // the authored backdrop chain followed by the ordinary CSS filter
            // chain. Box reflection is appended only by
            // FilterOperationsIncludingReflection() for the foreground filter
            // node, so it must not leak into this combined backdrop chain.
            let backdrop_filter_inputs = if node.applies_backdrop_filter {
                let mut inputs = authored_backdrop_filters.clone();
                inputs.extend(
                    authored_filters
                        .iter()
                        .filter(|filter| filter.r#type != PaintFilterType::kBoxReflect)
                        .cloned(),
                );
                ResolveFilterImages(&node, &inputs)
            } else {
                Vec::new()
            };
            let filter_reference_scale = if IsSvgEffectChild(fragment.paint.source_kind) {
                1.0
            } else {
                f64::from(fragment.paint.effective_zoom)
            };
            let inverse_filter_scale = 1.0 / filter_reference_scale.max(f64::MIN_POSITIVE);
            // Sources: paint_layer.cc:2606-2611 and
            // filter_effect_builder.cc:527-537. Compositor reference geometry
            // is unzoomed before SVG filter resolution, and the viewport is a
            // SizeF rather than a positioned rectangle.
            let filter_reference_box = (!backdrop_filter_inputs.is_empty()).then(|| {
                let mut rect = NinePieceImageGeometry::ReferenceRect(&node);
                rect.x *= inverse_filter_scale;
                rect.y *= inverse_filter_scale;
                rect.width *= inverse_filter_scale;
                rect.height *= inverse_filter_scale;
                PropertySpaceRect(rect, property_space_delta)
            });
            let filter_viewport =
                fragment
                    .paint
                    .paint_layer_filter_viewport
                    .map(|size| PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: size.width * inverse_filter_scale,
                        height: size.height * inverse_filter_scale,
                    });
            let filter_space_origin = Offset::default();
            let mut backdrop_filters = if node.applies_backdrop_filter {
                paint_model::property_filter_operations::PropertyFilterOperations::from_authored(
                    &backdrop_filter_inputs,
                    filter_reference_box,
                    filter_viewport,
                    filter_space_origin,
                    filter_reference_scale,
                )
            } else {
                Default::default()
            };
            // Source: paint_property_tree_builder.cc:4423-4433. Canvas
            // draw-element must not expose a backdrop whose filter depends on
            // currentColor/origin-tainted input. Keep the effect node and its
            // other properties, but publish no backdrop FilterInfo.
            let backdrop_filter_disallowed = fragment.paint.CanvasDrawElementEnabled()
                && fragment.paint.IsInCanvasSubtree()
                && backdrop_filters.origin_tainted();
            if backdrop_filter_disallowed {
                backdrop_filters = Default::default();
            }
            let mut effect_reasons = primary_effect_reasons;
            effect_reasons.PutAll(CompositingReasonFinder::AdditionalEffectCompositingTrigger(
                fragment,
            ));
            let mut compositor = EffectCompositorState(
                fragment,
                PaintPropertyRole::Effect,
                !effect_reasons.IsEmpty(),
                node.applies_mask && node.applies_backdrop_filter,
                inherited_tainted_subtree,
                is_in_drawable_canvas_subtree,
            );
            compositor.filter_output_bounds = filter_reference_box
                .and_then(|reference| backdrop_filters.reference_output_bounds(reference));
            node.properties.nodes.effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor,

                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: node.properties.effect_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(node.properties.nodes.clip.clone()),
                    filter_reference_box,
                    filter_viewport,
                    filter_space_origin,
                    filter_reference_scale,
                    opacity: MaskAdjustedOpacity(style),
                    blend_mode: style.blend_mode(),
                    filters: Default::default(),
                    backdrop_filters,
                    backdrop_filter_has_reference: backdrop_filter_has_reference
                        && !backdrop_filter_disallowed,
                    backdrop_filter_bounds: backdrop_filter_clip.as_ref().map(|clip| {
                        paint_model::paint_clip_path::rounded_bounds(
                            clip.rect.expect("source finite backdrop bounds"),
                            clip.radii,
                        )
                    }),
                    backdrop_filter_clip,
                    isolates_blending: isolates_descendant_blending,
                    has_mask: false,
                    is_mask: false,
                    // Source: core/paint/compositing/compositing_reason_finder.cc:461-462;
                    // core/paint/paint_property_tree_builder.cc:2232-2240.
                    // BackdropFilter is a direct effect reason: decompositing it
                    // would restrict backdrop reads to individual raster tiles.
                    // The additional effect trigger also carries transform hints.
                    direct_compositing_reasons: effect_reasons,
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
        }
        if node.applies_mask {
            // CSS masks establish a stacking context and isolated layer even
            // when a standalone source owner has no exported PaintLayer yet.
            node.is_paint_layer = true;
            node.is_stacking_context = true;
            let mask_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::Mask, mask_id);
            let mask_effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor: EffectCompositorState(
                        fragment,
                        PaintPropertyRole::Mask,
                        false,
                        false,
                        node.properties
                            .nodes
                            .effect
                            .compositor
                            .is_in_tainted_subtree,
                        is_in_drawable_canvas_subtree,
                    ),
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: mask_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(mask_output_clip.clone()),
                    filter_reference_box: None,
                    filter_viewport: None,
                    filter_space_origin: Offset::default(),
                    filter_reference_scale: 1.0,
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters: Default::default(),
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: true,
                    direct_compositing_reasons: Default::default(),
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
            node.mask_properties = Some(paint_model::paint_property_tree::PropertyTreeState {
                transform: node.properties.nodes.transform.clone(),
                clip: mask_output_clip.clone(),
                effect: mask_effect,
            });
            let mask_bounds = node.ResolvedMaskClipRect().expect("resolved MaskClip");
            let clip_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::MaskClip, clip_id);
            node.properties.clip_id = clip_id;
            node.properties.nodes.clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id: clip_id,
                    parent: Some(mask_output_clip),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    // Source: paint_property_tree_builder.cc UpdateEffect.
                    // Chromium retains the precise MaskBoundingBox separately,
                    // but supplies MaskClip's raster geometry through
                    // FloatRoundedRect(ToEnclosingRect(combined_clip)).  Using
                    // the fractional bounds directly clips the first content
                    // row/column before the mask source is composited.
                    rect: Some(EnclosingMaskClipRectInPropertySpace(
                        mask_bounds,
                        property_space_delta,
                    )),
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
        }
        // Blink UpdateCssClip runs after UpdateEffect and before UpdateFilter.
        // Unlike overflow clipping, the legacy positioned-box clip applies to
        // the box's own background as well as all descendants.
        if establishes {
            if let Some(css_clip) = fragment.paint.css_clip {
                let local = node.PixelSnappedRect(&PaintRect {
                    x: node.paint_offset.x + css_clip.offset.x,
                    y: node.paint_offset.y + css_clip.offset.y,
                    width: css_clip.size.width.max(0.0),
                    height: css_clip.size.height.max(0.0),
                });
                node.local_css_clip = Some(local);
                if let Some(mapped) = MapRectToRoot(local, &node.transforms) {
                    node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
                }
                node.properties.clip_id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk.RegisterProperty(
                    fragment,
                    PaintPropertyRole::CssClip,
                    node.properties.clip_id,
                );
                node.properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id: node.properties.clip_id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(local, property_space_delta)),
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                );
                node.applies_clip = true;
                node.applies_css_clip = true;
            }
        }
        if node.applies_filter {
            style.expect("filter must have source style");
            node.recording_filters = ResolveFilterImages(&node, &authored_filters);
            node.properties.effect_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk.RegisterProperty(
                fragment,
                PaintPropertyRole::Filter,
                node.properties.effect_id,
            );
            // Source: paint_layer.cc:2044-2047 and
            // paint_property_tree_builder.cc:2602,2609-2612. A pixel-moving layer
            // uses its stitched visual overflow including self-painting layer
            // descendants, not this physical fragment's border box.
            let local_reference_box = if IsSvgEffectChild(fragment.paint.source_kind) {
                // Source: paint_property_tree_builder.cc:2615-2627 and
                // LayoutSVGModelObject::VisualRectInLocalSVGCoordinates().
                // SVG children do not use a PaintLayer filter reference box;
                // their decorated local SVG visual rect is the filter input
                // and output-bounds reference.  The anonymous NG fragment can
                // be empty, so a box-decoration reference rect is not an
                // equivalent fallback.
                fragment
                    .paint
                    .svg_visual_rect
                    .map_or_else(PaintRect::default, |rect| PaintRect {
                        x: node.paint_offset.x + rect.offset.x,
                        y: node.paint_offset.y + rect.offset.y,
                        width: rect.size.width,
                        height: rect.size.height,
                    })
            } else {
                fragment
                    .paint
                    .paint_layer_filter_reference_box
                    .map(|rect| PaintRect {
                        x: node.paint_offset.x + rect.offset.x,
                        y: node.paint_offset.y + rect.offset.y,
                        width: rect.size.width,
                        height: rect.size.height,
                    })
                    .unwrap_or_else(|| {
                        if fragment.paint.PaintLayerStateIsAuthoritative() && node.is_paint_layer {
                            PaintRect::default()
                        } else {
                            NinePieceImageGeometry::ReferenceRect(&node)
                        }
                    })
            };
            let filter_reference_scale = if IsSvgEffectChild(fragment.paint.source_kind) {
                1.0
            } else {
                f64::from(fragment.paint.effective_zoom)
            };
            let inverse_filter_scale = 1.0 / filter_reference_scale.max(f64::MIN_POSITIVE);
            // Sources: paint_layer.cc:2575-2589 and
            // filter_effect_builder.cc:527-537. Keep Blink's unzoomed builder
            // inputs; Filter::Scale() is applied exactly once downstream.
            let mut local_reference_box = local_reference_box;
            local_reference_box.x *= inverse_filter_scale;
            local_reference_box.y *= inverse_filter_scale;
            local_reference_box.width *= inverse_filter_scale;
            local_reference_box.height *= inverse_filter_scale;
            let filter_reference_box = PropertySpaceRect(local_reference_box, property_space_delta);
            let filter_viewport =
                fragment
                    .paint
                    .paint_layer_filter_viewport
                    .map(|size| PaintRect {
                        x: 0.0,
                        y: 0.0,
                        width: size.width * inverse_filter_scale,
                        height: size.height * inverse_filter_scale,
                    });
            let filter_space_origin = Offset::default();
            let mut filters =
                paint_model::property_filter_operations::PropertyFilterOperations::from_authored(
                    &node.recording_filters,
                    Some(filter_reference_box),
                    filter_viewport,
                    filter_space_origin,
                    filter_reference_scale,
                );
            // Source: paint_property_tree_builder.cc:2632-2653. A tainted CSS
            // filter is omitted inside a Canvas draw-element subtree. Outside
            // that restricted subtree its taint becomes inherited Effect
            // state for descendants.
            let filter_disallowed = fragment.paint.CanvasDrawElementEnabled()
                && fragment.paint.IsInCanvasSubtree()
                && filters.origin_tainted();
            if filter_disallowed {
                filters = Default::default();
            }
            filter_reasons.PutAll(CompositingReasonFinder::AdditionalEffectCompositingTrigger(
                fragment,
            ));
            let filter_taints_subtree = filters.origin_tainted();
            let mut compositor = EffectCompositorState(
                fragment,
                PaintPropertyRole::Filter,
                !filter_reasons.IsEmpty(),
                false,
                node.properties
                    .nodes
                    .effect
                    .compositor
                    .is_in_tainted_subtree
                    || filter_taints_subtree,
                is_in_drawable_canvas_subtree,
            );
            compositor.filter_output_bounds = filters.reference_output_bounds(filter_reference_box);
            // UpdateFilter follows UpdateEffect and has its own persistent
            // ObjectPaintProperties slot (paint_property_tree_builder.cc:2630).
            node.properties.nodes.effect = Arc::new(
                EffectPaintPropertyNode {
                    compositor,
                    backdrop_filter_bounds: None,
                    self_or_ancestor_participates_in_view_transition: false,
                    lifecycle: Default::default(),
                    id: node.properties.effect_id,
                    parent: Some(node.properties.nodes.effect.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    output_clip: Some(node.properties.nodes.clip.clone()),
                    filter_reference_box: Some(filter_reference_box),
                    filter_viewport,
                    filter_space_origin,
                    filter_reference_scale,
                    opacity: 1.0,
                    blend_mode: PaintBlendMode::kNormal,
                    filters,
                    backdrop_filters: Default::default(),
                    backdrop_filter_has_reference: false,
                    backdrop_filter_clip: None,
                    isolates_blending: false,
                    has_mask: false,
                    is_mask: false,
                    direct_compositing_reasons: filter_reasons,
                }
                .with_filter_presence()
                .with_inherited_view_transition(),
            );
            if authored_filters.iter().any(|filter| {
                matches!(
                    filter.r#type,
                    PaintFilterType::kBlur
                        | PaintFilterType::kDropShadow
                        | PaintFilterType::kBoxReflect
                )
            }) {
                node.properties.clip_id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk.RegisterProperty(
                    fragment,
                    PaintPropertyRole::PixelMovingFilterClipExpander,
                    node.properties.clip_id,
                );
                node.properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id: node.properties.clip_id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: None,
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: Some(node.properties.nodes.effect.clone()),
                    }
                    .with_layout_clips(),
                );
            }
        }
        // Chromium's composited background-attachment:fixed path records the
        // viewport-sized background in the LayoutView transform, then clips
        // it with a node whose local transform space is this element.  The
        // clip is deliberately not installed into the descendant context.
        if fragment.paint.can_composite_background_attachment_fixed {
            let style = style.expect("composited fixed background requires style");
            let background_images = style.background_images();
            let layer = background_images
                .first()
                .expect("composited fixed background requires one image layer");
            let border = IncludedBorderEdges(&fragment.paint);
            let mut local = PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            };
            if matches!(
                layer.clip,
                BackgroundBox::kPaddingBox | BackgroundBox::kContentBox
            ) {
                local.x += border.left;
                local.y += border.top;
                local.width = (local.width - border.left - border.right).max(0.0);
                local.height = (local.height - border.top - border.bottom).max(0.0);
            }
            if layer.clip == BackgroundBox::kContentBox {
                local.x += fragment.paint.padding.left;
                local.y += fragment.paint.padding.top;
                local.width =
                    (local.width - fragment.paint.padding.left - fragment.paint.padding.right)
                        .max(0.0);
                local.height =
                    (local.height - fragment.paint.padding.top - fragment.paint.padding.bottom)
                        .max(0.0);
            }
            let id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::BackgroundClip, id);
            let clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id,
                    parent: Some(node.properties.nodes.clip.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: Some(PropertySpaceRect(
                        node.PixelSnappedRect(&local),
                        property_space_delta,
                    )),
                    radii: PaintCornerRadii::default(),
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips(),
            );
            node.fixed_background_properties =
                Some(paint_model::paint_property_tree::PropertyTreeState {
                    transform: self.walk.viewport_properties.nodes.transform.clone(),
                    clip,
                    effect: node.properties.nodes.effect.clone(),
                });
        }
        // Source: paint_property_tree_builder.cc:2998-3062. This clip belongs
        // only to overflow controls and never enters the descendant context.
        if let Some(scrollbars) = fragment.paint.scrollbars.as_ref() {
            let mut controls_bounds: Option<PaintRect> = None;
            // Source: paint_layer_scrollable_area.cc:2304-2308 and
            // paint_property_tree_builder.cc:3004-3005. With no scroll corner,
            // a pointer-sized resizer is still part of the controls bounds.
            let corner_or_resizer =
                if scrollbars.corner_size.width > 0.0 && scrollbars.corner_size.height > 0.0 {
                    (scrollbars.corner_offset, scrollbars.corner_size)
                } else {
                    (
                        scrollbars.resizer_pointer_offset,
                        scrollbars.resizer_pointer_size,
                    )
                };
            for (offset, size) in [
                corner_or_resizer,
                (
                    scrollbars
                        .horizontal
                        .as_ref()
                        .map_or(Offset::default(), |axis| axis.track_offset),
                    scrollbars
                        .horizontal
                        .as_ref()
                        .map_or(Size::default(), |axis| axis.track_size),
                ),
                (
                    scrollbars
                        .vertical
                        .as_ref()
                        .map_or(Offset::default(), |axis| axis.track_offset),
                    scrollbars
                        .vertical
                        .as_ref()
                        .map_or(Size::default(), |axis| axis.track_size),
                ),
            ] {
                if size.width <= 0.0 || size.height <= 0.0 {
                    continue;
                }
                let rect = PaintRect {
                    x: offset.x,
                    y: offset.y,
                    width: size.width,
                    height: size.height,
                };
                controls_bounds = Some(controls_bounds.map_or(rect, |bounds| Union(bounds, &rect)));
            }
            let border_box = node.PixelSnappedRect(&PaintRect {
                x: node.paint_offset.x,
                y: node.paint_offset.y,
                width: fragment.size.width,
                height: fragment.size.height,
            });
            let controls_outside_border = controls_bounds.is_some_and(|bounds| {
                let bounds = PaintRect {
                    x: node.paint_offset.x + bounds.x,
                    y: node.paint_offset.y + bounds.y,
                    ..bounds
                };
                bounds.x < border_box.x
                    || bounds.y < border_box.y
                    || bounds.x + bounds.width > border_box.x + border_box.width
                    || bounds.y + bounds.height > border_box.y + border_box.height
            });
            if controls_outside_border {
                let id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk
                    .RegisterProperty(fragment, PaintPropertyRole::OverflowControlsClip, id);
                let clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(border_box, property_space_delta)),
                        radii: PaintCornerRadii::default(),
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips(),
                );
                node.overflow_controls_properties =
                    Some(paint_model::paint_property_tree::PropertyTreeState {
                        transform: node.properties.nodes.transform.clone(),
                        clip,
                        effect: node.properties.nodes.effect.clone(),
                    });
            }
        }
        // LocalBorderBoxProperties excludes this box's overflow clip and
        // scroll translation. ContentsProperties includes both (Blink
        // paint_property_tree_builder.cc UpdateForChildren).
        let border_box_properties = node.properties.clone();
        let mut scroll_node = None;
        if ClipsOverflow(fragment) || NeedsScrollAndScrollTranslation(fragment) {
            let mut clip_inset = IncludedBorderEdges(&fragment.paint);
            let replaced_always_clips = ReplacedElementAlwaysClipsToContentBox(fragment);
            if replaced_always_clips {
                clip_inset.top += fragment.paint.padding.top;
                clip_inset.right += fragment.paint.padding.right;
                clip_inset.bottom += fragment.paint.padding.bottom;
                clip_inset.left += fragment.paint.padding.left;
            } else if let Some(outsets) = fragment.paint.overflow_clip_margin_outsets {
                clip_inset = Edges {
                    top: -outsets.top,
                    right: -outsets.right,
                    bottom: -outsets.bottom,
                    left: -outsets.left,
                };
            }
            let mut local = PaintRect {
                x: node.paint_offset.x + clip_inset.left,
                y: node.paint_offset.y + clip_inset.top,
                width: (fragment.size.width - clip_inset.left - clip_inset.right).max(0.0),
                height: (fragment.size.height - clip_inset.top - clip_inset.bottom).max(0.0),
            };
            if let Some(replaced) = fragment
                .paint
                .replaced_content
                .as_ref()
                .filter(|replaced| replaced_always_clips && replaced.is_layout_embedded_content)
            {
                // Source: paint_property_tree_builder.cc:3273-3279. Embedded
                // content may differ by one device pixel after persistent
                // sizing; match the source replaced-content size.
                local.width = replaced.size.width;
                local.height = replaced.size.height;
            }
            let inner_radius_clip = local;
            if let Some(scrollbars) = fragment.paint.scrollbars.as_ref() {
                if !scrollbars.uses_overlay_scrollbars {
                    if let Some(vertical) = scrollbars.vertical.as_ref() {
                        let thickness = vertical.track_size.width;
                        if vertical.track_offset.x <= clip_inset.left {
                            local.x += thickness;
                        }
                        local.width = (local.width - thickness).max(0.0);
                    }
                    if let Some(horizontal) = scrollbars.horizontal.as_ref() {
                        let thickness = horizontal.track_size.height;
                        if horizontal.track_offset.y <= clip_inset.top {
                            local.y += thickness;
                        }
                        local.height = (local.height - thickness).max(0.0);
                    }
                }
            }
            // Source: core/paint/paint_property_tree_builder.cc:3286-3307.
            // Layout mapping retains the unsnapped clip independently of the
            // painting clip. Scroll container output is the authoritative
            // physical clip geometry when present.
            let mut precise_layout_clip = local;
            local = node.PixelSnappedRect(&local);
            if let Some(scroll) = fragment.paint.scroll_container.as_ref() {
                precise_layout_clip = PaintRect {
                    x: node.paint_offset.x + scroll.container_rect.offset.x,
                    y: node.paint_offset.y + scroll.container_rect.offset.y,
                    width: scroll.container_rect.size.width,
                    height: scroll.container_rect.size.height,
                };
                local = node.PixelSnappedRect(&precise_layout_clip);
            }
            if let Some(viewport) = fragment.paint.svg_viewport {
                if let Some(clip) = viewport.clip {
                    local = PaintRect {
                        x: clip.offset.x,
                        y: clip.offset.y,
                        width: clip.size.width,
                        height: clip.size.height,
                    };
                    precise_layout_clip = local;
                }
            }
            node.local_clip = Some(local);
            let mut resolved_clip_radii = PaintCornerRadii::default();
            if let Some(style) =
                style.filter(|_| fragment.paint.source_kind != NodeKind::kSvgViewportContainer)
            {
                if let Some(outsets) = fragment
                    .paint
                    .overflow_clip_margin_outsets
                    .filter(|_| !replaced_always_clips)
                {
                    resolved_clip_radii = ExpandCornerRadiiEdges(
                        ResolveCornerRadii(
                            style,
                            &fragment.paint.border_sides,
                            fragment.size,
                            Edges::default(),
                        ),
                        outsets,
                        Size {
                            width: inner_radius_clip.width,
                            height: inner_radius_clip.height,
                        },
                    );
                } else {
                    resolved_clip_radii = ResolveCornerRadii(
                        style,
                        &fragment.paint.border_sides,
                        fragment.size,
                        clip_inset,
                    );
                }
            }
            if NeedsInnerBorderRadiusClip(fragment) {
                let inner_clip = node.PixelSnappedRect(&inner_radius_clip);
                node.local_inner_border_radius_clip = Some(inner_clip);
                node.local_inner_border_radius_radii = resolved_clip_radii;
                let id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk
                    .RegisterProperty(fragment, PaintPropertyRole::InnerBorderRadiusClip, id);
                node.properties.nodes.clip = Arc::new(
                    ClipPaintPropertyNode {
                        geometry_mapper_cache: Default::default(),
                        layout_clip_rects: Default::default(),
                        lifecycle: Default::default(),
                        id,
                        parent: Some(node.properties.nodes.clip.clone()),
                        local_transform_space: node.properties.nodes.transform.clone(),
                        rect: Some(PropertySpaceRect(inner_clip, property_space_delta)),
                        radii: resolved_clip_radii,
                        clip_path: None,
                        clip_path_error: None,
                        pixel_moving_filter: None,
                    }
                    .with_layout_clips()
                    .with_layout_clip_values(
                        Some(PropertySpaceRect(inner_radius_clip, property_space_delta)),
                        None,
                        None,
                    ),
                );
            }
            node.local_clip_radii = if replaced_always_clips {
                resolved_clip_radii
            } else {
                PaintCornerRadii::default()
            };
            node.local_clip_radius = UniformCornerRadius(&resolved_clip_radii);
            if let Some(mapped) = MapRectToRoot(local, &node.transforms) {
                node.clip = Some(node.clip.map_or(mapped, |clip| Intersect(clip, &mapped)));
            }
            node.properties.clip_id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk.RegisterProperty(
                fragment,
                PaintPropertyRole::OverflowClip,
                node.properties.clip_id,
            );
            let mut excluding_overlay = precise_layout_clip;
            if let Some(scrollbars) = fragment
                .paint
                .scrollbars
                .as_ref()
                .filter(|bars| bars.uses_overlay_scrollbars)
            {
                if let Some(vertical) = &scrollbars.vertical {
                    if vertical.track_offset.x <= clip_inset.left {
                        excluding_overlay.x += vertical.track_size.width;
                    }
                    excluding_overlay.width =
                        (excluding_overlay.width - vertical.track_size.width).max(0.0);
                }
                if let Some(horizontal) = &scrollbars.horizontal {
                    if horizontal.track_offset.y <= clip_inset.top {
                        excluding_overlay.y += horizontal.track_size.height;
                    }
                    excluding_overlay.height =
                        (excluding_overlay.height - horizontal.track_size.height).max(0.0);
                }
            }
            node.properties.nodes.clip = Arc::new(
                ClipPaintPropertyNode {
                    geometry_mapper_cache: Default::default(),
                    layout_clip_rects: Default::default(),
                    lifecycle: Default::default(),
                    id: node.properties.clip_id,
                    parent: Some(node.properties.nodes.clip.clone()),
                    local_transform_space: node.properties.nodes.transform.clone(),
                    rect: Some(PropertySpaceRect(local, property_space_delta)),
                    radii: node.local_clip_radii,
                    clip_path: None,
                    clip_path_error: None,
                    pixel_moving_filter: None,
                }
                .with_layout_clips()
                .with_layout_clip_values(
                    Some(PropertySpaceRect(precise_layout_clip, property_space_delta)),
                    None,
                    (fragment.paint.source_kind != NodeKind::kSvgViewportContainer)
                        .then_some(PropertySpaceRect(excluding_overlay, property_space_delta)),
                ),
            );
            node.applies_clip = true;
            node.applies_overflow_clip = true;
            if fragment.paint.source_kind != NodeKind::kSvgViewportContainer
                && NeedsScrollAndScrollTranslation(fragment)
            {
                node.properties.scroll_id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk.RegisterProperty(
                    fragment,
                    PaintPropertyRole::Scroll,
                    node.properties.scroll_id,
                );
                // Blink's UpdateScrollNode passes
                // PaintLayerScrollableArea::PixelSnappedContentsSize() to
                // cc. Snap the contents with the same physical origin as the
                // overflow clip; enclosing the unsnapped LayoutUnit size at
                // the compositor boundary can grow the scroll range by one
                // pixel and make main/impl offsets alternate at the end.
                // Source: core/paint/paint_property_tree_builder.cc:3706-3726;
                //         core/paint/paint_layer_scrollable_area.cc.
                let snapped_contents = node.PixelSnappedRect(&PaintRect {
                    x: precise_layout_clip.x,
                    y: precise_layout_clip.y,
                    width: fragment
                        .paint
                        .scroll_size
                        .width
                        .max(precise_layout_clip.width),
                    height: fragment
                        .paint
                        .scroll_size
                        .height
                        .max(precise_layout_clip.height),
                });
                scroll_node = Some(Arc::new(ScrollPaintPropertyNode {
                    // Source: core/paint/paint_property_tree_builder.cc:3570-3648,
                    // 4611-4628. A non-none source scroll-snap payload remains
                    // unsupported; PaintInput rejects it rather than inventing one.
                    compositor: {
                        let mut state =
                            paint_model::paint_property_tree::ScrollCompositorState::default();
                        let direct_reasons =
                            CompositingReasonFinder::DirectReasonsForPaintProperties(fragment);
                        state.composited_scrolling_preference =
                            if CompositingReasonFinder::ShouldForcePreferCompositingToLCDText(
                                fragment,
                                direct_reasons,
                            ) {
                                paint_model::paint_property_tree::PaintCompositedScrollingPreference::Preferred
                            } else if fragment
                                .paint
                                .scroll_container
                                .as_ref()
                                .and_then(|scroll| scroll.prefers_non_composited_scrolling)
                                == Some(true)
                            {
                                paint_model::paint_property_tree::PaintCompositedScrollingPreference::NotPreferred
                            } else {
                                paint_model::paint_property_tree::PaintCompositedScrollingPreference::Default
                            };
                        state.prevent_scroll_axis_locking =
                            fragment.paint.prevent_scroll_axis_locking;
                        // Source: core/paint/paint_property_tree_builder.cc:3436-3439.
                        if self
                            .walk
                            .requires_main_thread_for_background_attachment_fixed
                            == Some(true)
                        {
                            state.main_thread_repaint_reasons |= 1;
                        }
                        state.overscroll_behavior = [
                            fragment.paint.overscroll_behavior_x,
                            fragment.paint.overscroll_behavior_y,
                        ];
                        if let Some(scroll) = fragment.paint.scroll_container.as_ref() {
                            state.compositor_element_id = scroll.compositor_element_id;
                            if let Some(scroll_snap_type) = scroll.scroll_snap_type {
                                if scroll_snap_type.is_none {
                                    state.snap_container_data_absent = true;
                                } else {
                                    use layoutng_assembly::fragment_tree::{
                                        ScrollSnapAxisPaintData as Axis,
                                        ScrollSnapStrictnessPaintData as Strictness,
                                    };
                                    use paint_model::paint_property_tree::{
                                        PaintScrollSnapAxis as PaintAxis,
                                        PaintScrollSnapStrictness as PaintStrictness,
                                        PaintScrollSnapType,
                                    };
                                    state.snap_container_data = Some(PaintScrollSnapType {
                                        axis: match scroll_snap_type.axis {
                                            Axis::Both => PaintAxis::Both,
                                            Axis::X => PaintAxis::X,
                                            Axis::Y => PaintAxis::Y,
                                            Axis::Block => PaintAxis::Block,
                                            Axis::Inline => PaintAxis::Inline,
                                            Axis::Pair => PaintAxis::Pair,
                                        },
                                        strictness: match scroll_snap_type.strictness {
                                            Strictness::Proximity => PaintStrictness::Proximity,
                                            Strictness::Mandatory => PaintStrictness::Mandatory,
                                        },
                                    });
                                }
                            }
                            // Source: core/paint/paint_property_tree_builder.cc:3440-3455;
                            // cc/input/main_thread_scrolling_reason.h:25-37.
                            if scroll.background_needs_repaint_on_scroll == Some(true) {
                                state.main_thread_repaint_reasons |= 1 << 3;
                            }
                            if !(scroll.user_scrollable_horizontal
                                || scroll.user_scrollable_vertical)
                            {
                                state.main_thread_repaint_reasons |= 1 << 2;
                            }
                            for (behavior, scrollable) in state
                                .overscroll_behavior
                                .iter_mut()
                                .zip(scroll.scrollable_axes)
                            {
                                if !scrollable {
                                    *behavior = Default::default();
                                }
                            }
                        }
                        state
                    },
                    lifecycle: Default::default(),
                    id: node.properties.scroll_id,
                    owner_client_id: fragment.paint.display_item_client_id,
                    owner_node_id: u64::try_from(
                        fragment.paint.display_item_client_owner_node_id,
                    )
                    .unwrap_or_default(),
                    parent: parent_properties.nodes.nearest_scroll(),
                    overflow_clip: Some(node.properties.nodes.clip.clone()),
                    container_rect: PropertySpaceRect(local, property_space_delta),
                    contents_rect: PaintRect {
                        x: local.x + property_space_delta.x,
                        y: local.y + property_space_delta.y,
                        width: snapped_contents.width.max(local.width),
                        height: snapped_contents.height.max(local.height),
                    },
                    minimum_scroll_offset: fragment
                        .paint
                        .scroll_container
                        .as_ref()
                        .map_or(Offset::default(), |scroll| scroll.minimum_scroll_offset),
                    maximum_scroll_offset: fragment.paint.scroll_container.as_ref().map_or_else(
                        || Offset {
                            x: (fragment.paint.scroll_size.width - local.width).max(0.0),
                            y: (fragment.paint.scroll_size.height - local.height).max(0.0),
                        },
                        |scroll| scroll.maximum_scroll_offset,
                    ),
                    user_scrollable_horizontal: fragment
                        .paint
                        .scroll_container
                        .as_ref()
                        .map_or_else(
                            || {
                                matches!(
                                    fragment.paint.overflow_x,
                                    Overflow::kScroll | Overflow::kAuto
                                )
                            },
                            |scroll| scroll.user_scrollable_horizontal,
                        ),
                    user_scrollable_vertical: fragment.paint.scroll_container.as_ref().map_or_else(
                        || {
                            matches!(
                                fragment.paint.overflow_y,
                                Overflow::kScroll | Overflow::kAuto
                            )
                        },
                        |scroll| scroll.user_scrollable_vertical,
                    ),
                }));
            }
        }

        // Source: core/paint/paint_property_tree_builder.cc:3666-3741.
        // Overlay scrollbar opacity is animated on the compositor. Each axis
        // therefore needs its own effect node; sharing the page effect would
        // make the scrollbar fade mutate the entire document's opacity.
        if let Some(scrollbars) = fragment
            .paint
            .scrollbars
            .as_ref()
            .filter(|scrollbars| scrollbars.uses_overlay_scrollbars)
        {
            for (axis, role, horizontal) in [
                (
                    scrollbars.horizontal.as_ref(),
                    PaintPropertyRole::HorizontalScrollbarEffect,
                    true,
                ),
                (
                    scrollbars.vertical.as_ref(),
                    PaintPropertyRole::VerticalScrollbarEffect,
                    false,
                ),
            ] {
                let Some(axis) = axis else { continue };
                let id = self.walk.next_property_id;
                self.walk.next_property_id += 1;
                self.walk.RegisterProperty(fragment, role, id);
                let mut scrollbar_reasons = Default::default();
                // Source: paint_property_tree_builder.cc:3706-3721. Canvas
                // children render their overflow controls into the drawable
                // canvas subtree, so Blink deliberately omits the ordinary
                // compositor opacity-animation reason there.
                if !is_in_drawable_canvas_subtree {
                    scrollbar_reasons = CompositingReasonFinder::ActiveOpacityAnimation();
                }
                let effect = Arc::new(
                    EffectPaintPropertyNode {
                        compositor: paint_model::paint_property_tree::EffectCompositorState {
                            compositor_element_id: axis.compositor_element_id,
                            is_in_tainted_subtree: node
                                .properties
                                .nodes
                                .effect
                                .compositor
                                .is_in_tainted_subtree,
                            is_in_drawable_canvas_subtree,
                            ..Default::default()
                        },
                        backdrop_filter_bounds: None,
                        self_or_ancestor_participates_in_view_transition: false,
                        lifecycle: Default::default(),
                        id,
                        parent: Some(node.properties.nodes.effect.clone()),
                        local_transform_space: border_box_properties.nodes.transform.clone(),
                        output_clip: Some(border_box_properties.nodes.clip.clone()),
                        filter_reference_box: None,
                        filter_viewport: None,
                        filter_space_origin: Offset::default(),
                        filter_reference_scale: 1.0,
                        opacity: 1.0,
                        blend_mode: PaintBlendMode::kNormal,
                        filters: Default::default(),
                        backdrop_filters: Default::default(),
                        backdrop_filter_has_reference: false,
                        backdrop_filter_clip: None,
                        isolates_blending: false,
                        has_mask: false,
                        is_mask: false,
                        direct_compositing_reasons: scrollbar_reasons,
                    }
                    .with_filter_presence()
                    .with_inherited_view_transition(),
                );
                let state = paint_model::paint_property_tree::PropertyTreeState {
                    transform: border_box_properties.nodes.transform.clone(),
                    clip: node.overflow_controls_properties.as_ref().map_or_else(
                        || border_box_properties.nodes.clip.clone(),
                        |properties| properties.clip.clone(),
                    ),
                    effect,
                };
                if horizontal {
                    node.horizontal_scrollbar_properties = Some(state);
                } else {
                    node.vertical_scrollbar_properties = Some(state);
                }
            }
        }

        // Source: core/paint/paint_property_tree_builder.cc
        // UpdateScrollTranslation. Descendant display items stay in the
        // scroller's local coordinate space; the ScrollTranslation property
        // alone moves them. Baking the offset into child geometry and also
        // publishing the property applies every committed scroll twice.
        let geometry_scroll = fragment.paint.scroll_offset;
        // Source: paint_property_tree_builder.cc:3516-3561,3769-3799.
        // ScrollTranslation uses ScrollPosition (ScrollOrigin + offset), then
        // the child paint offset gains ScrollOrigin. The two origin terms
        // cancel visually while the property tree retains Chromium's exact
        // scroll-coordinate basis.
        let (scroll_origin, scroll_position) = fragment
            .paint
            .scroll_container
            .as_ref()
            .map_or((Offset::default(), geometry_scroll), |scroll| {
                (scroll.scroll_origin, scroll.scroll_position)
            });
        let child_origin = node.absolute_offset;
        let mut child_scroll = node.scroll_offset;
        if fragment.paint.establishes_paint_state
            && (geometry_scroll.x != 0.0 || geometry_scroll.y != 0.0)
        {
            let local_scroll = geometry_scroll;
            child_scroll.x += local_scroll.x;
            child_scroll.y += local_scroll.y;
        }
        let children_are_flex_or_grid_items = fragment.paint.HasSource()
            && matches!(
                fragment.paint.display,
                Display::kFlex
                    | Display::kInlineFlex
                    | Display::kGrid
                    | Display::kInlineGrid
                    | Display::kGridLanes
            );
        let svg_view_box = SvgTransformToPixelSnappedBorderBox(
            fragment,
            node.paint_offset,
            node.paint_snap_offset,
            node.pixel_snap_scale,
        );
        let mut child_properties = node.properties.clone();
        let mut child_transforms = node.transforms.clone();
        let preserves_3d = style.is_some_and(|style| style.preserves_3d());
        let mut child_rendering_context_id = if preserves_3d {
            node.properties.nodes.transform.rendering_context_id
        } else {
            0
        };
        let pre_perspective_should_flatten = !preserves_3d;
        // This is captured before Perspective: perspective changes flattening
        // for descendant geometry, but not backface inheritance.
        let child_can_inherit_backface = pre_perspective_should_flatten;
        let mut child_should_flatten = pre_perspective_should_flatten;
        if let Some((depth, origin)) = style.and_then(|style| {
            style
                .perspective()
                .map(|depth| (depth, style.perspective_origin()))
        }) {
            if !depth.is_finite() || depth <= 0.0 {
                panic!("used perspective depth must be finite and positive");
            }
            let pivot = Offset {
                x: node.paint_offset.x
                    + property_space_delta.x
                    + origin.pixels.x
                    + fragment.size.width * origin.percentages.x / 100.0,
                y: node.paint_offset.y
                    + property_space_delta.y
                    + origin.pixels.y
                    + fragment.size.height * origin.percentages.y / 100.0,
            };
            let mut perspective = TransformMatrix::default();
            perspective.values[11] = -1.0 / depth;
            let matrix = MultiplyTransforms(
                &TranslationTransform(pivot.x, pivot.y),
                &MultiplyTransforms(&perspective, &TranslationTransform(-pivot.x, -pivot.y)),
            );
            let id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::Perspective, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(fragment, PaintPropertyRole::Perspective),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix,
                origin: [0.0; 3],
                flattens_inherited_transform: pre_perspective_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: None,
                sticky_constraint: None,
                direct_compositing_reasons: Default::default(),
            });
            child_properties.transform_id = id;
            child_transforms.push(matrix);
            // Chromium's perspective node itself must not flatten, otherwise
            // its child 3D transforms never receive perspective.
            child_should_flatten = false;
        }
        // Chromium inserts ReplacedContentTransform after Perspective and
        // before ScrollTranslation, so the content transform scrolls with the
        // replaced element rather than wrapping its scroll node.
        if let Some(matrix) = svg_view_box {
            let id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::ReplacedContentTransform, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(
                    fragment,
                    PaintPropertyRole::ReplacedContentTransform,
                ),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix,
                origin: [0.0; 3],
                flattens_inherited_transform: child_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: None,
                sticky_constraint: None,
                direct_compositing_reasons: Default::default(),
            });
            child_properties.transform_id = id;
            child_should_flatten = true;
            child_rendering_context_id = 0;
        }
        // UpdateScrollTranslation records negative ScrollPosition and
        // associates the Scroll node with that transform.
        let has_scroll_translation = scroll_node.is_some()
            || (establishes && (geometry_scroll.x != 0.0 || geometry_scroll.y != 0.0))
            || scroll_origin.x != 0.0
            || scroll_origin.y != 0.0;
        if has_scroll_translation {
            let id = self.walk.next_property_id;
            self.walk.next_property_id += 1;
            self.walk
                .RegisterProperty(fragment, PaintPropertyRole::ScrollTranslation, id);
            child_properties.nodes.transform = Arc::new(TransformPaintPropertyNode {
                compositor: TransformCompositorState(
                    fragment,
                    PaintPropertyRole::ScrollTranslation,
                ),
                geometry_mapper_cache: Default::default(),
                scroll_parent_scroll_translation: None,
                lifecycle: Default::default(),
                id,
                parent: Some(child_properties.nodes.transform.clone()),
                matrix: TranslationTransform(-scroll_position.x, -scroll_position.y),
                origin: [0.0; 3],
                flattens_inherited_transform: child_should_flatten,
                rendering_context_id: child_rendering_context_id,
                backface_visibility: PaintBackfaceVisibility::Inherited,
                scroll: scroll_node,
                sticky_constraint: None,
                direct_compositing_reasons: CompositingReasonFinder::TransformReasons(
                    fragment,
                    PaintPropertyRole::ScrollTranslation,
                    false,
                ),
            });
            child_properties.transform_id = id;
            child_transforms.push(TranslationTransform(-scroll_position.x, -scroll_position.y));
        }
        node.cull_decision_inputs = crate::cull_rect_updater::CullRectDecisionInputs {
            has_filter_that_moves_pixels,
            has_perspective,
            has_active_transform_animation,
            has_non_composited_transform,
        };
        let infinite_cull = crate::cull_rect_updater::ShouldUseInfiniteCullRect(
            fragment,
            inherited.subtree_should_use_infinite_cull_rect,
            has_filter_that_moves_pixels,
            has_perspective,
            has_active_transform_animation,
            has_non_composited_transform,
            self.previous_paint_result,
        );
        node.contents_properties = child_properties.clone();
        node.properties = border_box_properties;
        if infinite_cull.use_infinite {
            node.cull_rect = None;
            node.self_cull_rect = None;
        } else {
            let expansion_ratio = crate::cull_rect_updater::ExpansionRatio(fragment);
            let (self_cull_rect, _) = CullRectUpdater::ComputeFragmentCullRect(
                parent_cull_rect,
                old_self_cull_rect,
                &self.walk.viewport_properties.nodes,
                &parent_properties.nodes,
                &node.properties.nodes,
                expansion_ratio,
            );
            node.self_cull_rect = self_cull_rect;
            let (contents_cull_rect, _) = CullRectUpdater::ComputeFragmentContentsCullRect(
                node.self_cull_rect,
                old_contents_cull_rect,
                &self.walk.viewport_properties.nodes,
                &node.properties.nodes,
                &node.contents_properties.nodes,
                expansion_ratio,
            );
            node.cull_rect = contents_cull_rect;
        }
        if let Some(svg) = svg_view_box {
            child_transforms.push(svg);
        }
        let own_keys: Vec<_> = self
            .walk
            .property_keys
            .range(own_key_start..self.walk.next_property_id)
            .map(|(&id, &key)| (id, key))
            .collect();
        node.cache = Some(NodeCache {
            inherited,
            children: ChildContext {
                inside_blocking_wheel_event_handler: fragment
                    .paint
                    .inside_blocking_wheel_event_handler,
                blocking_wheel_event_handler_changed: false,
                origin: child_origin,
                paint_space: if has_scroll_translation {
                    Offset {
                        x: paint_space_origin.x - scroll_origin.x,
                        y: paint_space_origin.y - scroll_origin.y,
                    }
                } else {
                    paint_space_origin
                },
                clip: node.clip,
                cull: node.cull_rect,
                transforms: child_transforms,
                opacity: node.opacity,
                scroll: child_scroll,
                fixed_background_acts_as_scroll: node.fixed_background_acts_as_scroll,
                canvas_contents: fragment
                    .paint
                    .CanvasLayoutObjectIdentity()
                    .map(|layout_object| CanvasContentsContext {
                        layout_object,
                        properties: child_properties.clone(),
                    })
                    .or(canvas_contents),
                properties: child_properties,
                rendering_context_id: child_rendering_context_id,
                should_flatten_inherited_transform: child_should_flatten,
                can_inherit_backface_visibility: child_can_inherit_backface,
                directly_composited_container_paint_offset_subpixel_delta: node
                    .directly_composited_container_paint_offset_subpixel_delta,
                subtree_should_use_infinite_cull_rect: infinite_cull.subtree_should_use_infinite,
                flex_or_grid_items: children_are_flex_or_grid_items,
                svg: svg_view_box,
            },
            revision: fragment.pre_paint_revision,
            subtree_revision: fragment.pre_paint_subtree_revision,
            raw_properties: node.properties.clone(),
            raw_contents: node.contents_properties.clone(),
            raw_fixed_background: node.fixed_background_properties.clone(),
            raw_horizontal_scrollbar: node.horizontal_scrollbar_properties.clone(),
            raw_vertical_scrollbar: node.vertical_scrollbar_properties.clone(),
            raw_mask: node.mask_properties.clone(),
            raw_opacity: node.opacity,
            subtree_keys: own_keys.clone(),
            own_keys,
            nodes: 1,
            computed_pass: self.walk.walk_generation,
            physical_children: Vec::new(),
            contains_local_frames: false,
        });
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolation_without_blending_descendants_does_not_create_effect() {
        assert!(!NeedsCssBlendEffect(
            PaintBlendMode::kNormal,
            true,
            true,
            false,
        ));
        assert!(NeedsCssBlendEffect(
            PaintBlendMode::kNormal,
            true,
            true,
            true,
        ));
        assert!(NeedsCssBlendEffect(
            PaintBlendMode::kMultiply,
            false,
            false,
            false,
        ));
    }
}
