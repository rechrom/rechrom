// C++: layoutng_inline/text_diff_range.h/.cc.
#![allow(non_snake_case)]

use foundation::{String, StringView, WtfSizeT};

// cpp: layoutng_inline/text_diff_range.h:18-20,39-43
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextDiffRange {
    pub offset: WtfSizeT,
    pub old_size: WtfSizeT,
    pub new_size: WtfSizeT,
}

impl TextDiffRange {
    // cpp: layoutng_inline/text_diff_range.h:22-24
    pub fn Delete(offset: WtfSizeT, old_size: WtfSizeT) -> Self {
        Self {
            offset,
            old_size,
            ..Self::default()
        }
    }

    // cpp: layoutng_inline/text_diff_range.h:25-27
    pub fn Insert(offset: WtfSizeT, new_size: WtfSizeT) -> Self {
        Self {
            offset,
            new_size,
            ..Self::default()
        }
    }

    // cpp: layoutng_inline/text_diff_range.h:28-32
    pub fn Replace(offset: WtfSizeT, old_size: WtfSizeT, new_size: WtfSizeT) -> Self {
        Self {
            offset,
            old_size,
            new_size,
        }
    }

    // cpp: layoutng_inline/text_diff_range.h:34-34
    pub fn OldEndOffset(&self) -> WtfSizeT {
        self.offset.wrapping_add(self.old_size)
    }

    // cpp: layoutng_inline/text_diff_range.h:35-35
    pub fn NewEndOffset(&self) -> WtfSizeT {
        self.offset.wrapping_add(self.new_size)
    }

    // cpp: layoutng_inline/text_diff_range.h:48-50
    #[cfg(not(feature = "expensive_dchecks"))]
    pub fn CheckValid(&self, _old_text: &String, _new_text: &String) {}

    // cpp: layoutng_inline/text_diff_range.cc:12-24
    #[cfg(feature = "expensive_dchecks")]
    pub fn CheckValid(&self, old_text: &String, new_text: &String) {
        debug_assert_eq!(
            old_text
                .length()
                .wrapping_sub(self.old_size)
                .wrapping_add(self.new_size),
            new_text.length(),
            "{old_text} => {new_text} {}-{}+{} => {}",
            old_text.length(),
            self.old_size,
            self.new_size,
            new_text.length()
        );

        let old_view = StringView::from(old_text);
        let new_view = StringView::from(new_text);
        debug_assert_eq!(
            old_view.Substring(0, self.offset),
            new_view.Substring(0, self.offset),
            "{old_text} => {new_text}"
        );

        let old_end = self.OldEndOffset();
        let new_end = self.NewEndOffset();
        debug_assert_eq!(
            old_view.Substring(old_end, old_view.length() - old_end),
            new_view.Substring(new_end, new_view.length() - new_end),
            "{old_text} => {new_text}"
        );
    }
}
