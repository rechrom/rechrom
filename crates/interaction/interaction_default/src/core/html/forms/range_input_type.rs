/*
 * Copyright (C) 2010 Google Inc. All rights reserved.
 * Copyright (C) 2011 Apple Inc. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met:
 *
 *     * Redistributions of source code must retain the above copyright
 * notice, this list of conditions and the following disclaimer.
 *     * Redistributions in binary form must reproduce the above
 * copyright notice, this list of conditions and the following disclaimer
 * in the documentation and/or other materials provided with the
 * distribution.
 *     * Neither the name of Google Inc. nor the names of its
 * contributors may be used to endorse or promote products derived from
 * this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/html/forms/range_input_type.h:1-107
// Source: third_party/blink/renderer/core/html/forms/range_input_type.cc:1-498

use super::input_type::{TextFieldEventBehavior, ValueMode};
use super::step_range::{
    AnyStepHandling, Decimal, NumberTypeParser, StepDescription, StepRange, StepValueShouldBe,
};
use interaction_dispatch::core::dom::events::event::EventRef;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

// Borrowed source identities; the element/shadow owner retains the same thumb,
// track, datalist and style. These ports contain no copied DOM/layout tree.
pub trait RangeThumb {
    fn DragFrom(&self, x: i32, y: i32);
    fn SetPositionFromValue(&self);
    fn StopDragging(&self, allow_event_dispatch: bool);
    fn IsActive(&self) -> bool;
}
pub trait RangeTrack {}
pub trait RangeDataList {
    fn Len(&self) -> usize;
    fn OptionValue(&self, index: usize) -> String;
    fn OptionDisabled(&self, index: usize) -> bool;
}
#[derive(Clone, Copy)]
pub struct RangeWritingDirection {
    // Source cc:235-239 PhysicalToLogical mapping, supplied by canonical style.
    pub inline_end: &'static str,
    pub line_over: &'static str,
    pub inline_start: &'static str,
    pub line_under: &'static str,
    pub horizontal: bool,
    pub slider_vertical_appearance: bool,
}
pub enum RangeShadowMutation {
    CreateTrackThumbContainer,
}
pub enum RangeLayoutObject {
    FlexibleBox,
}
pub enum RangeAppearance {
    SliderHorizontal,
}
pub enum RangeInvalidation {
    TrackLayoutAndSubtreePaint,
}
pub fn InvalidRangeValueWarning(value: &[u16]) -> Vec<u16> {
    "The specified value "
        .encode_utf16()
        .chain(crate::platform::json_values::QuoteString(value))
        .chain(" cannot be parsed, or is out of range.".encode_utf16())
        .collect()
}
pub enum RangeValidationMessage {
    NotEqual,
    Overflow,
    Underflow,
    Reversed,
}
/// Source primitives consumed by range keyboard stepping. Geometry/shadow
/// services used only by pointer dragging need not cross this boundary.
pub trait RangeKeyboardOperations<D: Decimal>: NumberTypeParser<D> {
    fn Value(&self) -> String;
    fn Attribute(&self, name: &'static str) -> String;
    fn FindStepBase(&self, default_base: D) -> D;
    fn SetValueAsDecimal(&self, value: D, behavior: TextFieldEventBehavior);
    fn IsDisabledFormControl(&self) -> bool;
    fn WritingDirection(&self) -> Option<RangeWritingDirection>;
    fn NonStandardSliderVerticalEnabled(&self) -> bool;
    fn BeginEventQueueScope(&self) -> Box<dyn EventQueueScope + '_>;
    fn NotifyAccessibilityValueChanged(&self);
}
pub trait RangeControl<D: Decimal>: RangeKeyboardOperations<D> {
    // Source: range_input_type.cc:135-139 -> Decimal::FromDouble. The required
    // number-to-string leaf belongs to the assembled element/conversion owner.
    fn FromDouble(&self, value: f64) -> D;
    fn Thumb(&self) -> Rc<dyn RangeThumb>;
    fn Track(&self) -> Option<Rc<dyn RangeTrack>>;
    fn TargetIsHostOrShadowDescendant(&self, target: &EventRef) -> bool;
    fn TargetIsThumb(&self, target: &EventRef) -> bool;
    fn EnsureUserAgentShadowRoot(&self);
    fn ShadowMutation(&self, mutation: RangeShadowMutation);
    fn HasDirtyValue(&self) -> bool;
    fn SetValue(&self, value: &str);
    fn SetNonDirtyValue(&self, value: &str);
    fn UpdateView(&self);
    fn WarnInvalidValue(&self, value: &str);
    fn ValidationMessage(
        &self,
        kind: RangeValidationMessage,
        first: &str,
        second: Option<&str>,
    ) -> String;
    fn LocalizeValue(&self, value: &str) -> String;
    fn DisableChangeDuringMutationEnabled(&self) -> bool;
    fn StatePreservingAtomicMoveInProgress(&self) -> bool;
    fn Invalidate(&self, invalidation: RangeInvalidation);
    fn DataList(&self) -> Option<Rc<dyn RangeDataList>>;
    fn IsValidValue(&self, value: &str) -> bool;
}
pub use super::input_type::EventQueueScope;
#[derive(Clone, Copy)]
pub enum DisabledChangedReason {
    FieldsetChildrenChanged,
    Other,
}

// Source: range_input_type.h:43-95; cc:71-98. Rc/Drop retains the source's two
// base subobjects' same element; only tick values belong to RangeInputType.
pub struct RangeInputType<D: Decimal> {
    element: Rc<dyn RangeControl<D>>,
    has_created_shadow_subtree_: Cell<bool>,
    tick_mark_values_dirty_: Cell<bool>,
    tick_mark_values_: RefCell<Vec<D>>,
}
impl<D: Decimal> RangeInputType<D> {
    pub fn new(element: Rc<dyn RangeControl<D>>) -> Self {
        Self {
            element,
            has_created_shadow_subtree_: Cell::new(false),
            tick_mark_values_dirty_: Cell::new(true),
            tick_mark_values_: RefCell::new(Vec::new()),
        }
    }
    pub fn CreateView(&self) -> &Self {
        self
    }
    pub fn GetValueMode(&self) -> ValueMode {
        ValueMode::kValue
    }
    // Source: cc:100-129. TODO: CountUsage and writing-direction UseCounters.
    pub fn CountUsage(&self) {}
    pub fn DidRecalcStyle(&self) {}
    // Source: cc:131-147.
    pub fn ValueAsDouble(&self) -> f64 {
        self.element
            .ParseToDoubleForNumberType(&self.element.Value())
    }
    pub fn SetValueAsDouble(&self, value: f64, behavior: TextFieldEventBehavior) {
        self.element
            .SetValueAsDecimal(self.element.FromDouble(value), behavior);
    }
    pub fn TypeMismatchFor(&self, value: &str) -> bool {
        !value.is_empty() && !self.element.ParseToDoubleForNumberType(value).is_finite()
    }
    // Source: InputType::ValidateInputType (input_type.cc:205-213),
    // StepMismatch/RangeUnderflow/RangeOverflow:449-486,554-563. Range has
    // no reversed interval and inherits the false pattern/value-missing checks.
    pub fn IsValidValue(&self, value: &str) -> bool {
        if self.TypeMismatchFor(value) {
            return false;
        }
        let number = self.ParseToNumber(value, D::Nan());
        if !number.IsFinite() {
            return true;
        }
        let range = self.CreateStepRange(AnyStepHandling::kRejectAny);
        !range.StepMismatch(number) && number >= range.Minimum() && number <= range.Maximum()
    }
    pub fn SupportsRequired(&self) -> bool {
        false
    }
    // Source: cc:149-174.
    pub fn CreateStepRange(&self, handling: AnyStepHandling) -> StepRange<D> {
        Self::CreateStepRangeWithOperations(self.element.as_ref(), handling)
    }
    pub fn CreateStepRangeWithOperations(
        element: &dyn RangeKeyboardOperations<D>,
        handling: AnyStepHandling,
    ) -> StepRange<D> {
        let description = StepDescription::new(1, 0, 1, StepValueShouldBe::kStepValueShouldBeReal);
        let base = element.FindStepBase(D::FromInt(0));
        let minimum = element.ParseToDecimalForNumberType(&element.Attribute("min"), D::FromInt(0));
        let proposed =
            element.ParseToDecimalForNumberType(&element.Attribute("max"), D::FromInt(100));
        let maximum = if proposed >= minimum {
            proposed
        } else {
            minimum
        };
        let step = StepRange::ParseStep(handling, description, &element.Attribute("step"), element);
        StepRange::new(base, minimum, maximum, true, true, false, step, description)
    }
    // Source: cc:176-197. Scalar mouse data is captured before EventRef borrow
    // is released; native node identity tests use the same retained target.
    pub fn HandleMouseDownEvent(
        &self,
        event: &EventRef,
        button: i16,
        has_node: bool,
        x: f64,
        y: f64,
    ) {
        if !self.HasCreatedShadowSubtree() || self.element.IsDisabledFormControl() {
            return;
        }
        if button != 0 || !has_node {
            return;
        }
        if !self.element.TargetIsHostOrShadowDescendant(event) || self.element.TargetIsThumb(event)
        {
            return;
        }
        self.element
            .Thumb()
            .DragFrom(x.floor() as i32, y.floor() as i32);
    }
    // Source: cc:199-272.
    pub fn HandleKeydownEvent(&self, event: &EventRef, key: &str) {
        if Self::HandleKeydownWithOperations(self.element.as_ref(), key) {
            event.borrow_mut().BaseEventMut().SetDefaultHandled();
        }
    }
    pub fn HandleKeydownWithOperations(
        element: &dyn RangeKeyboardOperations<D>,
        key: &str,
    ) -> bool {
        if element.IsDisabledFormControl() {
            return false;
        }
        let current = element.ParseToDecimalForNumberType(&element.Value(), D::Nan());
        debug_assert!(current.IsFinite());
        let range = Self::CreateStepRangeWithOperations(element, AnyStepHandling::kRejectAny);
        let step = if element.Attribute("step").eq_ignore_ascii_case("any") {
            (range.Maximum() - range.Minimum()) / D::FromInt(100)
        } else {
            range.Step()
        };
        let candidate = (range.Maximum() - range.Minimum()) / D::FromInt(10);
        let big_step = if candidate < step { step } else { candidate };
        let mut direction = element.WritingDirection().unwrap_or(RangeWritingDirection {
            inline_end: "ArrowRight",
            line_over: "ArrowUp",
            inline_start: "ArrowLeft",
            line_under: "ArrowDown",
            horizontal: true,
            slider_vertical_appearance: false,
        });
        if element.NonStandardSliderVerticalEnabled()
            && direction.horizontal
            && direction.slider_vertical_appearance
        {
            direction = RangeWritingDirection {
                inline_end: "ArrowUp",
                line_over: "ArrowRight",
                inline_start: "ArrowDown",
                line_under: "ArrowLeft",
                horizontal: false,
                slider_vertical_appearance: true,
            };
        }
        let new_value = if key == direction.inline_end || key == direction.line_over {
            current + step
        } else if key == direction.inline_start || key == direction.line_under {
            current - step
        } else {
            match key {
                "PageUp" => current + big_step,
                "PageDown" => current - big_step,
                "Home" => range.Minimum(),
                "End" => range.Maximum(),
                _ => return false,
            }
        };
        let new_value = range.ClampValue(new_value);
        if new_value != current {
            let _scope = element.BeginEventQueueScope();
            element.SetValueAsDecimal(
                new_value,
                TextFieldEventBehavior::kDispatchInputAndChangeEvent,
            );
            element.NotifyAccessibilityValueChanged();
        }
        true
    }
    // Source: cc:274-299. Typed DOM/style/layout allocation outputs are routed
    // at these source points by the owning adapter.
    pub fn HasCreatedShadowSubtree(&self) -> bool {
        self.has_created_shadow_subtree_.get()
    }
    pub fn CreateShadowSubtree(&self) {
        // InputTypeView::CreateShadowSubtreeIfNeeded, cc:144-150: commit
        // creation on this retained view before synchronous DOM child effects.
        if self.has_created_shadow_subtree_.get() {
            return;
        }
        self.element.EnsureUserAgentShadowRoot();
        self.has_created_shadow_subtree_.set(true);
        self.element
            .ShadowMutation(RangeShadowMutation::CreateTrackThumbContainer);
    }
    pub fn CreateLayoutObject(&self) -> RangeLayoutObject {
        RangeLayoutObject::FlexibleBox
    }
    pub fn AdjustStyle(
        &self,
        builder: &mut layoutng_style::style::computed_style::ComputedStyleBuilder,
    ) {
        layoutng_style::style::range_input_style::AdjustRangeInputStyle(builder);
    }
    // Source: cc:301-310.
    pub fn ParseToNumber(&self, value: &str, default: D) -> D {
        self.element.ParseToDecimalForNumberType(value, default)
    }
    pub fn Serialize(&self, value: D) -> String {
        if !value.IsFinite() {
            String::new()
        } else {
            self.element.SerializeForNumberType(value)
        }
    }
    // Source: cc:312-341.
    pub fn SanitizeValueInResponseToMinOrMaxAttributeChange(&self) {
        self.ResanitizeValue();
    }
    pub fn StepAttributeChanged(&self) {
        self.ResanitizeValue();
    }
    fn ResanitizeValue(&self) {
        let value = self.element.Value();
        if self.element.HasDirtyValue() {
            self.element.SetValue(&value);
        } else {
            self.element.SetNonDirtyValue(&value);
        }
        self.element.UpdateView();
    }
    pub fn DidSetValue(&self, _old: &str, changed: bool) {
        if changed {
            self.element.UpdateView();
        }
    }
    pub fn AutoAppearance(&self) -> RangeAppearance {
        RangeAppearance::SliderHorizontal
    }
    pub fn UpdateView(&self) {
        if self.HasCreatedShadowSubtree() {
            self.element.Thumb().SetPositionFromValue();
        }
    }
    // Source: cc:343-355.
    pub fn SanitizeValue(&self, proposed: &str) -> String {
        let range = self.CreateStepRange(AnyStepHandling::kRejectAny);
        self.element.SerializeForNumberType(
            range.ClampValue(self.ParseToNumber(proposed, range.DefaultValue())),
        )
    }
    pub fn WarnIfValueIsInvalid(&self, value: &str) {
        if value.is_empty() || !self.SanitizeValue(value).is_empty() {
            return;
        }
        self.element.WarnInvalidValue(value);
    }
    // Source: cc:357-377.
    pub fn ValueNotEqualText(&self, value: D) -> String {
        self.ValidationMessage(RangeValidationMessage::NotEqual, value, None)
    }
    pub(super) fn LocalizedValidationValue(&self, value: D) -> String {
        self.element.LocalizeValue(&self.Serialize(value))
    }
    pub fn RangeOverflowText(&self, value: D) -> String {
        self.ValidationMessage(RangeValidationMessage::Overflow, value, None)
    }
    pub fn RangeUnderflowText(&self, value: D) -> String {
        self.ValidationMessage(RangeValidationMessage::Underflow, value, None)
    }
    pub fn RangeInvalidText(&self, minimum: D, maximum: D) -> String {
        self.ValidationMessage(RangeValidationMessage::Reversed, minimum, Some(maximum))
    }
    fn ValidationMessage(
        &self,
        kind: RangeValidationMessage,
        first: D,
        second: Option<D>,
    ) -> String {
        let first = self.element.LocalizeValue(&self.Serialize(first));
        let second = second.map(|value| self.element.LocalizeValue(&self.Serialize(value)));
        self.element
            .ValidationMessage(kind, &first, second.as_deref())
    }
    // Source: cc:379-424.
    pub fn DisabledAttributeChanged(&self, reason: DisabledChangedReason) {
        if !self.HasCreatedShadowSubtree() {
            return;
        }
        if self.element.IsDisabledFormControl() {
            let avoid = self.element.DisableChangeDuringMutationEnabled()
                && (self.element.StatePreservingAtomicMoveInProgress()
                    || matches!(reason, DisabledChangedReason::FieldsetChildrenChanged));
            self.element.Thumb().StopDragging(!avoid);
        }
    }
    pub fn ShouldRespectListAttribute(&self) -> bool {
        true
    }
    pub fn GetSliderThumbElement(&self) -> Rc<dyn RangeThumb> {
        self.element.Thumb()
    }
    pub fn SliderTrackElement(&self) -> Option<Rc<dyn RangeTrack>> {
        if !self.HasCreatedShadowSubtree() {
            return None;
        }
        self.element.Track()
    }
    pub fn ListAttributeTargetChanged(&self) {
        self.tick_mark_values_dirty_.set(true);
        self.element
            .Invalidate(RangeInvalidation::TrackLayoutAndSubtreePaint);
    }
    // Source: cc:426-451. Retain and clear the source vector's existing owner
    // and capacity. The datalist accessors are script-free DOM reads.
    pub fn UpdateTickMarkValues(&self) {
        if !self.tick_mark_values_dirty_.replace(false) {
            return;
        }
        let mut values = self.tick_mark_values_.borrow_mut();
        values.clear();
        let Some(list) = self.element.DataList() else {
            return;
        };
        values.reserve(list.Len());
        for i in 0..list.Len() {
            let value = list.OptionValue(i);
            if list.OptionDisabled(i) || value.is_empty() || !self.element.IsValidValue(&value) {
                continue;
            }
            values.push(self.ParseToNumber(&value, D::Nan()));
        }
        values.shrink_to_fit();
        values.sort_by(|a, b| a.partial_cmp(b).expect("valid range ticks are finite"));
    }
    // Source: cc:453-488.
    pub fn FindClosestTickMarkValue(&self, value: D) -> D {
        self.UpdateTickMarkValues();
        let ticks = self.tick_mark_values_.borrow();
        if ticks.is_empty() {
            return D::Nan();
        }
        let mut left = 0;
        let mut right = ticks.len();
        let mut middle;
        loop {
            debug_assert!(left <= right);
            middle = left + (right - left) / 2;
            if middle == 0 {
                break;
            }
            if middle == ticks.len() - 1 && ticks[middle] < value {
                middle += 1;
                break;
            }
            if ticks[middle - 1] <= value && ticks[middle] >= value {
                break;
            }
            if ticks[middle] < value {
                left = middle;
            } else {
                right = middle;
            }
        }
        let left = if middle != 0 {
            ticks[middle - 1]
        } else {
            D::Infinity(true)
        };
        let right = if middle != ticks.len() {
            ticks[middle]
        } else {
            D::Infinity(false)
        };
        if right - value < value - left {
            right
        } else {
            left
        }
    }
    // Source: cc:490-496.
    pub fn ValueAttributeChanged(&self) {
        self.UpdateView();
    }
    pub fn IsDraggedSlider(&self) -> bool {
        self.element.Thumb().IsActive()
    }
}
