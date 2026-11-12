//! Live Document ports for the translated editing fragment owner. Parsing and
//! text creation allocate into that same canonical DocumentFragment arena.
use crate::ownership::{InteractionDOMMutationEmitter, InteractionDocument};
use dom::persistent_document::{DOMAttribute, DOMNamespace, PersistentDocument};
use interaction_default::core::editing::serializers::serialization::{
    self, FragmentDocument, TextFragmentContext,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FragmentError {
    InvalidSelection,
    InvalidOffsets,
    ParseFailed,
    DocumentChanged,
    RequiresStyleSanitization,
    UnsupportedResource,
}
#[derive(Clone)]
pub struct RetainedFragment {
    pub root: u64,
    pub(super) document: InteractionDocument,
    pub(super) identity: u128,
}
impl RetainedFragment {
    pub fn IsCurrent(&self) -> bool {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        document.IsActive()
            && document.RootHandle().Identity() == self.identity
            && document.FindNodeById(self.root).is_some()
    }
}
/// InsertCommands::ExecuteInsertElement allocates an actual detached native
/// element; attribute strings never pass through the HTML parser.
pub(crate) fn CreateFragmentFromElement(
    document: &InteractionDocument,
    tag: &str,
    attribute: &str,
    value: &str,
) -> Result<RetainedFragment, FragmentError> {
    let mut owner = document.borrow_mut();
    let native = owner.GetDocumentMut();
    if !native.IsActive() {
        return Err(FragmentError::DocumentChanged);
    }
    let identity = native.RootHandle().Identity();
    let root = native.CreateDocumentFragment();
    let element = native.CreateElement(DOMNamespace::kHTML, tag.into(), Vec::new());
    if !value.is_empty() {
        native.SetAttribute(
            element,
            DOMAttribute {
                local_name: attribute.into(),
                value: value.into(),
                ..Default::default()
            },
        );
    }
    native.AppendChild(root, element);
    Ok(RetainedFragment {
        root: native.Node(root).Id(),
        document: document.clone(),
        identity,
    })
}
pub(crate) fn FragmentTextContentWithBreaks(fragment: &RetainedFragment) -> Vec<u16> {
    let owner = fragment.document.borrow();
    let document = owner.GetDocument();
    fn append(document: &PersistentDocument, index: usize, text: &mut Vec<u16>) {
        let node = document.Node(index);
        if node.Type() == dom::DOMNodeType::kText {
            text.extend_from_slice(&document.CharacterDataUTF16(index));
        } else if node.IsHTMLElement("br") {
            text.push(10);
        } else {
            for child in node.Children() {
                append(document, *child, text);
            }
        }
    }
    let mut text = Vec::new();
    if let Some(root) = document.FindNodeById(fragment.root) {
        append(document, root, &mut text);
    }
    text
}
struct NativeFragmentDocument<'a>(&'a mut PersistentDocument);
impl FragmentDocument for NativeFragmentDocument<'_> {
    type Node = usize;
    fn CreateDocumentFragment(&mut self) -> usize {
        self.0.CreateDocumentFragment()
    }
    fn CreateText(&mut self, text: &[u16]) -> usize {
        self.0.CreateTextUTF16(text)
    }
    fn CreateElement(&mut self, name: &str) -> usize {
        self.0
            .CreateElement(DOMNamespace::kHTML, name.into(), Vec::new())
    }
    fn CloneElementWithoutChildren(&mut self, node: &usize) -> usize {
        self.0.CloneNode(*node, false)
    }
    fn SetAttribute(&mut self, node: &usize, name: &str, value: &str) {
        self.0.SetAttribute(
            *node,
            DOMAttribute {
                local_name: name.into(),
                value: value.into(),
                ..Default::default()
            },
        );
    }
    fn AppendChild(&mut self, parent: &usize, child: &usize) {
        self.0.AppendChild(*parent, *child);
    }
}
/// The caller obtains this context from FrameSelection::CompositionText after
/// lifecycle preparation. Detached fragment construction does not execute script
/// or enter undo history; ReplaceSelection inserts its existing nodes through
/// CompositeEditCommand and the synchronous mutation emitter.
pub fn CreateFragmentFromText(
    document: &InteractionDocument,
    _emit: &InteractionDOMMutationEmitter,
    context: &TextFragmentContext<u64>,
    text: &[u16],
) -> Result<RetainedFragment, FragmentError> {
    let mut owner = document.borrow_mut();
    let native = owner.GetDocumentMut();
    if !native.IsActive() {
        return Err(FragmentError::DocumentChanged);
    }
    let identity = native.RootHandle().Identity();
    let context = TextFragmentContext {
        richly_editable: context.richly_editable,
        preserve_newlines: context.preserve_newlines,
        default_paragraph_separator: context.default_paragraph_separator,
        enclosing_block_to_clone: match context.enclosing_block_to_clone {
            Some(node) => Some(
                native
                    .FindNodeById(node)
                    .ok_or(FragmentError::InvalidSelection)?,
            ),
            None => None,
        },
    };
    let root =
        serialization::CreateFragmentFromText(&mut NativeFragmentDocument(native), &context, text);
    Ok(RetainedFragment {
        root: native.Node(root).Id(),
        document: document.clone(),
        identity,
    })
}
/// Source serialization.cc:468–527. Parser policy removes scripts, plug-ins and
/// scripting attributes while preserving native node, attribute and WTF-16 owners.
/// Strict clipboard sanitization is a separate staging lifecycle operation; this
/// primitive deliberately exposes only the inert parser policy.
pub fn CreateFragmentFromMarkup(
    document: &InteractionDocument,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
) -> Result<RetainedFragment, FragmentError> {
    CreateFragmentFromMarkupPolicy(
        document,
        markup,
        start,
        end,
        base_url,
        html::parser::parser_content_policy::ParserContentPolicy::DisallowScriptingAndPluginContent,
    )
}
/// InsertHTML uses serialization.h's default AllowScriptingContent policy;
/// clipboard imports retain their separate inert/sanitized parser policy.
pub(crate) fn CreateFragmentForInsertHTML(
    document: &InteractionDocument,
    markup: &[u16],
) -> Result<RetainedFragment, FragmentError> {
    CreateFragmentFromMarkupPolicy(
        document,
        markup,
        0,
        markup.len(),
        "",
        html::parser::parser_content_policy::ParserContentPolicy::AllowScriptingContent,
    )
}
fn CreateFragmentFromMarkupPolicy(
    document: &InteractionDocument,
    markup: &[u16],
    start: usize,
    end: usize,
    base_url: &str,
    policy: html::parser::parser_content_policy::ParserContentPolicy,
) -> Result<RetainedFragment, FragmentError> {
    let mut owner = document.borrow_mut();
    let native = owner.GetDocumentMut();
    if !native.IsActive() {
        return Err(FragmentError::DocumentChanged);
    }
    let identity = native.RootHandle().Identity();
    let root = html::markup_fragment::CreateFragmentFromMarkupWithContext(
        native, markup, start, end, base_url, policy, false,
    )
    .map_err(|_| FragmentError::InvalidOffsets)?
    .ok_or(FragmentError::ParseFailed)?;
    Ok(RetainedFragment {
        root: native.Node(root).Id(),
        document: document.clone(),
        identity,
    })
}

