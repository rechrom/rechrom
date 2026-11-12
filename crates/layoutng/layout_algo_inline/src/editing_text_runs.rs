//! Source: core/editing/iterators/text_iterator_text_node_handler.cc:24-210;
//! core/layout/inline/offset_mapping.cc:224-233,301-355.
//! The handler keeps a borrowed span and index into the canonical mapping.
//! Clipping produces only a per-unit view; it never copies a mapping/tree.
use crate::internal::inline_node::InlineNode;
use crate::internal::layout_node_metadata::{Node, Text};
use crate::internal::layout_object::{AssociatedLayoutObjectOfDefault, LayoutObject};
use crate::internal::layout_text::LayoutText;
use crate::layout_text_fragment::LayoutTextFragment;
use crate::offset_mapping::{OffsetMapping, OffsetMappingUnit};
use foundation::{DynamicTo, EDisplay, EVisibility, String};

/// Only behavior inputs consumed by Chromium's text-node handler cross this
/// boundary. DOM traversal and the remaining TextIterator behavior stay with
/// the editing owner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditingTextIteratorBehavior {
    pub ignores_style_visibility: bool,
    pub ignores_css_text_transforms: bool,
    pub emits_original_text: bool,
    pub emits_space_for_nbsp: bool,
}

/// One source EmitText invocation. `text` retains its immutable String owner;
/// the UTF-16 substring is [text_start,text_end), associated with the original
/// DOM range [dom_start,dom_end). No owned document/paragraph model is built.
#[derive(Clone, Debug)]
pub struct EditingTextRun {
    pub node_id: u64,
    pub text: String,
    pub text_start: u32,
    pub text_end: u32,
    pub dom_start: u32,
    pub dom_end: u32,
    pub next_offset: u32,
    /// Source TextIteratorTextState::IsTextSecurityNode reads the DOM anchor's
    /// own LayoutObject, distinct from a first-letter unit's LayoutText style.
    pub is_text_security: bool,
}

/// Scoped to the borrowed LayoutOutput. It retains no LayoutObject, DOM or
/// OffsetMapping owner beyond that result's existing lifetime.
pub struct EditingTextRunCursor<'a> {
    node: &'a Node,
    offset: u32,
    end_offset: u32,
    units: &'a [OffsetMappingUnit],
    unit_index: usize,
    units_start_offset: u32,
    behavior: EditingTextIteratorBehavior,
}

fn ForceGetMapping<'a>(node: &'a Node, offset: u32) -> Option<&'a OffsetMapping> {
    // Source ForceGetFor(Position(text,offset)): the associated first-letter
    // object may live in a different inline formatting context from the
    // remaining text. Always resolve from the current DOM offset.
    let object =
        unsafe { AssociatedLayoutObjectOfDefault(node, i32::try_from(offset).ok()?).as_ref() }?;
    let context = OffsetMapping::GetInlineFormattingContextOfObject(object);
    if context.is_null() {
        return None;
    }
    unsafe { InlineNode::GetOffsetMapping(context).as_ref() }
}

fn ShouldSkipInvisible(node: &Node, offset: u32, ignores_visibility: bool) -> bool {
    let Some(object) = (unsafe { AssociatedLayoutObjectOfDefault(node, offset as i32).as_ref() })
    else {
        return true;
    };
    if object.StyleRef().Display() == EDisplay::kNone {
        return true;
    }
    !ignores_visibility && object.StyleRef().Visibility() != EVisibility::kVisible
}

struct ClippedUnit<'a> {
    source: &'a OffsetMappingUnit,
    dom_start: u32,
    dom_end: u32,
    text_start: u32,
    text_end: u32,
}
impl<'a> ClippedUnit<'a> {
    fn new(source: &'a OffsetMappingUnit, start: u32, end: u32) -> Self {
        let dom_start = source.DOMStart().max(start);
        let dom_end = source.DOMEnd().min(end);
        debug_assert!(dom_start <= dom_end);
        Self {
            source,
            dom_start,
            dom_end,
            text_start: source.ConvertDOMOffsetToTextContent(dom_start),
            text_end: source.ConvertDOMOffsetToTextContent(dom_end),
        }
    }
}

fn OriginalCompleteText(text: &LayoutText) -> String {
    let fragment = DynamicTo::<LayoutTextFragment>(text as *const LayoutText);
    if let Some(fragment) = unsafe { fragment.as_ref() } {
        fragment.CompleteText()
    } else {
        text.OriginalText()
    }
}
fn CopySubstring(text: &String, start: u32, end: u32) -> String {
    // This allocation is Chromium's behavior-specific String::substr, not a
    // copy of the resident mapping. Ordinary runs keep the shared full String.
    // Source String::substr checks the starting offset and StringImpl's
    // Substring clamps only the requested length.
    assert!(start <= text.length());
    debug_assert!(start <= end);
    let end = end.min(text.length());
    if start == 0 && end == text.length() {
        return text.clone();
    }
    let units = text.Span16().unwrap_or_default();
    String::Create8BitIfPossible(&units[start as usize..end as usize])
}
fn ReplaceUnits(text: &String, replace: impl Fn(u16) -> u16) -> String {
    let source = text.Span16().unwrap_or_default();
    // StringImpl::Replace retains its owner if the character is absent.
    let Some(first) = source.iter().position(|&unit| replace(unit) != unit) else {
        return text.clone();
    };
    let mut units = Vec::with_capacity(source.len());
    units.extend_from_slice(&source[..first]);
    units.extend(source[first..].iter().copied().map(replace));
    String::Create8BitIfPossible(&units)
}

