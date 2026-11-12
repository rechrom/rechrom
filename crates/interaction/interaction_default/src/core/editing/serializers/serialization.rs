// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! serialization.cc:544–685 and editing_utilities.cc:1319–1338, pinned
//! 6c1d401fcca5. Fragment nodes belong to the destination Document throughout.
use super::super::editing_utilities::StringWithRebalancedWhitespace;

pub const APPLE_INTERCHANGE_NEWLINE: &str = "Apple-interchange-newline";

/// Values read from the live context range after preparing layout. The block
/// identity is borrowed from that document, not a cloned editing model.
#[derive(Clone, Debug)]
pub struct TextFragmentContext<N> {
    pub richly_editable: bool,
    pub preserve_newlines: bool,
    pub enclosing_block_to_clone: Option<N>,
    pub default_paragraph_separator: super::super::editor::EditorParagraphSeparator,
}
pub trait FragmentDocument {
    type Node: Clone;
    fn CreateDocumentFragment(&mut self) -> Self::Node;
    fn CreateText(&mut self, text: &[u16]) -> Self::Node;
    fn CreateElement(&mut self, name: &str) -> Self::Node;
    fn CloneElementWithoutChildren(&mut self, node: &Self::Node) -> Self::Node;
    fn SetAttribute(&mut self, node: &Self::Node, name: &str, value: &str);
    fn AppendChild(&mut self, parent: &Self::Node, child: &Self::Node);
}
fn InterchangeNewline<D: FragmentDocument>(document: &mut D) -> D::Node {
    let br = document.CreateElement("br");
    document.SetAttribute(&br, "class", APPLE_INTERCHANGE_NEWLINE);
    br
}
fn AppendText<D: FragmentDocument>(document: &mut D, parent: &D::Node, text: &[u16]) {
    let text = document.CreateText(text);
    document.AppendChild(parent, &text);
}
fn AppendTabs<D: FragmentDocument>(document: &mut D, parent: &D::Node, count: usize) {
    let span = document.CreateElement("span");
    document.SetAttribute(&span, "style", "white-space:pre");
    AppendText(document, &span, &vec![9; count]);
    document.AppendChild(parent, &span);
}
fn FillContainerFromString<D: FragmentDocument>(
    document: &mut D,
    paragraph: &D::Node,
    text: &[u16],
) {
    if text.is_empty() {
        let br = document.CreateElement("br");
        document.AppendChild(paragraph, &br);
        return;
    }
    debug_assert!(!text.contains(&10));
    let mut tabs = 0;
    let count = text.split(|c| *c == 9).count();
    for (index, part) in text.split(|c| *c == 9).enumerate() {
        if !part.is_empty() {
            if tabs != 0 {
                AppendTabs(document, paragraph, tabs);
                tabs = 0;
            }
            AppendText(
                document,
                paragraph,
                &StringWithRebalancedWhitespace(part, index == 0, index + 1 == count),
            );
        }
        if index + 1 != count {
            tabs += 1;
        } else if tabs != 0 {
            AppendTabs(document, paragraph, tabs);
        }
    }
}
/// Source NormalizeLineEndingsToLf: CRLF becomes LF and isolated CR becomes LF;
/// WTF-16 code units, including lone surrogates, otherwise remain unchanged.
fn NormalizeLineEndingsToLf(text: &[u16]) -> Vec<u16> {
    let mut result = Vec::with_capacity(text.len());
    let mut index = 0;
    while let Some(&unit) = text.get(index) {
        if unit == 13 {
            result.push(10);
            index += usize::from(text.get(index + 1) == Some(&10));
        } else {
            result.push(unit);
        }
        index += 1;
    }
    result
}
pub fn CreateFragmentFromText<D: FragmentDocument>(
    document: &mut D,
    context: &TextFragmentContext<D::Node>,
    text: &[u16],
) -> D::Node {
    let fragment = document.CreateDocumentFragment();
    if text.is_empty() {
        return fragment;
    }
    let text = NormalizeLineEndingsToLf(text);
    if !context.richly_editable || context.preserve_newlines {
        AppendText(document, &fragment, &text);
        if text.last() == Some(&10) {
            let br = InterchangeNewline(document);
            document.AppendChild(&fragment, &br);
        }
        return fragment;
    }
    if !text.contains(&10) {
        FillContainerFromString(document, &fragment, &text);
        return fragment;
    }
    let count = text.split(|c| *c == 10).count();
    for (index, line) in text.split(|c| *c == 10).enumerate() {
        let element = if line.is_empty() && index + 1 == count {
            InterchangeNewline(document)
        } else {
            let element = if let Some(block) = &context.enclosing_block_to_clone {
                document.CloneElementWithoutChildren(block)
            } else {
                document.CreateElement(super::super::editor::DefaultParagraphElementTag(
                    context.default_paragraph_separator,
                ))
            };
            FillContainerFromString(document, &element, line);
            element
        };
        document.AppendChild(&fragment, &element);
    }
    fragment
}

