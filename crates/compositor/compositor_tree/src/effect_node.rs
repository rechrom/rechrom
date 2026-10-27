// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::{
    geometry::{Size, Vector2dF},
    property_ids::{kContentsRootPropertyNodeId, kInvalidPropertyNodeId, kRootPropertyNodeId},
    property_types::{
        AddCornerRadiiToTracedValue, AddLinearGradientToTracedValue, AddRectFToTracedValue,
        BlendMode, FilterOperations, MaskFilterInfo, PropertyPath, PropertyTrace, SubtreeCaptureId,
        ViewTransitionElementResourceId,
    },
    transform::PropertyTreeTransform,
    ElementId,
};

/// Filter capabilities required by retained effect-tree update policy.
pub trait EffectTreeFilters: FilterOperations {
    fn AllowsLCDText(&self) -> bool;
    fn HasZoomFilter(&self) -> bool;
}

impl EffectTreeFilters for paint_model::cc::paint::filter_operations::FilterOperations {
    fn AllowsLCDText(&self) -> bool {
        Self::AllowsLCDText(self)
    }

    fn HasZoomFilter(&self) -> bool {
        self.HasFilterOfType(paint_model::cc::paint::filter_operation::FilterType::ZOOM)
    }
}

// Source: cc/trees/effect_node.h:35-68
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderSurfaceReason {
    kNone,
    kRoot,
    k3dTransformFlattening,
    kBackdropScope,
    kBlendMode,
    kBlendModeDstIn,
    kOpacity,
    kOpacityAnimation,
    kFilter,
    kFilterAnimation,
    kBackdropFilter,
    kBackdropFilterAnimation,
    kRoundedCorner,
    kClipPath,
    kClipAxisAlignment,
    kMask,
    kTrilinearFiltering,
    kCache,
    kCopyRequest,
    kMirrored,
    kSubtreeIsBeingCaptured,
    kViewTransitionParticipant,
    kGradientMask,
    k2DScaleTransformWithCompositedDescendants,
    kUnboundedElement,
    kTest,
}
// Source: cc/trees/effect_node.cc:23-78
pub fn RenderSurfaceReasonToString(reason: RenderSurfaceReason) -> &'static str {
    match reason {
        RenderSurfaceReason::kNone => "none",
        RenderSurfaceReason::kRoot => "root",
        RenderSurfaceReason::k3dTransformFlattening => "3d transform flattening",
        RenderSurfaceReason::kBackdropScope => "backdrop scope",
        RenderSurfaceReason::kBlendMode => "blend mode",
        RenderSurfaceReason::kBlendModeDstIn => "blend mode kDstIn",
        RenderSurfaceReason::kOpacity => "opacity",
        RenderSurfaceReason::kOpacityAnimation => "opacity animation",
        RenderSurfaceReason::kFilter => "filter",
        RenderSurfaceReason::kFilterAnimation => "filter animation",
        RenderSurfaceReason::kBackdropFilter => "backdrop filter",
        RenderSurfaceReason::kBackdropFilterAnimation => "backdrop filter animation",
        RenderSurfaceReason::kRoundedCorner => "rounded corner",
        RenderSurfaceReason::kGradientMask => "gradient mask",
        RenderSurfaceReason::kClipPath => "clip path",
        RenderSurfaceReason::kClipAxisAlignment => "clip axis alignment",
        RenderSurfaceReason::kMask => "mask",
        RenderSurfaceReason::kTrilinearFiltering => "trilinear filtering",
        RenderSurfaceReason::kCache => "cache",
        RenderSurfaceReason::kCopyRequest => "copy request",
        RenderSurfaceReason::kMirrored => "mirrored",
        RenderSurfaceReason::kSubtreeIsBeingCaptured => "subtree being captured",
        RenderSurfaceReason::kViewTransitionParticipant => "view transition participant",
        RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants => {
            "2D scale transform with composited descendants"
        }
        RenderSurfaceReason::kUnboundedElement => "unbounded element",
        RenderSurfaceReason::kTest => "test",
    }
}

