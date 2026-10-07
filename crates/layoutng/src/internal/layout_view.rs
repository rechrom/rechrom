#![allow(non_snake_case)]

use foundation::graphics_types;
use std::cell::Cell;
use std::ops::{Deref, DerefMut};

use foundation::{
    gfx, AtomicString, CompositingReasons, DynamicTo, EOverflow, GCedHeapHashSet, HeapVector,
    LayoutUnit, MakeGarbageCollected, Member, OverlayScrollbarClipBehavior, PhysicalOffset,
    PhysicalRect, PhysicalSize, RuntimeEnabledFeatures, ToFlooredSize, TransformState, Vector,
    Visitor, WeakHeapHashMap, WeakMember,
};
use graphics_types::graphics::visual_rect_flags::VisualRectFlags;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

use super::block_node::BlockNode;
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_layout::ShouldIncludeScrollbarGutter;
use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayerType};
use super::layout_input_node::MinMaxSizesFloatInput;
use super::layout_invalidation_reason;
use super::layout_node_metadata::ContainerNode;
use super::layout_object::{
    BoxQuadType, HitTestLocation, HitTestResult, LayoutObject, MarkingBehavior, PositionedState,
};
use super::layout_text::LayoutText;
use super::length_utils::SizeType;
use super::map_coordinates_flags::MapCoordinatesFlags;
use super::scroll_types::{IncludeScrollbarsInRect, ScrollOffset};
use super::scrollbar_mode::mojom::blink::ScrollbarMode;
use super::variable_length_transform_result::VariableLengthTransformResult;

// The corresponding C++ classes are only forward-declared in layout_view.h.
#[repr(C)]
pub struct HitTestCache {
    _opaque: [u8; 0],
}
#[repr(C)]
pub struct LayoutViewTransitionRoot {
    _opaque: [u8; 0],
}

// The source tree supplies declarations but no bodies for these view methods.
// The pagination factory is defined in //src/layoutng_paged, outside the
// selected packages; the other contracts belong to the external assembly.
unsafe extern "Rust" {
    fn LayoutViewCreateAnonymousPageLayoutObjectProvider(
        view: &mut LayoutView,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow;
    fn LayoutViewHitTestProvider(
        view: &mut LayoutView,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool;
    fn LayoutViewHitTestNoLifecycleUpdateProvider(
        view: &mut LayoutView,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool;
    fn LayoutViewClearHitTestCacheProvider(view: &mut LayoutView);
    fn LayoutViewMapToVisualRectInAncestorSpaceInternalProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutViewCommitPendingSelectionProvider(view: &mut LayoutView);
    fn LayoutViewQuadsInAncestorInternalProvider(
        view: &LayoutView,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        box_type: BoxQuadType,
    );
    fn LayoutViewUpdateHitTestResultProvider(
        view: &LayoutView,
        result: &mut HitTestResult,
        offset: &PhysicalOffset,
    );
    fn LayoutViewNamedPageAtIndexProvider(view: &LayoutView, page_index: usize) -> AtomicString;
    fn LayoutViewInvalidateLayoutForCounterStyleChangesProvider(view: &mut LayoutView);
    fn LayoutViewInvalidatePaintForViewAndDescendantsProvider(view: &mut LayoutView);
    fn LayoutViewMapAncestorToLocalProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    );
    fn LayoutViewMapLocalToAncestorProvider(
        view: &LayoutView,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    );
    fn LayoutViewGetViewTransitionRootProvider(view: &LayoutView) -> *mut LayoutViewTransitionRoot;
    fn LayoutViewCacheScrollDimensionsProvider(view: &mut LayoutView);
}

// cpp: layoutng/internal/layout_view.h:45-47
pub type SVGTextDescendantsMap =
    WeakHeapHashMap<LayoutBlock, Member<GCedHeapHashSet<Member<LayoutBox>>>>;

// cpp: layoutng/internal/layout_view.h:407-412
#[derive(Clone, Copy, Debug, Default)]
pub struct CachedScrollDimensions {
    pub width: LayoutUnit,
    pub height: LayoutUnit,
    pub origin: gfx::Point,
    pub offset: ScrollOffset,
}

// cpp: layoutng/internal/layout_view.h:50-68
// cpp: layoutng/internal/layout_view.h:349-434
#[repr(C)]
pub struct LayoutView {
    base_: LayoutBlockFlow,
    initial_containing_block_resize_handled_list_: Member<GCedHeapHashSet<Member<LayoutObject>>>,
    initial_containing_block_size_for_printing_: PhysicalSize,
    pagination_scale_factor_: f32,
    anonymous_page_objects_: HeapVector<Member<LayoutObject>>,
    layout_counter_count_: u32,
    layout_list_item_count_: u32,
    svg_text_descendants_: SVGTextDescendantsMap,
    text_to_variable_length_transform_result_:
        WeakHeapHashMap<LayoutText, VariableLengthTransformResult>,
    hit_test_count_: u32,
    hit_test_cache_hits_: u32,
    hit_test_cache_: Member<HitTestCache>,
    autosize_h_scrollbar_mode_: ScrollbarMode,
    autosize_v_scrollbar_mode_: ScrollbarMode,
    cached_scroll_dimensions_: Option<CachedScrollDimensions>,
    previous_background_rect_: Cell<PhysicalRect>,
    vertical_scrollbar_width_for_viewport_units_: i32,
    horizontal_scrollbar_height_for_viewport_units_: i32,
    contains_annotations_: bool,
    contains_non_scaling_stroke_: bool,
}

// cpp: layoutng/internal/layout_view.h:437-442
impl foundation::DowncastFrom<LayoutObject> for LayoutView {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutView()
    }
}

impl foundation::DowncastFrom<LayoutBox> for LayoutView {
    fn AllowFrom(box_: &LayoutBox) -> bool {
        box_.IsLayoutView()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutView, base_) == 0);

impl Deref for LayoutView {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

impl DerefMut for LayoutView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_
    }
}

