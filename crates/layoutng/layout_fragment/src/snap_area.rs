use super::super::layout_node_metadata::Element;
use foundation::{Member, Visitor, WritingDirectionMode};
use layoutng_geometry::geometry::axis::{kPhysicalAxesNone, PhysicalAxes};

#[derive(Clone)]
pub struct SnapArea {
    element_: Member<Element>,
    consumed_axes_: PhysicalAxes,
    pending_axes_: PhysicalAxes,
    writing_direction_mode_: Option<WritingDirectionMode>,
}

impl Default for SnapArea {
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
    pub fn from_element(element: *mut Element) -> Self {
        Self {
            element_: Member::from_ptr(element),
            ..Self::default()
        }
    }

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

    pub fn GetElementIfConsumed(&self) -> *mut Element {
        assert!(self.Resolved());
        if self.consumed_axes_ != kPhysicalAxesNone {
            self.element_.Get()
        } else {
            std::ptr::null_mut()
        }
    }

    pub fn IsPending(&self) -> bool {
        !self.Resolved() || self.pending_axes_ != kPhysicalAxesNone
    }

    pub fn GetElement(&self) -> *mut Element {
        self.element_.Get()
    }

    pub fn ConsumedAxes(&self) -> PhysicalAxes {
        assert!(self.Resolved());
        self.consumed_axes_
    }

    pub fn PendingAxes(&self) -> PhysicalAxes {
        assert!(self.Resolved());
        self.pending_axes_
    }

    pub fn Resolved(&self) -> bool {
        self.writing_direction_mode_.is_some()
    }

    pub fn ContainerWritingDirectionMode(&self) -> Option<WritingDirectionMode> {
        self.writing_direction_mode_
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.element_);
    }
}

impl PartialEq for SnapArea {
    fn eq(&self, other: &Self) -> bool {
        self.element_.Get() == other.element_.Get()
            && self.consumed_axes_ == other.consumed_axes_
            && self.pending_axes_ == other.pending_axes_
            && self.writing_direction_mode_ == other.writing_direction_mode_
    }
}

// The memset-clear vector trait awaits the external foundation vector interface.
