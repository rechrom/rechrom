// C++: src/foundation/text/native/writing_mode_utils.h

use crate::blink_geometry::geometry::PhysicalDirection;

use super::writing_direction_mode::WritingDirectionMode;
use super::writing_mode::WritingMode;

// C++ template class PhysicalToLogical<Value> -> Rust struct + impl.
#[derive(Clone, Copy)]
pub struct PhysicalToLogical<Value> {
    writing_direction_: WritingDirectionMode,
    top_: Value,
    right_: Value,
    bottom_: Value,
    left_: Value,
}

impl<Value: Copy> PhysicalToLogical<Value> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        top: Value,
        right: Value,
        bottom: Value,
        left: Value,
    ) -> Self {
        Self {
            writing_direction_: writing_direction,
            top_: top,
            right_: right,
            bottom_: bottom,
            left_: left,
        }
    }

    pub fn InlineStart(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return if self.writing_direction_.IsLtr() {
                self.left_
            } else {
                self.right_
            };
        }
        self.ValueFor(self.writing_direction_.InlineStart())
    }

    pub fn InlineEnd(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return if self.writing_direction_.IsLtr() {
                self.right_
            } else {
                self.left_
            };
        }
        self.ValueFor(self.writing_direction_.InlineEnd())
    }

    pub fn BlockStart(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return self.top_;
        }
        self.ValueFor(self.writing_direction_.BlockStart())
    }

    pub fn BlockEnd(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return self.bottom_;
        }
        self.ValueFor(self.writing_direction_.BlockEnd())
    }

    pub fn LineOver(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            self.top_
        } else {
            self.ValueFor(self.writing_direction_.LineOver())
        }
    }

    pub fn LineUnder(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            self.bottom_
        } else {
            self.ValueFor(self.writing_direction_.LineUnder())
        }
    }

    fn ValueFor(&self, direction: PhysicalDirection) -> Value {
        match direction {
            PhysicalDirection::kUp => self.top_,
            PhysicalDirection::kRight => self.right_,
            PhysicalDirection::kDown => self.bottom_,
            PhysicalDirection::kLeft => self.left_,
        }
    }
}

// C++ template class LogicalToPhysical<Value> -> Rust struct + impl.
#[derive(Clone, Copy)]
pub struct LogicalToPhysical<Value> {
    writing_direction_: WritingDirectionMode,
    inline_start_: Value,
    inline_end_: Value,
    block_start_: Value,
    block_end_: Value,
}

impl<Value: Copy> LogicalToPhysical<Value> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        inline_start: Value,
        inline_end: Value,
        block_start: Value,
        block_end: Value,
    ) -> Self {
        Self {
            writing_direction_: writing_direction,
            inline_start_: inline_start,
            inline_end_: inline_end,
            block_start_: block_start,
            block_end_: block_end,
        }
    }

    pub fn Left(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return if self.writing_direction_.IsLtr() {
                self.inline_start_
            } else {
                self.inline_end_
            };
        }
        if self.writing_direction_.IsFlippedBlocks() {
            self.block_end_
        } else {
            self.block_start_
        }
    }

    pub fn Right(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return if self.writing_direction_.IsLtr() {
                self.inline_end_
            } else {
                self.inline_start_
            };
        }
        if self.writing_direction_.IsFlippedBlocks() {
            self.block_start_
        } else {
            self.block_end_
        }
    }

    pub fn Top(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return self.block_start_;
        }
        if self.writing_direction_.GetWritingMode() == WritingMode::kSidewaysLr {
            return if self.writing_direction_.IsLtr() {
                self.inline_end_
            } else {
                self.inline_start_
            };
        }
        if self.writing_direction_.IsLtr() {
            self.inline_start_
        } else {
            self.inline_end_
        }
    }

    pub fn Bottom(&self) -> Value {
        if self.writing_direction_.IsHorizontal() {
            return self.block_end_;
        }
        if self.writing_direction_.GetWritingMode() == WritingMode::kSidewaysLr {
            return if self.writing_direction_.IsLtr() {
                self.inline_start_
            } else {
                self.inline_end_
            };
        }
        if self.writing_direction_.IsLtr() {
            self.inline_end_
        } else {
            self.inline_start_
        }
    }
}

// C++ template class LogicalToLogical<Value> -> Rust struct + impl.
#[derive(Clone, Copy)]
pub struct LogicalToLogical<Value> {
    logical_: PhysicalToLogical<Value>,
}

impl<Value: Copy> LogicalToLogical<Value> {
    pub fn new(
        from_writing_direction: WritingDirectionMode,
        to_writing_direction: WritingDirectionMode,
        inline_start: Value,
        inline_end: Value,
        block_start: Value,
        block_end: Value,
    ) -> Self {
        Self::from_physical(
            to_writing_direction,
            LogicalToPhysical::new(
                from_writing_direction,
                inline_start,
                inline_end,
                block_start,
                block_end,
            ),
        )
    }

