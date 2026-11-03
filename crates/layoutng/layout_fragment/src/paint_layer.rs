#![allow(non_snake_case)]

//! Chromium PaintLayer identity and ownership. Compositing and paint-property
//! construction are owned by the paint package; this source subset supplies
//! the persistent DisplayItemClient used by PaintLayerPainter chunk scopes.

use foundation::graphics_types::graphics::dom_node_id::DOMNodeId;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{
    gfx, DynamicTo, EPosition, EVisibility, FloatValueForLength, Member, PaintInvalidationReason,
    PhysicalOffset, PhysicalRect, RuntimeEnabledFeatures, String as BlinkString, Traceable,
    Visitor,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::static_position::LogicalStaticPosition;
use layoutng_style::css::css_reflection_direction::CSSReflectionDirection;
use layoutng_style::style::box_reflection::{BoxReflection, ReflectionDirection};

use super::super::layout_object::LayoutObject;
#[cfg(test)]
use super::super::layout_object::StyleChangeContext;

#[cfg(test)]
use super::super::layout_block::LayoutBlock;
use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::super::layout_inline::LayoutInline;
use super::super::layout_node_metadata::Node;
use super::super::map_coordinates_flags::MapCoordinatesMode;

// Source: core/paint/paint_layer.h:93-105.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayerPositionVisibility {
    kAnchorsIntersectionVisible = 1,
    kAnchorsCssVisible = 1 << 1,
    kChainedAnchorsVisible = 1 << 2,
    kNoOverflow = 1 << 3,
}

// Source: core/paint/paint_layer.h:693-694.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaintingContainerType {
    kParent,
    kStackingContext,
}

// Source: core/paint/paint_layer.h:675-680.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DescendantDependentFlagsUpdateFlag {
    kNeedsDescendantDependentUpdate,
    kDoesNotNeedDescendantDependentUpdate,
}

// The GarbageCollected base has no data. Preserve the DisplayItemClient base
// as a real subobject, independently of the owning LayoutObject's client.
// cpp: core/paint/paint_layer.h:168-171,688-738
#[repr(C)]
pub struct PaintLayer {
    display_item_client_: DisplayItemClient,
    layout_object_: Member<LayoutBoxModelObject>,
    is_self_painting_layer_: bool,
    has_visible_content_: bool,
    needs_descendant_dependent_flags_update_: bool,
    needs_visual_overflow_recalc_: bool,
    has_visible_self_painting_descendant_: bool,
    has_non_isolated_descendant_with_blend_mode_: bool,
    has_fixed_position_descendant_: bool,
    has_non_contained_absolute_position_descendant_: bool,
    has_stacked_descendant_in_current_stacking_context_: bool,
    has_descendant_with_transform_anim_: bool,
    has_self_painting_layer_descendant_: bool,
    has_backdrop_filter_descendant_: bool,
    has_3d_transformed_descendant_: bool,
    // Source: core/paint/paint_layer.h:786-788,812-815. Chromium stores the
    // edges as bitfields; the extracted owner retains the same value together.
    static_position_: LogicalStaticPosition,
    invisible_for_position_visibility_: u8,
    descendant_needs_check_position_visibility_: bool,
    is_destroyed_: bool,
}

impl PaintLayer {
    fn ObjectIsVisible(object: &LayoutObject) -> bool {
        let style = object.StyleRef();
        style.Visibility() == EVisibility::kVisible
            || (style.HasReferenceFilter()
                && RuntimeEnabledFeatures::SvgFilterPaintsForHiddenContentEnabled())
    }

    fn ObjectOwnsSelfPaintingLayer(object: *const LayoutObject) -> bool {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if box_model.is_null() {
            return false;
        }
        let layer = unsafe { &*box_model }.Layer();
        !layer.is_null() && unsafe { &*layer }.IsSelfPaintingLayer()
    }

    // cpp: core/paint/paint_layer.cc:178-217
    pub fn new(layout_object: *mut LayoutBoxModelObject) -> Self {
        assert!(!layout_object.is_null());
        let mut layer = Self {
            display_item_client_: DisplayItemClient::default(),
            layout_object_: Member::from_ptr(layout_object),
            is_self_painting_layer_: unsafe { &*layout_object }.LayerTypeRequired()
                == PaintLayerType::kNormalPaintLayer,
            has_visible_content_: false,
            needs_descendant_dependent_flags_update_: true,
            needs_visual_overflow_recalc_: true,
            has_visible_self_painting_descendant_: false,
            has_non_isolated_descendant_with_blend_mode_: false,
            has_fixed_position_descendant_: false,
            has_non_contained_absolute_position_descendant_: false,
            has_stacked_descendant_in_current_stacking_context_: false,
            has_descendant_with_transform_anim_: false,
            has_self_painting_layer_descendant_: false,
            has_backdrop_filter_descendant_: false,
            has_3d_transformed_descendant_: false,
            static_position_: LogicalStaticPosition::default(),
            invisible_for_position_visibility_: 0,
            descendant_needs_check_position_visibility_: false,
            is_destroyed_: false,
        };
        // Source: core/paint/paint_layer.cc:216.
        layer.UpdateScrollableArea();
        layer
    }

    // cpp: core/paint/paint_layer.h:185-190
    pub fn GetLayoutObject(&self) -> &LayoutBoxModelObject {
        debug_assert!(!self.is_destroyed_);
        unsafe { &*self.layout_object_.Get() }
    }

    // Source: core/paint/paint_layer.h:183-185.
    pub fn GetLayoutBox(&self) -> *mut LayoutBox {
        DynamicTo::<LayoutBox>(self.layout_object_.Get())
    }

    // Source: core/paint/paint_layer.cc:956-963 and paint_layer.h:187.
    pub fn GetLayoutBoxWithBlockFragments(&self) -> *const LayoutBox {
        let layout_box = self.GetLayoutBox();
        if layout_box.is_null() || !unsafe { &*layout_box }.CanTraversePhysicalFragments() {
            return std::ptr::null();
        }
        debug_assert!(!unsafe { &*layout_box }.IsFragmentLessBox());
        layout_box
    }

