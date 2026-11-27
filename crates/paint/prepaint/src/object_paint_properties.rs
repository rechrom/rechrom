//! Source: third_party/blink/renderer/core/paint/object_paint_properties.h
//! (Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15).
//!
//! The paint property nodes one fragment occurrence of a LayoutObject owns,
//! one typed slot per Chromium `NodeId` (object_paint_properties.h:116-178).
//! Each slot holds the one live node of that role: `Update*` creates it
//! (kNodeAddedOrRemoved) or updates the resident node in place
//! (`Node::Update`, keeping its pointer identity and every child's parent
//! link), and `Clear*` drops the slot and reports whether a node existed
//! (object_paint_properties.h:34-44,64-67). Artifacts that still name a
//! cleared node keep it alive through their `Arc`.
//!
//! # Mapping from the walker-owned keyed store (S2 switch-over)
//!
//! Today `PrePaintTreeWalk` keeps these nodes outside the fragment:
//!
//! - `property_node_store` (`PaintPropertyNodeStore`): four
//!   `KeyedPropertyNodes` maps keyed by `PaintPropertyKey { owner, role }`
//!   with `owner = Fragment { client, fragment }`. Each
//!   `(owner, role) -> Arc<Node>` entry becomes the slot
//!   `ObjectPaintPropertySlot::FromRole(role)` of the
//!   `ObjectPaintProperties` held by that occurrence's
//!   `PrePaintFragmentData` (the occurrence *is* the owner, so the key
//!   disappears). `UpdateTransform/Clip/Effect/Scroll` become the slot's
//!   `Update*`; the store's change bookkeeping (`record_change`,
//!   `NoteCompositorChange`, intersection-observation state) stays the
//!   builder's `OnUpdate*` / `OnClear*` work
//!   (paint_property_tree_builder.cc `OnUpdateTransform` etc.), driven by the
//!   value `Update*` returns and the bool `Clear*` returns. The transform
//!   animation state the store derives from the resident node
//!   (`HasActiveTransformAnimation` on old and new state) becomes the
//!   caller's `animation_state` argument, as in Chromium.
//!   `PaintPropertyOwner::Viewport` entries are not ObjectPaintProperties
//!   (see the role table on [`ObjectPaintPropertySlot::FromRole`]) and stay
//!   in the store.
//! - `property_keys` (node id -> key) and `RegisterProperty` /
//!   `PropertySlotKey`: the id-to-slot indirection is not needed once the
//!   builder calls the slot's `Update*` directly. `SlotId` only supplies the
//!   dump id of a node being created: the `new_id` closure of `Update*`.
//! - `own_key_log` / `NodeCache::own_keys`: the set of slots an object
//!   registered in its build is the set of present slots
//!   ([`ObjectPaintProperties::HasSlot`], [`ObjectPaintProperties::Slots`]);
//!   `HasOwnPaintProperties` / `HasOwnPropertyRole` become
//!   `PrePaintFragmentData::HasPaintProperties` / `HasPropertyRole`.
//! - `own_change_log`: the per-build maximum of the `Update*` results, which
//!   the builder accumulates in its context as Chromium's
//!   `full_context_.force_subtree_update_reasons` / `property_changed` do.
//! - `released_property_keys`, `full_slot_sweep` and `RetainLiveSlots`:
//!   a slot dies with its role (`Clear*` returns true) or with the
//!   `PrePaintFragmentData` (FragmentData destruction, fragment_data.cc
//!   ClearPaintProperties / FragmentDataList::Shrink). No end-of-walk sweep
//!   remains; the builder raises kNodeAddedOrRemoved and clears the
//!   GeometryMapper caches when a `Clear*` returns true.

use std::sync::Arc;

use paint_input_types::layout_input::TransformMatrix;
use paint_model::paint_property_tree::{
    ClipPaintPropertyNode, ClipPaintPropertyNodeState, EffectAnimationState,
    EffectPaintPropertyNode, EffectPaintPropertyNodeState, PaintPropertyChangeType,
    PaintPropertyNodeLifecycle, PaintPropertyRole, ScrollPaintPropertyNode,
    ScrollPaintPropertyNodeState, TransformAnimationState, TransformPaintPropertyNode,
    TransformPaintPropertyNodeState,
};
use paint_model::property_cell::PropertyWriter;

