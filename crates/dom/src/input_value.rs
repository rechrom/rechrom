//! HTMLInputElement's canonical value/storage boundary. The injected source
//! InputType supplies sanitization; DOM retains the resulting UTF-16 owner.
use crate::{persistent_document::DOMOwnerHandle, DOMInputType};
use std::rc::Rc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DOMInputValueMode {
    Value,
    Default,
    DefaultOn,
    Filename,
}
pub struct DOMInputValueRequest<'a> {
    pub kind: DOMInputType,
    pub proposed: Rc<[u16]>,
    pub minimum: &'a [u16],
    pub maximum: &'a [u16],
    pub step: &'a str,
    pub value_attribute: &'a [u16],
    pub multiple: bool,
}
pub trait DOMInputValueSanitizer {
    fn ValueMode(&self, kind: DOMInputType) -> DOMInputValueMode;
    fn SanitizeValue(&self, request: DOMInputValueRequest<'_>) -> Rc<[u16]>;
    fn DidCommitValue(
        &self,
        owner: DOMOwnerHandle,
        kind: DOMInputType,
        value: &[u16],
        revision: u64,
    );
}
