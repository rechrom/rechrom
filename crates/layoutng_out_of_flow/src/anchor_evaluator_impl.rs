#![allow(non_snake_case)]

use std::cell::{Cell, RefCell};
use std::fmt::Debug;

use foundation::style_values::css::anchor_query::AnchorQuery;
use foundation::style_values::css::css_anchor_query_enums::{
    CSSAnchorQueryType, CSSAnchorSizeValue, CSSAnchorValue,
};
use foundation::style_values::style::anchor_specifier_value::AnchorSpecifierValue;
use foundation::{
    DynamicTo, GCedHeapHashSet, LayoutUnit, MakeGarbageCollected, Member, PhysicalOffset,
    PhysicalRect, To, WritingDirectionMode,
};
use layoutng_assembly::internal::anchor_map::{AnchorMap, PhysicalAnchorReference};
use layoutng_assembly::internal::anchor_scope::ToAnchorScopedName;
use layoutng_assembly::internal::css::out_of_flow_data::RememberedScrollOffsets;
use layoutng_assembly::internal::grid_layout_data::GridLayoutData;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::LayoutObject;
use layoutng_geometry::geometry::axis::PhysicalAxis;
use layoutng_geometry::geometry::box_sides::PhysicalBoxSides;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyleBuilder;
use layoutng_style::style::default_anchor_data::DefaultAnchorData;
use layoutng_style::style::position_area::{PositionAreaOffsets, PositionAreaRegion};
use layoutng_style::style::style_position_anchor::Type as StylePositionAnchorType;

use crate::anchor_evaluator::{AnchorEvaluator, AnchorEvaluatorMode, AnchorScope};
use crate::anchor_query_calculations::{ResolveAnchorSizeValue, ResolveAnchorValue};
use crate::anchor_scroll_services::ComputeAnchorScrollOffsets;

// base::AutoReset<bool> resets even if evaluation exits by unwinding. The
// pointer remains valid because each guard is scoped inside a &mut self call.
// cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:225-226
struct AutoResetBool {
    target_: *mut bool,
    original_: bool,
}

impl AutoResetBool {
    fn new(target: &mut bool, value: bool) -> Self {
        let original_ = std::mem::replace(target, value);
        Self {
            target_: target,
            original_,
        }
    }
}

impl Drop for AutoResetBool {
    fn drop(&mut self) {
        unsafe { *self.target_ = self.original_ };
    }
}

// The C++ single-value cache is mutable even through const evaluator methods.
// RefCell preserves that interface without changing its key or update order.
// cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:181-203
pub struct CachedValue<KeyType, ValueType> {
    entry_: RefCell<Option<(KeyType, ValueType)>>,
}

impl<KeyType, ValueType> Default for CachedValue<KeyType, ValueType> {
    fn default() -> Self {
        Self {
            entry_: RefCell::new(None),
        }
    }
}

impl<KeyType: Clone + PartialEq, ValueType: Copy + Debug + PartialEq>
    CachedValue<KeyType, ValueType>
{
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:189-198
    pub fn Get(&self, key: &KeyType, create: impl Fn() -> ValueType) -> ValueType {
        if let Some((cached_key, cached_value)) = self.entry_.borrow().as_ref() {
            if cached_key == key {
                debug_assert_eq!(*cached_value, create());
                return *cached_value;
            }
        }
        let value = create();
        *self.entry_.borrow_mut() = Some((key.clone(), value));
        value
    }
}

// The C++ class is stack-allocated and stores non-owning pointers to its
// query and containing boxes. Its GC-owned values retain their original owner.
// cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:37-230
pub struct AnchorEvaluatorImpl {
    mode_: Cell<AnchorEvaluatorMode>,
    query_box_: *const LayoutBox,
    anchor_map_: *const AnchorMap,
    implicit_anchor_: *const LayoutObject,
    containing_block_: *const LayoutObject,
    query_box_actual_containing_block_: *const LayoutObject,
    grid_layout_data_: *const GridLayoutData,
    container_writing_direction_: WritingDirectionMode,
    container_size_: LogicalSize,
    container_rect_: LogicalRect,
    scroll_rect_: Option<LogicalRect>,
    cached_default_anchor_: CachedValue<DefaultAnchorData, *const LayoutObject>,
    cached_default_anchor_scroll_container_x_: CachedValue<DefaultAnchorData, *const LayoutBox>,
    cached_default_anchor_scroll_container_y_: CachedValue<DefaultAnchorData, *const LayoutBox>,
    needs_scroll_adjustment_in_x_: bool,
    needs_scroll_adjustment_in_y_: bool,
    did_resolve_anchor_with_running_transform_animation_: bool,
    display_locks_affected_by_anchors_: *mut GCedHeapHashSet<Member<Element>>,
    remembered_scroll_offsets_: *const RememberedScrollOffsets,
    used_scroll_offsets_: *mut RememberedScrollOffsets,
}

