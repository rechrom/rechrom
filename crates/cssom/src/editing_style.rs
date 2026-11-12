//! Opaque declaration operations exchanged by EditingStyle and the Style owner.
use crate::CSSInlineStyle;
/// StyleChange result. The declaration keeps its compiled property owner;
/// qualified names retain the author's original HTML equivalent tags.
pub struct EditingStyleApplication {
    pub declaration: CSSInlineStyle,
    pub tags: Vec<foundation::QualifiedName>,
}
#[derive(Clone)]
pub enum EditingStyleOperation {
    Capture {
        node_id: u64,
        in_effect: bool,
    },
    Prepare {
        node_id: u64,
        declaration: CSSInlineStyle,
    },
    Merge {
        base: CSSInlineStyle,
        override_style: CSSInlineStyle,
    },
    RemoveStyleAddedByElement {
        node_id: u64,
        declaration: CSSInlineStyle,
    },
}
#[derive(Clone, Copy, Debug)]
pub enum EditingStyleError {
    MissingStyle,
    MissingCompiledDeclaration,
    UnavailableComputedValue,
    UnavailableMatchedRules,
    UnavailableSerializationValue,
    UnsupportedResource,
}

/// Range serializer requests. The Style owner resolves rules and mutates
/// compiled declarations; callers receive only the existing opaque handle.
#[derive(Clone)]
pub enum SerializationStyleOperation {
    Wrapping {
        node_id: u64,
        annotate: bool,
    },
    Inline {
        node_id: u64,
        wrapping: Option<CSSInlineStyle>,
        annotate: bool,
        sanitize: bool,
        force_inline: bool,
        remove_float: bool,
    },
    Text {
        wrapping: CSSInlineStyle,
        sanitize: bool,
    },
    FullySelectedRoot {
        node_id: u64,
    },
}

/// Replacement editing asks Style to operate on compiled declarations and the
/// element's original computed/matched-rule owners. No CSS source is reparsed.
#[derive(Clone)]
pub enum ReplacementStyleOperation {
    Inline {
        node_id: u64,
    },
    PropertyCount {
        declaration: CSSInlineStyle,
    },
    ImplicitConflicts {
        node_id: u64,
        declaration: CSSInlineStyle,
    },
    RemoveContext {
        node_id: u64,
        context_id: Option<u64>,
        declaration: CSSInlineStyle,
        include_rules: bool,
    },
    WhiteSpaceFacts {
        node_id: u64,
        include_author_rules: bool,
    },
    WhiteSpaceNormal,
    InsertionStyleMatchesAttribute {
        node_id: u64,
        span_id: u64,
    },
    VisualDifference {
        node_id: u64,
        other_id: u64,
    },
}
#[derive(Clone, Copy, Default)]
pub struct ReplacementWhiteSpaceFacts {
    pub has_inline_white_space: bool,
    pub has_author_white_space: bool,
    pub collapse_breaks_and_wrap: Option<(bool, bool)>,
}
pub enum ReplacementStyleResult {
    Inline(Option<CSSInlineStyle>),
    Declaration(CSSInlineStyle),
    Count(usize),
    ImplicitConflicts {
        replace_with_span: bool,
        attributes: Vec<foundation::QualifiedName>,
    },
    WhiteSpaceFacts(ReplacementWhiteSpaceFacts),
    Bool(bool),
    VisualDifference(Option<bool>),
}

/// ApplyStyleCommand declaration operations. The command owns DOM traversal;
/// Style owns the compiled CSSValue algebra and computed-value interpretation.
#[derive(Clone, Copy)]
pub enum ApplyStyleExtraction {
    Block,
    Direction,
    Container,
}
#[derive(Clone)]
pub enum EditingStylePropertyValue {
    Identifier(foundation::CSSValueID),
    Text(String),
    LegacyFontSize(String),
}
#[derive(Clone)]
pub enum ApplyStyleOperation {
    Empty,
    Create {
        properties: Vec<(foundation::CSSPropertyID, EditingStylePropertyValue)>,
    },
    CommandStart {
        node_id: u64,
        typing: Option<CSSInlineStyle>,
    },
    CommandPresent {
        node_id: u64,
        declaration: CSSInlineStyle,
        typing: Option<CSSInlineStyle>,
    },
    CommandValue {
        node_id: u64,
        property: foundation::CSSPropertyID,
        typing: Option<CSSInlineStyle>,
    },
    ToggleList {
        declaration: CSSInlineStyle,
        property: foundation::CSSPropertyID,
        value: foundation::CSSValueID,
    },
    Count {
        declaration: CSSInlineStyle,
    },
    Extract {
        declaration: CSSInlineStyle,
        kind: ApplyStyleExtraction,
    },
    Merge {
        base: CSSInlineStyle,
        overriding: CSSInlineStyle,
    },
    RemoveProperties {
        declaration: CSSInlineStyle,
        properties: Vec<foundation::CSSPropertyID>,
    },
    RemovePresentProperties {
        declaration: CSSInlineStyle,
        present: CSSInlineStyle,
    },
    Conflicts {
        node_id: u64,
        declaration: CSSInlineStyle,
        matching: bool,
        extract: bool,
        preserve_direction: bool,
    },
    Present {
        node_id: u64,
        declaration: CSSInlineStyle,
    },
    Change {
        node_id: u64,
        declaration: CSSInlineStyle,
        originals: Vec<(
            foundation::CSSPropertyID,
            foundation::CSSValueID,
            foundation::QualifiedName,
        )>,
        css: bool,
    },
    FontDelta {
        declaration: CSSInlineStyle,
    },
    FontSize {
        node_id: u64,
    },
    SetFontSize {
        declaration: CSSInlineStyle,
        pixels: f32,
    },
    Bidi {
        node_id: u64,
    },
    StyleDirection {
        declaration: CSSInlineStyle,
    },
    NormalBidi {
        declaration: CSSInlineStyle,
    },
}
pub struct ApplyStyleConflict {
    pub implicit_element: bool,
    pub attributes: Vec<foundation::QualifiedName>,
    pub properties: Vec<foundation::CSSPropertyID>,
    pub extracted: CSSInlineStyle,
}
pub struct ApplyStyleChange {
    pub declaration: CSSInlineStyle,
    pub tags: Vec<foundation::QualifiedName>,
    pub font_attributes: Vec<(foundation::QualifiedName, std::rc::Rc<[u16]>)>,
}
pub enum ApplyStyleResult {
    Text(String),
    InvalidValue,
    ToggledList {
        declaration: CSSInlineStyle,
        is_none: bool,
    },
    Declaration(CSSInlineStyle),
    Extract {
        extracted: CSSInlineStyle,
        remaining: CSSInlineStyle,
    },
    Count(usize),
    Bool(bool),
    Conflict(ApplyStyleConflict),
    Change(ApplyStyleChange),
    FontDelta {
        delta: f32,
        declaration: CSSInlineStyle,
    },
    Number(f32),
    Bidi {
        unicode_bidi: foundation::CSSValueID,
        direction: Option<foundation::TextDirection>,
    },
    Direction(Option<Option<foundation::TextDirection>>),
}
