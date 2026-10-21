use foundation::{AtomicString, CSSBitset, Vector, Visitor};

// cpp: layoutng_style/style/style_will_change_data.h:18-51
/// Immutable after construction, matching the C++ const data members.
pub struct StyleWillChangeData {
    values: Vector<AtomicString>,
    resolved_longhand_ids: CSSBitset,
    has_scroll_position_value: bool,
    has_transform_property: bool,
    has_any_transform_property: bool,
}

#[allow(non_snake_case)]
impl StyleWillChangeData {
    // cpp: layoutng_style/style/style_will_change_data.h:23-32
    pub fn new(
        values: Vector<AtomicString>,
        resolved_longhand_ids: CSSBitset,
        has_scroll_position_value: bool,
        has_transform_property: bool,
        has_any_transform_property: bool,
    ) -> Self {
        Self {
            values,
            resolved_longhand_ids,
            has_scroll_position_value,
            has_transform_property,
            has_any_transform_property,
        }
    }

    // cpp: layoutng_style/style/style_will_change_data.h:38
    pub fn Trace(&self, _visitor: Option<&mut Visitor>) {}

    // cpp: layoutng_style/style/style_will_change_data.h:40-50
    pub fn values(&self) -> &Vector<AtomicString> {
        &self.values
    }
    pub fn resolved_longhand_ids(&self) -> &CSSBitset {
        &self.resolved_longhand_ids
    }
    pub fn has_scroll_position_value(&self) -> bool {
        self.has_scroll_position_value
    }
    pub fn has_transform_property(&self) -> bool {
        self.has_transform_property
    }
    pub fn has_any_transform_property(&self) -> bool {
        self.has_any_transform_property
    }
}

// cpp: layoutng_style/style/style_will_change_data.h:34-36
impl PartialEq for StyleWillChangeData {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Eq for StyleWillChangeData {}

// The immutable strings/bitset contain no layout-heap edges.
// cpp: core/style/style_will_change_data.h:38
impl foundation::Traceable for StyleWillChangeData {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        StyleWillChangeData::Trace(self, Some(visitor));
    }
}
