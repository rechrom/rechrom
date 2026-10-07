// C++: font_engine/text/native/hyphenation.h/.cc
// The pure virtual LastHyphenLocation is supplied by an algorithm object.
// Source declarations Initialize, ShouldHyphenateWord, and
// PlatformGetHyphenation have no definitions in this checkout and remain
// unconnected.
use foundation::{String, WtfSizeT};
use std::cell::Cell;
use std::sync::Arc;

// cpp: font_engine/text/native/hyphenation.h:26-31
#[allow(non_snake_case)]
pub trait HyphenationBackend {
    fn LastHyphenLocation(&self, word: &String, before_index: WtfSizeT) -> WtfSizeT;
}

// cpp: font_engine/text/native/hyphenation.h:24-71
pub struct Hyphenation {
    backend_: Arc<dyn HyphenationBackend>,
    // The source mutates limits on a ref-counted dictionary. Cell preserves
    // that shared ownership without fabricating a mutable reference to Arc data.
    min_word_length_: Cell<WtfSizeT>,
    min_prefix_length_: Cell<WtfSizeT>,
    min_suffix_length_: Cell<WtfSizeT>,
    hyphenate_capitalized_word_: bool,
}

#[allow(non_snake_case)]
impl Hyphenation {
    pub const kDefaultMinPrefixLength: WtfSizeT = 2;
    pub const kDefaultMinSuffixLength: WtfSizeT = 2;
    pub const kDefaultMinWordLength: WtfSizeT = 5;

    // Rust representation of the C++ abstract class: construction requires
    // the implementation of its pure virtual lookup method.
    pub fn new_with_backend(backend: Arc<dyn HyphenationBackend>) -> Self {
        Self {
            backend_: backend,
            min_word_length_: Cell::new(Self::kDefaultMinWordLength),
            min_prefix_length_: Cell::new(Self::kDefaultMinPrefixLength),
            min_suffix_length_: Cell::new(Self::kDefaultMinSuffixLength),
            hyphenate_capitalized_word_: false,
        }
    }

    // cpp: font_engine/text/native/hyphenation.h:28-29
    pub fn LastHyphenLocation(&self, word: &String, before_index: WtfSizeT) -> WtfSizeT {
        self.backend_.LastHyphenLocation(word, before_index)
    }

    // cpp: font_engine/text/native/hyphenation.cc:11-22
    pub fn FirstHyphenLocation(&self, word: &String, after_index: WtfSizeT) -> WtfSizeT {
        let mut first = 0;
        let mut before = word.encode_utf16().count() as WtfSizeT;
        while before > after_index {
            let location = self.LastHyphenLocation(word, before);
            if location == 0 || location <= after_index {
                break;
            }
            first = location;
            before = location;
        }
        first
    }

    // cpp: font_engine/text/native/hyphenation.cc:25-35
    pub fn HyphenLocations(&self, word: &String) -> Vec<WtfSizeT> {
        let mut locations = Vec::with_capacity(8);
        let mut before = word.encode_utf16().count() as WtfSizeT;
        while before != 0 {
            let location = self.LastHyphenLocation(word, before);
            if location == 0 || location >= before {
                break;
            }
            locations.push(location);
            before = location;
        }
        locations
    }

    // cpp: font_engine/text/native/hyphenation.h:40-43
    pub fn MinPrefixLength(&self) -> WtfSizeT {
        self.min_prefix_length_.get()
    }
    pub fn MinSuffixLength(&self) -> WtfSizeT {
        self.min_suffix_length_.get()
    }
    pub fn MinWordLength(&self) -> WtfSizeT {
        self.min_word_length_.get()
    }

    // cpp: font_engine/text/native/hyphenation.cc:38-49
    pub fn SetLimits(
        &self,
        min_prefix_length: WtfSizeT,
        min_suffix_length: WtfSizeT,
        min_word_length: WtfSizeT,
    ) {
        let min_prefix_length = if min_prefix_length != 0 {
            min_prefix_length
        } else {
            Self::kDefaultMinPrefixLength
        };
        let min_suffix_length = if min_suffix_length != 0 {
            min_suffix_length
        } else {
            Self::kDefaultMinSuffixLength
        };
        let min_word_length = if min_word_length != 0 {
            min_word_length
        } else {
            Self::kDefaultMinWordLength
        };
        self.min_prefix_length_.set(min_prefix_length);
        self.min_suffix_length_.set(min_suffix_length);
        self.min_word_length_
            .set(min_word_length.max(min_prefix_length.wrapping_add(min_suffix_length)));
    }

    // cpp: font_engine/text/native/hyphenation.h:47
    pub fn ResetLimits(&self) {
        self.SetLimits(0, 0, 0);
    }
}
