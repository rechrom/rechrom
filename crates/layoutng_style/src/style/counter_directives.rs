use std::ops::{Deref, DerefMut};

use foundation::{AtomicString, HashMap, Vector};

// cpp: layoutng_style/style/counter_directives.h:45-134
#[derive(Clone, Default, PartialEq, Eq)]
pub struct CounterDirectives {
    reset_value_: Option<i64>,
    increment_value_: Option<i32>,
    set_value_: Option<i32>,
    is_reset_reversed_: bool,
}

#[allow(non_snake_case)]
impl CounterDirectives {
    // cpp: layoutng_style/style/counter_directives.h:58-60
    pub fn IsReset(&self) -> bool {
        self.reset_value_.is_some() || self.is_reset_reversed_
    }

    // cpp: layoutng_style/style/counter_directives.h:64-67
    pub fn ResetValue(&self) -> Option<i32> {
        self.reset_value_.map(saturated_i32)
    }

    // cpp: layoutng_style/style/counter_directives.h:68-70
    pub fn ResetValueInt64(&self) -> Option<i64> {
        self.reset_value_
    }
    pub fn SetResetValue(&mut self, value: i64) {
        self.reset_value_ = Some(value);
    }
    pub fn IsResetReversed(&self) -> bool {
        self.is_reset_reversed_
    }

    // cpp: layoutng_style/style/counter_directives.h:71-73
    pub fn IsContentBasedReset(&self) -> bool {
        self.is_reset_reversed_ && self.reset_value_.is_none()
    }

    // cpp: layoutng_style/style/counter_directives.h:74
    pub fn SetIsResetReversed(&mut self) {
        self.is_reset_reversed_ = true;
    }

    // cpp: layoutng_style/style/counter_directives.h:75-78
    pub fn ClearReset(&mut self) {
        self.reset_value_ = None;
        self.is_reset_reversed_ = false;
    }

    // cpp: layoutng_style/style/counter_directives.h:79-82
    pub fn InheritReset(&mut self, parent: &CounterDirectives) {
        self.reset_value_ = parent.reset_value_;
        self.is_reset_reversed_ = parent.is_reset_reversed_;
    }

    // cpp: layoutng_style/style/counter_directives.h:86
    pub fn HasIncrement(&self) -> bool {
        self.increment_value_.is_some()
    }

    // cpp: layoutng_style/style/counter_directives.h:91
    pub fn IncrementValue(&self) -> i32 {
        self.increment_value_.expect("counter increment is absent")
    }

    // cpp: layoutng_style/style/counter_directives.h:92-94
    pub fn AddIncrementValue(&mut self, value: i32) {
        self.increment_value_ = Some(self.increment_value_.unwrap_or(0).saturating_add(value));
    }

    // cpp: layoutng_style/style/counter_directives.h:95
    pub fn ClearIncrement(&mut self) {
        self.increment_value_ = None;
    }

    // cpp: layoutng_style/style/counter_directives.h:96-98
    pub fn InheritIncrement(&mut self, parent: &CounterDirectives) {
        self.increment_value_ = parent.increment_value_;
    }

    // cpp: layoutng_style/style/counter_directives.h:100
    pub fn HasSet(&self) -> bool {
        self.set_value_.is_some()
    }

    // cpp: layoutng_style/style/counter_directives.h:104-106
    pub fn SetValue(&self) -> i32 {
        self.set_value_.expect("counter set is absent")
    }
    pub fn SetSetValue(&mut self, value: i32) {
        self.set_value_ = Some(value);
    }
    pub fn ClearSet(&mut self) {
        self.set_value_ = None;
    }

    // cpp: layoutng_style/style/counter_directives.h:107-109
    pub fn InheritSet(&mut self, parent: &CounterDirectives) {
        self.set_value_ = parent.set_value_;
    }

    // cpp: layoutng_style/style/counter_directives.h:111
    pub fn IsDefined(&self) -> bool {
        self.IsReset() || self.HasIncrement() || self.HasSet()
    }

    // cpp: layoutng_style/style/counter_directives.h:113-125
    pub fn CombinedValue(&self) -> i32 {
        if self.HasSet() {
            return self.SetValue();
        }
        let sum = self
            .ResetValueInt64()
            .unwrap_or(0)
            .checked_add(i64::from(self.increment_value_.unwrap_or(0)))
            .expect("C++ signed counter overflow is undefined");
        saturated_i32(sum)
    }
}

// cpp: layoutng_style/style/counter_directives.h:65-66
// cpp: layoutng_style/style/counter_directives.h:123-124
fn saturated_i32(value: i64) -> i32 {
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

// cpp: layoutng_style/style/counter_directives.h:142-148
#[derive(Clone, Default, PartialEq, Eq)]
pub struct CounterPropertyEntry {
    pub name: AtomicString,
    pub value: Option<i32>,
    pub is_reversed: bool,
}

// cpp: layoutng_style/style/counter_directives.h:150-156
#[derive(Clone, Default)]
pub struct CounterPropertyList(pub Vector<CounterPropertyEntry>);

#[allow(non_snake_case)]
impl CounterPropertyList {
    // cpp: layoutng_style/style/counter_directives.h:153-155
    pub fn Clone(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

impl Deref for CounterPropertyList {
    type Target = Vector<CounterPropertyEntry>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CounterPropertyList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// cpp: layoutng_style/style/counter_directives.h:158-164
#[derive(Clone, Default, PartialEq, Eq)]
pub struct CounterDirectiveMap(pub HashMap<AtomicString, CounterDirectives>);

#[allow(non_snake_case)]
impl CounterDirectiveMap {
    // cpp: layoutng_style/style/counter_directives.h:161-163
    pub fn Clone(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

impl Deref for CounterDirectiveMap {
    type Target = HashMap<AtomicString, CounterDirectives>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CounterDirectiveMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