/// FrameSelection::ComputeRangeForSerialization, frame_selection.cc:1307–1329.
/// DOM endpoints convert to flat-tree positions before visible canonicalization;
/// rendering normalization uses the original caret owners for both endpoints.
pub fn ComputeRangeForSerialization<D, F>(
    dom: &D,
    flat: &F,
    anchor: &super::super::position::Position<D::Node>,
    focus: &super::super::position::Position<D::Node>,
) -> Option<(
    super::super::position::Position<D::Node>,
    super::super::position::Position<D::Node>,
)>
where
    D: super::super::position_conversion::PositionTreeScope,
    F: super::super::visible_position::VisiblePositionContext<Node = D::Node>
        + super::super::editing_boundaries::EditableBoundaryContext,
{
    use super::super::{
        compare_positions,
        position::{PositionWithAffinity, TextAffinity},
        position_conversion,
        visible_position::VisiblePosition,
        visible_selection,
    };
    let anchor = position_conversion::ToPositionInFlatTree(dom, flat, anchor);
    let focus = position_conversion::ToPositionInFlatTree(dom, flat, focus);
    if anchor.IsNull() || focus.IsNull() {
        return None;
    }
    let (start, end) = if compare_positions::CompareFlatPositions(flat, &anchor, &focus)?
        == std::cmp::Ordering::Greater
    {
        (focus, anchor)
    } else {
        (anchor, focus)
    };
    let start = VisiblePosition::Create(
        flat,
        PositionWithAffinity {
            position: start,
            affinity: TextAffinity::Downstream,
        },
    )
    .DeepEquivalent()
    .clone();
    let end = VisiblePosition::Create(
        flat,
        PositionWithAffinity {
            position: end,
            affinity: TextAffinity::Downstream,
        },
    )
    .DeepEquivalent()
    .clone();
    if start.IsNull()
        || end.IsNull()
        || compare_positions::CompareFlatPositions(flat, &start, &end)? != std::cmp::Ordering::Less
    {
        return None;
    }
    visible_selection::NormalizeSelectionRange(flat, &start, &end)
}

/// A canonical inert staging Document supplies parser/style lifecycle leaves.
/// The source loop owns stabilization and reparses; the host owns the single
/// DocumentFragment being processed and never returns a parallel node model.
pub trait MarkupSanitizationContext {
    type Fragment;
    type Error;
    fn ParseWithContext(
        &mut self,
        markup: &[u16],
        start: usize,
        end: usize,
    ) -> Result<Self::Fragment, Self::Error>;
    fn ContainsStyleElements(&self, fragment: &Self::Fragment) -> bool;
    fn StripSvgUseNonLocalHrefs(&mut self, fragment: &Self::Fragment) -> bool;
    fn Serialize(
        &mut self,
        fragment: &Self::Fragment,
        inline_styles: bool,
    ) -> Result<Vec<u16>, Self::Error>;
    fn ParseAndSerializeFinal(
        &mut self,
        markup: &[u16],
        base_url: &str,
    ) -> Result<Vec<u16>, Self::Error>;
}
/// serialization.cc:854–937. A stable source string is mandatory: successful
/// parsing alone does not establish markup sanitization.
pub fn CreateStrictlyProcessedMarkupWithContext<C: MarkupSanitizationContext>(
    context: &mut C,
    raw: &[u16],
    mut start: usize,
    mut end: usize,
    base_url: &str,
) -> Result<Option<Vec<u16>>, C::Error> {
    if raw.is_empty() {
        return Ok(None);
    }
    let mut last_markup = Vec::new();
    let mut markup = raw.to_vec();
    for _ in 0..16 {
        if last_markup == markup {
            break;
        }
        last_markup = markup;
        let fragment = context.ParseWithContext(&last_markup, start, end)?;
        let needs_style = context.ContainsStyleElements(&fragment);
        let stripped_use = context.StripSvgUseNonLocalHrefs(&fragment);
        markup = context.Serialize(&fragment, needs_style || stripped_use)?;
        start = 0;
        end = markup.len();
    }
    if last_markup == markup {
        context.ParseAndSerializeFinal(&markup, base_url).map(Some)
    } else {
        Ok(None)
    }
}
