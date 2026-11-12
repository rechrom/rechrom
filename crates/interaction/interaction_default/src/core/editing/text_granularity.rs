// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/text_granularity.h:31-43
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextGranularity {
    #[default]
    kCharacter,
    kWord,
    kSentence,
    kLine,
    kParagraph,
    kSentenceBoundary,
    kLineBoundary,
    kParagraphBoundary,
    kDocumentBoundary,
}
