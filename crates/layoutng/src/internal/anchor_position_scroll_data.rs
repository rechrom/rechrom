#![allow(non_snake_case)]

use foundation::{
    gfx, CompositorElementId, CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace,
    GCedHeapVector, HeapHashSet, LayoutUnit, Member, PhysicalOffset, RuntimeEnabledFeatures,
    Visitor, WeakMember,
};
use layoutng_style::style::style_position_anchor::Type as StylePositionAnchorType;

use super::anchor_scroll_services::{
    AnchorViewportUsesTransformOverscroll, ReadAnchorScrollContainerState,
};
use super::css::out_of_flow_data::RememberedScrollOffsetType;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_node_metadata::{Element, Node};
use super::layout_object::LayoutObject;
use super::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
use super::node_rare_data_field::NodeRareDataField;
use super::non_overflowing_scroll_range::NonOverflowingScrollRange;

// cpp: layoutng/internal/anchor_position_scroll_data.cc:46-61
fn PositionAnchorObject(box_: &LayoutBox) -> *const LayoutObject {
    let default_anchor_data = box_.StyleRef().GetDefaultAnchorData();
    match default_anchor_data.GetType() {
        StylePositionAnchorType::kNone => std::ptr::null(),
        StylePositionAnchorType::kAuto => box_.AcceptableImplicitAnchor(),
        StylePositionAnchorType::kName => box_.FindTargetAnchor(default_anchor_data.GetName()),
        StylePositionAnchorType::kNormal => panic!("unreachable position-anchor type"),
    }
}

// cpp: layoutng/internal/anchor_position_scroll_data.cc:63-70
fn GetNonOverflowingScrollRanges(
    layout_object: *const LayoutObject,
) -> *const GCedHeapVector<NonOverflowingScrollRange> {
    if layout_object.is_null() || !unsafe { &*layout_object }.IsOutOfFlowPositioned() {
        return std::ptr::null();
    }
    assert!(unsafe { &*layout_object }.IsBox());
    unsafe { &*layout_object.cast::<LayoutBox>() }.NonOverflowingScrollRanges()
}

// cpp: layoutng/internal/anchor_position_scroll_data.cc:72-82
fn CheckHasDefaultAnchorReferences(layout_object: *const LayoutObject) -> (bool, bool) {
    if layout_object.is_null() || !unsafe { &*layout_object }.IsOutOfFlowPositioned() {
        return (false, false);
    }
    assert!(unsafe { &*layout_object }.IsBox());
    let box_ = unsafe { &*layout_object.cast::<LayoutBox>() };
    (
        box_.NeedsAnchorPositionScrollAdjustmentInX(),
        box_.NeedsAnchorPositionScrollAdjustmentInY(),
    )
}

// cpp: layoutng/internal/anchor_position_scroll_data.cc:84-91
fn ContainerIgnoreLayoutViewForFixedPos(object: &LayoutObject) -> *const LayoutBoxModelObject {
    let container = object.Container();
    if container.is_null() || (object.IsFixedPositioned() && unsafe { &*container }.IsLayoutView())
    {
        return std::ptr::null();
    }
    assert!(unsafe { &*container }.IsBoxModelObject());
    container.cast::<LayoutBoxModelObject>()
}

// cpp: layoutng/internal/anchor_position_scroll_data.h:119-178
pub struct AdjustmentData {
    pub anchor_element: Member<Element>,
    pub adjustment_container_ids: Vec<CompositorElementId>,
    pub accumulated_adjustment: PhysicalOffset,
    pub accumulated_range_adjustment_offset: PhysicalOffset,
    pub accumulated_adjustment_scroll_origin: gfx::Vector2d,
    pub anchored_element_container_scroll_offset: PhysicalOffset,
    pub containers_include_viewport: bool,
    pub needs_scroll_adjustment_in_x: bool,
    pub needs_scroll_adjustment_in_y: bool,
    pub has_chained_anchor: bool,
}

impl Default for AdjustmentData {
    fn default() -> Self {
        Self {
            anchor_element: Member::default(),
            adjustment_container_ids: Vec::new(),
            accumulated_adjustment: PhysicalOffset::default(),
            accumulated_range_adjustment_offset: PhysicalOffset::default(),
            accumulated_adjustment_scroll_origin: gfx::Vector2d::default(),
            anchored_element_container_scroll_offset: PhysicalOffset::default(),
            containers_include_viewport: false,
            needs_scroll_adjustment_in_x: false,
            needs_scroll_adjustment_in_y: false,
            has_chained_anchor: false,
        }
    }
}

