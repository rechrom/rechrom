use foundation::{Member, Visitor};
use layoutng_geometry::geometry::axis::{kPhysicalAxesNone, PhysicalAxes};

// cpp: layoutng/internal/split_axis_item.h:20-55
pub struct SplitAxisItem<T> {
    value_: Member<T>,
    consumed_axes_: PhysicalAxes,
    pending_axes_: PhysicalAxes,
}

impl<T> Default for SplitAxisItem<T> {
    fn default() -> Self {
        Self {
            value_: Member::default(),
            consumed_axes_: kPhysicalAxesNone,
            pending_axes_: kPhysicalAxesNone,
        }
    }
}

#[allow(non_snake_case)]
impl<T> SplitAxisItem<T> {
    pub fn new(value: *mut T, consumed_axes: PhysicalAxes, pending_axes: PhysicalAxes) -> Self {
        Self {
            value_: Member::from_ptr(value),
            consumed_axes_: consumed_axes,
            pending_axes_: pending_axes,
        }
    }

    pub fn GetIfConsumed(&self) -> *mut T {
        if self.consumed_axes_ != kPhysicalAxesNone {
            self.value_.Get()
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn GetIfPending(&self) -> *mut T {
        if self.pending_axes_ != kPhysicalAxesNone {
            self.value_.Get()
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn ConsumedAxes(&self) -> PhysicalAxes {
        self.consumed_axes_
    }

    pub fn PendingAxes(&self) -> PhysicalAxes {
        self.pending_axes_
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.value_);
    }
}

impl<T> PartialEq for SplitAxisItem<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value_.Get() == other.value_.Get()
            && self.consumed_axes_ == other.consumed_axes_
            && self.pending_axes_ == other.pending_axes_
    }
}

// cpp: layoutng/internal/split_axis_item.h:57-61
// The foundation VectorTraits specialization must permit clearing unused
// SplitAxisItem slots with memset when that external trait is connected.
