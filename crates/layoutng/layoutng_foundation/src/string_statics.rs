#![allow(non_snake_case)]

use std::sync::LazyLock;

use crate::AtomicString;

// The source installs these process-lifetime atoms before building LayoutObjects.
// Rust's LazyLock provides the same one-time initialization and stable identity.
// cpp: foundation/style_values/style/string_statics.cc:137-164
static STATIC_ATOMS: LazyLock<[AtomicString; 8]> = LazyLock::new(|| {
    ["xmlns:", "*", "xml", "xmlns", "xlink", "http", "https", ""].map(AtomicString::from_str)
});

// cpp: foundation/blink_base/wtf/text/string_statics.h:39-39
// cpp: foundation/style_values/style/string_statics.cc:137-164
pub fn InitStringStatics() {
    let _ = LazyLock::force(&STATIC_ATOMS);
    crate::keywords::Init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_atoms_have_stable_interned_identity() {
        InitStringStatics();
        assert_eq!(STATIC_ATOMS[2], AtomicString::from_str("xml"));
        InitStringStatics();
        assert_eq!(STATIC_ATOMS[2], AtomicString::from_str("xml"));
    }
}
