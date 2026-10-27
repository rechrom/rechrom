//! Native hyphenation, text-transform, and phrase-break environment.
//!
//! These callbacks feed inline shaping and line breaking. Keeping them in
//! `layout_algo_inline` makes provider and validation costs directly visible
//! without emitting per-word trace events.

#![allow(non_snake_case)]

use super::*;

// cpp: layoutng/internal/boundary/layout_boundary.cc:413-426
#[inline(never)]
fn ProviderLastHyphenLocation(
    provider: &dyn HyphenationProvider,
    locale: &str,
    word: &[u16],
    before_index: u32,
) -> u32 {
    let result = provider.LastHyphenLocation(locale, word, before_index as usize);
    if result > word.len() || (result != 0 && result >= before_index as usize) {
        panic!("HyphenationProvider returned an invalid UTF-16 boundary");
    }
    result as u32
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:407-431
struct ProviderHyphenation {
    provider_: Arc<dyn HyphenationProvider>,
    locale_: String,
}

impl ProviderHyphenation {
    fn new(provider: Arc<dyn HyphenationProvider>, locale: String) -> Self {
        Self {
            provider_: provider,
            locale_: locale,
        }
    }
}

impl HyphenationBackend for ProviderHyphenation {
    fn LastHyphenLocation(&self, word: &BlinkString, before_index: u32) -> u32 {
        let characters: Vec<u16> = word.encode_utf16().collect();
        ProviderLastHyphenLocation(
            self.provider_.as_ref(),
            &self.locale_,
            &characters,
            before_index,
        )
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:433-458
pub(super) struct InputHyphenationResolver {
    provider_: Option<Arc<dyn HyphenationProvider>>,
    dictionaries_: HashMap<String, Arc<Hyphenation>>,
}

impl InputHyphenationResolver {
    pub(super) fn new(provider: Option<Arc<dyn HyphenationProvider>>) -> Self {
        Self {
            provider_: provider,
            dictionaries_: HashMap::new(),
        }
    }
}

impl NativeHyphenationResolver for InputHyphenationResolver {
    fn Resolve(&mut self, locale: &AtomicString) -> *mut Hyphenation {
        let Some(provider) = self.provider_.as_ref() else {
            return std::ptr::null_mut();
        };
        let key = locale.Utf8();
        if let Some(dictionary) = self.dictionaries_.get(&key) {
            return Arc::as_ptr(dictionary) as *mut Hyphenation;
        }
        // The font_engine base retains this backend and owns the inherited
        // hyphenation limits and virtual-default algorithms.
        let backend = Arc::new(ProviderHyphenation::new(Arc::clone(provider), key.clone()));
        let dictionary = Arc::new(Hyphenation::new_with_backend(backend));
        let result = Arc::as_ptr(&dictionary) as *mut Hyphenation;
        self.dictionaries_.insert(key, dictionary);
        result
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:472-497
#[inline(never)]
fn TransformWithProvider(
    provider: Option<&dyn TextTransformProvider>,
    transform: TextTransform,
    locale: &str,
    text: &[u16],
    previous_character: u16,
    mut append_offset: impl FnMut(u32, u32),
) -> Vec<u16> {
    let Some(provider) = provider else {
        panic!("text-transform requires ConstraintSpace.text_transform");
    };
    let output = provider.Transform(transform, locale, text, previous_character);
    let mut previous_source = 0;
    let mut previous_target = 0;
    for entry in &output.offset_map {
        if entry.source > text.len()
            || entry.target > output.text.len()
            || entry.source < previous_source
            || entry.target < previous_target
        {
            panic!("TextTransformProvider returned an invalid offset map");
        }
        append_offset(entry.source as u32, entry.target as u32);
        previous_source = entry.source;
        previous_target = entry.target;
    }
    if text.len() != output.text.len() && output.offset_map.is_empty() {
        panic!("TextTransformProvider must map length-changing edits");
    }
    output.text
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:460-471
// cpp: layoutng/internal/boundary/layout_boundary.cc:498-503
pub(super) struct InputTextTransformResolver {
    provider_: Option<Arc<dyn TextTransformProvider>>,
}

impl InputTextTransformResolver {
    pub(super) fn new(provider: Option<Arc<dyn TextTransformProvider>>) -> Self {
        Self {
            provider_: provider,
        }
    }
}

impl NativeTextTransformResolver for InputTextTransformResolver {
    fn Transform(
        &self,
        transform: ETextTransform,
        locale: &AtomicString,
        text: &BlinkString,
        previous_character: u16,
        mut offset_map: Option<&mut TextOffsetMap>,
    ) -> BlinkString {
        let input: Vec<u16> = text.Span16().unwrap_or_default().to_vec();
        let output = TransformWithProvider(
            self.provider_.as_deref(),
            TextTransform(transform.bits()),
            &locale.Utf8(),
            &input,
            previous_character,
            |source, target| {
                if let Some(offset_map) = offset_map.as_deref_mut() {
                    offset_map.Append(source, target);
                }
            },
        );
        BlinkString::from_utf16(&output)
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:516-534
#[inline(never)]
fn NextPhraseBreak(
    provider: Option<&dyn PhraseBreakProvider>,
    locale: &str,
    text: &[u16],
    from_index: u32,
    range_end: u32,
) -> u32 {
    let Some(provider) = provider else {
        panic!("word-break:auto-phrase requires ConstraintSpace.phrase_break");
    };
    if from_index > range_end || range_end as usize > text.len() {
        panic!("Invalid native phrase-break range");
    }
    let result = provider.NextBreakLocation(locale, text, from_index as usize, range_end as usize);
    if result < from_index as usize
        || result > range_end as usize
        || (result > 0
            && result < text.len()
            && (0xdc00..=0xdfff).contains(&text[result])
            && (0xd800..=0xdbff).contains(&text[result - 1]))
    {
        panic!("PhraseBreakProvider returned an invalid UTF-16 boundary");
    }
    result as u32
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:505-515
// cpp: layoutng/internal/boundary/layout_boundary.cc:535-539
pub(super) struct InputPhraseBreakResolver {
    provider_: Option<Arc<dyn PhraseBreakProvider>>,
}

impl InputPhraseBreakResolver {
    pub(super) fn new(provider: Option<Arc<dyn PhraseBreakProvider>>) -> Self {
        Self {
            provider_: provider,
        }
    }
}

impl NativePhraseBreakResolver for InputPhraseBreakResolver {
    fn NextBreak(
        &self,
        locale: &AtomicString,
        text: &BlinkString,
        from_index: u32,
        range_end: u32,
    ) -> u32 {
        let input: Vec<u16> = text.Span16().unwrap_or_default().to_vec();
        NextPhraseBreak(
            self.provider_.as_deref(),
            &locale.Utf8(),
            &input,
            from_index,
            range_end,
        )
    }
}