/// The Create / in-place Update pair of one paint property node kind
/// (transform_paint_property_node.h:150-185, clip_paint_property_node.h:
/// 150-178, effect_paint_property_node.h:197-227,
/// scroll_paint_property_node.h:99-120). The parent is the `State`'s
/// `parent` link. Clip and scroll nodes take Chromium's empty
/// `AnimationState`, spelled `()`.
pub trait PaintPropertySlotNode: Sized {
    type State;
    type AnimationState: Default;
    /// `NodeType::Create(parent, std::move(state))`; `id` is the dump id.
    fn CreateSlotNode(id: u64, state: Self::State) -> Self;
    /// `node->Update(parent, std::move(state), animation_state)`.
    fn UpdateSlotNode(
        &self,
        writer: &mut PropertyWriter,
        state: Self::State,
        animation_state: Self::AnimationState,
    ) -> PaintPropertyChangeType;
}

impl PaintPropertySlotNode for TransformPaintPropertyNode {
    type State = TransformPaintPropertyNodeState;
    type AnimationState = TransformAnimationState;
    fn CreateSlotNode(id: u64, state: Self::State) -> Self {
        Self::new(PaintPropertyNodeLifecycle::default(), id, state)
    }
    fn UpdateSlotNode(
        &self,
        writer: &mut PropertyWriter,
        state: Self::State,
        animation_state: Self::AnimationState,
    ) -> PaintPropertyChangeType {
        self.Update(writer, state, animation_state)
    }
}

impl PaintPropertySlotNode for ClipPaintPropertyNode {
    type State = ClipPaintPropertyNodeState;
    type AnimationState = ();
    fn CreateSlotNode(id: u64, state: Self::State) -> Self {
        Self::new(PaintPropertyNodeLifecycle::default(), id, state)
    }
    fn UpdateSlotNode(
        &self,
        writer: &mut PropertyWriter,
        state: Self::State,
        _: (),
    ) -> PaintPropertyChangeType {
        self.Update(writer, state)
    }
}

impl PaintPropertySlotNode for EffectPaintPropertyNode {
    type State = EffectPaintPropertyNodeState;
    type AnimationState = EffectAnimationState;
    fn CreateSlotNode(id: u64, state: Self::State) -> Self {
        Self::new(PaintPropertyNodeLifecycle::default(), id, state)
    }
    fn UpdateSlotNode(
        &self,
        writer: &mut PropertyWriter,
        state: Self::State,
        animation_state: Self::AnimationState,
    ) -> PaintPropertyChangeType {
        self.Update(writer, state, animation_state)
    }
}

impl PaintPropertySlotNode for ScrollPaintPropertyNode {
    type State = ScrollPaintPropertyNodeState;
    type AnimationState = ();
    fn CreateSlotNode(id: u64, state: Self::State) -> Self {
        Self::new(PaintPropertyNodeLifecycle::default(), id, state)
    }
    fn UpdateSlotNode(
        &self,
        writer: &mut PropertyWriter,
        state: Self::State,
        _: (),
    ) -> PaintPropertyChangeType {
        self.Update(writer, state)
    }
}

/// Source: object_paint_properties.h:528-551 `Update<NodeType>`: create the
/// slot's node when absent, otherwise update the resident node in place.
fn UpdateSlot<T: PaintPropertySlotNode>(
    slot: &mut Option<Arc<T>>,
    writer: &mut PropertyWriter,
    new_id: impl FnOnce() -> u64,
    state: T::State,
    animation_state: T::AnimationState,
) -> PaintPropertyChangeType {
    if let Some(node) = slot {
        return node.UpdateSlotNode(writer, state, animation_state);
    }
    *slot = Some(Arc::new(T::CreateSlotNode(new_id(), state)));
    PaintPropertyChangeType::NodeAddedOrRemoved
}

/// The node kind of a slot (object_paint_properties.h:180-188 NodeIdRange).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintPropertyNodeKind {
    Transform,
    Scroll,
    Effect,
    Clip,
}

/// A shared reference to an existing live node of one kind, for
/// [`ObjectPaintProperties::AdoptSlotNode`].
#[derive(Clone, Debug)]
pub enum PaintPropertySlotNodeRef {
    Transform(Arc<TransformPaintPropertyNode>),
    Scroll(Arc<ScrollPaintPropertyNode>),
    Effect(Arc<EffectPaintPropertyNode>),
    Clip(Arc<ClipPaintPropertyNode>),
}

