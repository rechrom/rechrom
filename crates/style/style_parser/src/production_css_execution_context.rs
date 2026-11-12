//! CSSStyleDeclaration's retained ExecutionContext identity. The document
//! owns the lifetime and counters; no Document borrow escapes a synchronous call.
//! Source: CSSStyleDeclaration.h, PropertySetCSSStyleDeclaration.h:40-45.
use crate::css_parser_context::SecureContextMode;
use foundation::css_use_counter::CSSUseCounter;
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
pub struct ProductionCSSExecutionContext {
    document_identity: u128,
    lifetime: Arc<AtomicBool>,
    secure: Cell<SecureContextMode>,
    counters: Rc<CSSUseCounter>,
}
#[allow(non_snake_case)]
impl ProductionCSSExecutionContext {
    pub fn new(
        document_identity: u128,
        lifetime: Arc<AtomicBool>,
        secure: SecureContextMode,
        counters: Rc<CSSUseCounter>,
    ) -> Rc<Self> {
        Rc::new(Self {
            document_identity,
            lifetime,
            secure: Cell::new(secure),
            counters,
        })
    }
    pub fn DocumentIdentity(&self) -> u128 {
        self.document_identity
    }
    pub fn IsContextDestroyed(&self) -> bool {
        !self.lifetime.load(Ordering::Acquire)
    }
    pub fn GetSecureContextMode(&self) -> SecureContextMode {
        self.secure.get()
    }
    pub fn SynchronizeSecureContextMode(&self, mode: SecureContextMode) {
        self.secure.set(mode);
    }
    pub fn UseCounter(&self) -> &Rc<CSSUseCounter> {
        &self.counters
    }
}
/// This is the actual optional current-world identity, not a fabricated main
/// world. Native editing has no entered script world and supplies None.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionCSSDOMWrapperWorld {
    pub identity: u64,
    pub world_id: i32,
}
pub use foundation::referrer_policy::ReferrerPolicy as CSSReferrerPolicy;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductionCSSReferrer {
    pub referrer: foundation::String,
    pub policy: CSSReferrerPolicy,
}
impl ProductionCSSReferrer {
    pub fn Empty() -> Self {
        Self {
            referrer: foundation::String::from(""),
            policy: CSSReferrerPolicy::StrictOriginWhenCrossOrigin,
        }
    }
}
