// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! platform/network/mime/content_type.cc:33-105 and
//! mime_type_from_url.cc:35-49, Chromium revision 6c1d401f.
fn strip(text: &str) -> &str {
    text.trim_matches(|c: char| {
        (c as u32) <= u16::MAX as u32 && crate::unicode::IsSpaceOrNewline(c as u16)
    })
}
pub struct ContentType {
    value: std::string::String,
}
impl ContentType {
    pub fn new(text: &str) -> Self {
        Self {
            value: strip(text).to_owned(),
        }
    }
    pub fn GetType(&self) -> &str {
        strip(
            self.value
                .split_once(';')
                .map_or(self.value.as_str(), |(head, _)| head),
        )
    }
    pub fn Parameter(&self, name: &str) -> std::string::String {
        let mut start = 0;
        let mut quoted = false;
        for (index, ch) in self
            .value
            .char_indices()
            .chain(std::iter::once((self.value.len(), ';')))
        {
            if ch == '"' {
                quoted = !quoted;
            }
            if ch == ';' && !quoted {
                if index != start {
                    let parameter = strip(&self.value[start..index]);
                    if let Some((attribute, value)) = parameter.split_once('=') {
                        if strip(attribute).eq_ignore_ascii_case(name) {
                            return strip(value).chars().filter(|c| *c != '"').collect();
                        }
                    }
                }
                start = index + 1;
            }
        }
        // Source adds the remaining parameter even when its final quote is
        // unmatched (the sentinel above is intentionally ignored then).
        if start < self.value.len() {
            let parameter = strip(&self.value[start..]);
            if let Some((attribute, value)) = parameter.split_once('=') {
                if strip(attribute).eq_ignore_ascii_case(name) {
                    return strip(value).chars().filter(|c| *c != '"').collect();
                }
            }
        }
        std::string::String::new()
    }
}
pub fn MimeTypeFromDataURL(url: &str) -> std::string::String {
    debug_assert!(url.starts_with("data:"));
    let Some(index) = url.find(';').or_else(|| url.find(',')) else {
        return std::string::String::new();
    };
    if index > 5 {
        crate::String::from(&url[5..index]).DeprecatedLower().Utf8()
    } else {
        "text/plain".into()
    }
}
