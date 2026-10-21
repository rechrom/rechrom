#![allow(non_snake_case)]

use std::cell::{Cell, OnceCell, UnsafeCell};

use foundation::{
    gfx, DisallowNewWrapper, IsFlippedBlocksWritingMode, IsHorizontalWritingMode, LayoutUnit,
    MakeGarbageCollected, Member, Persistent, PhysicalOffset, PhysicalRect, PhysicalSize, Visitor,
    WeakHeapHashMap, WeakMember,
};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToPhysicalSize};
use layoutng_style::style::computed_style_constants::ShapeBox;

use super::shape::Shape;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_pass_scope::LayoutPassScope;

type ShapeOutsideInfoMapHolder = DisallowNewWrapper<ShapeOutsideInfoMap>;

thread_local! {
    // Rust's GC root is scoped to the layout thread, like the existing
    // scroll-clamp persistent root. Every access still reaches one map for
    // that thread's layout objects.
    static INFO_MAP: OnceCell<Persistent<ShapeOutsideInfoMapHolder>> = const { OnceCell::new() };
}

// cpp: layoutng/internal/shapes/shape_outside_info.h:86-88
pub type ShapeOutsideInfoMap = WeakHeapHashMap<LayoutBox, Member<ShapeOutsideInfo>>;

// cpp: layoutng/internal/shapes/shape_outside_info.h:47-48
// cpp: layoutng/internal/shapes/shape_outside_info.h:90-95
pub struct ShapeOutsideInfo {
    layout_box_: Member<LayoutBox>,
    shape_: UnsafeCell<Option<Box<dyn Shape>>>,
    reference_box_logical_size_: LogicalSize,
    percentage_resolution_inline_size_: LayoutUnit,
    is_computing_shape_: Cell<bool>,
}