use interaction_default::core::editing::serializers::{
    markup_accumulator::{self, MarkupAttribute, MarkupDocument, MarkupNodeKind},
    markup_formatter::{ResolveUrls, SerializationType},
};
pub(super) struct NativeMarkupDocument<'a>(pub(super) &'a PersistentDocument);
impl MarkupDocument for NativeMarkupDocument<'_> {
    type Node = usize;
    fn SerializationType(&self) -> SerializationType {
        if self.0.DocumentType(self.0.Root()) == dom::persistent_document::DOMDocumentType::kHTML {
            SerializationType::Html
        } else {
            SerializationType::Xml
        }
    }
    fn ElementNamespace(&self, node: usize) -> &[u16] {
        &self.0.Node(node).QualifiedNameUTF16().namespace_uri
    }
    fn IsHTMLElement(&self, node: usize) -> bool {
        let node = self.0.Node(node);
        node.Type() == dom::DOMNodeType::kElement && node.Namespace() == DOMNamespace::kHTML
    }
    fn IsCDATASection(&self, node: usize) -> bool {
        self.0.Node(node).IsCDATASection()
    }
    fn XMLDeclaration(&self) -> Option<(&[u16], &[u16], Option<bool>)> {
        self.0
            .XMLDeclaration()
            .map(|d| (d.version.as_ref(), d.encoding.as_ref(), d.standalone))
    }
    fn DocumentTypeIdentifiers(&self, node: usize) -> (&[u16], &[u16]) {
        let attributes = self.0.Node(node).AttributesUTF16();
        (
            attributes.first().map_or(&[], |a| a.value.as_ref()),
            attributes.get(1).map_or(&[], |a| a.value.as_ref()),
        )
    }
    fn Kind(&self, node: usize) -> MarkupNodeKind {
        use dom::DOMNodeType as K;
        match self.0.Node(node).Type() {
            K::kElement => MarkupNodeKind::Element,
            K::kText => MarkupNodeKind::Text,
            K::kComment => MarkupNodeKind::Comment,
            K::kDocument => MarkupNodeKind::Document,
            K::kDocumentFragment => MarkupNodeKind::DocumentFragment,
            K::kDocumentType => MarkupNodeKind::DocumentType,
            K::kProcessingInstruction => MarkupNodeKind::ProcessingInstruction,
        }
    }
    fn QualifiedName(&self, node: usize) -> Vec<u16> {
        let node = self.0.Node(node);
        if node.Type() != dom::DOMNodeType::kElement {
            return node.NameUTF16().to_vec();
        }
        let name = node.QualifiedNameUTF16();
        let mut result = name.prefix.to_vec();
        if !result.is_empty() {
            result.push(58);
        }
        result.extend_from_slice(&name.local_name);
        result
    }
    fn Data(&self, node: usize) -> &[u16] {
        self.0.CharacterDataUTF16Slice(node)
    }
    fn ChildCount(&self, node: usize) -> usize {
        self.0.Node(node).Children().len()
    }
    fn ChildAt(&self, node: usize, index: usize) -> usize {
        self.0.Node(node).Children()[index]
    }
    fn TemplateContents(&self, node: usize) -> Option<usize> {
        self.0.TemplateContents(node)
    }
    fn IsHTMLVoidElement(&self, node: usize) -> bool {
        let node = self.0.Node(node);
        node.Namespace() == DOMNamespace::kHTML
            && matches!(
                node.Name(),
                "area"
                    | "base"
                    | "basefont"
                    | "bgsound"
                    | "br"
                    | "col"
                    | "embed"
                    | "frame"
                    | "hr"
                    | "img"
                    | "input"
                    | "keygen"
                    | "link"
                    | "meta"
                    | "param"
                    | "source"
                    | "track"
                    | "wbr"
            )
    }
    fn TextIsRaw(&self, node: usize) -> bool {
        self.0.Node(node).Parent().is_some_and(|parent| {
            let parent = self.0.Node(parent);
            parent.Namespace() == DOMNamespace::kHTML
                && (matches!(
                    parent.Name(),
                    "script" | "style" | "xmp" | "iframe" | "plaintext" | "noembed" | "noframes"
                ) || parent.Name() == "noscript" && self.0.ScriptingEnabled())
        })
    }
    fn AttributeCount(&self, node: usize) -> usize {
        self.0.Node(node).AttributesUTF16().len()
    }
    fn Attribute(&self, node: usize, index: usize) -> MarkupAttribute<'_> {
        let attribute = &self.0.Node(node).AttributesUTF16()[index];
        MarkupAttribute {
            prefix: &attribute.name.prefix,
            local_name: &attribute.name.local_name,
            namespace: &attribute.name.namespace_uri,
            value: &attribute.value,
        }
    }
    fn ResolvedAttributeValue(
        &self,
        node: usize,
        index: usize,
        resolve: ResolveUrls,
    ) -> Option<Vec<u16>> {
        if resolve == ResolveUrls::None
            || resolve == ResolveUrls::NonLocal
                && self
                    .0
                    .DocumentURL()
                    .get(..5)
                    .is_some_and(|scheme| scheme.eq_ignore_ascii_case("file:"))
            || !self.0.Node(node).IsURLAttribute(index)
        {
            return None;
        }
        let value = &self.0.Node(node).AttributesUTF16()[index].value;
        Some(
            self.0
                .CompleteURLValue(&String::from_utf16_lossy(value))
                .AsStr()
                .encode_utf16()
                .collect(),
        )
    }
    fn ElementIsValue(&self, node: usize) -> Option<&[u16]> {
        self.0.ElementIsValueUTF16(node)
    }
}
/// Canonical HTML/XML node serialization used by link/image drag DataTransfer.
pub fn CreateMarkup(
    document: &InteractionDocument,
    node: u64,
    resolve: ResolveUrls,
) -> Result<Vec<u16>, FragmentError> {
    let owner = document.borrow();
    let document = owner.GetDocument();
    let node = document
        .FindNodeById(node)
        .ok_or(FragmentError::InvalidSelection)?;
    Ok(markup_accumulator::CreateMarkup(
        &NativeMarkupDocument(document),
        node,
        false,
        resolve,
    ))
}