// Source: cc/trees/effect_node.h:72-233
// Source: cc/trees/effect_node.cc:15-21
#[derive(Clone, Debug, PartialEq)]
pub struct EffectNode<F: FilterOperations, P: PropertyPath> {
    // Source: cc/trees/effect_node.h:74
    pub id: i32,
    // Source: cc/trees/effect_node.h:76
    pub parent_id: i32,
    // Source: cc/trees/effect_node.h:82
    pub element_id: ElementId,
    // Source: cc/trees/effect_node.h:84
    pub opacity: f32,
    // Source: cc/trees/effect_node.h:85
    pub screen_space_opacity: f32,
    // Source: cc/trees/effect_node.h:87
    pub filters: F,
    // Source: cc/trees/effect_node.h:88
    pub backdrop_filters: F,
    // Source: cc/trees/effect_node.h:89
    pub backdrop_filter_bounds: Option<P>,
    // Source: cc/trees/effect_node.h:90
    pub backdrop_filter_quality: f32,
    // Source: cc/trees/effect_node.h:94
    pub backdrop_mask_element_id: ElementId,
    // Source: cc/trees/effect_node.h:99
    pub mask_filter_info: MaskFilterInfo,
    // Source: cc/trees/effect_node.h:101
    pub surface_contents_scale: Vector2dF,
    // Source: cc/trees/effect_node.h:103
    pub subtree_capture_id: SubtreeCaptureId,
    // Source: cc/trees/effect_node.h:104
    pub subtree_size: Size,
    // Source: cc/trees/effect_node.h:108
    pub transform_id: i32,
    // Source: cc/trees/effect_node.h:111
    pub clip_id: i32,
    // Source: cc/trees/effect_node.h:116
    pub target_id: i32,
    // Source: cc/trees/effect_node.h:122
    pub view_transition_target_id: i32,
    // Source: cc/trees/effect_node.h:123
    pub closest_ancestor_with_cached_render_surface_id: i32,
    // Source: cc/trees/effect_node.h:124
    pub closest_ancestor_with_copy_request_id: i32,
    // Source: cc/trees/effect_node.h:125
    pub closest_ancestor_being_captured_id: i32,
    // Source: cc/trees/effect_node.h:126
    pub closest_ancestor_with_shared_element_id: i32,
    // Source: cc/trees/effect_node.h:130
    pub view_transition_element_resource_id: ViewTransitionElementResourceId,
    // Source: cc/trees/effect_node.h:132
    pub blend_mode: BlendMode,
    // Source: cc/trees/effect_node.h:136
    pub render_surface_reason: RenderSurfaceReason,
    // Source: cc/trees/effect_node.h:138
    pub cache_render_surface: bool,
    // Source: cc/trees/effect_node.h:142
    pub has_copy_request: bool,
    // Source: cc/trees/effect_node.h:148
    pub hidden_by_backface_visibility: bool,
    // Source: cc/trees/effect_node.h:152
    pub double_sided: bool,
    // Source: cc/trees/effect_node.h:153
    pub trilinear_filtering: bool,
    // Source: cc/trees/effect_node.h:159
    pub is_drawn: bool,
    // Source: cc/trees/effect_node.h:167
    pub only_draws_visible_content: bool,
    // Source: cc/trees/effect_node.h:171
    pub subtree_hidden: bool,
    // Source: cc/trees/effect_node.h:174
    pub has_potential_filter_animation: bool,
    // Source: cc/trees/effect_node.h:177
    pub has_potential_backdrop_filter_animation: bool,
    // Source: cc/trees/effect_node.h:180
    pub has_potential_opacity_animation: bool,
    // Source: cc/trees/effect_node.h:185
    pub has_masking_child: bool,
    // Source: cc/trees/effect_node.h:189
    pub effect_changed: bool,
    // Source: cc/trees/effect_node.h:193
    pub subtree_has_copy_request: bool,
    // Source: cc/trees/effect_node.h:197
    pub is_fast_rounded_corner: bool,
    // Source: cc/trees/effect_node.h:201
    pub node_or_ancestor_has_fast_rounded_corner: bool,
    // Source: cc/trees/effect_node.h:207
    pub lcd_text_disallowed_by_filter: bool,
    // Source: cc/trees/effect_node.h:215
    pub lcd_text_disallowed_by_backdrop_filter: bool,
    // Source: cc/trees/effect_node.h:219
    pub may_have_backdrop_effect: bool,
    // Source: cc/trees/effect_node.h:222
    pub needs_effect_for_2d_scale_transform: bool,
}
impl<F: FilterOperations, P: PropertyPath> Default for EffectNode<F, P> {
    // Source: cc/trees/effect_node.h:78-222
    fn default() -> Self {
        Self {
            id: kInvalidPropertyNodeId,
            parent_id: kInvalidPropertyNodeId,
            element_id: ElementId::default(),
            opacity: 1.0,
            screen_space_opacity: 1.0,
            filters: F::default(),
            backdrop_filters: F::default(),
            backdrop_filter_bounds: None,
            backdrop_filter_quality: 1.0,
            backdrop_mask_element_id: ElementId::default(),
            mask_filter_info: MaskFilterInfo::default(),
            surface_contents_scale: Vector2dF::default(),
            subtree_capture_id: SubtreeCaptureId::default(),
            subtree_size: Size::default(),
            transform_id: kRootPropertyNodeId,
            clip_id: kRootPropertyNodeId,
            target_id: 1,
            view_transition_target_id: kInvalidPropertyNodeId,
            closest_ancestor_with_cached_render_surface_id: kInvalidPropertyNodeId,
            closest_ancestor_with_copy_request_id: kInvalidPropertyNodeId,
            closest_ancestor_being_captured_id: kInvalidPropertyNodeId,
            closest_ancestor_with_shared_element_id: kInvalidPropertyNodeId,
            view_transition_element_resource_id: ViewTransitionElementResourceId::default(),
            blend_mode: BlendMode::kSrcOver,
            render_surface_reason: RenderSurfaceReason::kNone,
            cache_render_surface: false,
            has_copy_request: false,
            hidden_by_backface_visibility: false,
            double_sided: true,
            trilinear_filtering: false,
            is_drawn: true,
            only_draws_visible_content: true,
            subtree_hidden: false,
            has_potential_filter_animation: false,
            has_potential_backdrop_filter_animation: false,
            has_potential_opacity_animation: false,
            has_masking_child: false,
            effect_changed: false,
            subtree_has_copy_request: false,
            is_fast_rounded_corner: false,
            node_or_ancestor_has_fast_rounded_corner: false,
            lcd_text_disallowed_by_filter: false,
            lcd_text_disallowed_by_backdrop_filter: false,
            may_have_backdrop_effect: false,
            needs_effect_for_2d_scale_transform: false,
        }
    }
}
impl<F: FilterOperations, P: PropertyPath> EffectNode<F, P> {
    // Source: cc/trees/effect_node.cc:15-17
    pub fn new() -> Self {
        Self::default()
    }
    // Source: cc/trees/effect_node.h:224-226
    pub fn HasRenderSurface(&self) -> bool {
        self.render_surface_reason != RenderSurfaceReason::kNone
    }
    // Source: cc/trees/effect_node.cc:80-152
    pub fn AsValueInto(&self, value: &mut impl PropertyTrace) {
        value.SetString(
            "backdrop_mask_element_id",
            &self.backdrop_mask_element_id.ToString(),
        );
        value.SetInteger("id", self.id);
        value.SetInteger("parent_id", self.parent_id);
        value.SetString("element_id", &self.element_id.ToString());
        value.SetDouble("opacity", f64::from(self.opacity));
        if !self.filters.IsEmpty() {
            value.SetString("filters", &self.filters.ToString());
        }
        if !self.backdrop_filters.IsEmpty() {
            value.SetString("backdrop_filters", &self.backdrop_filters.ToString());
        }
        value.SetDouble(
            "backdrop_filter_quality",
            f64::from(self.backdrop_filter_quality),
        );
        value.SetBoolean(
            "lcd_text_disallowed_by_filter",
            self.lcd_text_disallowed_by_filter,
        );
        value.SetBoolean(
            "lcd_text_disallowed_by_backdrop_filter",
            self.lcd_text_disallowed_by_backdrop_filter,
        );
        if !self.mask_filter_info.IsEmpty() {
            AddRectFToTracedValue("mask_filter_bounds", &self.mask_filter_info.bounds(), value);
            if self.mask_filter_info.HasRoundedCorners() {
                AddCornerRadiiToTracedValue(
                    "mask_filter_rounded_corners_radii",
                    &self.mask_filter_info.rounded_corner_bounds,
                    value,
                );
                value.SetBoolean(
                    "mask_filter_is_fast_rounded_corner",
                    self.is_fast_rounded_corner,
                );
            }
            if self.mask_filter_info.HasGradientMask() {
                AddLinearGradientToTracedValue(
                    "mask_filter_gradient_mask",
                    self.mask_filter_info.gradient_mask.as_ref().unwrap(),
                    value,
                );
            }
            if let Some(clip_id) = self.mask_filter_info.clip_id {
                value.SetInteger("mask_filter_clip_id", clip_id);
            }
        }
        value.SetString("blend_mode", self.blend_mode.Name());
        value.SetString("subtree_capture_id", &self.subtree_capture_id.ToString());
        value.SetString(
            "subtree_size",
            &format!(
                "{}x{}",
                self.subtree_size.width(),
                self.subtree_size.height()
            ),
        );
        value.SetBoolean("cache_render_surface", self.cache_render_surface);
        value.SetBoolean("has_copy_request", self.has_copy_request);
        value.SetBoolean(
            "hidden_by_backface_visibility",
            self.hidden_by_backface_visibility,
        );
        value.SetBoolean("double_sided", self.double_sided);
        value.SetBoolean("trilinear_filtering", self.trilinear_filtering);
        value.SetBoolean("is_drawn", self.is_drawn);
        value.SetBoolean(
            "only_draws_visible_content",
            self.only_draws_visible_content,
        );
        value.SetBoolean("subtree_hidden", self.subtree_hidden);
        value.SetBoolean(
            "has_potential_filter_animation",
            self.has_potential_filter_animation,
        );
        value.SetBoolean(
            "has_potential_backdrop_filter_animation",
            self.has_potential_backdrop_filter_animation,
        );
        value.SetBoolean(
            "has_potential_opacity_animation",
            self.has_potential_opacity_animation,
        );
        value.SetBoolean("has_masking_child", self.has_masking_child);
        value.SetBoolean("effect_changed", self.effect_changed);
        value.SetBoolean("subtree_has_copy_request", self.subtree_has_copy_request);
        value.SetString(
            "render_surface_reason",
            RenderSurfaceReasonToString(self.render_surface_reason),
        );
        value.SetInteger("transform_id", self.transform_id);
        value.SetInteger("clip_id", self.clip_id);
        value.SetInteger("target_id", self.target_id);
        value.SetInteger(
            "closest_ancestor_with_cached_render_surface_id",
            self.closest_ancestor_with_cached_render_surface_id,
        );
        value.SetInteger(
            "closest_ancestor_with_copy_request_id",
            self.closest_ancestor_with_copy_request_id,
        );
        value.SetInteger(
            "closest_ancestor_being_captured_id",
            self.closest_ancestor_being_captured_id,
        );
        if self.view_transition_element_resource_id.IsValid() {
            value.SetString(
                "view_transition_element_resource_id",
                &self.view_transition_element_resource_id.ToString(),
            );
        }
    }
}

