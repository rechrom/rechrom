// Source: base/check_version_internal.h.in:10-11
// Source: base/BUILD.gn:6571-6596
// Mandatory resolved process_version output. In this Chromium tree GN reads
// chrome/VERSION MAJOR; no WebRTC alternative or guessed current milestone.
const fn version(text: &str) -> i32 {
    let bytes = text.as_bytes();
    assert!(!bytes.is_empty(), "empty native check version");
    let mut i = 0;
    let mut value = 0i32;
    while i < bytes.len() {
        assert!(
            bytes[i] >= b'0' && bytes[i] <= b'9',
            "invalid native check version"
        );
        value = value * 10 + (bytes[i] - b'0') as i32;
        i += 1;
    }
    value
}
pub const BASE_CHECK_VERSION_INTERNAL: i32 = version(env!("BASE_CHECK_VERSION_INTERNAL"));
pub const BASE_CHECK_NEXT_VERSION_INTERNAL: i32 = version(env!("BASE_CHECK_NEXT_VERSION_INTERNAL"));
