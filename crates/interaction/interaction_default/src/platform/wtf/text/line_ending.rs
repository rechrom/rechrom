// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/wtf/text/line_ending.h:44-45
// Source: third_party/blink/renderer/platform/wtf/text/line_ending.cc:94-131,174-188
use std::borrow::Cow;

// Source: line_ending.cc:94-113.
fn RequiredSizeForLf(src: &[u16]) -> Option<usize> {
    let mut change = false;
    let mut len = 0;
    let mut index = 0;
    while index < src.len() {
        let c = src[index];
        index += 1;
        if c == 13 {
            if index < src.len() && src[index] == 10 {
                index += 1;
            }
            change = true;
        }
        len += 1;
    }
    change.then_some(len)
}
// Source: line_ending.cc:115-131,174-188. Cow retains the source fast-path
// sharing when no normalization is needed.
pub fn NormalizeLineEndingsToLf(src: &[u16]) -> Cow<'_, [u16]> {
    if src.is_empty() {
        return Cow::Borrowed(src);
    }
    let Some(len) = RequiredSizeForLf(src) else {
        return Cow::Borrowed(src);
    };
    let mut dst = Vec::with_capacity(len);
    let mut index = 0;
    while index < src.len() {
        let mut c = src[index];
        index += 1;
        if c == 13 {
            if index < src.len() && src[index] == 10 {
                index += 1;
            }
            c = 10;
        }
        dst.push(c);
    }
    Cow::Owned(dst)
}