impl AnchorEvaluatorImpl {
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:43-62
    pub fn new(
        query_box: &LayoutBox,
        anchor_map: *const AnchorMap,
        implicit_anchor: *const LayoutObject,
        containing_block: *const LayoutObject,
        actual_containing_block: *const LayoutObject,
        grid_layout_data: *const GridLayoutData,
        container_writing_direction: WritingDirectionMode,
        container_size: LogicalSize,
        container_rect: LogicalRect,
        scroll_rect: Option<LogicalRect>,
    ) -> Self {
        Self {
            mode_: Cell::new(AnchorEvaluatorMode::kNone),
            query_box_: query_box as *const _,
            anchor_map_: anchor_map,
            implicit_anchor_: implicit_anchor,
            containing_block_: containing_block,
            query_box_actual_containing_block_: actual_containing_block,
            grid_layout_data_: grid_layout_data,
            container_writing_direction_: container_writing_direction,
            container_size_: container_size,
            container_rect_: container_rect,
            scroll_rect_: scroll_rect,
            cached_default_anchor_: CachedValue::default(),
            cached_default_anchor_scroll_container_x_: CachedValue::default(),
            cached_default_anchor_scroll_container_y_: CachedValue::default(),
            needs_scroll_adjustment_in_x_: false,
            needs_scroll_adjustment_in_y_: false,
            did_resolve_anchor_with_running_transform_animation_: false,
            display_locks_affected_by_anchors_: std::ptr::null_mut(),
            remembered_scroll_offsets_: std::ptr::null(),
            used_scroll_offsets_: std::ptr::null_mut(),
        }
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:67-72
    pub fn NeedsScrollAdjustmentInX(&self) -> bool {
        self.needs_scroll_adjustment_in_x_
    }
    pub fn NeedsScrollAdjustmentInY(&self) -> bool {
        self.needs_scroll_adjustment_in_y_
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:91
    pub fn GetAnchorMap(&self) -> *const AnchorMap {
        self.anchor_map_
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:103-105
    pub fn GetDisplayLocksAffectedByAnchors(&self) -> *mut GCedHeapHashSet<Member<Element>> {
        self.display_locks_affected_by_anchors_
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:107-115
    pub fn LastUsedScrollOffsets(&self) -> *const RememberedScrollOffsets {
        self.used_scroll_offsets_
    }
    pub fn ClearLastUsedScrollOffsets(&mut self) {
        self.used_scroll_offsets_ = std::ptr::null_mut();
    }
    pub fn SetRememberedScrollOffsets(&mut self, offsets: *const RememberedScrollOffsets) {
        self.remembered_scroll_offsets_ = offsets;
    }
    pub fn ClearRememberedScrollOffsets(&mut self) {
        self.remembered_scroll_offsets_ = std::ptr::null();
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:117-119
    pub fn DidResolveAnchorWithRunningTransformAnimation(&self) -> bool {
        self.did_resolve_anchor_with_running_transform_animation_
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:58-70
    fn AllowAnchor(&self) -> bool {
        matches!(
            self.mode_.get(),
            AnchorEvaluatorMode::kLeft
                | AnchorEvaluatorMode::kRight
                | AnchorEvaluatorMode::kTop
                | AnchorEvaluatorMode::kBottom
        )
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:72-84
    fn AllowAnchorSize(&self) -> bool {
        self.mode_.get() != AnchorEvaluatorMode::kNone
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:86-89
    fn IsYAxis(&self) -> bool {
        matches!(
            self.mode_.get(),
            AnchorEvaluatorMode::kTop | AnchorEvaluatorMode::kBottom | AnchorEvaluatorMode::kHeight
        )
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:91-93
    fn IsRightOrBottom(&self) -> bool {
        matches!(
            self.mode_.get(),
            AnchorEvaluatorMode::kRight | AnchorEvaluatorMode::kBottom
        )
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:36-43
    pub fn DefaultAnchor(&self, default_anchor_data: &DefaultAnchorData) -> *const LayoutObject {
        self.cached_default_anchor_.Get(default_anchor_data, || {
            let reference = self.ResolveAnchorReference(
                unsafe { &*AnchorSpecifierValue::Default() },
                default_anchor_data,
            );
            if reference.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*reference }.GetLayoutObject()
            }
        })
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:45-56
    fn DefaultAnchorScrollContainer(
        &self,
        axis: PhysicalAxis,
        default_anchor_data: &DefaultAnchorData,
    ) -> *const LayoutBox {
        let cache = if axis == PhysicalAxis::kVertical {
            &self.cached_default_anchor_scroll_container_y_
        } else {
            &self.cached_default_anchor_scroll_container_x_
        };
        cache.Get(default_anchor_data, || {
            let default_anchor = self.DefaultAnchor(default_anchor_data);
            if default_anchor.is_null() {
                std::ptr::null()
            } else {
                unsafe { &*default_anchor }.ContainingScrollContainer(axis)
            }
        })
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:95-107
    fn ShouldUseScrollAdjustmentFor(
        &self,
        anchor: *const LayoutObject,
        axis: PhysicalAxis,
        default_anchor_data: &DefaultAnchorData,
    ) -> bool {
        if anchor.is_null() {
            return false;
        }
        if anchor == self.DefaultAnchor(default_anchor_data) {
            return true;
        }
        unsafe { &*anchor }.ContainingScrollContainer(axis)
            == self.DefaultAnchorScrollContainer(axis, default_anchor_data)
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:156-183
    fn EvaluateAnchorSize(
        &mut self,
        anchor_specifier: &AnchorSpecifierValue,
        mut anchor_size_value: CSSAnchorSizeValue,
        default_anchor_data: &DefaultAnchorData,
    ) -> Option<LayoutUnit> {
        if !self.AllowAnchorSize() {
            return None;
        }
        if anchor_size_value == CSSAnchorSizeValue::kImplicit {
            anchor_size_value = if self.IsYAxis() {
                CSSAnchorSizeValue::kHeight
            } else {
                CSSAnchorSizeValue::kWidth
            };
        }
        let anchor_reference =
            self.ResolveAnchorForEvaluation(anchor_specifier, default_anchor_data);
        if anchor_reference.is_null() {
            return None;
        }
        let anchor_rect = self.CalculateAnchorRectWithScrollOffset(unsafe { &*anchor_reference });
        Some(ResolveAnchorSizeValue(
            &anchor_rect.size,
            anchor_size_value,
            self.container_writing_direction_.GetWritingMode(),
            unsafe { &*self.query_box_ }.StyleRef().GetWritingMode(),
        ))
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:185-210
    fn ResolveAnchorForEvaluation(
        &mut self,
        anchor_specifier: &AnchorSpecifierValue,
        default_anchor_data: &DefaultAnchorData,
    ) -> *const PhysicalAnchorReference {
        let anchor_reference = self.ResolveAnchorReference(anchor_specifier, default_anchor_data);
        if anchor_reference.is_null() {
            return std::ptr::null();
        }
        let reference = unsafe { &*anchor_reference };
        let display_locks = reference.GetDisplayLocks().Get();
        if !display_locks.is_null() {
            if self.display_locks_affected_by_anchors_.is_null() {
                self.display_locks_affected_by_anchors_ =
                    MakeGarbageCollected(GCedHeapHashSet::<Member<Element>>::default());
            }
            for display_lock in unsafe { &*display_locks }.iter() {
                unsafe { &mut *self.display_locks_affected_by_anchors_ }.insert(*display_lock);
            }
        }
        self.did_resolve_anchor_with_running_transform_animation_ |=
            reference.HasRunningTransformAnimation();
        anchor_reference
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:109-154
    fn EvaluateAnchor(
        &mut self,
        anchor_specifier: &AnchorSpecifierValue,
        anchor_value: CSSAnchorValue,
        percentage: f32,
        default_anchor_data: &DefaultAnchorData,
        position_area_offsets: &Option<PositionAreaOffsets>,
    ) -> Option<LayoutUnit> {
        if !self.AllowAnchor() {
            return None;
        }

        let anchor_reference =
            self.ResolveAnchorForEvaluation(anchor_specifier, default_anchor_data);
        if anchor_reference.is_null() {
            return None;
        }

        let has_default_anchor = !self.DefaultAnchor(default_anchor_data).is_null();
        let containing_block_rect = WritingModeConverter::from_logical_size(
            self.container_writing_direction_,
            self.container_size_,
        )
        .ToPhysicalRect(
            self.AdjustedContainingBlockRect(position_area_offsets, has_default_anchor),
        );
        let is_y_axis = self.IsYAxis();
        let axis_size = if is_y_axis {
            containing_block_rect.Height()
        } else {
            containing_block_rect.Width()
        };

        let anchor_rect = self.CalculateAnchorRectWithScrollOffset(unsafe { &*anchor_reference });
        let result = ResolveAnchorValue(
            anchor_rect,
            anchor_value,
            percentage,
            axis_size,
            self.container_writing_direction_,
            unsafe { &*self.query_box_ }
                .StyleRef()
                .GetWritingDirection(),
            &containing_block_rect.offset,
            is_y_axis,
            self.IsRightOrBottom(),
        );
        if result.is_some() {
            let needs_adjustment = if is_y_axis {
                self.needs_scroll_adjustment_in_y_
            } else {
                self.needs_scroll_adjustment_in_x_
            };
            if !needs_adjustment
                && self.ShouldUseScrollAdjustmentFor(
                    unsafe { &*anchor_reference }.GetLayoutObject(),
                    if is_y_axis {
                        PhysicalAxis::kVertical
                    } else {
                        PhysicalAxis::kHorizontal
                    },
                    default_anchor_data,
                )
            {
                if is_y_axis {
                    self.needs_scroll_adjustment_in_y_ = true;
                } else {
                    self.needs_scroll_adjustment_in_x_ = true;
                }
            }
        }
        result
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:352-376
    pub fn AdjustedContainingBlockRect(
        &self,
        position_area_offsets: &Option<PositionAreaOffsets>,
        has_default_anchor: bool,
    ) -> LogicalRect {
        let mut rect = if has_default_anchor {
            self.scroll_rect_.unwrap_or(self.container_rect_)
        } else {
            self.container_rect_
        };
        let style = unsafe { &*self.containing_block_ }.StyleRef();
        let containing_box = DynamicTo::<LayoutBox>(self.containing_block_ as *mut LayoutObject);
        if !containing_box.is_null() {
            rect = unsafe { &*containing_box }.AdjustOutOfFlowContainingBlockForAnchor(
                self.grid_layout_data_,
                style,
                &rect,
                unsafe { &*self.query_box_ },
            );
        }
        if let Some(position_area_offsets) = position_area_offsets {
            rect.Contract(
                &position_area_offsets
                    .insets
                    .ConvertToLogical(self.container_writing_direction_),
            );
        }
        debug_assert!(rect.size.inline_size >= LayoutUnit::default());
        debug_assert!(rect.size.block_size >= LayoutUnit::default());
        rect
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_services.cc:14-46
    fn ResolveAnchorReference(
        &self,
        anchor_specifier: &AnchorSpecifierValue,
        default_anchor_data: &DefaultAnchorData,
    ) -> *const PhysicalAnchorReference {
        if self.anchor_map_.is_null() {
            return std::ptr::null();
        }

        let anchor_map = unsafe { &*self.anchor_map_ };
        let query_box = unsafe { &*self.query_box_ };
        if anchor_specifier.IsNamed() {
            return anchor_map.AnchorReference(
                query_box,
                self.query_box_actual_containing_block_,
                ToAnchorScopedName(anchor_specifier.GetName(), query_box),
            );
        }

        debug_assert!(anchor_specifier.IsDefault());
        match default_anchor_data.GetType() {
            StylePositionAnchorType::kNone => std::ptr::null(),
            StylePositionAnchorType::kAuto => {
                if self.implicit_anchor_.is_null() {
                    return std::ptr::null();
                }
                anchor_map.AnchorReference(
                    query_box,
                    self.query_box_actual_containing_block_,
                    To::<Element>(unsafe { &*self.implicit_anchor_ }.GetNode()),
                )
            }
            StylePositionAnchorType::kName => anchor_map.AnchorReference(
                query_box,
                self.query_box_actual_containing_block_,
                ToAnchorScopedName(default_anchor_data.GetName(), query_box),
            ),
            StylePositionAnchorType::kNormal => unreachable!("normal default anchor data"),
        }
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_services.cc:48-84
    fn CalculateAnchorRectWithScrollOffset(
        &mut self,
        anchor_reference: &PhysicalAnchorReference,
    ) -> PhysicalRect {
        let mut result = anchor_reference.TransformedBoundingRect();
        let element = anchor_reference.GetElement();
        let scroll_offsets = if !self.remembered_scroll_offsets_.is_null() {
            unsafe { &*self.remembered_scroll_offsets_ }
                .GetOffsetsForAnchor(element)
                .or_else(|| {
                    if self.used_scroll_offsets_.is_null() {
                        None
                    } else {
                        unsafe { &*self.used_scroll_offsets_ }.GetOffsetsForAnchor(element)
                    }
                })
        } else if !self.used_scroll_offsets_.is_null() {
            unsafe { &*self.used_scroll_offsets_ }.GetOffsetsForAnchor(element)
        } else {
            None
        }
        .unwrap_or_else(|| {
            let anchored_element = To::<Element>(unsafe { &*self.query_box_ }.GetNode());
            let anchor_object = anchor_reference.GetLayoutObject();
            assert!(!anchored_element.is_null() && !anchor_object.is_null());
            ComputeAnchorScrollOffsets(unsafe { &*anchored_element }, unsafe { &*anchor_object })
        });

        result.Move(&(-scroll_offsets.scroll_offset_for_layout));
        if self.used_scroll_offsets_.is_null() {
            self.used_scroll_offsets_ = MakeGarbageCollected(RememberedScrollOffsets::default());
        }
        unsafe { &mut *self.used_scroll_offsets_ }.SetOffsetsForAnchor(element, &scroll_offsets);
        result
    }
}

impl AnchorEvaluator for AnchorEvaluatorImpl {
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:76-79
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:20-34
    fn Evaluate(
        &mut self,
        anchor_query: &AnchorQuery,
        default_anchor_data: &DefaultAnchorData,
        position_area_offsets: &Option<PositionAreaOffsets>,
    ) -> Option<LayoutUnit> {
        match anchor_query.Type() {
            CSSAnchorQueryType::kAnchor => self.EvaluateAnchor(
                anchor_query.AnchorSpecifier(),
                anchor_query.AnchorSide(),
                anchor_query.AnchorSidePercentageOrZero(),
                default_anchor_data,
                position_area_offsets,
            ),
            CSSAnchorQueryType::kAnchorSize => self.EvaluateAnchorSize(
                anchor_query.AnchorSpecifier(),
                anchor_query.AnchorSize(),
                default_anchor_data,
            ),
        }
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:81-82
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:250-350
    fn ComputePositionAreaOffsetsForLayout(
        &mut self,
        default_anchor_data: &DefaultAnchorData,
    ) -> Option<PositionAreaOffsets> {
        let position_area = default_anchor_data.GetPositionArea();
        assert!(!position_area.IsNone());
        if self.DefaultAnchor(default_anchor_data).is_null() {
            return None;
        }
        let physical_position_area = position_area.ToPhysical(
            &self.container_writing_direction_,
            &unsafe { &*self.query_box_ }
                .StyleRef()
                .GetWritingDirection(),
        );
        assert!(
            !position_area.ContainsAny(),
            "The 'any' keyword can only be used for anchored(fallback) container queries"
        );

        let mut offsets = PhysicalBoxStrut::default();
        let mut behaves_as_auto = PhysicalBoxSides::default();

        offsets.top = {
            let area = physical_position_area.FirstStart();
            if area == PositionAreaRegion::kNone {
                LayoutUnit::default()
            } else {
                behaves_as_auto.top = area == PositionAreaRegion::kTop;
                let side = if area == PositionAreaRegion::kBottom {
                    CSSAnchorValue::kBottom
                } else {
                    CSSAnchorValue::kTop
                };
                let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kTop, Some(self));
                if let Some(value) = anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                    unsafe { &*AnchorSpecifierValue::Default() },
                    side,
                    0.0,
                    default_anchor_data,
                    &None,
                ) {
                    if area == PositionAreaRegion::kTop && value > LayoutUnit::default() {
                        LayoutUnit::default()
                    } else {
                        value
                    }
                } else {
                    LayoutUnit::default()
                }
            }
        };

        offsets.bottom = {
            let area = physical_position_area.FirstEnd();
            if area == PositionAreaRegion::kNone {
                LayoutUnit::default()
            } else {
                behaves_as_auto.bottom = area == PositionAreaRegion::kBottom;
                let side = if area == PositionAreaRegion::kTop {
                    CSSAnchorValue::kTop
                } else {
                    CSSAnchorValue::kBottom
                };
                let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kBottom, Some(self));
                if let Some(value) = anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                    unsafe { &*AnchorSpecifierValue::Default() },
                    side,
                    0.0,
                    default_anchor_data,
                    &None,
                ) {
                    if area == PositionAreaRegion::kBottom && value > LayoutUnit::default() {
                        LayoutUnit::default()
                    } else {
                        value
                    }
                } else {
                    LayoutUnit::default()
                }
            }
        };

        offsets.left = {
            let area = physical_position_area.SecondStart();
            if area == PositionAreaRegion::kNone {
                LayoutUnit::default()
            } else {
                behaves_as_auto.left = area == PositionAreaRegion::kLeft;
                let side = if area == PositionAreaRegion::kRight {
                    CSSAnchorValue::kRight
                } else {
                    CSSAnchorValue::kLeft
                };
                let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kLeft, Some(self));
                if let Some(value) = anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                    unsafe { &*AnchorSpecifierValue::Default() },
                    side,
                    0.0,
                    default_anchor_data,
                    &None,
                ) {
                    if area == PositionAreaRegion::kLeft && value > LayoutUnit::default() {
                        LayoutUnit::default()
                    } else {
                        value
                    }
                } else {
                    LayoutUnit::default()
                }
            }
        };

        offsets.right = {
            let area = physical_position_area.SecondEnd();
            if area == PositionAreaRegion::kNone {
                LayoutUnit::default()
            } else {
                behaves_as_auto.right = area == PositionAreaRegion::kRight;
                let side = if area == PositionAreaRegion::kLeft {
                    CSSAnchorValue::kLeft
                } else {
                    CSSAnchorValue::kRight
                };
                let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kRight, Some(self));
                if let Some(value) = anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                    unsafe { &*AnchorSpecifierValue::Default() },
                    side,
                    0.0,
                    default_anchor_data,
                    &None,
                ) {
                    if area == PositionAreaRegion::kRight && value > LayoutUnit::default() {
                        LayoutUnit::default()
                    } else {
                        value
                    }
                } else {
                    LayoutUnit::default()
                }
            }
        };
        Some(PositionAreaOffsets::new(offsets, behaves_as_auto))
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:84-85
    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.cc:216-248
    fn ComputeAnchorCenterOffsets(
        &mut self,
        builder: &ComputedStyleBuilder,
    ) -> Option<PhysicalOffset> {
        let dummy_percentage = 0.0f32;
        let _reset_adjust_x = AutoResetBool::new(&mut self.needs_scroll_adjustment_in_x_, true);
        let _reset_adjust_y = AutoResetBool::new(&mut self.needs_scroll_adjustment_in_y_, true);
        let top = {
            let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kTop, Some(self));
            anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                unsafe { &*AnchorSpecifierValue::Default() },
                CSSAnchorValue::kCenter,
                dummy_percentage,
                &builder.GetDefaultAnchorData(),
                builder.PositionAreaOffsets(),
            )
        };
        let left = {
            let mut anchor_scope = AnchorScope::new(AnchorEvaluatorMode::kLeft, Some(self));
            anchor_scope.Evaluator().unwrap().EvaluateAnchor(
                unsafe { &*AnchorSpecifierValue::Default() },
                CSSAnchorValue::kCenter,
                dummy_percentage,
                &builder.GetDefaultAnchorData(),
                builder.PositionAreaOffsets(),
            )
        };
        assert_eq!(top.is_some(), left.is_some());
        top.map(|top| PhysicalOffset::new(left.unwrap(), top))
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator_impl.h:87-89
    fn GetContainerWritingDirection(&self) -> WritingDirectionMode {
        self.container_writing_direction_
    }

    // cpp: layoutng_out_of_flow/anchor_evaluator.h:99-100
    fn ModeCell(&self) -> &Cell<AnchorEvaluatorMode> {
        &self.mode_
    }
}
