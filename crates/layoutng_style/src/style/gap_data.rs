use foundation::{EBorderStyle, HeapVector, Member, Vector, Visitor, WtfSizeT};

use crate::css::style_color::StyleColor;

// cpp: layoutng_style/style/gap_data.h:17-18
pub trait GapValue: Clone + Default + PartialEq {
    type VectorType: GapRepeatedValues<Self>;
    fn TraceValue(&self, visitor: &mut Visitor<'_>);
}

pub trait GapRepeatedValues<T: GapValue>: Clone + Default + PartialEq {
    fn size(&self) -> WtfSizeT;
    fn at(&self, index: WtfSizeT) -> &T;

    fn TraceValues(&self, visitor: &mut Visitor<'_>) {
        for index in 0..self.size() {
            self.at(index).TraceValue(visitor);
        }
    }
}

impl GapValue for StyleColor {
    type VectorType = HeapVector<StyleColor, 1>;

    fn TraceValue(&self, visitor: &mut Visitor<'_>) {
        StyleColor::Trace(self, visitor);
    }
}

impl GapValue for i32 {
    type VectorType = Vector<i32>;
    fn TraceValue(&self, _visitor: &mut Visitor<'_>) {}
}

impl GapValue for EBorderStyle {
    type VectorType = Vector<EBorderStyle>;
    fn TraceValue(&self, _visitor: &mut Visitor<'_>) {}
}

impl GapRepeatedValues<StyleColor> for HeapVector<StyleColor, 1> {
    fn size(&self) -> WtfSizeT {
        HeapVector::size(self)
    }
    fn at(&self, index: WtfSizeT) -> &StyleColor {
        HeapVector::at(self, index)
    }
}

// The plain C++ Vector is currently a Rust Vec; translate its 32-bit size
// and at() checks at this boundary rather than claiming Vec has those methods.
impl GapRepeatedValues<i32> for Vector<i32> {
    fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.len()).expect("WTF vector size exceeds 32 bits")
    }
    fn at(&self, index: WtfSizeT) -> &i32 {
        &self[index as usize]
    }
}

impl GapRepeatedValues<EBorderStyle> for Vector<EBorderStyle> {
    fn size(&self) -> WtfSizeT {
        WtfSizeT::try_from(self.len()).expect("WTF vector size exceeds 32 bits")
    }
    fn at(&self, index: WtfSizeT) -> &EBorderStyle {
        &self[index as usize]
    }
}

// cpp: layoutng_style/style/gap_data.h:14-46
#[derive(Clone)]
pub struct ValueRepeater<T: GapValue> {
    repeated_values_: T::VectorType,
    repeat_count_: Option<WtfSizeT>,
}

impl<T: GapValue> foundation::Traceable for ValueRepeater<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        ValueRepeater::Trace(self, visitor);
    }
}

#[allow(non_snake_case)]
impl<T: GapValue> ValueRepeater<T> {
    // cpp: layoutng_style/style/gap_data.h:21-25
    pub fn new(repeated_values: T::VectorType, repeat_count: Option<WtfSizeT>) -> Self {
        assert!(repeated_values.size() > 0);
        Self {
            repeated_values_: repeated_values,
            repeat_count_: repeat_count,
        }
    }

    // cpp: layoutng_style/style/gap_data.h:32
    pub fn IsAutoRepeater(&self) -> bool {
        self.repeat_count_.is_none()
    }

    // cpp: layoutng_style/style/gap_data.h:33
    pub fn RepeatedValues(&self) -> &T::VectorType {
        &self.repeated_values_
    }

    // cpp: layoutng_style/style/gap_data.h:34-37
    pub fn RepeatCount(&self) -> WtfSizeT {
        self.repeat_count_.expect("repeat count required")
    }

    // cpp: layoutng_style/style/gap_data.h:39-41
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.repeated_values_.TraceValues(visitor);
    }
}

// cpp: layoutng_style/style/gap_data.h:19
impl<T: GapValue> Default for ValueRepeater<T> {
    fn default() -> Self {
        Self {
            repeated_values_: Default::default(),
            repeat_count_: None,
        }
    }
}

// cpp: layoutng_style/style/gap_data.h:27-30
impl<T: GapValue> PartialEq for ValueRepeater<T> {
    fn eq(&self, other: &Self) -> bool {
        self.repeated_values_ == other.repeated_values_ && self.repeat_count_ == other.repeat_count_
    }
}

// cpp: layoutng_style/style/gap_data.h:48-92
#[derive(Clone)]
pub struct GapData<T: GapValue> {
    value_: T,
    value_repeater_: Member<ValueRepeater<T>>,
}

impl<T: GapValue> foundation::Traceable for GapData<T> {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        GapData::Trace(self, visitor);
    }
}

#[allow(non_snake_case)]
impl<T: GapValue> GapData<T> {
    // cpp: layoutng_style/style/gap_data.h:55
    pub fn from_value(value: T) -> Self {
        Self {
            value_: value,
            value_repeater_: Member::default(),
        }
    }

    // cpp: layoutng_style/style/gap_data.h:56-57
    pub fn from_repeater(value_repeater: *mut ValueRepeater<T>) -> Self {
        Self {
            value_: T::default(),
            value_repeater_: Member::from_ptr(value_repeater),
        }
    }

    // cpp: layoutng_style/style/gap_data.h:58-61
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.value_.TraceValue(visitor);
        visitor.Trace(&self.value_repeater_);
    }

    // cpp: layoutng_style/style/gap_data.h:68-71
    pub fn GetValue(&self) -> T {
        assert!(self.value_repeater_.Get().is_null());
        self.value_.clone()
    }

    // cpp: layoutng_style/style/gap_data.h:73-76
    pub fn GetValueRepeater(&self) -> &ValueRepeater<T> {
        let ptr = self.value_repeater_.Get();
        assert!(!ptr.is_null());
        unsafe { &*ptr }
    }

    // cpp: layoutng_style/style/gap_data.h:78
    pub fn IsRepeaterData(&self) -> bool {
        !self.value_repeater_.Get().is_null()
    }

    // cpp: layoutng_style/style/gap_data.h:80-87
    pub fn GetFixedSlotCount(&self) -> WtfSizeT {
        if !self.IsRepeaterData() {
            return 1;
        }
        assert!(!self.GetValueRepeater().IsAutoRepeater());
        self.GetValueRepeater().RepeatCount() * self.GetValueRepeater().RepeatedValues().size()
    }
}

// cpp: layoutng_style/style/gap_data.h:54
impl<T: GapValue> Default for GapData<T> {
    fn default() -> Self {
        Self {
            value_: T::default(),
            value_repeater_: Member::default(),
        }
    }
}

// cpp: layoutng_style/style/gap_data.h:63-66
impl<T: GapValue> PartialEq for GapData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value_ == other.value_
            && foundation::ValuesEquivalent(&self.value_repeater_, &other.value_repeater_)
    }
}

// cpp: layoutng_style/style/gap_data.h:96-100
pub trait GapDataClearUnusedSlots: GapValue {
    const kCanClearUnusedSlotsWithMemFunctions: bool;
}

#[allow(non_upper_case_globals)]
impl GapDataClearUnusedSlots for StyleColor {
    const kCanClearUnusedSlotsWithMemFunctions: bool = true;
}

#[allow(non_upper_case_globals)]
impl GapDataClearUnusedSlots for i32 {
    const kCanClearUnusedSlotsWithMemFunctions: bool = true;
}

#[allow(non_upper_case_globals)]
impl GapDataClearUnusedSlots for EBorderStyle {
    const kCanClearUnusedSlotsWithMemFunctions: bool = true;
}