impl LayoutView {
    // cpp: layoutng/internal/layout_view_layout.cc:22-33
    pub fn new(input_root: *mut ContainerNode) -> Self {
        assert!(!input_root.is_null());
        let mut view = Self {
            base_: LayoutBlockFlow::new(input_root),
            initial_containing_block_resize_handled_list_: Member::default(),
            initial_containing_block_size_for_printing_: PhysicalSize::default(),
            pagination_scale_factor_: 1.0,
            anonymous_page_objects_: HeapVector::default(),
            layout_counter_count_: 0,
            layout_list_item_count_: 0,
            svg_text_descendants_: SVGTextDescendantsMap::default(),
            text_to_variable_length_transform_result_: WeakHeapHashMap::default(),
            hit_test_count_: 0,
            hit_test_cache_hits_: 0,
            hit_test_cache_: Member::default(),
            autosize_h_scrollbar_mode_: ScrollbarMode::kAuto,
            autosize_v_scrollbar_mode_: ScrollbarMode::kAuto,
            cached_scroll_dimensions_: None,
            previous_background_rect_: Cell::new(PhysicalRect::default()),
            vertical_scrollbar_width_for_viewport_units_: 0,
            horizontal_scrollbar_height_for_viewport_units_: 0,
            contains_annotations_: false,
            contains_non_scaling_stroke_: false,
        };
        view.SetRuntimeClass(super::layout_object::LayoutObjectClass::View);
        view.SetInline(false);
        view.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkOnlyThis);
        view.SetPositionState(PositionedState::kIsOutOfFlowPositioned);
        view
    }

    // cpp: layoutng/internal/layout_view_layout.cc:36-42
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.anonymous_page_objects_);
        visitor.Trace(&self.svg_text_descendants_);
        visitor.Trace(&self.text_to_variable_length_transform_result_);
        visitor.Trace(&self.initial_containing_block_resize_handled_list_);
        self.base_.Trace(visitor);
    }

    // cpp: layoutng/internal/layout_view_data.cc:34-45
    pub fn ComputeMinimumWidth(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let mode = style.GetWritingMode();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            mode,
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetContainsAnnotations(self.contains_annotations_);
        let space = builder.ToConstraintSpace();
        BlockNode::new(self as *const LayoutView as *mut LayoutBox)
            .ComputeMinMaxSizes(
                mode,
                SizeType::kIntrinsic,
                &space,
                MinMaxSizesFloatInput::default(),
            )
            .sizes
            .min_size
    }

    // cpp: layoutng/internal/layout_view_data.cc:47-50
    pub fn InitialContainingBlockSize(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        let size = self.GetLayoutSize(IncludeScrollbarsInRect::kIncludeScrollbars);
        PhysicalSize::new(
            LayoutUnit::from_signed(size.width()),
            LayoutUnit::from_signed(size.height()),
        )
    }

    // cpp: layoutng/internal/layout_view_data.cc:52-58
    pub fn RegisterVariableLengthTransformResult(
        &mut self,
        text: &LayoutText,
        result: &VariableLengthTransformResult,
    ) {
        self.CheckIsNotDestroyed();
        assert!(text.HasVariableLengthTransform());
        self.text_to_variable_length_transform_result_.Set(
            WeakMember::from_ptr(text as *const LayoutText as *mut LayoutText),
            result.clone(),
        );
    }

    // cpp: layoutng/internal/layout_view_data.cc:60-64
    pub fn UnregisterVariableLengthTransformResult(&mut self, text: &LayoutText) {
        self.CheckIsNotDestroyed();
        self.text_to_variable_length_transform_result_
            .remove(&WeakMember::from_ptr(
                text as *const LayoutText as *mut LayoutText,
            ));
    }

    // cpp: layoutng/internal/layout_view_data.cc:66-71
    pub fn GetVariableLengthTransformResult(
        &self,
        text: &LayoutText,
    ) -> VariableLengthTransformResult {
        self.CheckIsNotDestroyed();
        assert!(text.HasVariableLengthTransform());
        self.text_to_variable_length_transform_result_
            .get(&WeakMember::from_ptr(
                text as *const LayoutText as *mut LayoutText,
            ))
            .expect("variable-length transform result registered")
            .clone()
    }

    // cpp: layoutng/internal/layout_view_data.cc:73-80
    pub fn SetAutosizeScrollbarModes(&mut self, h_mode: ScrollbarMode, v_mode: ScrollbarMode) {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(
            v_mode == ScrollbarMode::kAuto,
            h_mode == ScrollbarMode::kAuto
        );
        self.autosize_v_scrollbar_mode_ = v_mode;
        self.autosize_h_scrollbar_mode_ = h_mode;
    }

    // cpp: layoutng/internal/layout_view_data.cc:82-85
    pub fn DocumentRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.ScrollableOverflowRect()
    }

    // cpp: layoutng/internal/layout_view_data.cc:87-98
    pub fn SubtractUnconditionalScrollbarsFromViewportUnits(
        &self,
        viewport_size: &gfx::SizeF,
    ) -> gfx::SizeF {
        if !RuntimeEnabledFeatures::SmallerViewportUnitsEnabled() {
            return *viewport_size;
        }
        let mut removed = *viewport_size
            - gfx::SizeF::new(
                self.vertical_scrollbar_width_for_viewport_units_ as f32,
                self.horizontal_scrollbar_height_for_viewport_units_ as f32,
            );
        removed.SetToMax(&gfx::SizeF::new(0.0, 0.0));
        removed
    }

    // cpp: layoutng/internal/layout_view_data.cc:100-112
    pub fn AffectedByResizedInitialContainingBlock(&mut self, result: &LayoutResult) -> bool {
        self.CheckIsNotDestroyed();
        let handled = self.initial_containing_block_resize_handled_list_.Get();
        if handled.is_null() {
            return false;
        }
        let object = result.GetPhysicalFragment().GetLayoutObject();
        debug_assert!(!object.is_null());
        unsafe { &mut *handled }.insert(Member::from_ptr(object as *mut LayoutObject))
    }

    // cpp: layoutng/internal/layout_view_data.cc:114-126
    pub fn SetScrollbarSizesForViewportUnits(&mut self, size: &gfx::Size) -> bool {
        self.CheckIsNotDestroyed();
        let mut changed = false;
        if size.width() != self.vertical_scrollbar_width_for_viewport_units_ {
            self.vertical_scrollbar_width_for_viewport_units_ = size.width();
            changed = true;
        }
        if size.height() != self.horizontal_scrollbar_height_for_viewport_units_ {
            self.horizontal_scrollbar_height_for_viewport_units_ = size.height();
            changed = true;
        }
        changed
    }

    // cpp: layoutng/internal/layout_view_data.cc:128-136
    pub fn SetContainsAnnotations(&mut self) {
        self.CheckIsNotDestroyed();
        if self.contains_annotations_ {
            return;
        }
        self.contains_annotations_ = true;
        self.SetNeedsLayout(&raw const layout_invalidation_reason::kStyleChange);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:44-47
    pub fn IsBeingAutoSized(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_view_layout.cc:49-52
    pub fn PreviousBackgroundRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.previous_background_rect_.get()
    }
    // cpp: layoutng/internal/layout_view_layout.cc:54-57
    pub fn SetPreviousBackgroundRect(&self, rect: PhysicalRect) {
        self.CheckIsNotDestroyed();
        self.previous_background_rect_.set(rect);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:59-65
    pub fn AddChild(&mut self, child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        assert_ne!(
            unsafe { &*child }.StyleRef().StyleType(),
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdViewTransition
        );
        self.base_.AddChild(child, before_child);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:67-71
    pub fn IsChildAllowed(&self, child: *const LayoutObject, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { &*child }.IsBox()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:73-76
    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_view_layout.cc:78-84
    pub fn GetLayoutSize(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Size {
        self.CheckIsNotDestroyed();
        if self.InputOwnerForLayout().InputPrinting() {
            return ToFlooredSize(&self.initial_containing_block_size_for_printing_);
        }
        self.GetNonPrintingLayoutSize(inclusion)
    }

    // cpp: layoutng/internal/layout_view_layout.cc:86-97
    pub fn GetNonPrintingLayoutSize(&self, inclusion: IncludeScrollbarsInRect) -> gfx::Size {
        self.CheckIsNotDestroyed();
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        let mut rect = PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(viewport.size.width),
                LayoutUnit::from_signed(viewport.size.height),
            ),
        );
        if inclusion == IncludeScrollbarsInRect::kExcludeScrollbars && self.IsScrollContainer() {
            self.ExcludeScrollbars(
                &mut rect,
                OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            );
        }
        ToFlooredSize(&rect.size)
    }

    // cpp: layoutng/internal/layout_view_layout.cc:99-105
    pub fn ViewLogicalHeightForPercentages(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let size = if self.InputOwnerForLayout().InputPrinting() {
            self.initial_containing_block_size_for_printing_
        } else {
            let size = self.GetLayoutSize(IncludeScrollbarsInRect::kExcludeScrollbars);
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width()),
                LayoutUnit::from_signed(size.height()),
            )
        };
        if self.IsHorizontalWritingMode() {
            size.height
        } else {
            size.width
        }
    }

    // cpp: layoutng/internal/layout_view_layout.cc:107-111
    pub fn RootBox(&self) -> &LayoutBox {
        self.CheckIsNotDestroyed();
        let child = DynamicTo::<LayoutBox>(self.FirstChild());
        if child.is_null() {
            unsafe { &*(self as *const LayoutView as *const LayoutBox) }
        } else {
            unsafe { &*child }
        }
    }

    // cpp: layoutng/internal/layout_view_layout.cc:113-126
    pub fn LayoutRoot(&self) {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let writing_mode = style.GetWritingMode();
        let size = ToLogicalSize(self.InitialContainingBlockSize(), writing_mode);
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            writing_mode,
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(size);
        builder.SetIsFixedInlineSize(true);
        builder.SetIsFixedBlockSize(true);
        builder.SetContainsAnnotations(self.contains_annotations_);
        let space = builder.ToConstraintSpace();
        BlockNode::new(self as *const LayoutView as *mut LayoutBox).Layout(
            &space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
    }

    // cpp: layoutng/internal/layout_view_layout.cc:128-139
    pub fn ViewRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        if self.InputOwnerForLayout().InputPrinting() {
            return PhysicalRect::new(PhysicalOffset::default(), self.StitchedSize());
        }
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        let size = viewport
            .transition_snapshot_size
            .as_ref()
            .unwrap_or(&viewport.size);
        PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width),
                LayoutUnit::from_signed(size.height),
            ),
        )
    }

    // cpp: layoutng/internal/layout_view_layout.cc:141-145
    pub fn OverflowClipRectWithBehavior(
        &self,
        behavior: OverlayScrollbarClipBehavior,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRectInternal(behavior, false)
    }
    // cpp: layoutng/internal/layout_view_layout.cc:147-150
    pub fn OverflowClipRectForScrollNode(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRectInternal(
            OverlayScrollbarClipBehavior::kIgnoreOverlayScrollbarSize,
            true,
        )
    }
    // cpp: layoutng/internal/layout_view_layout.cc:152-162
    fn OverflowClipRectInternal(
        &self,
        behavior: OverlayScrollbarClipBehavior,
        _for_scroll_node: bool,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let mut rect = self.ViewRect();
        if rect.IsEmpty() {
            return self.base_.OverflowClipRectWithBehavior(behavior);
        }
        if self.IsScrollContainer() {
            self.ExcludeScrollbars(
                &mut rect,
                behavior,
                ShouldIncludeScrollbarGutter::kIncludeScrollbarGutter,
            );
        }
        rect
    }

    // cpp: layoutng/internal/layout_view_layout.cc:164-182
    pub fn CalculateScrollbarModes(
        &self,
        horizontal: &mut ScrollbarMode,
        vertical: &mut ScrollbarMode,
        overflow_x: Option<EOverflow>,
        overflow_y: Option<EOverflow>,
    ) {
        self.CheckIsNotDestroyed();
        assert_eq!(overflow_x.is_some(), overflow_y.is_some());
        let x = overflow_x.unwrap_or_else(|| self.StyleRef().OverflowX());
        let y = overflow_y.unwrap_or_else(|| self.StyleRef().OverflowY());
        let mode = |overflow: EOverflow| match overflow {
            EOverflow::kScroll => ScrollbarMode::kAlwaysOn,
            EOverflow::kHidden | EOverflow::kClip => ScrollbarMode::kAlwaysOff,
            _ => ScrollbarMode::kAuto,
        };
        *horizontal = mode(x);
        *vertical = mode(y);
    }

    // cpp: layoutng/internal/layout_view_layout.cc:184-187
    pub fn ShouldPlaceBlockDirectionScrollbarOnLogicalLeft(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef()
            .ShouldPlaceBlockDirectionScrollbarOnLogicalLeft()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:189-192
    pub fn OffsetForFixedPosition(&self) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        PhysicalOffset::default()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:194-202
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        for object in self.anonymous_page_objects_.iter() {
            let object = object.Get();
            if !unsafe { &*object }.BeingDestroyed() {
                unsafe { &mut *object }.Destroy();
            }
        }
        self.anonymous_page_objects_.clear();
        self.base_.WillBeDestroyed();
    }

    // cpp: layoutng/internal/layout_view_layout.cc:204-207
    pub fn UpdateFromStyle(&mut self) {
        self.CheckIsNotDestroyed();
        self.base_.UpdateFromStyle();
    }
    // cpp: layoutng/internal/layout_view_layout.cc:209-212
    pub fn UpdateAfterLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.base_.UpdateAfterLayout();
    }
    // cpp: layoutng/internal/layout_view_layout.cc:214-221
    pub fn StyleDidChange(
        &mut self,
        diff: layoutng_style::style::style_difference::StyleDifference,
        old_style: Option<&ComputedStyle>,
        new_style: &ComputedStyle,
        context: &super::layout_object::StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.base_.StyleDidChange(
            diff,
            old_style.map_or(std::ptr::null(), |style| style),
            new_style,
            context,
        );
    }

    // cpp: layoutng/internal/layout_view_layout.cc:223-226
    pub fn IsFragmentationContextRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputPaginated()
    }
    // cpp: layoutng/internal/layout_view_layout.cc:228-231
    pub fn ShouldUsePaginatedLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputPaginated()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:233-238
    pub fn SmallViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        let viewport = self
            .InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("layout viewport required");
        self.SubtractUnconditionalScrollbarsFromViewportUnits(&gfx::SizeF::new(
            viewport.size.width as f32,
            viewport.size.height as f32,
        ))
    }
    // cpp: layoutng/internal/layout_view_layout.cc:240-242
    pub fn LargeViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        self.SmallViewportSizeForViewportUnits()
    }
    // cpp: layoutng/internal/layout_view_layout.cc:244-246
    pub fn DynamicViewportSizeForViewportUnits(&self) -> gfx::SizeF {
        self.SmallViewportSizeForViewportUnits()
    }
    // cpp: layoutng/internal/layout_view_layout.cc:248-250
    pub fn PaginationViewportSizeForMediaQueries(&self) -> gfx::SizeF {
        gfx::SizeF::from(self.initial_containing_block_size_for_printing_)
    }

    // cpp: layoutng/internal/layout_view_layout.cc:252-256
    pub fn BackgroundIsKnownToBeOpaqueInRect(&self, _local_rect: &PhysicalRect) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_view_layout.cc:258-261
    pub fn DebugRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let size = self.GetLayoutSize(IncludeScrollbarsInRect::kIncludeScrollbars);
        PhysicalRect::new(
            PhysicalOffset::default(),
            PhysicalSize::new(
                LayoutUnit::from_signed(size.width()),
                LayoutUnit::from_signed(size.height()),
            ),
        )
    }

    // cpp: layoutng/internal/layout_view_layout.cc:263-266
    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        self.CheckIsNotDestroyed();
        CompositingReasons::default()
    }

    // cpp: layoutng/internal/layout_view_layout.cc:268-271
    pub fn HasTickmarks(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }
    // cpp: layoutng/internal/layout_view_layout.cc:273-276
    pub fn GetTickmarks(&self) -> foundation::Vector<gfx::Rect> {
        self.CheckIsNotDestroyed();
        foundation::Vector::default()
    }

    // cpp: layoutng/internal/layout_view.h:90-98
    pub fn HitTestCount(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.hit_test_count_
    }
    pub fn HitTestCacheHits(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.hit_test_cache_hits_
    }

    // cpp: layoutng/internal/layout_view.h:102-115
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutView"
    }
    pub fn IsLayoutView(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }
    pub fn LayerTypeRequired(&self) -> PaintLayerType {
        self.CheckIsNotDestroyed();
        PaintLayerType::kNormalPaintLayer
    }

    // cpp: layoutng/internal/layout_view.h:77-88
    pub fn CreateAnonymousPageLayoutObject(
        &mut self,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow {
        unsafe { LayoutViewCreateAnonymousPageLayoutObjectProvider(self, style) }
    }
    pub fn HitTest(&mut self, location: &HitTestLocation, result: &mut HitTestResult) -> bool {
        unsafe { LayoutViewHitTestProvider(self, location, result) }
    }
    pub fn HitTestNoLifecycleUpdate(
        &mut self,
        location: &HitTestLocation,
        result: &mut HitTestResult,
    ) -> bool {
        unsafe { LayoutViewHitTestNoLifecycleUpdateProvider(self, location, result) }
    }
    // cpp: layoutng/internal/layout_view.h:100-100
    pub fn ClearHitTestCache(&mut self) {
        unsafe { LayoutViewClearHitTestCacheProvider(self) }
    }

    // cpp: layoutng/internal/layout_view.h:135-146
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe {
            LayoutViewMapToVisualRectInAncestorSpaceInternalProvider(self, ancestor, state, flags)
        }
    }
    pub fn CommitPendingSelection(&mut self) {
        unsafe { LayoutViewCommitPendingSelectionProvider(self) }
    }
    // cpp: layoutng/internal/layout_view.h:148-151
    pub fn QuadsInAncestorInternal(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        flags: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        unsafe { LayoutViewQuadsInAncestorInternalProvider(self, quads, ancestor, flags, box_type) }
    }

    // cpp: layoutng/internal/layout_view.h:182-183
    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutViewUpdateHitTestResultProvider(self, result, offset) }
    }

    // cpp: layoutng/internal/layout_view.h:205-205
    pub fn NamedPageAtIndex(&self, page_index: usize) -> AtomicString {
        unsafe { LayoutViewNamedPageAtIndexProvider(self, page_index) }
    }

    // cpp: layoutng/internal/layout_view.h:263-265
    pub fn InvalidateLayoutForCounterStyleChanges(&mut self) {
        unsafe { LayoutViewInvalidateLayoutForCounterStyleChangesProvider(self) }
    }
    // cpp: layoutng/internal/layout_view.h:288-288
    pub fn InvalidatePaintForViewAndDescendants(&mut self) {
        unsafe { LayoutViewInvalidatePaintForViewAndDescendantsProvider(self) }
    }

    // cpp: layoutng/internal/layout_view.h:313-315
    pub fn MapAncestorToLocal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    ) {
        unsafe { LayoutViewMapAncestorToLocalProvider(self, ancestor, state, flags) }
    }
    // cpp: layoutng/internal/layout_view.h:319-321
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: MapCoordinatesFlags,
    ) {
        unsafe { LayoutViewMapLocalToAncestorProvider(self, ancestor, state, flags) }
    }

    // cpp: layoutng/internal/layout_view.h:338-340
    pub fn GetViewTransitionRoot(&self) -> *mut LayoutViewTransitionRoot {
        unsafe { LayoutViewGetViewTransitionRootProvider(self) }
    }
    pub fn CacheScrollDimensions(&mut self) {
        unsafe { LayoutViewCacheScrollDimensionsProvider(self) }
    }

    // cpp: layoutng/internal/layout_view.h:158-168
    pub fn AutosizeHorizontalScrollbarMode(&self) -> ScrollbarMode {
        self.CheckIsNotDestroyed();
        self.autosize_h_scrollbar_mode_
    }
    pub fn AutosizeVerticalScrollbarMode(&self) -> ScrollbarMode {
        self.CheckIsNotDestroyed();
        self.autosize_v_scrollbar_mode_
    }

    // cpp: layoutng/internal/layout_view.h:176-180
    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_view.h:187-203
    pub fn SetInitialContainingBlockSizeForPrinting(&mut self, size: PhysicalSize) {
        self.CheckIsNotDestroyed();
        self.initial_containing_block_size_for_printing_ = size;
    }
    pub fn InitialContainingBlockSizeForPrinting(&self) -> PhysicalSize {
        self.CheckIsNotDestroyed();
        self.initial_containing_block_size_for_printing_
    }
    pub fn SetPaginationScaleFactor(&mut self, factor: f32) {
        self.CheckIsNotDestroyed();
        self.pagination_scale_factor_ = factor;
    }
    pub fn PaginationScaleFactor(&self) -> f32 {
        self.CheckIsNotDestroyed();
        self.pagination_scale_factor_
    }

    // cpp: layoutng/internal/layout_view.h:214-241
    pub fn AddLayoutCounter(&mut self) {
        self.CheckIsNotDestroyed();
        self.layout_counter_count_ += 1;
    }
    pub fn RemoveLayoutCounter(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.layout_counter_count_ > 0);
        self.layout_counter_count_ -= 1;
    }
    pub fn HasLayoutCounters(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.layout_counter_count_ != 0
    }
    pub fn AddLayoutListItem(&mut self) {
        self.CheckIsNotDestroyed();
        self.layout_list_item_count_ += 1;
    }
    pub fn RemoveLayoutListItem(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.layout_list_item_count_ > 0);
        self.layout_list_item_count_ -= 1;
    }
    pub fn HasLayoutListItems(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.layout_list_item_count_ != 0
    }

    // cpp: layoutng/internal/layout_view.h:249-256
    pub fn SetContainsNonScalingStroke(&mut self) {
        self.CheckIsNotDestroyed();
        self.contains_non_scaling_stroke_ = true;
    }
    pub fn ContainsNonScalingStroke(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.contains_non_scaling_stroke_
    }

    // cpp: layoutng/internal/layout_view.h:301-307
    pub fn BackgroundRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.OverflowClipRect()
    }

    // cpp: layoutng/internal/layout_view.h:323-328
    pub fn SvgTextDescendantsMap(&mut self) -> &mut SVGTextDescendantsMap {
        self.CheckIsNotDestroyed();
        &mut self.svg_text_descendants_
    }

    // cpp: layoutng/internal/layout_view.h:358-363
    fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_view.h:437-442
    pub fn AllowFromLayoutObject(object: &LayoutObject) -> bool {
        object.IsLayoutView()
    }
}