impl PaintPropertySlotNodeRef {
    pub fn Kind(&self) -> PaintPropertyNodeKind {
        match self {
            Self::Transform(_) => PaintPropertyNodeKind::Transform,
            Self::Scroll(_) => PaintPropertyNodeKind::Scroll,
            Self::Effect(_) => PaintPropertyNodeKind::Effect,
            Self::Clip(_) => PaintPropertyNodeKind::Clip,
        }
    }

    /// Whether both name the same live node (node identity is its pointer).
    pub fn PtrEq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Transform(a), Self::Transform(b)) => Arc::ptr_eq(a, b),
            (Self::Scroll(a), Self::Scroll(b)) => Arc::ptr_eq(a, b),
            (Self::Effect(a), Self::Effect(b)) => Arc::ptr_eq(a, b),
            (Self::Clip(a), Self::Clip(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

macro_rules! object_paint_properties {
    ($(
        $kind:ident $node:ty, $has:ident, $for_nodes:ident => [
            $($slot:ident $field:ident $get:ident $update:ident $clear:ident;)*
        ]
    )*) => {
        /// Source: object_paint_properties.h:45-597 `ObjectPaintProperties`.
        /// Chromium stores the nodes in a `SparseVector`; Rechrom keeps one
        /// typed `Option<Arc<_>>` per slot inside the boxed allocation that
        /// `PrePaintFragmentData` creates only for objects with nodes (the
        /// same rare-data placement, fragment_data.h:206-235).
        #[derive(Debug, Default)]
        pub struct ObjectPaintProperties {
            $($($field: Option<Arc<$node>>,)*)*
        }

        /// Source: object_paint_properties.h:116-178 `NodeId`, in Chromium
        /// order within each kind, followed by the Rechrom-only slots (see
        /// [`ObjectPaintPropertySlot::FromRole`]). The parent alias ids
        /// (kTransformAlias, kEffectAlias, kClipAlias) have no slot: see
        /// `PrePaintFragmentData::ContentsTransform`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ObjectPaintPropertySlot {
            $($($slot,)*)*
        }

        impl ObjectPaintPropertySlot {
            /// Every slot in `NodeId` order.
            pub const ALL: &'static [Self] = &[$($(Self::$slot,)*)*];

            pub const fn Kind(self) -> PaintPropertyNodeKind {
                match self {
                    $($(Self::$slot => PaintPropertyNodeKind::$kind,)*)*
                }
            }
        }

        impl ObjectPaintProperties {
            $($(
                pub fn $get(&self) -> Option<&Arc<$node>> {
                    self.$field.as_ref()
                }

                /// Source: object_paint_properties.h:68-87 `ADD_NODE`
                /// `Update##function`. `new_id` supplies the dump id when the
                /// slot is empty and a node is created.
                pub fn $update(
                    &mut self,
                    writer: &mut PropertyWriter,
                    new_id: impl FnOnce() -> u64,
                    state: <$node as PaintPropertySlotNode>::State,
                    animation_state: <$node as PaintPropertySlotNode>::AnimationState,
                ) -> PaintPropertyChangeType {
                    UpdateSlot(&mut self.$field, writer, new_id, state, animation_state)
                }

                /// Source: object_paint_properties.h:86 `Clear##function`:
                /// true when an existing node was removed.
                pub fn $clear(&mut self) -> bool {
                    self.$field.take().is_some()
                }
            )*)*

            $(
                /// Source: object_paint_properties.h:184-188 `HasNode<NodeType>`.
                pub fn $has(&self) -> bool {
                    false $(|| self.$field.is_some())*
                }

                /// Source: object_paint_properties.h:455-469 `ForNodes`: runs
                /// `action` on each present node of the kind in slot order and
                /// returns true as soon as `action` does.
                pub fn $for_nodes(&self, mut action: impl FnMut(&Arc<$node>) -> bool) -> bool {
                    $(
                        if let Some(node) = &self.$field {
                            if action(node) {
                                return true;
                            }
                        }
                    )*
                    false
                }
            )*

            /// Whether `slot` holds a node.
            pub fn HasSlot(&self, slot: ObjectPaintPropertySlot) -> bool {
                match slot {
                    $($(ObjectPaintPropertySlot::$slot => self.$field.is_some(),)*)*
                }
            }

            /// Stores `node`, an existing live node of `slot`'s kind, in
            /// `slot` without creating a second node. S2 dual-write
            /// transition: the walker's keyed store still creates and
            /// updates the node and this slot names the same `Arc`, so each
            /// slot keeps one live node (object_paint_properties.h:528-551).
            pub fn AdoptSlotNode(
                &mut self,
                slot: ObjectPaintPropertySlot,
                node: PaintPropertySlotNodeRef,
            ) {
                match (slot, node) {
                    $($(
                        (
                            ObjectPaintPropertySlot::$slot,
                            PaintPropertySlotNodeRef::$kind(node),
                        ) => self.$field = Some(node),
                    )*)*
                    (slot, node) => {
                        panic!("{slot:?} cannot hold a {:?} node", node.Kind())
                    }
                }
            }

            /// The node `slot` holds, as a shared reference.
            pub fn SlotNode(&self, slot: ObjectPaintPropertySlot) -> Option<PaintPropertySlotNodeRef> {
                match slot {
                    $($(
                        ObjectPaintPropertySlot::$slot => {
                            self.$field.clone().map(PaintPropertySlotNodeRef::$kind)
                        }
                    )*)*
                }
            }

            /// Removes the node of `slot`; true when one existed.
            pub fn ClearSlot(&mut self, slot: ObjectPaintPropertySlot) -> bool {
                match slot {
                    $($(ObjectPaintPropertySlot::$slot => self.$clear(),)*)*
                }
            }
        }
    };
}

object_paint_properties! {
    // object_paint_properties.h:117-139,302-317.
    Transform TransformPaintPropertyNode, HasTransformNode, ForTransformNodes => [
        PaintOffsetTranslation paint_offset_translation PaintOffsetTranslation
            UpdatePaintOffsetTranslation ClearPaintOffsetTranslation;
        StickyTranslation sticky_translation StickyTranslation
            UpdateStickyTranslation ClearStickyTranslation;
        AnchorPositionScrollTranslation anchor_position_scroll_translation
            AnchorPositionScrollTranslation UpdateAnchorPositionScrollTranslation
            ClearAnchorPositionScrollTranslation;
        ElementCanvasTransform element_canvas_transform ElementCanvasTransform
            UpdateElementCanvasTransform ClearElementCanvasTransform;
        Translate translate Translate UpdateTranslate ClearTranslate;
        Rotate rotate Rotate UpdateRotate ClearRotate;
        Scale scale Scale UpdateScale ClearScale;
        Offset offset Offset UpdateOffset ClearOffset;
        Transform transform Transform UpdateTransform ClearTransform;
        ContentTranslation content_translation ContentTranslation
            UpdateContentTranslation ClearContentTranslation;
        Perspective perspective Perspective UpdatePerspective ClearPerspective;
        ReplacedContentTransform replaced_content_transform ReplacedContentTransform
            UpdateReplacedContentTransform ClearReplacedContentTransform;
        ScrollTranslation scroll_translation ScrollTranslation
            UpdateScrollTranslation ClearScrollTranslation;
        UnboundedWrapperTransform unbounded_wrapper_transform UnboundedWrapperTransform
            UpdateUnboundedWrapperTransform ClearUnboundedWrapperTransform;
        UnboundedInnerTransform unbounded_inner_transform UnboundedInnerTransform
            UpdateUnboundedInnerTransform ClearUnboundedInnerTransform;
        // Rechrom-only (see FromRole).
        SvgLocalTransform svg_local_transform SvgLocalTransform
            UpdateSvgLocalTransform ClearSvgLocalTransform;
        TextCombineTransform text_combine_transform TextCombineTransform
            UpdateTextCombineTransform ClearTextCombineTransform;
    ]
    // object_paint_properties.h:141-143,321-324.
    Scroll ScrollPaintPropertyNode, HasScrollNode, ForScrollNodes => [
        Scroll scroll Scroll UpdateScroll ClearScroll;
    ]
    // object_paint_properties.h:145-159,370-381.
    Effect EffectPaintPropertyNode, HasEffectNode, ForEffectNodes => [
        ElementCaptureEffect element_capture_effect ElementCaptureEffect
            UpdateElementCaptureEffect ClearElementCaptureEffect;
        ViewTransitionScopeRootEffect view_transition_scope_root_effect
            ViewTransitionScopeRootEffect UpdateViewTransitionScopeRootEffect
            ClearViewTransitionScopeRootEffect;
        ViewTransitionEffect view_transition_effect ViewTransitionEffect
            UpdateViewTransitionEffect ClearViewTransitionEffect;
        UnboundedWrapperEffect unbounded_wrapper_effect UnboundedWrapperEffect
            UpdateUnboundedWrapperEffect ClearUnboundedWrapperEffect;
        Effect effect Effect UpdateEffect ClearEffect;
        Filter filter Filter UpdateFilter ClearFilter;
        Mask mask Mask UpdateMask ClearMask;
        ClipPathMask clip_path_mask ClipPathMask UpdateClipPathMask ClearClipPathMask;
        VerticalScrollbarEffect vertical_scrollbar_effect VerticalScrollbarEffect
            UpdateVerticalScrollbarEffect ClearVerticalScrollbarEffect;
        HorizontalScrollbarEffect horizontal_scrollbar_effect HorizontalScrollbarEffect
            UpdateHorizontalScrollbarEffect ClearHorizontalScrollbarEffect;
        ScrollCornerEffect scroll_corner_effect ScrollCornerEffect
            UpdateScrollCornerEffect ClearScrollCornerEffect;
    ]
    // object_paint_properties.h:161-174,436-446.
    Clip ClipPaintPropertyNode, HasClipNode, ForClipNodes => [
        ClipPathClip clip_path_clip ClipPathClip UpdateClipPathClip ClearClipPathClip;
        MaskClip mask_clip MaskClip UpdateMaskClip ClearMaskClip;
        CssClip css_clip CssClip UpdateCssClip ClearCssClip;
        OverflowControlsClip overflow_controls_clip OverflowControlsClip
            UpdateOverflowControlsClip ClearOverflowControlsClip;
        BackgroundClip background_clip BackgroundClip
            UpdateBackgroundClip ClearBackgroundClip;
        PixelMovingFilterClipExpander pixel_moving_filter_clip_expander
            PixelMovingFilterClipExpander UpdatePixelMovingFilterClipExpander
            ClearPixelMovingFilterClipExpander;
        InnerBorderRadiusClip inner_border_radius_clip InnerBorderRadiusClip
            UpdateInnerBorderRadiusClip ClearInnerBorderRadiusClip;
        OverflowClip overflow_clip OverflowClip UpdateOverflowClip ClearOverflowClip;
        CssClipFixedPosition css_clip_fixed_position CssClipFixedPosition
            UpdateCssClipFixedPosition ClearCssClipFixedPosition;
        InnerBorderShapeClip inner_border_shape_clip InnerBorderShapeClip
            UpdateInnerBorderShapeClip ClearInnerBorderShapeClip;
        // Rechrom-only (see FromRole).
        BackdropFilterClip backdrop_filter_clip BackdropFilterClip
            UpdateBackdropFilterClip ClearBackdropFilterClip;
    ]
}

impl ObjectPaintPropertySlot {
    /// The ObjectPaintProperties slot of a walker `PaintPropertyRole` with a
    /// `PaintPropertyOwner::Fragment` owner. `None` for the frame-level
    /// roles, which are not ObjectPaintProperties:
    ///
    /// | Rechrom role | Chromium owner / slot |
    /// |---|---|
    /// | ViewportClip | none: Rechrom's frame-view viewport clip (parent of the LayoutView clips); Chromium clips the root with LayoutView's OverflowClip |
    /// | OverscrollElasticityTransform | VisualViewport `overscroll_elasticity_transform_node_` (visual_viewport.h:370) |
    /// | PageScale | VisualViewport `page_scale_node_` (visual_viewport.h:371) |
    /// | VisualViewportScrollTranslation | VisualViewport `scroll_translation_node_` (visual_viewport.h:372) |
    /// | VisualViewportScroll | VisualViewport `scroll_node_` (visual_viewport.h:373) |
    ///
    /// Every other role maps to the Chromium slot of the same name, except
    /// three Rechrom-only slots kept separate so S2 switches writers without
    /// changing behaviour:
    ///
    /// - SvgLocalTransform: Chromium folds the SVG local transform into the
    ///   `Transform` slot (object_paint_properties.h:228-232,
    ///   UpdateTransformForSVGChild). Rechrom's builder can emit it next to
    ///   CSS transform roles (the SVG shape local-transform adaptation), so it
    ///   keeps its own slot until that adaptation folds into `Transform`.
    /// - TextCombineTransform: Chromium applies the text-combine scale while
    ///   painting (text_combine_painter.cc); Rechrom builds a transform node.
    /// - BackdropFilterClip: Chromium carries the backdrop bounds as the
    ///   Effect state's `backdrop_filter_info` (paint_property_tree_builder.cc:
    ///   4429); Rechrom's Effect state names a clip node
    ///   (`backdrop_filter_clip`).
    ///
    /// Chromium slots without a Rechrom role today: AnchorPositionScroll
    /// Translation (Rechrom puts anchor scroll data on PaintOffsetTranslation
    /// state), ContentTranslation, UnboundedWrapperTransform,
    /// UnboundedInnerTransform, ElementCaptureEffect,
    /// ViewTransitionScopeRootEffect, ViewTransitionEffect,
    /// UnboundedWrapperEffect, ClipPathMask, ScrollCornerEffect,
    /// CssClipFixedPosition, and the three isolation aliases.
    pub const fn FromRole(role: PaintPropertyRole) -> Option<Self> {
        use PaintPropertyRole as R;
        Some(match role {
            R::ViewportClip
            | R::OverscrollElasticityTransform
            | R::PageScale
            | R::VisualViewportScroll
            | R::VisualViewportScrollTranslation => return None,
            R::PaintOffsetTranslation => Self::PaintOffsetTranslation,
            R::StickyTranslation => Self::StickyTranslation,
            R::ElementCanvasTransform => Self::ElementCanvasTransform,
            R::TextCombineTransform => Self::TextCombineTransform,
            R::Translate => Self::Translate,
            R::Rotate => Self::Rotate,
            R::Scale => Self::Scale,
            R::Offset => Self::Offset,
            R::Transform => Self::Transform,
            R::Perspective => Self::Perspective,
            R::SvgLocalTransform => Self::SvgLocalTransform,
            R::ClipPathClip => Self::ClipPathClip,
            R::CssClip => Self::CssClip,
            R::BackgroundClip => Self::BackgroundClip,
            R::BackdropFilterClip => Self::BackdropFilterClip,
            R::MaskClip => Self::MaskClip,
            R::Effect => Self::Effect,
            R::Mask => Self::Mask,
            R::Filter => Self::Filter,
            R::PixelMovingFilterClipExpander => Self::PixelMovingFilterClipExpander,
            R::OverflowControlsClip => Self::OverflowControlsClip,
            R::InnerBorderRadiusClip => Self::InnerBorderRadiusClip,
            R::InnerBorderShapeClip => Self::InnerBorderShapeClip,
            R::OverflowClip => Self::OverflowClip,
            R::HorizontalScrollbarEffect => Self::HorizontalScrollbarEffect,
            R::VerticalScrollbarEffect => Self::VerticalScrollbarEffect,
            R::Scroll => Self::Scroll,
            R::ScrollTranslation => Self::ScrollTranslation,
            R::ReplacedContentTransform => Self::ReplacedContentTransform,
        })
    }

    /// The walker role of this slot, for the role-keyed consumers
    /// (compositing reasons, compositor element ids) until they read slots.
    /// `None` for the Chromium slots Rechrom does not build yet.
    pub const fn Role(self) -> Option<PaintPropertyRole> {
        use PaintPropertyRole as R;
        Some(match self {
            Self::PaintOffsetTranslation => R::PaintOffsetTranslation,
            Self::StickyTranslation => R::StickyTranslation,
            Self::ElementCanvasTransform => R::ElementCanvasTransform,
            Self::Translate => R::Translate,
            Self::Rotate => R::Rotate,
            Self::Scale => R::Scale,
            Self::Offset => R::Offset,
            Self::Transform => R::Transform,
            Self::Perspective => R::Perspective,
            Self::ReplacedContentTransform => R::ReplacedContentTransform,
            Self::ScrollTranslation => R::ScrollTranslation,
            Self::SvgLocalTransform => R::SvgLocalTransform,
            Self::TextCombineTransform => R::TextCombineTransform,
            Self::Scroll => R::Scroll,
            Self::Effect => R::Effect,
            Self::Filter => R::Filter,
            Self::Mask => R::Mask,
            Self::VerticalScrollbarEffect => R::VerticalScrollbarEffect,
            Self::HorizontalScrollbarEffect => R::HorizontalScrollbarEffect,
            Self::ClipPathClip => R::ClipPathClip,
            Self::MaskClip => R::MaskClip,
            Self::CssClip => R::CssClip,
            Self::OverflowControlsClip => R::OverflowControlsClip,
            Self::BackgroundClip => R::BackgroundClip,
            Self::PixelMovingFilterClipExpander => R::PixelMovingFilterClipExpander,
            Self::InnerBorderRadiusClip => R::InnerBorderRadiusClip,
            Self::OverflowClip => R::OverflowClip,
            Self::InnerBorderShapeClip => R::InnerBorderShapeClip,
            Self::BackdropFilterClip => R::BackdropFilterClip,
            Self::AnchorPositionScrollTranslation
            | Self::ContentTranslation
            | Self::UnboundedWrapperTransform
            | Self::UnboundedInnerTransform
            | Self::ElementCaptureEffect
            | Self::ViewTransitionScopeRootEffect
            | Self::ViewTransitionEffect
            | Self::UnboundedWrapperEffect
            | Self::ClipPathMask
            | Self::ScrollCornerEffect
            | Self::CssClipFixedPosition => return None,
        })
    }
}

impl ObjectPaintProperties {
    /// `HasNode<PaintPropertyNode>()`: whether any slot holds a node.
    pub fn HasAnyNode(&self) -> bool {
        self.HasTransformNode()
            || self.HasScrollNode()
            || self.HasEffectNode()
            || self.HasClipNode()
    }

    /// The present slots in `NodeId` order (replaces `NodeCache::own_keys`).
    pub fn Slots(&self) -> impl Iterator<Item = ObjectPaintPropertySlot> + '_ {
        ObjectPaintPropertySlot::ALL
            .iter()
            .copied()
            .filter(|slot| self.HasSlot(*slot))
    }

    /// Whether the slot of a walker role holds a node; false for the
    /// frame-level roles.
    pub fn HasRole(&self, role: PaintPropertyRole) -> bool {
        ObjectPaintPropertySlot::FromRole(role).is_some_and(|slot| self.HasSlot(slot))
    }

    /// Source: object_paint_properties.h:266-269.
    pub fn HasCSSTransformPropertyNode(&self) -> bool {
        self.translate.is_some()
            || self.rotate.is_some()
            || self.scale.is_some()
            || self.offset.is_some()
            || self.transform.is_some()
    }

    /// Source: object_paint_properties.h:271-300: the existing CSS transform
    /// nodes (Translate, Rotate, Scale, Offset, Transform) outside to inside.
    pub fn CSSTransformPropertiesOutsideToInside(
        &self,
    ) -> impl Iterator<Item = &Arc<TransformPaintPropertyNode>> + '_ {
        [
            &self.translate,
            &self.rotate,
            &self.scale,
            &self.offset,
            &self.transform,
        ]
        .into_iter()
        .flatten()
    }

    /// Source: object_paint_properties.h:503-510: the resident Transform
    /// node updated in place.
    pub fn DirectlyUpdateTransformAndOrigin(
        &self,
        writer: &mut PropertyWriter,
        matrix: TransformMatrix,
        origin: [f64; 3],
        animation_state: TransformAnimationState,
    ) -> PaintPropertyChangeType {
        let node = self
            .transform
            .as_ref()
            .expect("DirectlyUpdateTransformAndOrigin needs a Transform node");
        node.DirectlyUpdateTransformAndOrigin(writer, matrix, origin, animation_state)
    }

    /// Source: object_paint_properties.h:511-523.
    pub fn DirectlyUpdateOpacity(
        &self,
        writer: &mut PropertyWriter,
        opacity: f32,
        animation_state: EffectAnimationState,
    ) -> PaintPropertyChangeType {
        let Some(node) = &self.effect else {
            debug_assert!(false, "DirectlyUpdateOpacity needs an Effect node");
            return PaintPropertyChangeType::NodeAddedOrRemoved;
        };
        node.DirectlyUpdateOpacity(writer, opacity, animation_state)
    }

    /// Source: object_paint_properties.h:479-491 `Validate` (DCHECK builds).
    pub fn Validate(&self) {
        debug_assert!(
            self.scroll_translation.is_none() || self.replaced_content_transform.is_none(),
            "Replaced elements don't scroll so there should never be both a scroll \
             translation and a replaced content transform."
        );
        debug_assert!(
            self.clip_path_clip.is_none() || self.clip_path_mask.is_none(),
            "ClipPathClip and ClipPathMask should be mutually exclusive."
        );
    }
}