/// A call-scoped Style engine leaf. The retained result shares the compiled
/// declaration owner; Interaction neither reparses style text nor owns a resolver.
pub trait MarkupStyleQueries {
    fn Query(
        &self,
        operation: cssom::editing_style::SerializationStyleOperation,
    ) -> Result<cssom::CSSInlineStyle, cssom::editing_style::EditingStyleError>;
}

pub use super::page_editing_visible_position::{
    SerializeMarkupBody, SerializeMarkupRange, SerializeMarkupRangeWithAncestor,
};

/// FrameSelection::SelectedHtmlForClipboard's exact CreateMarkup options.
pub fn SelectedHtmlForClipboard(
    document: &dom::Document,
    layout: layoutng_assembly::layout_engine::LayoutOutput<'_>,
    styles: &dyn MarkupStyleQueries,
    selection: layoutng_assembly::editing_state::DOMSelection,
) -> Result<
    Vec<u16>,
    interaction_default::core::editing::serializers::styled_markup_serializer::SerializationError,
> {
    use interaction_default::core::editing::serializers::styled_markup_accumulator::CreateMarkupOptions;
    SerializeMarkupRange(
        document,
        layout,
        styles,
        selection,
        CreateMarkupOptions {
            annotate_for_interchange: true,
            resolve_urls: ResolveUrls::NonLocal,
            ignore_css_text_transforms: true,
            skip_unselectable_content: true,
            ..Default::default()
        },
    )
}
