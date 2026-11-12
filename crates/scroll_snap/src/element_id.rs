// Chromium: cc/paint/element_id.h; same opaque cc identity owner.
/// Opaque scrolling/animation identity from cc/paint/element_id.h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ElementId {
    id_: u64,
}

impl ElementId {
    pub const kInvalidElementId: u64 = 0;
    pub const kDeletedElementId: u64 = u64::MAX;

    pub const fn new(id: u64) -> Self {
        debug_assert!(id != Self::kInvalidElementId);
        debug_assert!(id != Self::kDeletedElementId);
        Self { id_: id }
    }

    pub const fn DeletedValue() -> Self {
        Self {
            id_: Self::kDeletedElementId,
        }
    }

    pub const fn GetInternalValue(self) -> u64 {
        self.id_
    }

    pub const fn IsValidInternalValue(value: u64) -> bool {
        value != Self::kInvalidElementId && value != Self::kDeletedElementId
    }

    pub const fn IsValid(self) -> bool {
        Self::IsValidInternalValue(self.id_)
    }

    pub fn ToString(self) -> String {
        format!("({})", self.id_)
    }
}

pub fn RemapElementIdToCcNamespace(element_id: ElementId) -> ElementId {
    assert!(element_id.GetInternalValue() & 1 == 0);
    ElementId::new(element_id.GetInternalValue() | 1)
}