    // Source: core/paint/paint_layer.h:189. Rechrom does not retain a second
    // mutable PaintLayer tree; the nearest ancestor layer is derived from the
    // authoritative LayoutObject parent chain.
    pub fn Parent(&self) -> *mut PaintLayer {
        let parent = self.GetLayoutObject().Parent();
        if parent.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*parent }.EnclosingLayer()
        }
    }

    // Source: core/paint/paint_layer.h:191-192. Rechrom derives the links from
    // the authoritative LayoutObject tree instead of retaining a second
    // mutable child/sibling list.
    pub fn FirstChild(&self) -> *mut PaintLayer {
        let root = self.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
        let mut object = unsafe { &*root }.NextInPreOrder(root);
        while !object.is_null() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(object);
            if !box_model.is_null() {
                let layer = unsafe { &*box_model }.Layer();
                if !layer.is_null() {
                    return layer;
                }
            }
            object = unsafe { &*object }.NextInPreOrder(root);
        }
        std::ptr::null_mut()
    }

    // Source: core/paint/paint_layer.h:191. See FirstChild() for ownership.
    pub fn NextSibling(&self) -> *mut PaintLayer {
        let parent = self.Parent();
        if parent.is_null() {
            return std::ptr::null_mut();
        }
        let parent_root = unsafe { &*parent }.GetLayoutObject() as *const LayoutBoxModelObject
            as *const LayoutObject;
        let object = self.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
        let mut next = unsafe { &*object }.NextInPreOrderAfterChildren(parent_root);
        while !next.is_null() {
            let box_model = DynamicTo::<LayoutBoxModelObject>(next);
            if !box_model.is_null() {
                let layer = unsafe { &*box_model }.Layer();
                if !layer.is_null() {
                    return layer;
                }
            }
            next = unsafe { &*next }.NextInPreOrder(parent_root);
        }
        std::ptr::null_mut()
    }

    // Source: core/paint/paint_layer.h:190. See FirstChild() for ownership.
    pub fn PreviousSibling(&self) -> *mut PaintLayer {
        let parent = self.Parent();
        if parent.is_null() {
            return std::ptr::null_mut();
        }
        let this = self as *const PaintLayer as *mut PaintLayer;
        let mut previous = std::ptr::null_mut();
        let mut child = unsafe { &*parent }.FirstChild();
        while !child.is_null() && child != this {
            previous = child;
            child = unsafe { &*child }.NextSibling();
        }
        if child == this {
            previous
        } else {
            std::ptr::null_mut()
        }
    }

    // Source: core/paint/paint_layer.h:193. See FirstChild() for ownership.
    pub fn LastChild(&self) -> *mut PaintLayer {
        let mut child = self.FirstChild();
        if child.is_null() {
            return child;
        }
        loop {
            let next = unsafe { &*child }.NextSibling();
            if next.is_null() {
                return child;
            }
            child = next;
        }
    }

    // Source: core/paint/paint_layer.h:225.
    pub fn IsRootLayer(&self) -> bool {
        self.GetLayoutObject().IsLayoutView()
    }

    // Source: core/paint/paint_layer.h:218-223.
    pub fn Root(&self) -> *mut PaintLayer {
        let mut current = self as *const PaintLayer as *mut PaintLayer;
        loop {
            let parent = unsafe { &*current }.Parent();
            if parent.is_null() {
                return current;
            }
            current = parent;
        }
    }

    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.display_item_client_
    }

    // Source: core/paint/paint_layer.cc:260-262.
    pub fn DebugName(&self) -> BlinkString {
        self.GetLayoutObject().DebugName()
    }

    // Source: core/paint/paint_layer.cc:264-267. PaintLayer deliberately
    // forwards `false`, independently of the caller's internal-content flag.
    pub fn OwnerNodeId(&self, _is_internal_content: bool) -> DOMNodeId {
        self.GetLayoutObject().OwnerNodeId(false)
    }

    // Source: core/paint/paint_layer.cc:1118-1124 and paint_layer.h:379.
    pub fn EnclosingNode(&self) -> *mut Node {
        let mut object = self.GetLayoutObject() as *const LayoutBoxModelObject as *mut LayoutObject;
        while !object.is_null() {
            let current = unsafe { &*object };
            let node = current.GetNode();
            if !node.is_null() {
                return node;
            }
            object = current.Parent();
        }
        panic!("PaintLayer must have an enclosing Node")
    }

    // Source: core/paint/paint_layer.cc:269-273.
    pub fn PaintsWithFilters(&self) -> bool {
        self.HasFilterInducingProperty()
    }

    // Source: core/paint/paint_layer.cc:2088-2093 and paint_layer.h:365.
    pub fn BackdropFilterReferenceBox(&self) -> gfx::RectF {
        let layout_object = self.layout_object_.Get().cast::<LayoutObject>();
        let layout_inline = DynamicTo::<LayoutInline>(layout_object);
        if !layout_inline.is_null() {
            return gfx::RectF::from(unsafe { &*layout_inline }.PhysicalLinesBoundingBox());
        }
        let layout_box = self.GetLayoutBox();
        debug_assert!(!layout_box.is_null());
        gfx::RectF::from(unsafe { &*layout_box }.PhysicalBorderBoxRect())
    }

    // Source: core/paint/paint_layer.cc:164-174. The value is the viewport
    // passed to FilterEffectBuilder, not the visual-overflow reference box.
    pub fn ComputeFilterViewport(&self) -> Option<gfx::SizeF> {
        let layout_object = self.layout_object_.Get().cast::<LayoutObject>();
        let layout_inline = DynamicTo::<LayoutInline>(layout_object);
        if !layout_inline.is_null() {
            return Some(gfx::SizeF::from(
                unsafe { &*layout_inline }.PhysicalLinesBoundingBox().size,
            ));
        }
        let layout_box = self.GetLayoutBox();
        debug_assert!(!layout_box.is_null());
        let layout_box = unsafe { &*layout_box };
        if layout_box.IsSVGForeignObject() {
            return None;
        }
        Some(gfx::SizeF::from(layout_box.StitchedSize()))
    }

    // Source: core/paint/paint_layer.cc:2195-2202 and paint_layer.h:559.
    pub fn LocalBoundingBox(&self) -> PhysicalRect {
        let mut rect = self.GetLayoutObject().VisualOverflowRect();
        if self.GetLayoutObject().IsEffectiveRootScroller() || self.IsRootLayer() {
            let view = self.GetLayoutObject().View();
            assert!(
                !view.is_null(),
                "root PaintLayer must belong to a LayoutView"
            );
            rect.Unite(&PhysicalRect::new(
                rect.offset,
                unsafe { &*view }.ViewRect().size,
            ));
        }
        rect
    }

    // Source: core/paint/paint_layer.cc:2657-2666. The source appends the
    // box-reflect operation after the authored filter list. Keep that order
    // here; the reflection mask does not alter BoxReflection::MapRect().
    pub fn MapRectForFilterFloat(&self, rect: &gfx::RectF) -> gfx::RectF {
        if !self.HasFilterThatMovesPixels() {
            return *rect;
        }

        let object = self.GetLayoutObject();
        let style = object.StyleRef();
        let mut mapped = style.Filter().MapRect(rect);
        if object.HasReflection() && object.IsBox() {
            let reflection = unsafe { &*style.BoxReflect() };
            let box_ = unsafe { &*self.GetLayoutBox() };
            let frame_size =
                PhysicalRect::new(box_.FirstFragment().PaintOffset(), box_.StitchedSize())
                    .PixelSnappedSize();
            let (direction, offset) = match reflection.Direction() {
                CSSReflectionDirection::kReflectionAbove => (
                    ReflectionDirection::kVerticalReflection,
                    -FloatValueForLength(reflection.Offset(), frame_size.height() as f32),
                ),
                CSSReflectionDirection::kReflectionBelow => (
                    ReflectionDirection::kVerticalReflection,
                    2.0 * frame_size.height() as f32
                        + FloatValueForLength(reflection.Offset(), frame_size.height() as f32),
                ),
                CSSReflectionDirection::kReflectionLeft => (
                    ReflectionDirection::kHorizontalReflection,
                    -FloatValueForLength(reflection.Offset(), frame_size.width() as f32),
                ),
                CSSReflectionDirection::kReflectionRight => (
                    ReflectionDirection::kHorizontalReflection,
                    2.0 * frame_size.width() as f32
                        + FloatValueForLength(reflection.Offset(), frame_size.width() as f32),
                ),
            };
            mapped = BoxReflection::new(direction, offset).MapRect(&mapped);
        }
        mapped
    }

    pub fn MapRectForFilter(&self, rect: &PhysicalRect) -> PhysicalRect {
        if !self.HasFilterThatMovesPixels() {
            return *rect;
        }
        PhysicalRect::EnclosingRect(&self.MapRectForFilterFloat(&gfx::RectF::from(*rect)))
    }

    fn ExpandRectForSelfPaintingLayerChildren(
        &self,
        parent: &PaintLayer,
        result: &mut PhysicalRect,
    ) {
        let mut child = parent.FirstChild();
        while !child.is_null() {
            let child_layer = unsafe { &*child };
            if child_layer.IsSelfPaintingLayer() {
                let mut added_rect = child_layer.LocalBoundingBox();
                child_layer.ExpandRectForSelfPaintingDescendants(&mut added_rect);

                if child_layer.PaintsWithFilters() {
                    added_rect = child_layer.MapRectForFilter(&added_rect);
                }

                let transform = child_layer.GetLayoutObject().Transform();
                if !transform.is_null() {
                    added_rect = PhysicalRect::EnclosingRect(
                        &unsafe { &*transform }.MapRect(gfx::RectF::from(added_rect)),
                    );
                }

                let delta = child_layer.GetLayoutObject().LocalToAncestorPointPhysical(
                    &PhysicalOffset::default(),
                    self.GetLayoutObject(),
                    MapCoordinatesMode::kIgnoreTransforms.into(),
                );
                added_rect.Move(&delta);
                result.Unite(&added_rect);
            } else {
                // PaintLayerPaintOrderIterator's z-order lists surface stacked
                // descendants through non-self-painting layer bridges. Bounds
                // union is order-independent, so walking those bridges yields
                // the same participating layer set without moving the paint
                // iterator into Layout's ownership package.
                self.ExpandRectForSelfPaintingLayerChildren(child_layer, result);
            }
            child = child_layer.NextSibling();
        }
    }

    // Source: core/paint/paint_layer.cc:2204-2248.
    pub fn ExpandRectForSelfPaintingDescendants(&self, result: &mut PhysicalRect) {
        if self.GetLayoutObject().ChildPaintBlockedByDisplayLock() {
            return;
        }
        debug_assert_eq!(*result, self.LocalBoundingBox());
        if !self.HasSelfPaintingLayerDescendant() || self.KnownToClipSubtreeToPaddingBox() {
            return;
        }
        self.ExpandRectForSelfPaintingLayerChildren(self, result);
    }

    // Source: core/paint/paint_layer.cc:2250-2266.
    pub fn KnownToClipSubtreeToPaddingBox(&self) -> bool {
        let box_ = self.GetLayoutBox();
        if box_.is_null() {
            return false;
        }
        let box_ = unsafe { &*box_ };
        if !box_.ShouldClipOverflowAlongBothAxis()
            || self.HasNonContainedAbsolutePositionDescendant()
            || (self.HasFixedPositionDescendant() && !box_.CanContainFixedPositionObjects())
            || box_.StyleRef().OverflowClipMargin().is_some()
        {
            return false;
        }
        // A resident Layout tree is one local-frame tree, so its LayoutView
        // root layer is exactly the source's `is_root_layer_ && IsLocalRoot()`.
        !self.IsRootLayer()
    }

    // Source: core/paint/paint_layer.cc:2268-2273 and paint_layer.h:560.
    pub fn LocalBoundingBoxIncludingSelfPaintingDescendants(&self) -> PhysicalRect {
        let mut result = self.LocalBoundingBox();
        self.ExpandRectForSelfPaintingDescendants(&mut result);
        result
    }

    // Source: core/paint/paint_layer.cc:2275-2296.
    pub fn SupportsSubsequenceCaching(&self) -> bool {
        let box_ = self.GetLayoutBox();
        if !box_.is_null() {
            let box_ = unsafe { &*box_ };
            if box_.PhysicalFragmentCount() > 1 {
                return false;
            }
            if box_.IsSVGRoot() || box_.IsSVGForeignObject() {
                return true;
            }
            if box_.IsDocumentElement() {
                return false;
            }
        }
        self.GetLayoutObject().IsStacked()
    }

    /// Source contract: `paint_layer.h:451-457`.
    ///
    /// The extracted owner does not duplicate Chromium's two repaint
    /// bitfields. Their authoritative effect is the invalidation reason on the
    /// resident PaintLayer client, recursively across the same PaintLayer
    /// child links (which already stop at display-lock paint boundaries).
    pub fn SelfOrDescendantNeedsRepaint(&self) -> bool {
        if self.DisplayItemClient().GetPaintInvalidationReason()
            != foundation::PaintInvalidationReason::kNone
        {
            return true;
        }
        let mut child = self.FirstChild();
        while !child.is_null() {
            let child_layer = unsafe { &*child };
            if child_layer.SelfOrDescendantNeedsRepaint() {
                return true;
            }
            child = child_layer.NextSibling();
        }
        false
    }

    // Source: core/paint/paint_layer.h:263-289.
    pub fn SetStaticPositionFromNG(&mut self, position: &LogicalStaticPosition) {
        self.static_position_ = *position;
    }

    pub fn GetStaticPosition(&self) -> LogicalStaticPosition {
        self.static_position_
    }

    // Source: core/paint/paint_layer.cc:152-162.
    fn SlowContainingLayer(layout_object: &LayoutObject) -> *mut PaintLayer {
        let mut container = layout_object.ContainerWithSkipInfo(std::ptr::null_mut());
        while !container.is_null() {
            let current = unsafe { &*container };
            if current.HasLayer() {
                let box_model = DynamicTo::<LayoutBoxModelObject>(container);
                debug_assert!(!box_model.is_null());
                return unsafe { &*box_model }.Layer();
            }
            container = current.ContainerWithSkipInfo(std::ptr::null_mut());
        }
        std::ptr::null_mut()
    }

    // Source: core/paint/paint_layer.cc:606-636.
    pub fn ContainingLayer(&self) -> *mut PaintLayer {
        let layout_object =
            self.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
        let layout_object = unsafe { &*layout_object };
        if layout_object.IsOutOfFlowPositioned() {
            if layout_object.IsInsideMulticol() {
                return Self::SlowContainingLayer(layout_object);
            }
            let mut current = self.Parent();
            while !current.is_null() {
                let object = unsafe { &*current }.GetLayoutObject();
                let can_contain = if layout_object.IsFixedPositioned() {
                    object.CanContainFixedPositionObjects()
                } else {
                    object.CanContainAbsolutePositionObjects()
                };
                if can_contain {
                    break;
                }
                current = unsafe { &*current }.Parent();
            }
            return current;
        }
        if layout_object.IsColumnSpanAll() {
            return Self::SlowContainingLayer(layout_object);
        }
        self.Parent()
    }

    // Source: core/paint/paint_layer.cc:638-649.
    pub fn GetPaintingContainerType(&self) -> PaintingContainerType {
        let object = self.GetLayoutObject();
        if !RuntimeEnabledFeatures::StackingContextIsNotStackedEnabled()
            && object.IsReplacedNormalFlowStackingContext(object.StyleRef())
        {
            return PaintingContainerType::kParent;
        }
        if object.IsStacked() {
            PaintingContainerType::kStackingContext
        } else {
            PaintingContainerType::kParent
        }
    }

    // Source: core/paint/paint_layer.cc:651-655.
    pub fn PaintingContainer(&self) -> *mut PaintLayer {
        if self.GetPaintingContainerType() == PaintingContainerType::kParent {
            self.Parent()
        } else {
            self.AncestorStackingContext()
        }
    }

    // Source: core/paint/paint_layer.cc:657-664.
    pub fn AncestorStackingContext(&self) -> *mut PaintLayer {
        let mut ancestor = self.Parent();
        while !ancestor.is_null() {
            if unsafe { &*ancestor }.GetLayoutObject().IsStackingContext() {
                return ancestor;
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
        std::ptr::null_mut()
    }

    // Source: core/paint/paint_layer.cc:351-358 and paint_layer.h:241.
    pub fn DirtyVisibleContentStatus(&mut self) {
        self.MarkAncestorChainForFlagsUpdate(
            DescendantDependentFlagsUpdateFlag::kNeedsDescendantDependentUpdate,
        );
        if !self.IsSelfPaintingLayer() {
            let parent = self.Parent();
            if !parent.is_null() {
                unsafe { &mut *parent }.DirtyVisibleContentStatus();
            }
        }
    }

    // Source: core/paint/paint_layer.cc:360-376 and paint_layer.h:682-685.
    pub(crate) fn MarkAncestorChainForFlagsUpdate(
        &mut self,
        flag: DescendantDependentFlagsUpdateFlag,
    ) {
        let mut layer = self as *mut PaintLayer;
        while !layer.is_null() {
            let current = unsafe { &mut *layer };
            let object = unsafe { &mut *current.layout_object_.Get() };
            if current.needs_descendant_dependent_flags_update_ && object.NeedsPaintPropertyUpdate()
            {
                break;
            }
            if flag == DescendantDependentFlagsUpdateFlag::kNeedsDescendantDependentUpdate {
                current.needs_descendant_dependent_flags_update_ = true;
            }
            object.SetNeedsPaintPropertyUpdate();
            layer = current.Parent();
        }
    }

    // Source: core/paint/paint_layer.cc:378-384 and paint_layer.h:687.
    fn SetNeedsDescendantDependentFlagsUpdate(&mut self) {
        let mut layer = self as *mut PaintLayer;
        while !layer.is_null() {
            let current = unsafe { &mut *layer };
            if current.needs_descendant_dependent_flags_update_ {
                break;
            }
            current.needs_descendant_dependent_flags_update_ = true;
            layer = current.Parent();
        }
    }

    // Source: core/paint/paint_layer.cc:386-478,482-483,491,494,496,
    // 498-499,503,506,512-513
    // and paint_layer.h:406-425,440-442. Host display-lock notifications,
    // mutable StackingNode dirtiness, LayoutView background invalidation and
    // 3D paint-order status remain with their unavailable host owners; every
    // PaintLayer-owned descendant aggregate is maintained here.
    pub fn UpdateDescendantDependentFlags(&mut self) {
        if !self.needs_descendant_dependent_flags_update_ {
            return;
        }

        let old_has_non_isolated_descendant_with_blend_mode =
            self.has_non_isolated_descendant_with_blend_mode_;
        let mut has_visible_self_painting_descendant = false;
        self.has_non_isolated_descendant_with_blend_mode_ = false;
        self.has_fixed_position_descendant_ = false;
        self.has_non_contained_absolute_position_descendant_ = false;
        self.has_stacked_descendant_in_current_stacking_context_ = false;
        self.has_self_painting_layer_descendant_ = false;
        self.descendant_needs_check_position_visibility_ = false;
        self.has_backdrop_filter_descendant_ = false;
        self.has_descendant_with_transform_anim_ = false;

        let can_contain_abs = self.GetLayoutObject().CanContainAbsolutePositionObjects();
        let mut child = if self.GetLayoutObject().ChildPrePaintBlockedByDisplayLock() {
            std::ptr::null_mut()
        } else {
            self.FirstChild()
        };
        while !child.is_null() {
            let next = unsafe { &*child }.NextSibling();
            let child_layer = unsafe { &mut *child };
            child_layer.UpdateDescendantDependentFlags();
            let child_object = child_layer.GetLayoutObject();
            let child_style = child_object.StyleRef();

            if (child_layer.HasVisibleContent() && child_layer.IsSelfPaintingLayer())
                || child_layer.HasVisibleSelfPaintingDescendant()
            {
                has_visible_self_painting_descendant = true;
            }
            self.has_non_isolated_descendant_with_blend_mode_ |= (!child_object
                .IsStackingContext()
                && child_layer.HasNonIsolatedDescendantWithBlendMode())
                || child_style.HasBlendMode();
            self.has_fixed_position_descendant_ |= child_layer.HasFixedPositionDescendant()
                || child_style.GetPosition() == EPosition::kFixed;
            if !can_contain_abs {
                self.has_non_contained_absolute_position_descendant_ |= child_layer
                    .HasNonContainedAbsolutePositionDescendant()
                    || child_style.GetPosition() == EPosition::kAbsolute;
            }
            if !self.has_stacked_descendant_in_current_stacking_context_ {
                if child_object.IsStacked() {
                    self.has_stacked_descendant_in_current_stacking_context_ = true;
                } else if !child_object.IsStackingContext() {
                    self.has_stacked_descendant_in_current_stacking_context_ =
                        child_layer.has_stacked_descendant_in_current_stacking_context_;
                }
            }
            self.has_self_painting_layer_descendant_ |=
                child_layer.HasSelfPaintingLayerDescendant() || child_layer.IsSelfPaintingLayer();
            self.has_backdrop_filter_descendant_ |= child_layer.HasBackdropFilterDescendant()
                || child_style.HasNonInitialBackdropFilter();
            self.has_descendant_with_transform_anim_ |= child_layer
                .HasDescendantWithTransformAnim()
                || child_style.HasCurrentTransformRelatedAnimation();
            child = next;
        }

        if self.InvisibleForPositionVisibility()
            && !self.GetLayoutObject().IsStackingContext()
            && self.has_self_painting_layer_descendant_
        {
            let ancestor = self.AncestorStackingContext();
            if !ancestor.is_null() {
                unsafe { &mut *ancestor }.descendant_needs_check_position_visibility_ = true;
            }
        }

        self.SetHasVisibleSelfPaintingDescendant(has_visible_self_painting_descendant);
        self.UpdateHasVisibleContent();

        if old_has_non_isolated_descendant_with_blend_mode
            != self.has_non_isolated_descendant_with_blend_mode_
        {
            unsafe { &mut *self.layout_object_.Get() }.SetNeedsPaintPropertyUpdate();
        }

        self.Update3DTransformedDescendantStatus();

        self.needs_descendant_dependent_flags_update_ = false;
        if self.IsSelfPaintingLayer() && self.needs_visual_overflow_recalc_ {
            if !self.GetLayoutObject().ChildPrePaintBlockedByDisplayLock() {
                let object = self.layout_object_.Get().cast::<LayoutObject>();
                unsafe { &mut *object }.RecalcVisualOverflow();
            }
        }
        self.needs_visual_overflow_recalc_ = false;
    }

    // Source: core/paint/paint_layer.cc:517-550.
    fn UpdateHasVisibleContent(&mut self) {
        let previously_has_visible_content = self.has_visible_content_;
        let root = self.GetLayoutObject() as *const LayoutBoxModelObject as *const LayoutObject;
        if Self::ObjectIsVisible(unsafe { &*root }) {
            self.has_visible_content_ = true;
        } else {
            self.has_visible_content_ = false;
            let mut object = unsafe { &*root }.NextInPreOrder(root);
            while !object.is_null() {
                let current = unsafe { &*object };
                if Self::ObjectOwnsSelfPaintingLayer(object) {
                    object = current.NextInPreOrderAfterChildren(root);
                    continue;
                }
                if Self::ObjectIsVisible(current) {
                    self.has_visible_content_ = true;
                    break;
                }
                object = current.NextInPreOrder(root);
            }
        }
        if self.has_visible_content_ != previously_has_visible_content {
            unsafe { &mut *self.layout_object_.Get() }.SetShouldCheckForPaintInvalidation();
        }
    }

    // Source: core/paint/paint_layer.cc:578-592. Chromium's
    // PaintLayerPaintOrderIterator(kStackedChildren) walks stacked layers in
    // this stacking context, crossing non-stacking layer subtrees and treating
    // a descendant stacking context atomically. The live layer tree is derived
    // from LayoutObject ownership here, so this helper performs that same
    // membership walk without retaining a second mutable z-order list.
    fn Update3DTransformedDescendantStatus(&mut self) {
        self.has_3d_transformed_descendant_ = false;
        let mut child = self.FirstChild();
        while !child.is_null() {
            let child_layer = unsafe { &*child };
            let child_object = child_layer.GetLayoutObject();
            if child_object.IsStacked()
                && (child_layer.Has3DTransform()
                    || (child_layer.Preserves3D() && child_layer.Has3DTransformedDescendant()))
            {
                self.has_3d_transformed_descendant_ = true;
                return;
            }
            if !child_object.IsStackingContext()
                && Self::StackedSubtreeHas3DTransformedDescendant(child)
            {
                self.has_3d_transformed_descendant_ = true;
                return;
            }
            child = child_layer.NextSibling();
        }
    }

    fn StackedSubtreeHas3DTransformedDescendant(layer: *mut PaintLayer) -> bool {
        let mut child = unsafe { &*layer }.FirstChild();
        while !child.is_null() {
            let child_layer = unsafe { &*child };
            let child_object = child_layer.GetLayoutObject();
            if child_object.IsStacked()
                && (child_layer.Has3DTransform()
                    || (child_layer.Preserves3D() && child_layer.Has3DTransformedDescendant()))
            {
                return true;
            }
            if !child_object.IsStackingContext()
                && Self::StackedSubtreeHas3DTransformedDescendant(child)
            {
                return true;
            }
            child = child_layer.NextSibling();
        }
        false
    }

    // Source: core/paint/paint_layer.cc:561,570.
    fn SetHasVisibleSelfPaintingDescendant(&mut self, has_visible: bool) {
        self.has_visible_self_painting_descendant_ = has_visible;
    }

    // Source: core/paint/paint_layer.cc:666-672.
    pub fn SetNeedsCompositingInputsUpdate(&mut self) {
        self.MarkAncestorChainForFlagsUpdate(
            DescendantDependentFlagsUpdateFlag::kNeedsDescendantDependentUpdate,
        );
    }

    // Source: core/paint/paint_layer.cc:674-676.
    pub fn ScrollContainerStatusChanged(&mut self) {
        self.SetNeedsCompositingInputsUpdate();
    }

    // Source: core/paint/paint_layer.h:391-394.
    pub fn NeedsVisualOverflowRecalc(&self) -> bool {
        self.needs_visual_overflow_recalc_
    }

    // Source: core/paint/paint_layer.cc:678-690. Chromium's DCHECK-only ink
    // invalidation at 680-682 has no release behavior and is intentionally not
    // synthesized here.
    pub fn SetNeedsVisualOverflowRecalc(&mut self) {
        debug_assert!(self.IsSelfPaintingLayer());
        self.needs_visual_overflow_recalc_ = true;
        self.SetNeedsDescendantDependentFlagsUpdate();
    }

    // Source: core/paint/paint_layer.cc:275-291.
    pub fn ContainingScrollContainerLayer(&self, is_fixed_to_view: *mut bool) -> *const PaintLayer {
        let mut is_fixed = self.GetLayoutObject().IsFixedPositioned();
        let mut container = self.ContainingLayer();
        while !container.is_null() {
            let current = unsafe { &*container };
            if current.GetLayoutObject().IsScrollContainer() {
                if !is_fixed_to_view.is_null() {
                    unsafe { *is_fixed_to_view = is_fixed && current.IsRootLayer() };
                }
                debug_assert!(!current.GetLayoutObject().GetScrollableArea().is_null());
                return container;
            }
            is_fixed = current.GetLayoutObject().IsFixedPositioned();
            container = current.ContainingLayer();
        }
        if !is_fixed_to_view.is_null() {
            unsafe { *is_fixed_to_view = true };
        }
        std::ptr::null()
    }

    // Source: core/paint/paint_layer.cc:2331-2336.
    pub fn EnclosingSelfPaintingLayer(&self) -> *mut PaintLayer {
        let mut layer = self as *const PaintLayer as *mut PaintLayer;
        while !layer.is_null() && !unsafe { &*layer }.IsSelfPaintingLayer() {
            layer = unsafe { &*layer }.Parent();
        }
        layer
    }

    // Source: core/paint/paint_layer.cc:885-899.
    pub fn RequiresScrollableArea(&self) -> bool {
        let box_ = self.GetLayoutBox();
        if box_.is_null() {
            return false;
        }
        let box_ = unsafe { &*box_ };
        box_.IsScrollContainer() || box_.IsOverscrollContainer() || box_.CanResize()
    }

    // Source: core/paint/paint_layer.cc:901-907,917-922. Targeted snap-area
    // repair at 908-916 remains explicitly unsupported. The area allocation is
    // delegated across the existing Layout/Paint package boundary without
    // moving its storage or changing the public object graph.
    pub fn UpdateScrollableArea(&mut self) {
        let object = unsafe { &mut *self.layout_object_.Get() };
        if self.RequiresScrollableArea() == !object.GetScrollableArea().is_null() {
            return;
        }
        if object.GetScrollableArea().is_null() {
            object.CreateScrollableAreaForPaintLayer();
        } else {
            object.DisposeScrollableAreaForPaintLayer();
        }
        object.SetNeedsPaintPropertyUpdate();
    }

    // Source: core/paint/paint_layer.cc:594-599,601-604. VisibleSizeChanged()
    // at line 600 only notifies host-owned overlay-scrollbar/accessibility
    // systems and remains unsupported rather than being replaced by a signal.
    pub fn UpdateScrollingAfterLayout(&mut self) {
        if !self.RequiresScrollableArea() {
            return;
        }
        let area = self.GetLayoutObject().GetScrollableArea();
        debug_assert!(!area.is_null());
        unsafe { &mut *area }.UpdateAfterLayout();
        let box_ = self.GetLayoutBox();
        if !box_.is_null() && unsafe { &*box_ }.ScrollableAreaSizeChanged() {
            unsafe { &mut *box_ }.SetScrollableAreaSizeChanged(false);
        }
    }

    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    pub fn Id(&self) -> u64 {
        self.display_item_client_.Id() as u64
    }
    pub fn IsCacheable(&self) -> bool {
        self.display_item_client_.IsCacheable()
    }
    pub fn IsJustCreated(&self) -> bool {
        self.display_item_client_.IsJustCreated()
    }

    // cpp: core/paint/paint_layer.h:210
    pub fn IsSelfPaintingLayer(&self) -> bool {
        debug_assert!(!self.is_destroyed_);
        self.is_self_painting_layer_
    }

    // Source: core/paint/paint_layer.h:212-215.
    pub fn IsTransparent(&self) -> bool {
        let object = self.GetLayoutObject();
        object.StyleRef().HasOpacity() || object.StyleRef().HasBlendMode() || object.HasMask()
    }

    // Source: core/paint/paint_layer.h:302-305.
    pub fn Preserves3D(&self) -> bool {
        self.GetLayoutObject().Preserves3D()
    }

    pub fn Has3DTransform(&self) -> bool {
        let transform = self.GetLayoutObject().Transform();
        !transform.is_null() && !unsafe { &*transform }.Is2dTransform()
    }

    // Source: core/paint/paint_layer.h:509-512.
    pub fn Has3DTransformedDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_3d_transformed_descendant_
    }

    // Source: core/paint/paint_layer.h:307-311.
    pub fn HasFilterInducingProperty(&self) -> bool {
        self.GetLayoutObject().HasFilterInducingProperty()
    }

    // Source: core/paint/paint_layer.cc:2669-2677 and paint_layer.h:712.
    pub fn ComputeHasFilterThatMovesPixels(&self) -> bool {
        if !self.HasFilterInducingProperty() {
            return false;
        }
        let object = self.GetLayoutObject();
        let style = object.StyleRef();
        (style.HasFilter() && style.Filter().HasFilterThatMovesPixels()) || object.HasReflection()
    }

    // Source: core/paint/paint_layer.h:354-356. Chromium caches the result of
    // ComputeHasFilterThatMovesPixels(); Rechrom's immutable export boundary
    // computes the same value from the authoritative style instead.
    pub fn HasFilterThatMovesPixels(&self) -> bool {
        self.ComputeHasFilterThatMovesPixels()
    }

    // Source: core/paint/paint_layer.cc:2680-2686 and paint_layer.h:714.
    pub fn ComputeHasReferenceFilter(&self) -> bool {
        if !self.HasFilterInducingProperty() {
            return false;
        }
        let style = self.GetLayoutObject().StyleRef();
        style.HasFilter() && style.Filter().HasReferenceFilter()
    }

    // Source: core/paint/paint_layer.h:231-239.
    pub fn HasVisibleContent(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_visible_content_
    }

    pub fn HasVisibleSelfPaintingDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_visible_self_painting_descendant_
    }

    // Source: core/paint/paint_layer.h:406-425.
    pub fn HasFixedPositionDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_fixed_position_descendant_
    }

    pub fn HasNonContainedAbsolutePositionDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_non_contained_absolute_position_descendant_
    }

    pub fn HasDescendantWithTransformAnim(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_descendant_with_transform_anim_
    }

    pub fn HasSelfPaintingLayerDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_self_painting_layer_descendant_
    }

    pub fn HasBackdropFilterDescendant(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_backdrop_filter_descendant_
    }

    // Source: core/paint/paint_layer.cc:692-696,701-702 and paint_layer.h:440.
    // LayoutSVGRoot's distinct descendant-isolation state remains unsupported
    // until its real virtual dispatch owner is linked; do not replace it with
    // the undefined generic provider.
    pub fn HasNonIsolatedDescendantWithBlendMode(&self) -> bool {
        debug_assert!(!self.needs_descendant_dependent_flags_update_);
        self.has_non_isolated_descendant_with_blend_mode_
    }

    // Source: core/paint/paint_property_tree_builder.cc:4519-4537 and
    // paint_layer.h:337-342. The source stores the builder-context aggregate;
    // the immutable export computes the same ancestor fact once.
    pub fn IsUnderSVGHiddenContainer(&self) -> bool {
        debug_assert!(!self.is_destroyed_);
        let mut ancestor = self.GetLayoutObject().Parent();
        while !ancestor.is_null() {
            let current = unsafe { &*ancestor };
            if current.IsSVGHiddenContainer() {
                return true;
            }
            ancestor = current.Parent();
        }
        false
    }

    // Source: core/paint/paint_layer.cc:2834-2847 and
    // core/paint/paint_layer.h:541-548. The retained Paint boundary observes
    // this bit change in the same layout cycle; invalidating the layer client
    // prevents a previously visible/hidden subsequence from being reused.
    pub fn SetInvisibleForPositionVisibility(
        &mut self,
        visibility: LayerPositionVisibility,
        invisible: bool,
    ) {
        let was_invisible = self.InvisibleForPositionVisibility();
        if invisible {
            self.invisible_for_position_visibility_ |= visibility as u8;
        } else {
            self.invisible_for_position_visibility_ &= !(visibility as u8);
        }
        if self.InvisibleForPositionVisibility() != was_invisible {
            self.display_item_client_
                .Invalidate(PaintInvalidationReason::kStyle);
        }
    }

    pub fn InvisibleForPositionVisibility(&self) -> bool {
        self.invisible_for_position_visibility_ != 0
    }

    // Source: core/paint/paint_layer.cc:2867-2879. Chromium enables this walk
    // through CheckAncestorPositionVisibilityScope only for stacking contexts
    // that may contain an affected descendant. The immutable Layout->Paint
    // boundary can export the exact answer once, so it walks the same parent
    // layer chain unconditionally and stops before the nearest stacking
    // context layer.
    pub fn HasAncestorInvisibleForPositionVisibility(&self) -> bool {
        let mut object = self.GetLayoutObject().Parent();
        while !object.is_null() {
            let current = unsafe { &*object };
            let box_model = DynamicTo::<LayoutBoxModelObject>(object);
            if !box_model.is_null() {
                let layer = unsafe { &*box_model }.Layer();
                if !layer.is_null() {
                    if current.IsStackingContext() {
                        break;
                    }
                    if unsafe { &*layer }.InvisibleForPositionVisibility() {
                        return true;
                    }
                }
            }
            object = current.Parent();
        }
        false
    }

    // Source: core/paint/paint_layer.cc:573-575.
    pub fn IsZOrderListVisible(&self) -> bool {
        self.HasVisibleContent()
            || self.HasVisibleSelfPaintingDescendant()
            || self.HasViewTransitionName()
    }

    // Source: core/paint/paint_layer.h:551. Chromium caches this style fact on
    // PaintLayer and dirties mutable z-order lists when it changes. Rechrom's
    // paint-order lists are rebuilt from immutable PrePaint input, so reading
    // the authoritative style preserves the value without duplicating cache
    // invalidation state.
    pub fn HasViewTransitionName(&self) -> bool {
        !self
            .GetLayoutObject()
            .StyleRef()
            .ViewTransitionName()
            .Get()
            .is_null()
    }

    // cpp: core/paint/paint_layer.cc:2298-2310,2688-2693
    pub fn UpdateSelfPaintingLayer(&mut self) {
        let is_self_painting_layer =
            self.GetLayoutObject().LayerTypeRequired() == PaintLayerType::kNormalPaintLayer;
        if self.is_self_painting_layer_ == is_self_painting_layer {
            return;
        }
        // PaintLayer::SetNeedsRepaint invalidates the layer's own
        // DisplayItemClient before changing painting-container ownership.
        self.display_item_client_
            .Invalidate(PaintInvalidationReason::kLayout);
        self.is_self_painting_layer_ = is_self_painting_layer;
        if is_self_painting_layer {
            self.SetNeedsVisualOverflowRecalc();
        }
        let parent = self.Parent();
        if !parent.is_null() {
            unsafe { &mut *parent }.MarkAncestorChainForFlagsUpdate(
                DescendantDependentFlagsUpdateFlag::kNeedsDescendantDependentUpdate,
            );
        }
    }

    // Resource subscriptions/compositor trees are absent from this source
    // subset. Their owners remain outside this identity/lifetime translation.
    // cpp: core/paint/paint_layer.cc:225-257
    pub fn Destroy(&mut self) {
        debug_assert!(!self.is_destroyed_);
        self.is_destroyed_ = true;
    }
}

