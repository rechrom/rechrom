//! Chromium blink/common/scheme_registry.cc:14-70. Registration is product
//! initialization data, independent of Document URLs or input events.
use std::{
    collections::HashSet,
    sync::{OnceLock, RwLock},
};
fn extension_schemes() -> &'static RwLock<HashSet<String>> {
    static SCHEMES: OnceLock<RwLock<HashSet<String>>> = OnceLock::new();
    SCHEMES.get_or_init(|| RwLock::new(HashSet::new()))
}
pub struct CommonSchemeRegistry;
impl CommonSchemeRegistry {
    pub fn RegisterURLSchemeAsExtension(scheme: &str) {
        assert_eq!(scheme, scheme.to_ascii_lowercase());
        extension_schemes()
            .write()
            .expect("scheme registry")
            .insert(scheme.into());
    }
    pub fn RemoveURLSchemeAsExtensionForTest(scheme: &str) {
        extension_schemes()
            .write()
            .expect("scheme registry")
            .remove(scheme);
    }
    pub fn IsExtensionScheme(scheme: &str) -> bool {
        if scheme.is_empty() {
            return false;
        }
        assert_eq!(scheme, scheme.to_ascii_lowercase());
        extension_schemes()
            .read()
            .expect("scheme registry")
            .contains(scheme)
    }
}
