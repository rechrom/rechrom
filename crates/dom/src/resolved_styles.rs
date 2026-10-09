//! Immutable computed-style result used by the legacy static document adapter.
//!
//! The style algorithms live in the `style` crate.  This result type stays
//! beside the DOM projection because DOM-to-layout mapping consumes it and the
//! dependency direction must remain `style -> dom`, never `dom -> style`.

use crate::persistent_document::PseudoElement;
use layoutng_assembly::internal::layout_input::ComputedStyle;

#[derive(Clone, Default)]
pub struct ResolvedStyles {
    pub styles: Vec<ComputedStyle>,
    pub generates_box: Vec<bool>,
    pub display_contents: Vec<bool>,
    pub before: Vec<Option<PseudoElement>>,
    pub after: Vec<Option<PseudoElement>>,
}