// cpp: core/paint/paint_layer.cc:2881-2892
impl Traceable for PaintLayer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.layout_object_);
        self.display_item_client_.Trace(visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer_for(owner: &mut LayoutBoxModelObject) -> Box<PaintLayer> {
        Box::new(PaintLayer {
            display_item_client_: DisplayItemClient::default(),
            layout_object_: Member::from_ptr(owner),
            is_self_painting_layer_: true,
            has_visible_content_: false,
            needs_descendant_dependent_flags_update_: true,
            needs_visual_overflow_recalc_: true,
            has_visible_self_painting_descendant_: false,
            has_non_isolated_descendant_with_blend_mode_: false,
            has_fixed_position_descendant_: false,
            has_non_contained_absolute_position_descendant_: false,
            has_stacked_descendant_in_current_stacking_context_: false,
            has_descendant_with_transform_anim_: false,
            has_self_painting_layer_descendant_: false,
            has_backdrop_filter_descendant_: false,
            has_3d_transformed_descendant_: false,
            static_position_: LogicalStaticPosition::default(),
            invisible_for_position_visibility_: 0,
            descendant_needs_check_position_visibility_: false,
            is_destroyed_: false,
        })
    }

    #[test]
    fn static_position_is_owned_by_the_paint_layer() {
        use layoutng_geometry::geometry::logical_offset::LogicalOffset;
        use layoutng_geometry::geometry::static_position::{
            BlockEdge, InlineEdge, LogicalAlignmentDirection,
        };

        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = Box::new(LayoutBoxModelObject::new(std::ptr::null_mut()));
        let mut layer = layer_for(&mut owner);
        let position = LogicalStaticPosition::new(
            LogicalOffset::new(
                foundation::LayoutUnit::from_signed(17),
                foundation::LayoutUnit::from_signed(29),
            ),
            InlineEdge::kInlineEnd,
            BlockEdge::kBlockCenter,
            LogicalAlignmentDirection::kInline,
        );
        layer.SetStaticPositionFromNG(&position);
        let retained = layer.GetStaticPosition();
        assert_eq!(retained.offset, position.offset);
        assert_eq!(retained.inline_edge, position.inline_edge);
        assert_eq!(retained.block_edge, position.block_edge);
        assert_eq!(retained.align_self_direction, position.align_self_direction);
        layer.Destroy();
        owner.SetDestroyedForTesting();
    }

    #[test]
    fn subsequence_caching_policy_follows_atomic_stacking() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut block = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        block.SetStyleInternal(unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        });
        let box_model = (&mut *block as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let mut layer = layer_for(unsafe { &mut *box_model });
        let initial_style = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        block.SetStyleInternal(initial_style);
        assert!(!layer.SupportsSubsequenceCaching());

        let mut absolute_builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial_style);
        absolute_builder.SetPosition(EPosition::kAbsolute);
        block.SetStyleInternal(unsafe { &*absolute_builder.TakeStyle() });
        assert!(layer.SupportsSubsequenceCaching());

        layer.Destroy();
        block.SetDestroyedForTesting();
    }

    #[test]
    fn backdrop_filter_reference_box_uses_the_source_box_border_box() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut block = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let box_model = (&mut *block as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let mut layer = layer_for(unsafe { &mut *box_model });

        assert_eq!(
            layer.BackdropFilterReferenceBox(),
            gfx::RectF::from(block.PhysicalBorderBoxRect())
        );
        assert_eq!(
            layer.ComputeFilterViewport(),
            Some(gfx::SizeF::from(block.StitchedSize()))
        );
        assert_eq!(layer.LocalBoundingBox(), block.VisualOverflowRect());

        layer.Destroy();
        block.SetDestroyedForTesting();
    }

    #[test]
    fn containing_layer_is_derived_from_the_source_layout_parent() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = Box::new(LayoutBoxModelObject::new(std::ptr::null_mut()));
        let mut child = Box::new(LayoutBoxModelObject::new(std::ptr::null_mut()));
        child.SetParent(&mut *parent as *mut LayoutBoxModelObject as *mut LayoutObject);

        let mut parent_layer = layer_for(&mut parent);
        let mut child_layer = layer_for(&mut child);
        parent.SetHasLayer(true);
        child.SetHasLayer(true);
        parent
            .GetMutableForPainting()
            .FirstFragment()
            .SetLayer(&mut *parent_layer);
        child
            .GetMutableForPainting()
            .FirstFragment()
            .SetLayer(&mut *child_layer);

        let expected = &mut *parent_layer as *mut PaintLayer;
        let child_object = &mut *child as *mut LayoutBoxModelObject as *mut LayoutObject;
        assert_eq!(child_layer.Parent(), expected);
        assert_eq!(child_layer.Root(), expected);
        assert_eq!(parent_layer.Root(), expected);
        assert_eq!(child_layer.ContainingLayer(), expected);
        assert!(std::ptr::eq(child.PaintingLayerDefault(), &*child_layer,));
        child_layer.is_self_painting_layer_ = false;
        assert!(std::ptr::eq(child.PaintingLayerDefault(), expected));
        assert!(std::ptr::eq(
            child_layer.EnclosingSelfPaintingLayer(),
            expected,
        ));

        child
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        parent
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        child_layer.ScrollContainerStatusChanged();
        assert!(child.NeedsPaintPropertyUpdate());
        assert!(parent.NeedsPaintPropertyUpdate());

        child_layer.needs_descendant_dependent_flags_update_ = false;
        parent_layer.needs_descendant_dependent_flags_update_ = false;
        child
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        parent
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        let visible_style = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        let mut hidden_style_builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(visible_style);
        hidden_style_builder.SetVisibility(EVisibility::kHidden);
        let hidden_style = unsafe { &*hidden_style_builder.TakeStyle() };
        let mut style_change_context = StyleChangeContext::default();
        unsafe { &mut *child_object }.StyleWillChange(
            layoutng_style::style::style_difference::StyleDifference::default(),
            visible_style,
            hidden_style,
            &mut style_change_context,
        );
        assert!(child.NeedsPaintPropertyUpdate());
        assert!(parent.NeedsPaintPropertyUpdate());
        assert!(child_layer.needs_descendant_dependent_flags_update_);
        assert!(parent_layer.needs_descendant_dependent_flags_update_);

        child_layer.needs_descendant_dependent_flags_update_ = false;
        parent_layer.needs_descendant_dependent_flags_update_ = false;
        child_layer.needs_visual_overflow_recalc_ = false;
        child
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        parent
            .GetMutableForPainting()
            .ClearNeedsPaintPropertyUpdateForTesting();
        unsafe { &mut *child_object }.MarkSelfPaintingLayerForVisualOverflowRecalc();
        assert!(!child_layer.NeedsVisualOverflowRecalc());
        assert!(!child_layer.needs_descendant_dependent_flags_update_);
        assert!(!parent_layer.needs_descendant_dependent_flags_update_);

        child_layer.is_self_painting_layer_ = true;
        unsafe { &mut *child_object }.MarkSelfPaintingLayerForVisualOverflowRecalc();
        assert!(child_layer.NeedsVisualOverflowRecalc());
        assert!(child_layer.needs_descendant_dependent_flags_update_);
        assert!(parent_layer.needs_descendant_dependent_flags_update_);

        child_layer.Destroy();
        parent_layer.Destroy();
        child.SetDestroyedForTesting();
        parent.SetDestroyedForTesting();
    }

    #[test]
    fn descendant_dependent_flags_follow_the_source_layer_tree() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let mut child = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let parent_object = (&mut *parent as *mut LayoutBlock).cast::<LayoutObject>();
        let child_object = (&mut *child as *mut LayoutBlock).cast::<LayoutObject>();
        unsafe { &mut *parent.VirtualChildren() }
            .AppendInitialChildNode(parent_object, child_object);

        let parent_box_model = (&mut *parent as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let child_box_model = (&mut *child as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let mut parent_layer = layer_for(unsafe { &mut *parent_box_model });
        let mut child_layer = layer_for(unsafe { &mut *child_box_model });
        unsafe { &mut *parent_box_model }.SetHasLayer(true);
        unsafe { &mut *child_box_model }.SetHasLayer(true);
        unsafe { &mut *parent_box_model }
            .GetMutableForPainting()
            .FirstFragment()
            .SetLayer(&mut *parent_layer);
        unsafe { &mut *child_box_model }
            .GetMutableForPainting()
            .FirstFragment()
            .SetLayer(&mut *child_layer);

        let visible_style = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        let mut hidden_style_builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(visible_style);
        hidden_style_builder.SetVisibility(EVisibility::kHidden);
        let hidden_style = unsafe { &*hidden_style_builder.TakeStyle() };
        parent.SetStyleInternal(hidden_style);
        child.SetStyleInternal(visible_style);

        assert!(std::ptr::eq(parent_layer.FirstChild(), &*child_layer));
        assert!(std::ptr::eq(parent_layer.LastChild(), &*child_layer));
        assert!(child_layer.PreviousSibling().is_null());
        assert!(child_layer.NextSibling().is_null());
        parent_layer.UpdateDescendantDependentFlags();
        assert!(!parent_layer.needs_descendant_dependent_flags_update_);
        assert!(!child_layer.needs_descendant_dependent_flags_update_);
        assert!(!parent_layer.HasVisibleContent());
        assert!(child_layer.HasVisibleContent());
        assert!(parent_layer.HasVisibleSelfPaintingDescendant());
        assert!(parent_layer.HasSelfPaintingLayerDescendant());
        assert!(parent_layer.IsZOrderListVisible());

        child_layer.UpdateSelfPaintingLayer();
        assert!(!child_layer.IsSelfPaintingLayer());
        assert!(parent_layer.needs_descendant_dependent_flags_update_);
        parent_layer.UpdateDescendantDependentFlags();
        assert!(parent_layer.HasVisibleContent());
        assert!(!parent_layer.HasVisibleSelfPaintingDescendant());
        assert!(!parent_layer.HasSelfPaintingLayerDescendant());

        child_layer.Destroy();
        parent_layer.Destroy();
        child.SetDestroyedForTesting();
        parent.SetDestroyedForTesting();
    }

    #[test]
    fn three_d_descendant_status_crosses_non_stacking_layer_subtrees() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let mut bridge = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let mut transformed = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let parent_object = (&mut *parent as *mut LayoutBlock).cast::<LayoutObject>();
        let bridge_object = (&mut *bridge as *mut LayoutBlock).cast::<LayoutObject>();
        let transformed_object = (&mut *transformed as *mut LayoutBlock).cast::<LayoutObject>();
        unsafe { &mut *parent.VirtualChildren() }
            .AppendInitialChildNode(parent_object, bridge_object);
        unsafe { &mut *bridge.VirtualChildren() }
            .AppendInitialChildNode(bridge_object, transformed_object);

        let parent_box_model = (&mut *parent as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let bridge_box_model = (&mut *bridge as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let transformed_box_model =
            (&mut *transformed as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let mut parent_layer = layer_for(unsafe { &mut *parent_box_model });
        let mut bridge_layer = layer_for(unsafe { &mut *bridge_box_model });
        let mut transformed_layer = layer_for(unsafe { &mut *transformed_box_model });
        for (box_model, layer) in [
            (parent_box_model, &mut *parent_layer as *mut PaintLayer),
            (bridge_box_model, &mut *bridge_layer as *mut PaintLayer),
            (
                transformed_box_model,
                &mut *transformed_layer as *mut PaintLayer,
            ),
        ] {
            unsafe { &mut *box_model }.SetHasLayer(true);
            unsafe { &mut *box_model }
                .GetMutableForPainting()
                .FirstFragment()
                .SetLayer(layer);
        }

        let initial_style = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        parent.SetStyleInternal(initial_style);
        bridge.SetStyleInternal(initial_style);
        let mut transformed_style_builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial_style);
        transformed_style_builder.SetPosition(EPosition::kAbsolute);
        let transformed_style = unsafe { &*transformed_style_builder.TakeStyle() };
        transformed.SetStyleInternal(transformed_style);
        let mut transform = foundation::gfx::Transform::default();
        transform.ApplyPerspectiveDepth(100.0);
        transformed.transform_ = Some(Box::new(transform));

        assert!(!bridge.IsStackingContext());
        assert!(transformed.IsStacked());
        assert!(transformed_layer.Has3DTransform());
        parent_layer.UpdateDescendantDependentFlags();
        assert!(parent_layer.Has3DTransformedDescendant());
        assert!(bridge_layer.Has3DTransformedDescendant());
        assert!(!transformed_layer.Has3DTransformedDescendant());

        transformed_layer.Destroy();
        bridge_layer.Destroy();
        parent_layer.Destroy();
        transformed.SetDestroyedForTesting();
        bridge.SetDestroyedForTesting();
        parent.SetDestroyedForTesting();
    }

    #[test]
    fn descendant_bounds_cross_non_self_painting_layer_bridges() {
        use layoutng_geometry::geometry::overflow_clip_axes::kOverflowClipBothAxis;

        let _heap = foundation::LayoutHeapScope::new();
        let mut parent = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let mut bridge = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let mut child = Box::new(LayoutBlock::new(std::ptr::null_mut()));
        let parent_object = (&mut *parent as *mut LayoutBlock).cast::<LayoutObject>();
        let bridge_object = (&mut *bridge as *mut LayoutBlock).cast::<LayoutObject>();
        let child_object = (&mut *child as *mut LayoutBlock).cast::<LayoutObject>();
        unsafe { &mut *parent.VirtualChildren() }
            .AppendInitialChildNode(parent_object, bridge_object);
        unsafe { &mut *bridge.VirtualChildren() }
            .AppendInitialChildNode(bridge_object, child_object);

        let initial_style = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        parent.SetStyleInternal(initial_style);
        bridge.SetStyleInternal(initial_style);
        child.SetStyleInternal(initial_style);
        bridge.SetLocation(PhysicalOffset::new(
            foundation::LayoutUnit::from_signed(20),
            foundation::LayoutUnit::from_signed(30),
        ));
        child.SetLocation(PhysicalOffset::new(
            foundation::LayoutUnit::from_signed(5),
            foundation::LayoutUnit::from_signed(7),
        ));

        let parent_box_model = (&mut *parent as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let bridge_box_model = (&mut *bridge as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let child_box_model = (&mut *child as *mut LayoutBlock).cast::<LayoutBoxModelObject>();
        let mut parent_layer = layer_for(unsafe { &mut *parent_box_model });
        let mut bridge_layer = layer_for(unsafe { &mut *bridge_box_model });
        let mut child_layer = layer_for(unsafe { &mut *child_box_model });
        bridge_layer.is_self_painting_layer_ = false;
        for (box_model, layer) in [
            (parent_box_model, &mut *parent_layer as *mut PaintLayer),
            (bridge_box_model, &mut *bridge_layer as *mut PaintLayer),
            (child_box_model, &mut *child_layer as *mut PaintLayer),
        ] {
            unsafe { &mut *box_model }.SetHasLayer(true);
            unsafe { &mut *box_model }
                .GetMutableForPainting()
                .FirstFragment()
                .SetLayer(layer);
        }
        parent_layer.UpdateDescendantDependentFlags();
        parent.SetVisualOverflow(
            &PhysicalRect::new(
                PhysicalOffset::default(),
                foundation::PhysicalSize::new(
                    foundation::LayoutUnit::from_signed(10),
                    foundation::LayoutUnit::from_signed(10),
                ),
            ),
            &PhysicalRect::default(),
        );
        child.SetVisualOverflow(
            &PhysicalRect::new(
                PhysicalOffset::default(),
                foundation::PhysicalSize::new(
                    foundation::LayoutUnit::from_signed(4),
                    foundation::LayoutUnit::from_signed(6),
                ),
            ),
            &PhysicalRect::default(),
        );

        assert_eq!(
            parent_layer.LocalBoundingBoxIncludingSelfPaintingDescendants(),
            PhysicalRect::new(
                PhysicalOffset::default(),
                foundation::PhysicalSize::new(
                    foundation::LayoutUnit::from_signed(29),
                    foundation::LayoutUnit::from_signed(43),
                ),
            )
        );

        parent.SetOverflowClipAxes(kOverflowClipBothAxis);
        assert!(parent_layer.KnownToClipSubtreeToPaddingBox());
        assert_eq!(
            parent_layer.LocalBoundingBoxIncludingSelfPaintingDescendants(),
            parent_layer.LocalBoundingBox()
        );

        child_layer.Destroy();
        bridge_layer.Destroy();
        parent_layer.Destroy();
        child.SetDestroyedForTesting();
        bridge.SetDestroyedForTesting();
        parent.SetDestroyedForTesting();
    }
}
