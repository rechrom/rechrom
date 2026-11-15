//! Source suggestion_marker_replacement_scope.cc:9-26. Shared only to carry
//! this existing synchronous mutation context through DOM's neutral fact port;
//! marker algorithms and state remain in interaction default's controller.
use std::cell::Cell;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkerMutationKind {
    Ordinary,
    SuggestionReplacement,
}
thread_local! {static REPLACEMENT_SCOPE:Cell<bool>=const{Cell::new(false)};}
pub struct SuggestionMarkerReplacementScope;
impl SuggestionMarkerReplacementScope {
    pub fn new() -> Self {
        REPLACEMENT_SCOPE.with(|scope| {
            assert!(
                !scope.replace(true),
                "source replacement scopes do not nest"
            );
        });
        Self
    }
    pub fn CurrentlyInScope() -> bool {
        REPLACEMENT_SCOPE.with(|scope| scope.get())
    }
    pub fn CurrentMutationKind() -> MarkerMutationKind {
        if Self::CurrentlyInScope() {
            MarkerMutationKind::SuggestionReplacement
        } else {
            MarkerMutationKind::Ordinary
        }
    }
}
impl Drop for SuggestionMarkerReplacementScope {
    fn drop(&mut self) {
        REPLACEMENT_SCOPE.with(|scope| scope.set(false));
    }
}