pub fn EffectiveOpacity<F: FilterOperations, P: PropertyPath>(node: &EffectNode<F, P>) -> f32 {
    if node.subtree_hidden {
        0.0
    } else {
        node.opacity
    }
}

pub fn PropagateEffectChanged<F: FilterOperations, P: PropertyPath>(
    node: &mut EffectNode<F, P>,
    parent: Option<&EffectNode<F, P>>,
) {
    if parent.is_some_and(|parent| parent.effect_changed) {
        node.effect_changed = true;
    }
}

/// Updates the inherited/non-geometric state for one retained effect node.
/// The caller supplies the already-computed transform backface state and keeps
/// ownership of tree storage; all effect policy remains package-local.
pub fn UpdateEffectNodeInheritedState<F: EffectTreeFilters, P: PropertyPath>(
    node: &mut EffectNode<F, P>,
    mut parent: Option<&mut EffectNode<F, P>>,
    is_active_tree: bool,
    transform_is_showing_backface: bool,
) {
    node.screen_space_opacity = EffectiveOpacity(node);
    if let Some(parent) = parent.as_deref() {
        node.screen_space_opacity *= parent.screen_space_opacity;
        node.subtree_hidden |= parent.subtree_hidden;
    }

    if node.has_copy_request || node.cache_render_surface || node.subtree_capture_id.is_valid() {
        node.is_drawn = true;
    } else if EffectiveOpacity(node) == 0.0
        && (!node.has_potential_opacity_animation || is_active_tree)
        && node.backdrop_filters.IsEmpty()
    {
        node.is_drawn = false;
    } else if let Some(parent) = parent.as_deref() {
        node.is_drawn = parent.is_drawn;
    } else {
        node.is_drawn = true;
    }

    PropagateEffectChanged(node, parent.as_deref());
    node.lcd_text_disallowed_by_filter =
        node.has_potential_filter_animation || !node.filters.AllowsLCDText();
    if let Some(parent) = parent.as_deref() {
        node.lcd_text_disallowed_by_filter |= parent.lcd_text_disallowed_by_filter;
    }
    node.node_or_ancestor_has_fast_rounded_corner = node.is_fast_rounded_corner;
    if let Some(parent) = parent.as_deref() {
        node.node_or_ancestor_has_fast_rounded_corner |=
            parent.node_or_ancestor_has_fast_rounded_corner;
    }

    node.hidden_by_backface_visibility = if parent
        .as_deref()
        .is_some_and(|parent| parent.hidden_by_backface_visibility)
    {
        true
    } else if node.double_sided {
        false
    } else {
        transform_is_showing_backface
    };

    node.has_masking_child = false;
    if node.blend_mode == BlendMode::kDstIn {
        if let Some(parent) = parent.as_deref_mut() {
            parent.has_masking_child = true;
        }
    }

    node.only_draws_visible_content = !node.has_copy_request
        && !node.subtree_capture_id.is_valid()
        && !node.view_transition_element_resource_id.IsValid();
    if let Some(parent) = parent.as_deref() {
        node.only_draws_visible_content &= parent.only_draws_visible_content;
    }
    if !node.backdrop_filters.IsEmpty() {
        node.only_draws_visible_content &= !node.backdrop_filters.HasZoomFilter();
    }

    if node.view_transition_element_resource_id.IsValid() {
        node.closest_ancestor_with_shared_element_id = node.id;
    } else if let Some(parent) = parent.as_deref() {
        node.closest_ancestor_with_shared_element_id =
            parent.closest_ancestor_with_shared_element_id;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceContentsScaleInputs<'a, T: PropertyTreeTransform> {
    pub to_screen: &'a T,
    pub device_scale_factor: f32,
    pub device_transform_scale_factor: f32,
    pub page_scale_factor: f32,
    pub external_page_scale_factor: f32,
    pub transform_in_page_scale_subtree: bool,
    pub size_at_external_scale: bool,
}

/// Returns whether a render surface's contents scale changed and therefore
/// invalidates clip/transform-derived caches.
pub fn UpdateEffectSurfaceContentsScale<
    F: FilterOperations,
    P: PropertyPath,
    T: PropertyTreeTransform,
>(
    node: &mut EffectNode<F, P>,
    inputs: SurfaceContentsScaleInputs<'_, T>,
) -> bool {
    if !node.HasRenderSurface() {
        node.surface_contents_scale = Vector2dF::new(1.0, 1.0);
        return false;
    }
    let mut layer_scale_factor = inputs.device_scale_factor * inputs.device_transform_scale_factor;
    if inputs.transform_in_page_scale_subtree {
        layer_scale_factor *= inputs.page_scale_factor;
    }
    let old_scale = node.surface_contents_scale;
    node.surface_contents_scale = inputs
        .to_screen
        .ComputeTransform2dScaleComponents(layer_scale_factor);
    if node.id != kContentsRootPropertyNodeId && inputs.size_at_external_scale {
        node.surface_contents_scale
            .Scale(inputs.external_page_scale_factor);
    }
    if node.render_surface_reason == RenderSurfaceReason::k2DScaleTransformWithCompositedDescendants
    {
        node.surface_contents_scale
            .set_x(node.surface_contents_scale.x().abs().ceil());
        node.surface_contents_scale
            .set_y(node.surface_contents_scale.y().abs().ceil());
    }
    old_scale != node.surface_contents_scale
}