impl ShapeOutsideInfo {
    // cpp: layoutng/internal/shapes/shape_outside_info.h:49-54
    // cpp: layoutng/internal/layout_box_geometry.cc:786-789
    pub fn new(layout_box: &LayoutBox) -> Self {
        Self {
            layout_box_: Member::from_ptr(layout_box as *const LayoutBox as *mut LayoutBox),
            shape_: UnsafeCell::new(None),
            reference_box_logical_size_: LogicalSize::default(),
            percentage_resolution_inline_size_: LayoutUnit::default(),
            is_computing_shape_: Cell::new(false),
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:806-829
    pub fn SetReferenceBoxLogicalSize(&mut self, mut size: LogicalSize, margin_size: LogicalSize) {
        let layout_box = unsafe { &*self.layout_box_.Get() };
        let writing_direction = unsafe { &*layout_box.ContainingBlock() }
            .StyleRef()
            .GetWritingDirection();
        let shape_outside = unsafe { &*layout_box.StyleRef().ShapeOutside() };
        match shape_outside.CssBox() {
            ShapeBox::kMarginBox => size.Expand(margin_size.inline_size, margin_size.block_size),
            ShapeBox::kBorderBox => {}
            ShapeBox::kPaddingBox => {
                size -= layout_box
                    .BorderOutsets()
                    .ConvertToLogical(writing_direction)
            }
            ShapeBox::kContentBox => {
                size -= (layout_box.BorderOutsets() + layout_box.PaddingOutsets())
                    .ConvertToLogical(writing_direction)
            }
        }
        size = size.ClampNegativeToZero();
        if self.reference_box_logical_size_ == size {
            return;
        }
        self.reference_box_logical_size_ = size;
        self.MarkShapeAsDirty();
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:831-835
    pub fn SetPercentageResolutionInlineSize(&mut self, size: LayoutUnit) {
        if self.percentage_resolution_inline_size_ == size {
            return;
        }
        self.percentage_resolution_inline_size_ = size;
        self.MarkShapeAsDirty();
    }

    // cpp: layoutng/internal/shapes/shape_outside_info.h:56-67
    // A raw pointer retains C++'s cache-owned, aliasable reference semantics.
    pub fn EnsureInfo(key: &LayoutBox) -> *mut Self {
        let map = unsafe { &mut *Self::GetInfoMap() };
        let weak_key = WeakMember::from_ptr(key as *const LayoutBox as *mut LayoutBox);
        if let Some(existing) = map.get(&weak_key) {
            return existing.Get();
        }
        let info = MakeGarbageCollected(Self::new(key));
        map.insert(weak_key, Member::from_ptr(info));
        info
    }

    pub fn RemoveInfo(key: &LayoutBox) {
        let map = unsafe { &mut *Self::GetInfoMap() };
        map.remove(&WeakMember::from_ptr(
            key as *const LayoutBox as *mut LayoutBox,
        ));
    }

    pub fn Info(key: &LayoutBox) -> *mut Self {
        if !Self::IsEnabledFor(key) {
            return std::ptr::null_mut();
        }
        Self::EnsureInfo(key)
    }

    // cpp: layoutng/internal/shapes/shape_outside_info.h:70-88
    // cpp: layoutng/internal/layout_box_geometry.cc:837-839
    pub fn MarkShapeAsDirty(&mut self) {
        unsafe { *self.shape_.get() = None };
    }

    pub fn IsComputingShape(&self) -> bool {
        self.is_computing_shape_.get()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:883-902
    pub fn ComputedShapePhysicalBoundingBox(&self) -> PhysicalRect {
        let logical_box = self.ComputedShape().ShapeMarginLogicalBoundingBox();
        let mut physical_box = PhysicalRect::new(
            PhysicalOffset::new(
                logical_box.offset.inline_offset,
                logical_box.offset.block_offset,
            ),
            PhysicalSize::new(logical_box.size.inline_size, logical_box.size.block_size),
        );
        let start = self.LogicalStartOffset();
        physical_box.offset.left += start.inline_offset;
        let layout_box = unsafe { &*self.layout_box_.Get() };
        if layout_box.StyleRef().IsFlippedBlocksWritingMode() {
            physical_box.offset.top = layout_box.LogicalHeight() - physical_box.Bottom();
        } else {
            physical_box.offset.top += start.block_offset;
        }
        if !layout_box.StyleRef().IsHorizontalWritingMode() {
            physical_box = PhysicalRect::new(
                PhysicalOffset::new(physical_box.offset.top, physical_box.offset.left),
                PhysicalSize::new(physical_box.size.height, physical_box.size.width),
            );
        }
        physical_box
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:904-912
    pub fn ShapeToLayoutObjectPoint(&self, mut point: gfx::PointF) -> gfx::PointF {
        let start = self.LogicalStartOffset();
        point.Offset(start.inline_offset.ToFloat(), start.block_offset.ToFloat());
        let layout_box = unsafe { &*self.layout_box_.Get() };
        let writing_mode = layout_box.StyleRef().GetWritingMode();
        if IsFlippedBlocksWritingMode(writing_mode) {
            point.set_y(layout_box.LogicalHeight().ToFloat() - point.y());
        }
        if !IsHorizontalWritingMode(writing_mode) {
            point.Transpose();
        }
        point
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:851-864
    pub fn ComputedShape(&self) -> &dyn Shape {
        if unsafe { &*self.shape_.get() }.is_none() {
            assert!(!self.is_computing_shape_.get());
            self.is_computing_shape_.set(true);
            struct ResetComputingFlag<'a>(&'a Cell<bool>);
            impl Drop for ResetComputingFlag<'_> {
                fn drop(&mut self) {
                    self.0.set(false);
                }
            }
            let _reset = ResetComputingFlag(&self.is_computing_shape_);
            let support = unsafe { &*LayoutPassScope::RequireFloatSupport() };
            let create_shape = support.create_shape.expect("float shape factory required");
            let shape = create_shape(
                unsafe { &*self.layout_box_.Get() },
                self.reference_box_logical_size_,
                self.percentage_resolution_inline_size_,
            );
            unsafe { *self.shape_.get() = shape };
        }
        unsafe {
            (&*self.shape_.get())
                .as_deref()
                .expect("computed shape required")
        }
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:841-843
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_box_);
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:798-804
    fn IsEnabledFor(layout_box: &LayoutBox) -> bool {
        if !layout_box.IsFloating() || layout_box.StyleRef().ShapeOutside().is_null() {
            return false;
        }
        let node = layout_box.GetNode();
        if node.is_null() {
            return false;
        }
        unsafe { &*node }
            .InputStyle()
            .extended
            .as_ref()
            .is_some_and(|extended| extended.shape_outside.is_some())
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:845-849
    fn ReferenceBoxPhysicalSize(&self) -> PhysicalSize {
        let layout_box = unsafe { &*self.layout_box_.Get() };
        ToPhysicalSize(
            self.reference_box_logical_size_,
            unsafe { &*layout_box.ContainingBlock() }
                .StyleRef()
                .GetWritingMode(),
        )
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:867-881
    fn LogicalStartOffset(&self) -> LogicalOffset {
        let layout_box = unsafe { &*self.layout_box_.Get() };
        let shape_outside = unsafe { &*layout_box.StyleRef().ShapeOutside() };
        let outsets = match shape_outside.CssBox() {
            ShapeBox::kMarginBox => -layout_box.MarginOutsets(),
            ShapeBox::kBorderBox => Default::default(),
            ShapeBox::kPaddingBox => layout_box.BorderOutsets(),
            ShapeBox::kContentBox => layout_box.BorderOutsets() + layout_box.PaddingOutsets(),
        };
        outsets
            .ConvertToLogical(
                unsafe { &*layout_box.ContainingBlock() }
                    .StyleRef()
                    .GetWritingDirection(),
            )
            .StartOffset()
    }

    // cpp: layoutng/internal/layout_box_geometry.cc:791-796
    fn GetInfoMap() -> *mut ShapeOutsideInfoMap {
        INFO_MAP.with(|slot| {
            let root = slot.get_or_init(|| {
                Persistent::from_ptr(MakeGarbageCollected(ShapeOutsideInfoMapHolder::new(
                    ShapeOutsideInfoMap::default(),
                )))
            });
            unsafe { &mut *root.Get() }.Value() as *mut ShapeOutsideInfoMap
        })
    }
}