fn ComputeEmission(
    mapping: &OffsetMapping,
    unit: &ClippedUnit<'_>,
    behavior: EditingTextIteratorBehavior,
) -> (String, u32, u32) {
    let object = unit.source.GetLayoutObject();
    let text = unsafe { &*(object as *const LayoutObject as *const LayoutText) };
    debug_assert!(object.IsText());
    let mut string = mapping.GetText().clone();
    let mut start = unit.text_start;
    let mut end = unit.text_end;
    // Source's security guard applies to the CSS-transform bypass only;
    // EmitsOriginalText is its own explicit source behavior below.
    if behavior.ignores_css_text_transforms && text.HasTextTransform() && !text.IsSecure() {
        string = CopySubstring(&OriginalCompleteText(text), unit.dom_start, unit.dom_end);
        if !text.StyleRef().ShouldPreserveBreaks() {
            string = ReplaceUnits(&string, |unit| if unit == 0x0a { 0x20 } else { unit });
            string = ReplaceUnits(&string, |unit| if unit == 0x09 { 0x20 } else { unit });
        }
        start = 0;
        end = string.length();
    }
    if behavior.emits_original_text {
        string = CopySubstring(&OriginalCompleteText(text), unit.dom_start, unit.dom_end);
        start = 0;
        end = string.length();
    }
    if behavior.emits_space_for_nbsp {
        string = ReplaceUnits(&CopySubstring(&string, start, end), |unit| {
            if unit == 0xa0 {
                0x20
            } else {
                unit
            }
        });
        start = 0;
        end = string.length();
    }
    (string, start, end)
}

impl<'a> EditingTextRunCursor<'a> {
    pub(crate) fn New(
        node: &'a Node,
        start: u32,
        end: u32,
        behavior: EditingTextIteratorBehavior,
    ) -> Option<Self> {
        debug_assert!(start <= end);
        if !node.IsTextNode() || start > end {
            return None;
        }
        debug_assert!(unsafe { DynamicTo::<Text>(node).as_ref() }
            .is_some_and(|text| end <= text.data().length()));
        let mapping = ForceGetMapping(node, start)?;
        let mut cursor = Self {
            node,
            offset: start,
            end_offset: end,
            units: &[],
            unit_index: 0,
            units_start_offset: start,
            behavior,
        };
        cursor.ResetUnits(mapping);
        Some(cursor)
    }
    fn ResetUnits(&mut self, mapping: &'a OffsetMapping) {
        let units = mapping.GetMappingUnitsForNode(self.node);
        // Source GetMappingUnitsForDOMRange includes boundary-touching units.
        let first = units.partition_point(|unit| unit.DOMEnd() < self.offset);
        let last = units.partition_point(|unit| unit.DOMStart() <= self.end_offset);
        self.units = if first <= last {
            &units[first..last]
        } else {
            &[]
        };
        self.unit_index = 0;
        self.units_start_offset = self.offset;
    }
    pub fn NextOffset(&self) -> u32 {
        self.offset
    }
    pub fn NextRun(&mut self) -> Option<EditingTextRun> {
        while self.offset < self.end_offset {
            let Some(mapping) = ForceGetMapping(self.node, self.offset) else {
                self.offset = self.end_offset;
                return None;
            };
            if self.unit_index >= self.units.len() {
                self.ResetUnits(mapping);
            }
            let initial = self.offset;
            while let Some(unit) = self.units.get(self.unit_index) {
                let unit = ClippedUnit::new(unit, self.units_start_offset, self.end_offset);
                if unit.text_start == unit.text_end
                    || ShouldSkipInvisible(
                        self.node,
                        unit.dom_start,
                        self.behavior.ignores_style_visibility,
                    )
                {
                    self.offset = unit.dom_end;
                    self.unit_index += 1;
                    continue;
                }
                let (text, text_start, text_end) = ComputeEmission(mapping, &unit, self.behavior);
                self.offset = unit.dom_end;
                self.unit_index += 1;
                return Some(EditingTextRun {
                    node_id: self.node.InputId(),
                    text,
                    text_start,
                    text_end,
                    dom_start: unit.dom_start,
                    dom_end: unit.dom_end,
                    next_offset: self.offset,
                    is_text_security: unsafe { self.node.GetLayoutObject().as_ref() }.is_some_and(
                        |object| {
                            object.StyleRef().TextSecurity() != foundation::ETextSecurity::kNone
                        },
                    ),
                });
            }
            // Match source's explicit no-progress exit across first-letter
            // mappings. It never substitutes DOM text for an unavailable run.
            if self.offset == initial {
                self.offset = self.end_offset;
                return None;
            }
        }
        None
    }
}
impl Iterator for EditingTextRunCursor<'_> {
    type Item = EditingTextRun;
    fn next(&mut self) -> Option<Self::Item> {
        self.NextRun()
    }
}