// These two definitions live in layout_view_data.cc because the view owns the
// traced descendant map, while their receiver remains LayoutBlock.
impl LayoutBlock {
    // cpp: layoutng/internal/layout_block.h:142-142
    // cpp: layoutng/internal/layout_view_data.cc:163-173
    pub fn AddSvgTextDescendant(&mut self, svg_text: &mut LayoutBox) {
        self.CheckIsNotDestroyed();
        assert!(svg_text.IsSVGText());
        let view = unsafe { &mut *self.View() };
        let key = WeakMember::from_ptr(self as *mut LayoutBlock);
        let map = view.SvgTextDescendantsMap();
        let is_new_entry = map.insert(key.clone(), Member::default());
        if is_new_entry {
            *map.get_mut(&key).unwrap() = Member::from_ptr(MakeGarbageCollected(
                GCedHeapHashSet::<Member<LayoutBox>>::default(),
            ));
        }
        let descendants = unsafe { &mut *map.get(&key).unwrap().Get() };
        descendants.insert(Member::from_ptr(svg_text as *mut LayoutBox));
        self.SetHasSVGTextDescendants(true);
    }

    // cpp: layoutng/internal/layout_block.h:143-143
    // cpp: layoutng/internal/layout_view_data.cc:175-188
    pub fn RemoveSvgTextDescendant(&mut self, svg_text: &mut LayoutBox) {
        self.CheckIsNotDestroyed();
        assert!(svg_text.IsSVGText());
        let view = unsafe { &mut *self.View() };
        let map = view.SvgTextDescendantsMap();
        let key = WeakMember::from_ptr(self as *mut LayoutBlock);
        let Some(descendants_member) = map.get(&key) else {
            return;
        };
        let descendants = unsafe { &mut *descendants_member.Get() };
        descendants.erase(&Member::from_ptr(svg_text as *mut LayoutBox));
        if descendants.empty() {
            map.remove(&key);
            self.SetHasSVGTextDescendants(false);
        }
    }
}