    pub fn InlineStart(&self) -> Value {
        self.logical_.InlineStart()
    }

    pub fn BlockStart(&self) -> Value {
        self.logical_.BlockStart()
    }

    pub fn InlineEnd(&self) -> Value {
        self.logical_.InlineEnd()
    }

    pub fn BlockEnd(&self) -> Value {
        self.logical_.BlockEnd()
    }

    fn from_physical(
        to_writing_direction: WritingDirectionMode,
        physical: LogicalToPhysical<Value>,
    ) -> Self {
        Self {
            logical_: PhysicalToLogical::new(
                to_writing_direction,
                physical.Top(),
                physical.Right(),
                physical.Bottom(),
                physical.Left(),
            ),
        }
    }
}

// C++ template class LogicalToPhysicalGetter<Value, Object> -> Rust struct + impl.
pub struct LogicalToPhysicalGetter<'a, Value, Object> {
    object_: &'a Object,
    converter_: LogicalToPhysical<fn(&Object) -> Value>,
}

impl<'a, Value, Object> LogicalToPhysicalGetter<'a, Value, Object> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        object: &'a Object,
        inline_start_getter: fn(&Object) -> Value,
        inline_end_getter: fn(&Object) -> Value,
        block_start_getter: fn(&Object) -> Value,
        block_end_getter: fn(&Object) -> Value,
    ) -> Self {
        Self {
            object_: object,
            converter_: LogicalToPhysical::new(
                writing_direction,
                inline_start_getter,
                inline_end_getter,
                block_start_getter,
                block_end_getter,
            ),
        }
    }

    pub fn Left(&self) -> Value {
        (self.converter_.Left())(self.object_)
    }

    pub fn Right(&self) -> Value {
        (self.converter_.Right())(self.object_)
    }

    pub fn Top(&self) -> Value {
        (self.converter_.Top())(self.object_)
    }

    pub fn Bottom(&self) -> Value {
        (self.converter_.Bottom())(self.object_)
    }
}

// C++ template class PhysicalToLogicalGetter<Value, Object> -> Rust struct + impl.
pub struct PhysicalToLogicalGetter<'a, Value, Object> {
    object_: &'a Object,
    converter_: PhysicalToLogical<fn(&Object) -> Value>,
}

impl<'a, Value, Object> PhysicalToLogicalGetter<'a, Value, Object> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        object: &'a Object,
        top_getter: fn(&Object) -> Value,
        right_getter: fn(&Object) -> Value,
        bottom_getter: fn(&Object) -> Value,
        left_getter: fn(&Object) -> Value,
    ) -> Self {
        Self {
            object_: object,
            converter_: PhysicalToLogical::new(
                writing_direction,
                top_getter,
                right_getter,
                bottom_getter,
                left_getter,
            ),
        }
    }

    pub fn InlineStart(&self) -> Value {
        (self.converter_.InlineStart())(self.object_)
    }

    pub fn InlineEnd(&self) -> Value {
        (self.converter_.InlineEnd())(self.object_)
    }

    pub fn BlockStart(&self) -> Value {
        (self.converter_.BlockStart())(self.object_)
    }

    pub fn BlockEnd(&self) -> Value {
        (self.converter_.BlockEnd())(self.object_)
    }
}

// C++ template class LogicalToPhysicalSetter<Value, Object> -> Rust struct + impl.
pub struct LogicalToPhysicalSetter<'a, Value, Object> {
    object_: &'a mut Object,
    converter_: PhysicalToLogical<fn(&mut Object, Value)>,
}

impl<'a, Value, Object> LogicalToPhysicalSetter<'a, Value, Object> {
    pub fn new(
        writing_direction: WritingDirectionMode,
        object: &'a mut Object,
        top_setter: fn(&mut Object, Value),
        right_setter: fn(&mut Object, Value),
        bottom_setter: fn(&mut Object, Value),
        left_setter: fn(&mut Object, Value),
    ) -> Self {
        Self {
            object_: object,
            converter_: PhysicalToLogical::new(
                writing_direction,
                top_setter,
                right_setter,
                bottom_setter,
                left_setter,
            ),
        }
    }

    pub fn SetInlineStart(&mut self, value: Value) {
        (self.converter_.InlineStart())(self.object_, value);
    }

    pub fn SetInlineEnd(&mut self, value: Value) {
        (self.converter_.InlineEnd())(self.object_, value);
    }

    pub fn SetBlockStart(&mut self, value: Value) {
        (self.converter_.BlockStart())(self.object_, value);
    }

    pub fn SetBlockEnd(&mut self, value: Value) {
        (self.converter_.BlockEnd())(self.object_, value);
    }
}
