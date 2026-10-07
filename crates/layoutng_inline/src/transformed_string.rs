// C++: layoutng_inline/transformed_string.h/.cc.
// A borrowed length map retains span semantics; StringView owns an immutable
// UTF-16 snapshot because Rust cannot safely borrow a C++ StringImpl here.
#![allow(non_snake_case)]

use foundation::StringView;

// cpp: layoutng_inline/transformed_string.h:16-22,42-46
#[derive(Clone, Debug)]
pub struct TransformedString<'a> {
    view_: StringView,
    length_map_: &'a [u32],
}

impl<'a> TransformedString<'a> {
    // cpp: layoutng_inline/transformed_string.h:26-26
    pub fn new(view: StringView) -> Self {
        Self {
            view_: view,
            length_map_: &[],
        }
    }

    // cpp: layoutng_inline/transformed_string.h:27-29
    pub fn with_length_map(view: StringView, map: &'a [u32]) -> Self {
        Self {
            view_: view,
            length_map_: map,
        }
    }

    // cpp: layoutng_inline/transformed_string.h:31-31
    pub fn View(&self) -> &StringView {
        &self.view_
    }

    // cpp: layoutng_inline/transformed_string.h:32-32
    pub fn HasLengthMap(&self) -> bool {
        !self.length_map_.is_empty()
    }

    // cpp: layoutng_inline/transformed_string.h:33-35
    pub fn LengthMap(&self) -> &[u32] {
        self.length_map_
    }

    // cpp: layoutng_inline/transformed_string.h:37-37
    // cpp: layoutng_inline/transformed_string.cc:9-19
    pub fn Substring(&self, start: u32, length: u32) -> Self {
        let sub_view = self.view_.Substring(start, length);
        if self.length_map_.is_empty() {
            return Self::new(sub_view);
        }
        assert_eq!(self.view_.length() as usize, self.length_map_.len());
        assert!(start <= self.view_.length());
        assert!(length <= self.view_.length() - start);
        Self::with_length_map(
            sub_view,
            &self.length_map_[start as usize..(start + length) as usize],
        )
    }

    // cpp: layoutng_inline/transformed_string.h:38-40
    pub fn SubstringFrom(&self, start: u32) -> Self {
        self.Substring(start, self.view_.length() - start)
    }
}