impl AdjustmentData {
    // cpp: layoutng/internal/anchor_position_scroll_data.h:164-164
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.anchor_element);
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:166-177
    pub fn TotalOffset(&self) -> PhysicalOffset {
        if self.containers_include_viewport {
            self.accumulated_adjustment.clone()
                + self.anchored_element_container_scroll_offset.clone()
        } else {
            self.accumulated_adjustment.clone()
        }
    }

    pub fn TotalOffsetIncludingChained(&self) -> PhysicalOffset {
        if self.containers_include_viewport {
            self.accumulated_range_adjustment_offset.clone()
                + self.anchored_element_container_scroll_offset.clone()
        } else {
            self.accumulated_range_adjustment_offset.clone()
        }
    }
}

// cpp: layoutng/internal/anchor_position_scroll_data.h:186-187
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotDiff {
    kNone,
    kScrollersOrFallbackPosition,
    kOffsetOnly,
}

// cpp: layoutng/internal/anchor_position_scroll_data.h:50-205
pub struct AnchorPositionScrollData {
    rare_data_: NodeRareDataField,
    anchored_element_: Member<Element>,
    default_anchor_adjustment_data_: AdjustmentData,
    dependent_anchors_: HeapHashSet<WeakMember<Node>>,
}

impl AnchorPositionScrollData {
    // cpp: layoutng/internal/anchor_position_scroll_data.cc:95-100
    pub fn new(anchored_element: *mut Element) -> Self {
        assert!(!anchored_element.is_null());
        Self {
            rare_data_: NodeRareDataField,
            anchored_element_: Member::from_ptr(anchored_element),
            default_anchor_adjustment_data_: AdjustmentData::default(),
            dependent_anchors_: HeapHashSet::default(),
        }
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:57-83
    pub fn AnchoredElement(&self) -> *mut Element {
        self.anchored_element_.Get()
    }
    pub fn NeedsScrollAdjustment(&self) -> bool {
        !self
            .default_anchor_adjustment_data_
            .adjustment_container_ids
            .is_empty()
    }
    pub fn NeedsScrollAdjustmentInX(&self) -> bool {
        self.default_anchor_adjustment_data_
            .needs_scroll_adjustment_in_x
    }
    pub fn NeedsScrollAdjustmentInY(&self) -> bool {
        self.default_anchor_adjustment_data_
            .needs_scroll_adjustment_in_y
    }
    pub fn AccumulatedAdjustment(&self) -> PhysicalOffset {
        self.default_anchor_adjustment_data_
            .accumulated_adjustment
            .clone()
    }
    pub fn AccumulatedAdjustmentIncludingChained(&self) -> PhysicalOffset {
        self.default_anchor_adjustment_data_
            .accumulated_range_adjustment_offset
            .clone()
    }
    pub fn AccumulatedAdjustmentScrollOrigin(&self) -> gfx::Vector2d {
        self.default_anchor_adjustment_data_
            .accumulated_adjustment_scroll_origin
            .clone()
    }
    pub fn AdjustmentContainerIds(&self) -> &[CompositorElementId] {
        &self
            .default_anchor_adjustment_data_
            .adjustment_container_ids
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:87-89
    pub fn IsAffectedByViewportScrolling(&self) -> bool {
        self.default_anchor_adjustment_data_
            .containers_include_viewport
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:97-100
    pub fn TranslationAsPhysicalOffset(&self) -> PhysicalOffset {
        -self.AccumulatedAdjustmentIncludingChained()
            + self.SpeculativeDefaultAnchorRememberedOffsetIncludingChained()
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:113-115
    pub fn DefaultAnchorHasChainedAnchor(&self) -> bool {
        self.default_anchor_adjustment_data_.has_chained_anchor
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.h:184-184
    pub fn AddDependentAnchor(&mut self, node: *const Node) {
        self.dependent_anchors_
            .insert(WeakMember::from_ptr(node as *mut Node));
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:102-104
    pub fn IsActive(&self) -> bool {
        let anchored_element = self.anchored_element_.Get();
        unsafe { &*anchored_element }.GetAnchorPositionScrollData()
            == self as *const Self as *mut Self
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:106-123
    pub fn GetFilteredRememberedOffset(&self, kind: RememberedScrollOffsetType) -> PhysicalOffset {
        let out_of_flow_data = unsafe { &*self.anchored_element_.Get() }.GetOutOfFlowData();
        let offsets = if out_of_flow_data.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*out_of_flow_data }.GetSpeculativeRememberedScrollOffsets()
        };
        if offsets.is_null() {
            return PhysicalOffset::default();
        }
        unsafe { &*offsets }
            .GetOffset(
                self.default_anchor_adjustment_data_.anchor_element.Get(),
                kind,
                self.NeedsScrollAdjustmentInX(),
                self.NeedsScrollAdjustmentInY(),
            )
            .unwrap_or_default()
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:125-134
    pub fn SpeculativeDefaultAnchorRememberedOffset(&self) -> PhysicalOffset {
        self.GetFilteredRememberedOffset(RememberedScrollOffsetType::kLayout)
    }
    pub fn SpeculativeDefaultAnchorRememberedOffsetIncludingChained(&self) -> PhysicalOffset {
        self.GetFilteredRememberedOffset(RememberedScrollOffsetType::kRangeAdjustment)
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:136-251
    pub fn ComputeAdjustmentContainersData(
        anchored_element: *const Element,
        anchor: &LayoutObject,
    ) -> AdjustmentData {
        let mut result = AdjustmentData::default();
        let may_need_scroll_adjustment = |box_: &LayoutBox| -> bool {
            if RuntimeEnabledFeatures::AnchorPositionAdjustmentWithoutOverflowEnabled()
                && box_.IsLayoutView()
                && AnchorViewportUsesTransformOverscroll(box_)
            {
                return true;
            }
            box_.HasScrollableOverflow()
        };

        let anchored_layout_object = unsafe { &*anchored_element }
            .container
            .node
            .GetLayoutObject();
        assert!(!anchored_layout_object.is_null());
        let anchor_node = anchor.GetNode();
        assert!(!anchor_node.is_null() && unsafe { &*anchor_node }.IsElementNode());
        let anchor_element = anchor_node.cast::<Element>();
        result.anchor_element = Member::from_ptr(anchor_element);
        let bounding_container =
            ContainerIgnoreLayoutViewForFixedPos(unsafe { &*anchored_layout_object });

        if !bounding_container.is_null() && unsafe { &*bounding_container }.IsScrollContainer() {
            assert!(unsafe { &*bounding_container }.IsBox());
            let scroll_state =
                ReadAnchorScrollContainerState(unsafe { &*bounding_container.cast::<LayoutBox>() });
            result.anchored_element_container_scroll_offset =
                PhysicalOffset::FromVector2dFFloor(&scroll_state.offset);
        }

        let get_transformed_offset =
            |offset: PhysicalOffset, container: *const LayoutObject| -> PhysicalOffset {
                if offset.IsZero() {
                    return PhysicalOffset::default();
                }
                let flags = MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreScrollOffset);
                let transformed_origin = unsafe { &*container }.LocalToAncestorPointPhysical(
                    &PhysicalOffset::default(),
                    bounding_container.cast::<LayoutBoxModelObject>(),
                    flags,
                );
                let transformed_offset = unsafe { &*container }.LocalToAncestorPointPhysical(
                    &offset,
                    bounding_container.cast::<LayoutBoxModelObject>(),
                    flags,
                );
                transformed_offset - transformed_origin
            };

        let anchor_ptr = anchor as *const LayoutObject;
        let mut container = anchor_ptr;
        while !container.is_null() && container != bounding_container.cast::<LayoutObject>() {
            let object = unsafe { &*container };
            if object.IsScrollContainer() {
                assert!(object.IsBox());
                let box_ = unsafe { &*container.cast::<LayoutBox>() };
                let scroll_state = ReadAnchorScrollContainerState(box_);
                if container != anchor_ptr
                    && container != bounding_container.cast::<LayoutObject>()
                    && may_need_scroll_adjustment(box_)
                {
                    result.adjustment_container_ids.push(scroll_state.id);
                    let scroll_offset = PhysicalOffset::FromVector2dFFloor(&scroll_state.offset);
                    result.accumulated_adjustment +=
                        get_transformed_offset(scroll_offset, container);
                    result.accumulated_adjustment_scroll_origin += scroll_state.origin;
                    if scroll_state.is_layout_view {
                        result.containers_include_viewport = true;
                    }
                }
            }
            if object.IsBoxModelObject() {
                let box_model = unsafe { &*container.cast::<LayoutBoxModelObject>() };
                if box_model.StickyConstraints().HasAnyConstraint() {
                    result
                        .adjustment_container_ids
                        .push(CompositorElementIdFromUniqueObjectId(
                            box_model.UniqueId(),
                            CompositorElementIdNamespace::kStickyTranslation,
                        ));
                    result.accumulated_adjustment -=
                        get_transformed_offset(box_model.StickyPositionOffset(), container);
                }
            }
            if object.IsBox() {
                let box_ = unsafe { &*container.cast::<LayoutBox>() };
                let data = box_.GetAnchorPositionScrollData();
                if !data.is_null() {
                    let data = unsafe { &mut *data };
                    result.has_chained_anchor = true;
                    data.AddDependentAnchor(anchored_element.cast::<Node>());
                    if data.NeedsScrollAdjustment() {
                        result.adjustment_container_ids.push(
                            CompositorElementIdFromUniqueObjectId(
                                box_.UniqueId(),
                                CompositorElementIdNamespace::kAnchorPositionScrollTranslation,
                            ),
                        );
                        let adjustment_offset = data
                            .ComputeDefaultAnchorAdjustmentData()
                            .accumulated_range_adjustment_offset;
                        result.accumulated_range_adjustment_offset +=
                            get_transformed_offset(adjustment_offset, container);
                    }
                }
            }
            container = ContainerIgnoreLayoutViewForFixedPos(object).cast::<LayoutObject>();
        }
        result.accumulated_range_adjustment_offset += result.accumulated_adjustment.clone();
        result
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:253-288
    pub fn ComputeDefaultAnchorAdjustmentData(&self) -> AdjustmentData {
        let anchored_element = self.anchored_element_.Get();
        let layout_box = unsafe { &*anchored_element }.container.node.GetLayoutBox();
        let (mut needs_x, mut needs_y) =
            CheckHasDefaultAnchorReferences(layout_box.cast::<LayoutObject>());
        if !needs_x && !needs_y {
            return AdjustmentData::default();
        }
        let anchor_default_object = PositionAnchorObject(unsafe { &*layout_box });
        if anchor_default_object.is_null() {
            return AdjustmentData::default();
        }
        let mut result = Self::ComputeAdjustmentContainersData(anchored_element, unsafe {
            &*anchor_default_object
        });
        if result.adjustment_container_ids.is_empty() {
            needs_x = false;
            needs_y = false;
        }
        if !needs_x {
            result.accumulated_adjustment.left = LayoutUnit::default();
            result.accumulated_range_adjustment_offset.left = LayoutUnit::default();
            result.accumulated_adjustment_scroll_origin.set_x(0);
        }
        if !needs_y {
            result.accumulated_adjustment.top = LayoutUnit::default();
            result.accumulated_range_adjustment_offset.top = LayoutUnit::default();
            result.accumulated_adjustment_scroll_origin.set_y(0);
        }
        result.needs_scroll_adjustment_in_x = needs_x;
        result.needs_scroll_adjustment_in_y = needs_y;
        result
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:290-322
    pub fn TakeAndCompareSnapshot(&mut self, update: bool) -> SnapshotDiff {
        debug_assert!(self.IsActive());
        let new_adjustment_data = self.ComputeDefaultAnchorAdjustmentData();
        let mut diff = SnapshotDiff::kNone;
        if self.default_anchor_adjustment_data_.anchor_element.Get()
            != new_adjustment_data.anchor_element.Get()
            || self.AdjustmentContainerIds() != new_adjustment_data.adjustment_container_ids
            || !self.IsFallbackPositionValid(&new_adjustment_data)
        {
            diff = SnapshotDiff::kScrollersOrFallbackPosition;
        } else if self.NeedsScrollAdjustmentInX()
            != new_adjustment_data.needs_scroll_adjustment_in_x
            || self.NeedsScrollAdjustmentInY() != new_adjustment_data.needs_scroll_adjustment_in_y
            || self
                .default_anchor_adjustment_data_
                .TotalOffsetIncludingChained()
                != new_adjustment_data.TotalOffsetIncludingChained()
            || self.AccumulatedAdjustmentScrollOrigin()
                != new_adjustment_data.accumulated_adjustment_scroll_origin
        {
            diff = SnapshotDiff::kOffsetOnly;
        }
        if update && diff != SnapshotDiff::kNone {
            self.default_anchor_adjustment_data_ = new_adjustment_data;
        }
        diff
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:324-358
    pub fn IsFallbackPositionValid(&self, new_adjustment_data: &AdjustmentData) -> bool {
        let anchored_object = unsafe { &*self.anchored_element_.Get() }
            .container
            .node
            .GetLayoutObject();
        let ranges = GetNonOverflowingScrollRanges(anchored_object);
        if ranges.is_null() || unsafe { &*ranges }.is_empty() {
            return true;
        }
        for range in unsafe { &*ranges } {
            let range_element = new_adjustment_data.anchor_element.Get();
            let new_element = range.anchor_element.Get();
            let range_object = if range_element.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { &*range_element }.container.node.GetLayoutObject()
            };
            let new_object = if new_element.is_null() {
                std::ptr::null_mut()
            } else {
                unsafe { &*new_element }.container.node.GetLayoutObject()
            };
            if new_object != range_object {
                return false;
            } else if range.Contains(
                &self
                    .default_anchor_adjustment_data_
                    .TotalOffsetIncludingChained(),
            ) != range.Contains(&new_adjustment_data.TotalOffsetIncludingChained())
            {
                return false;
            }
        }
        true
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:360-377
    pub fn UpdateSnapshot(&mut self) -> bool {
        if !self.IsActive() {
            return false;
        }
        let diff = self.TakeAndCompareSnapshot(true);
        match diff {
            SnapshotDiff::kNone => false,
            SnapshotDiff::kOffsetOnly => false,
            SnapshotDiff::kScrollersOrFallbackPosition => {
                self.InvalidateLayoutDependentAndAncestors();
                true
            }
        }
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:379-382
    pub fn ShouldScheduleNextService(&mut self) -> bool {
        self.IsActive() && self.TakeAndCompareSnapshot(false) != SnapshotDiff::kNone
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:384-410
    pub fn InvalidateLayoutDependentAndAncestors(&mut self) {
        if !self.IsActive() {
            return;
        }
        self.InvalidateLayoutDependents();
        let positioned_box = unsafe { &*self.anchored_element_.Get() }
            .container
            .node
            .GetLayoutBox();
        assert!(!positioned_box.is_null());
        let anchor_default_object = PositionAnchorObject(unsafe { &*positioned_box });
        let positioned_container = ContainerIgnoreLayoutViewForFixedPos(unsafe {
            &*positioned_box.cast::<LayoutObject>()
        });
        let mut container = anchor_default_object;
        while !container.is_null() && container != positioned_container.cast::<LayoutObject>() {
            let object = unsafe { &*container };
            if object.IsBox() {
                let box_ = unsafe { &*container.cast::<LayoutBox>() };
                let data = box_.GetAnchorPositionScrollData();
                if !data.is_null() {
                    unsafe { &mut *data }.InvalidateLayoutDependentAndAncestors();
                }
            }
            container = ContainerIgnoreLayoutViewForFixedPos(object).cast::<LayoutObject>();
        }
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:412-432
    pub fn InvalidateLayoutDependents(&mut self) {
        if !self.IsActive() {
            return;
        }
        let anchored_element = unsafe { &*self.anchored_element_.Get() };
        let layout_object = anchored_element.container.node.GetLayoutObject();
        assert!(!layout_object.is_null());
        let out_of_flow_data = anchored_element.GetOutOfFlowData();
        if !out_of_flow_data.is_null() {
            unsafe { &mut *out_of_flow_data }.ClearRememberedScrollOffsets();
        }
        let reason = unsafe {
            &super::layout_invalidation_reason::kAnchorPositioning as *const std::ffi::c_char
        };
        unsafe { &mut *layout_object }.SetNeedsLayoutAndFullPaintInvalidation(reason);
        for dependent in self.dependent_anchors_.iter() {
            let node = dependent.Get();
            assert!(!node.is_null());
            let box_ = unsafe { &*node }.GetLayoutBox();
            if !box_.is_null() {
                let data = unsafe { &*box_ }.GetAnchorPositionScrollData();
                if !data.is_null() {
                    unsafe { &mut *data }.InvalidateLayoutDependents();
                }
            }
        }
        self.dependent_anchors_.clear();
    }

    // cpp: layoutng/internal/anchor_position_scroll_data.cc:434-439
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.anchored_element_);
        self.default_anchor_adjustment_data_.Trace(visitor);
        visitor.Trace(&self.dependent_anchors_);
        self.rare_data_.Trace(visitor);
    }
}
