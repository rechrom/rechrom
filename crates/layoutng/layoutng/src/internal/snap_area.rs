use super::layout_node_metadata::Element;
use foundation::{Member, Visitor, WritingDirectionMode};
use layoutng_geometry::geometry::axis::{kPhysicalAxesNone, PhysicalAxes};

// cpp: layoutng/internal/snap_area.h:22-78
#[derive(Clone)]
pub struct SnapArea {
    element_: Member<Element>,
    consumed_axes_: PhysicalAxes,
    pending_axes_: PhysicalAxes,
    writing_direction_mode_: Option<WritingDirectionMode>,
}

impl Default for SnapArea {
    // cpp: layoutng/internal/snap_area.h:31-31
    fn default() -> Self {
        Self {
            element_: Member::default(),
            consumed_axes_: kPhysicalAxesNone,
            pending_axes_: kPhysicalAxesNone,
            writing_direction_mode_: None,
        }
    }
}

#[allow(non_snake_case)]
impl SnapArea {
    // cpp: layoutng/internal/snap_area.h:32-32
    pub fn from_element(element: *mut Element) -> Self {
        Self {
            element_: Member::from_ptr(element),
            ..Self::default()
        }
    }

    // cpp: layoutng/internal/snap_area.h:34-41
    pub fn new(
        element: *mut Element,
        consumed: PhysicalAxes,
        pending: PhysicalAxes,
        writing_direction_mode: Option<WritingDirectionMode>,
    ) -> Self {
        Self {
            element_: Member::from_ptr(element),
            consumed_axes_: consumed,
            pending_axes_: pending,
            writing_direction_mode_: writing_direction_mode,
        }
    }

    // cpp: layoutng/internal/snap_area.h:43-46
    pub fn GetElementIfConsumed(&self) -> *mut Element {
        assert!(self.Resolved());
        if self.consumed_axes_ != kPhysicalAxesNone {
            self.element_.Get()
        } else {
            std::ptr::null_mut()
        }
    }

    // cpp: layoutng/internal/snap_area.h:48-50
    pub fn IsPending(&self) -> bool {
        !self.Resolved() || self.pending_axes_ != kPhysicalAxesNone
    }

    // cpp: layoutng/internal/snap_area.h:52-52
    pub fn GetElement(&self) -> *mut Element {
        self.element_.Get()
    }

    // cpp: layoutng/internal/snap_area.h:54-61
    pub fn ConsumedAxes(&self) -> PhysicalAxes {
        assert!(self.Resolved());
        self.consumed_axes_
    }

    pub fn PendingAxes(&self) -> PhysicalAxes {
        assert!(self.Resolved());
        self.pending_axes_
    }

    // cpp: layoutng/internal/snap_area.h:63-67
    pub fn Resolved(&self) -> bool {
        self.writing_direction_mode_.is_some()
    }

    pub fn ContainerWritingDirectionMode(&self) -> Option<WritingDirectionMode> {
        self.writing_direction_mode_
    }

    // cpp: layoutng/internal/snap_area.h:69-69
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.element_);
    }
}

// cpp: layoutng/internal/snap_area.h:71-71
impl PartialEq for SnapArea {
    fn eq(&self, other: &Self) -> bool {
        self.element_.Get() == other.element_.Get()
            && self.consumed_axes_ == other.consumed_axes_
            && self.pending_axes_ == other.pending_axes_
            && self.writing_direction_mode_ == other.writing_direction_mode_
    }
}

// cpp: layoutng/internal/snap_area.h:80-83
// The memset-clear vector trait awaits the external foundation vector interface.
