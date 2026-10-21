use std::ops::Deref;

use font_engine::FontHeight;
use foundation::{MakeGarbageCollected, TextDirection, Visitor};
use layoutng::internal::layout_object::LayoutObject;

use crate::physical_fragment::{FragmentType, PhysicalFragment, PhysicalFragmentFlags};

// cpp: layoutng_fragment_tree/physical_line_box_fragment.h:20-34
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineBoxType {
    kNormalLineBox = 0,
    kEmptyLineBox = 1,
    kRubyLineBox = 2,
}

impl LineBoxType {
    pub const kMaxValue: Self = Self::kRubyLineBox;
}

// The physical base is first, matching C++ inheritance and preserving the
// parent fields used by line-box accessors. The source's ASSERT_SIZE is a
// layout constraint to revisit when the PhysicalFragment base is translated.
// cpp: layoutng_fragment_tree/physical_line_box_fragment.h:18
// cpp: layoutng_fragment_tree/physical_line_box_fragment.h:76-78
// cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:24-29
#[repr(C)]
pub struct PhysicalLineBoxFragment {
    pub(crate) base_: PhysicalFragment,
    pub(crate) metrics_: FontHeight,
}

// cpp: layoutng_fragment_tree/physical_line_box_fragment.h:80-86
impl foundation::DowncastFrom<PhysicalFragment> for PhysicalLineBoxFragment {
    fn AllowFrom(fragment: &PhysicalFragment) -> bool {
        fragment.Type() == crate::physical_fragment::FragmentType::kFragmentLineBox
    }
}

const _: () = assert!(std::mem::offset_of!(PhysicalLineBoxFragment, base_) == 0);

impl Deref for PhysicalLineBoxFragment {
    type Target = PhysicalFragment;

    fn deref(&self) -> &Self::Target {
        &self.base_
    }
}

#[allow(non_snake_case)]
impl PhysicalLineBoxFragment {
    // The ordinary builder constructor and Create() are defined in
    // //src/layoutng_inline/physical_line_box_fragment_builder.cc and connect
    // through the shared inline assembly.
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:36
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:40-41

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:38
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:11-14
    pub fn Clone(other: &Self) -> *const Self {
        MakeGarbageCollected(Self::copy_from(other))
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:42
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:16-23
    fn copy_from(other: &Self) -> Self {
        let base = other.base_.clone();
        base.flags_.set(
            PhysicalFragmentFlags::BASE_DIRECTION,
            other
                .base_
                .flags_
                .get(PhysicalFragmentFlags::BASE_DIRECTION),
        );
        base.flags_.set(
            PhysicalFragmentFlags::HAS_HANGING,
            other.base_.flags_.get(PhysicalFragmentFlags::HAS_HANGING),
        );
        base.flags_.set(
            PhysicalFragmentFlags::HAS_PROPAGATED_DESCENDANTS,
            other
                .base_
                .flags_
                .get(PhysicalFragmentFlags::HAS_PROPAGATED_DESCENDANTS),
        );
        Self {
            base_: base,
            metrics_: other.metrics_,
        }
    }

    // C++ uses a default destructor; the Rust fields own their destruction.
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:43
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:31

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:45
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:40-42
    pub fn TraceAfterDispatch(&self, visitor: &mut Visitor) {
        self.base_.TraceAfterDispatch(visitor);
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:47-49
    pub fn GetLineBoxType(&self) -> LineBoxType {
        match self.base_.sub_type() {
            0 => LineBoxType::kNormalLineBox,
            1 => LineBoxType::kEmptyLineBox,
            2 => LineBoxType::kRubyLineBox,
            _ => unreachable!("invalid line-box subtype"),
        }
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:50
    pub fn IsEmptyLineBox(&self) -> bool {
        self.GetLineBoxType() == LineBoxType::kEmptyLineBox
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:52-53
    pub fn HasPropagatedDescendants(&self) -> bool {
        self.base_
            .flags_
            .get(PhysicalFragmentFlags::HAS_PROPAGATED_DESCENDANTS)
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:55-56
    pub fn HasHanging(&self) -> bool {
        self.base_.flags_.get(PhysicalFragmentFlags::HAS_HANGING)
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:58
    pub fn Metrics(&self) -> &FontHeight {
        &self.metrics_
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:60-65
    pub fn BaseDirection(&self) -> TextDirection {
        if self.base_.flags_.get(PhysicalFragmentFlags::BASE_DIRECTION) {
            TextDirection::kRtl
        } else {
            TextDirection::kLtr
        }
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:67-68
    // cpp: layoutng_fragment_tree/physical_line_box_fragment.cc:33-38
    pub fn BaselineMetrics(&self) -> FontHeight {
        self.metrics_
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:70-74
    pub fn ContainerLayoutObject(&self) -> *const LayoutObject {
        self.base_.layout_object_ptr()
    }

    // cpp: layoutng_fragment_tree/physical_line_box_fragment.h:80-85
    pub fn AllowFrom(fragment: &PhysicalFragment) -> bool {
        fragment.Type() == FragmentType::kFragmentLineBox
    }
}
