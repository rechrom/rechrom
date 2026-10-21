// C++: layoutng_inline/leading_floats.h.
#![allow(non_snake_case)]

use foundation::{HeapVector, Member, Traceable, Visitor, WtfSizeT};
use layoutng::internal::positioned_float::PositionedFloat;
use layoutng_fragment_tree::inline_break_token::InlineBreakToken;

// cpp: layoutng_inline/leading_floats.h:17-33
pub struct LeadingFloat {
    pub positioned_float: PositionedFloat,
    pub parallel_flow_break_token: Member<InlineBreakToken>,
}

impl LeadingFloat {
    // cpp: layoutng_inline/leading_floats.h:21-24
    pub fn new(
        positioned_float: &PositionedFloat,
        parallel_flow_break_token: *const InlineBreakToken,
    ) -> Self {
        Self {
            positioned_float: positioned_float.clone(),
            parallel_flow_break_token: Member::from_ptr(parallel_flow_break_token.cast_mut()),
        }
    }

    // cpp: layoutng_inline/leading_floats.h:29-32
    pub fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.positioned_float.Trace(visitor);
        visitor.Trace(&self.parallel_flow_break_token);
    }
}

impl Traceable for LeadingFloat {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        LeadingFloat::Trace(self, visitor);
    }
}

// cpp: layoutng_inline/leading_floats.h:35-57
pub struct LeadingFloats {
    floats_: HeapVector<LeadingFloat>,
    handled_index_: WtfSizeT,
}

impl Default for LeadingFloats {
    fn default() -> Self {
        Self {
            floats_: HeapVector::new(),
            handled_index_: 0,
        }
    }
}

#[allow(non_snake_case)]
impl LeadingFloats {
    // cpp: layoutng_inline/leading_floats.h:43-46
    pub fn Add(
        &mut self,
        positioned_float: &PositionedFloat,
        parallel_flow_break_token: *const InlineBreakToken,
    ) {
        self.floats_.push(LeadingFloat::new(
            positioned_float,
            parallel_flow_break_token,
        ));
    }

    // cpp: layoutng_inline/leading_floats.h:47-52
    pub fn SetHandledIndex(&mut self, index: WtfSizeT) {
        self.handled_index_ = index;
    }
    pub fn Empty(&self) -> bool {
        self.floats_.is_empty()
    }
    pub fn Count(&self) -> WtfSizeT {
        self.floats_.size()
    }
    pub fn At(&self, index: WtfSizeT) -> &LeadingFloat {
        &self.floats_[index as usize]
    }
    pub fn HandledIndex(&self) -> WtfSizeT {
        self.handled_index_
    }
}

// cpp: layoutng_inline/leading_floats.h:41-41
impl Drop for LeadingFloats {
    fn drop(&mut self) {
        self.floats_.clear();
    }
}

// cpp: layoutng_inline/leading_floats.h:61-61
// Vec drops and clears its live elements; unused capacity holds no initialized
// LeadingFloat values, replacing the C++ clear-unused-slots macro.
