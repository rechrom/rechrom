//! WTF String's nullity and exact UTF-16 owner at DOM control boundaries.
use std::{ops::Deref, rc::Rc};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NullableUTF16String {
    units: Rc<[u16]>,
    is_null: bool,
}
impl NullableUTF16String {
    pub fn Null() -> Self {
        Self {
            units: Rc::from([]),
            is_null: true,
        }
    }
    pub fn FromOwner(units: Rc<[u16]>) -> Self {
        Self {
            units,
            is_null: false,
        }
    }
    pub fn FromAttribute(units: Option<Rc<[u16]>>) -> Self {
        units.map(Self::FromOwner).unwrap_or_else(Self::Null)
    }
    pub fn IsNull(&self) -> bool {
        self.is_null
    }
    pub fn Owner(&self) -> Rc<[u16]> {
        self.units.clone()
    }
}
impl Deref for NullableUTF16String {
    type Target = [u16];
    fn deref(&self) -> &[u16] {
        &self.units
    }
}
impl AsRef<[u16]> for NullableUTF16String {
    fn as_ref(&self) -> &[u16] {
        &self.units
    }
}
impl From<&[u16]> for NullableUTF16String {
    fn from(value: &[u16]) -> Self {
        Self::FromOwner(Rc::from(value))
    }
}
