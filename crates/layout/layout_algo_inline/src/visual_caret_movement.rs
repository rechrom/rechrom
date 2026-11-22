//! Source: core/editing/visual_caret_movement.cc.
//!
//! Visual movement deliberately resolves the resident FragmentItems directly.
//! It does not pass a bidi-boundary position through visible-position
//! canonicalization, which would discard the fragment identity selected by the
//! previous keystroke.
use super::*;
use crate::internal::editing::forward::{
    Position, PositionAnchorType, PositionWithAffinity, TextAffinity,
};
use crate::offset_mapping::OffsetMapping;
use foundation::{DynamicTo, TextDirection};
use layout_fragment::inline_cursor::InlineCursor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualCaretMoveDirection {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VisualCaretMoveResult {
    pub position: PositionWithAffinity,
    pub bidi_level: Option<u8>,
    pub entered_bidi_run: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditingCaretDirections {
    pub resolved_direction: TextDirection,
    pub line_base_direction: TextDirection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AtomicInlineEdge {
    Before,
    After,
}

#[derive(Clone, Default)]
struct VisualFragment {
    cursor: InlineCursor,
}

impl VisualFragment {
    fn new(cursor: &InlineCursor) -> Self {
        if cursor.IsNull() {
            return Self::default();
        }
        let mut line = cursor.clone();
        line.MoveToContainingLine();
        let mut line_cursor = line.CursorForDescendants();
        line_cursor.MoveToCursor(cursor);
        Self {
            cursor: line_cursor,
        }
    }
    fn IsNull(&self) -> bool {
        self.cursor.IsNull()
    }
    fn IsText(&self) -> bool {
        self.cursor.IsNotNull() && self.cursor.Current().IsText()
    }
    fn IsAtomicInline(&self) -> bool {
        self.cursor.IsNotNull() && self.cursor.Current().IsAtomicInline()
    }
    fn Direction(&self) -> TextDirection {
        if self.IsText() || self.IsAtomicInline() {
            return self.cursor.Current().ResolvedDirection();
        }
        let mut line = self.cursor.clone();
        line.MoveToContainingLine();
        line.Current().BaseDirection()
    }
    fn BidiLevel(&self) -> u8 {
        self.cursor.Current().BidiLevel()
    }
    fn TextStartOffset(&self) -> u32 {
        self.cursor.Current().TextStartOffset()
    }
    fn TextEndOffset(&self) -> u32 {
        self.cursor.Current().TextEndOffset()
    }
    fn GetLayoutObject(&self) -> *const crate::internal::layout_object::LayoutObject {
        self.cursor.Current().GetLayoutObject()
    }
    fn NextLeafOnLine(&self) -> Self {
        let mut cursor = self.cursor.clone();
        cursor.MoveToNextInlineLeafIgnoringLineBreak();
        Self::new(&cursor)
    }
    fn PrevLeafOnLine(&self) -> Self {
        let mut cursor = self.cursor.clone();
        cursor.MoveToPreviousInlineLeafIgnoringLineBreak();
        Self::new(&cursor)
    }
}

fn VisualStartOffset(cursor: &InlineCursor) -> u32 {
    if cursor.Current().ResolvedDirection() == TextDirection::kLtr {
        cursor.Current().TextStartOffset()
    } else {
        cursor.Current().TextEndOffset()
    }
}

fn VisualEndOffset(cursor: &InlineCursor) -> u32 {
    if cursor.Current().ResolvedDirection() == TextDirection::kLtr {
        cursor.Current().TextEndOffset()
    } else {
        cursor.Current().TextStartOffset()
    }
}

fn FindFragmentForPosition(
    object: &crate::internal::layout_object::LayoutObject,
    position: &Position,
    bidi_level: Option<u8>,
) -> Option<(VisualFragment, u32)> {
    if position.IsNull() {
        return None;
    }
    let node = unsafe { object.NonPseudoNode().as_ref() }?;
    if position.node_id != Some(node.InputId()) {
        return None;
    }
    if !node.IsTextNode() {
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            object,
            &PositionWithAffinity {
                position: *position,
                affinity: TextAffinity::Downstream,
            },
            crate::EditingPositionOptions::default(),
        )?;
        let fragment = VisualFragment::new(&caret.cursor);
        let offset = if fragment.IsText() {
            caret
                .text_offset
                .unwrap_or_else(|| fragment.TextStartOffset())
        } else {
            0
        };
        return Some((fragment, offset));
    }
    let mapping = unsafe { OffsetMapping::GetForLayoutObject(object).as_ref() }?;
    let offset = mapping.GetTextContentOffsetForNode(node, position)?;
    let mut cursor = InlineCursor::default();
    cursor.MoveToLayoutObject(object);
    let mut first_match = None;
    let mut level_match = None;
    while cursor.IsNotNull() {
        if cursor.Current().IsText() {
            let start = cursor.Current().TextStartOffset();
            let end = cursor.Current().TextEndOffset();
            if offset >= start && offset <= end {
                if offset > start && offset < end {
                    return Some((VisualFragment::new(&cursor), offset));
                }
                first_match.get_or_insert_with(|| cursor.clone());
                if bidi_level == Some(cursor.Current().BidiLevel()) {
                    level_match.get_or_insert_with(|| cursor.clone());
                }
            }
        }
        cursor.MoveToNextForSameLayoutObject();
    }
    level_match
        .or(first_match)
        .map(|cursor| (VisualFragment::new(&cursor), offset))
}

fn MakePositionWithAffinity(
    fragment: &VisualFragment,
    offset: u32,
    affinity: TextAffinity,
) -> PositionWithAffinity {
    let Some(mapping) =
        (unsafe { OffsetMapping::GetForLayoutObject(fragment.GetLayoutObject()).as_ref() })
    else {
        return PositionWithAffinity::default();
    };
    let position = mapping.GetFirstPosition(offset);
    if position.IsNull() {
        PositionWithAffinity::default()
    } else {
        PositionWithAffinity { position, affinity }
    }
}

fn MakeAtomicInlinePosition(
    fragment: &VisualFragment,
    edge: AtomicInlineEdge,
    affinity: TextAffinity,
) -> PositionWithAffinity {
    let Some(object) = (unsafe { fragment.GetLayoutObject().as_ref() }) else {
        return PositionWithAffinity::default();
    };
    let Some(node) = (unsafe { object.GetNode().as_ref() }) else {
        return PositionWithAffinity::default();
    };
    PositionWithAffinity {
        position: Position::WithAnchor(
            node.InputId(),
            0,
            match edge {
                AtomicInlineEdge::Before => PositionAnchorType::BeforeAnchor,
                AtomicInlineEdge::After => PositionAnchorType::AfterAnchor,
            },
        ),
        affinity,
    }
}

fn GraphemeOffset(fragment: &VisualFragment, offset: u32, next: bool) -> u32 {
    let Some(object) = (unsafe { fragment.GetLayoutObject().as_ref() }) else {
        return offset;
    };
    let layout_text = DynamicTo::<crate::internal::layout_text::LayoutText>(object);
    let Some(layout_text) = (unsafe { layout_text.as_ref() }) else {
        return offset;
    };
    let Some(mapping) = (unsafe { OffsetMapping::GetForLayoutObject(object).as_ref() }) else {
        return offset;
    };
    let Some(unit) = mapping
        .GetMappingUnitsForLayoutObject(object)
        .iter()
        .find(|unit| unit.TextContentStart() <= offset && offset <= unit.TextContentEnd())
    else {
        return offset;
    };
    let dom_offset = unit.ConvertTextContentToFirstDOMOffset(offset);
    let text = layout_text.OriginalText();
    let Some(units) = text.Span16() else {
        return offset;
    };
    let iterator = font_engine::text::implementation::character_break_iterator::CharacterBreakIterator::from_utf16(units);
    let boundary = if next {
        iterator.Following(dom_offset.min(i32::MAX as u32) as i32)
    } else {
        iterator.Preceding(dom_offset.min(i32::MAX as u32) as i32)
    };
    let Ok(boundary) = u32::try_from(boundary) else {
        return offset;
    };
    if (next && boundary <= dom_offset) || (!next && boundary >= dom_offset) {
        return offset;
    }
    if boundary < unit.DOMStart() || boundary > unit.DOMEnd() {
        return offset;
    }
    unit.ConvertDOMOffsetToTextContent(boundary)
}

fn MoveToAdjacentVisualLine(cursor: &InlineCursor, next: bool) -> VisualCaretMoveResult {
    let mut line = cursor.clone();
    line.MoveToContainingLine();
    if next {
        line.MoveToNextLine();
    } else {
        line.MoveToPreviousLine();
    }
    if line.IsNull() {
        return VisualCaretMoveResult::default();
    }
    let mut descendants = line.CursorForDescendants();
    descendants.MoveToNextInlineLeafIgnoringLineBreak();
    if descendants.IsNull() {
        return VisualCaretMoveResult::default();
    }
    let fragment = if next {
        VisualFragment::new(&descendants)
    } else {
        let mut last = descendants.clone();
        loop {
            let mut following = last.clone();
            following.MoveToNextInlineLeafIgnoringLineBreak();
            if following.IsNull() {
                break;
            }
            last = following;
        }
        VisualFragment::new(&last)
    };
    if fragment.IsText() {
        let offset = if next {
            VisualStartOffset(&fragment.cursor)
        } else {
            VisualEndOffset(&fragment.cursor)
        };
        return MakeTextResult(
            &fragment,
            offset,
            if next {
                TextAffinity::Downstream
            } else {
                TextAffinity::Upstream
            },
        );
    }
    if fragment.IsAtomicInline() {
        return MakeAtomicResult(
            &fragment,
            if next {
                AtomicInlineEdge::Before
            } else {
                AtomicInlineEdge::After
            },
            if next {
                TextAffinity::Downstream
            } else {
                TextAffinity::Upstream
            },
        );
    }
    VisualCaretMoveResult::default()
}

#[derive(Default)]
struct VisualMovementState {
    fragment: VisualFragment,
    offset: u32,
    is_after_atomic: bool,
    is_before_atomic: bool,
}

fn ResolveStartState(
    object: &crate::internal::layout_object::LayoutObject,
    position: &PositionWithAffinity,
    bidi_level: Option<u8>,
) -> VisualMovementState {
    let mut state = VisualMovementState::default();
    if let Some((fragment, offset)) =
        FindFragmentForPosition(object, &position.position, bidi_level)
    {
        state.fragment = fragment;
        state.offset = offset;
        if state.fragment.IsText() {
            return state;
        }
    }
    let Some(caret) = crate::layout_algo_inline_editing_caret::ResolveForObject(
        object,
        position,
        crate::EditingPositionOptions::default(),
    ) else {
        return state;
    };
    state.fragment = VisualFragment::new(&caret.cursor);
    match caret.position_type {
        crate::layout_algo_inline_editing_caret::InlineCaretPositionType::AtTextOffset => {
            state.offset = caret.text_offset.unwrap_or(0);
        }
        crate::layout_algo_inline_editing_caret::InlineCaretPositionType::AfterBox => {
            state.is_after_atomic = true;
            if state.fragment.IsText() {
                state.offset = state.fragment.TextEndOffset();
            }
        }
        crate::layout_algo_inline_editing_caret::InlineCaretPositionType::BeforeBox => {
            state.is_before_atomic = true;
            if state.fragment.IsText() {
                state.offset = state.fragment.TextStartOffset();
            }
        }
    }
    state
}

fn MakeTextResult(
    fragment: &VisualFragment,
    offset: u32,
    affinity: TextAffinity,
) -> VisualCaretMoveResult {
    let position = MakePositionWithAffinity(fragment, offset, affinity);
    VisualCaretMoveResult {
        bidi_level: (!position.position.IsNull()).then(|| fragment.BidiLevel()),
        position,
        entered_bidi_run: false,
    }
}

fn MakeAtomicResult(
    fragment: &VisualFragment,
    edge: AtomicInlineEdge,
    affinity: TextAffinity,
) -> VisualCaretMoveResult {
    let position = MakeAtomicInlinePosition(fragment, edge, affinity);
    VisualCaretMoveResult {
        bidi_level: (!position.position.IsNull()).then(|| fragment.BidiLevel()),
        position,
        entered_bidi_run: false,
    }
}

impl LayoutNode<'_> {
    pub fn EditingCaretDirections(
        &self,
        position: &PositionWithAffinity,
        options: crate::EditingPositionOptions,
    ) -> Option<EditingCaretDirections> {
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let resolved_direction = caret.cursor.Current().ResolvedDirection();
        let mut line = caret.cursor;
        line.MoveToContainingLine();
        line.IsNotNull().then(|| EditingCaretDirections {
            resolved_direction,
            line_base_direction: line.Current().BaseDirection(),
        })
    }

    pub fn EditingCaretLineDirectionCoordinateIgnoringTransforms(
        &self,
        position: &PositionWithAffinity,
        options: crate::EditingPositionOptions,
        bar_width: f64,
    ) -> Option<foundation::LayoutUnit> {
        use crate::internal::map_coordinates_flags::{MapCoordinatesFlags, MapCoordinatesMode};
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let mut rect = crate::layout_algo_inline_editing_caret_rect::BarCaretRect(
            &caret,
            foundation::LayoutUnit::from_f64(bar_width),
        )?;
        if rect.IsEmpty() {
            return None;
        }
        if caret.cursor.Current().IsAtomicInline() {
            rect.offset -= caret.cursor.Current().OffsetInContainerFragment();
        }
        let object = unsafe { caret.cursor.Current().GetLayoutObject().as_ref() }?;
        let local = if object.HasFlippedBlocksWritingMode() {
            foundation::PhysicalOffset::new(rect.Right(), rect.offset.top)
        } else {
            rect.MinXMinYCorner()
        };
        let absolute = object.LocalToAbsolutePointPhysical(
            &local,
            MapCoordinatesFlags::from_mode(MapCoordinatesMode::kIgnoreTransforms),
        );
        Some(if object.IsHorizontalWritingMode() {
            absolute.left
        } else {
            absolute.top
        })
    }

    pub fn MoveCaretVisually(
        &self,
        position: &PositionWithAffinity,
        input_bidi_level: Option<u8>,
        entered_bidi_run: bool,
        direction: VisualCaretMoveDirection,
    ) -> VisualCaretMoveResult {
        let _scope = crate::internal::layout_pass_scope::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let right = direction == VisualCaretMoveDirection::Right;
        let state = ResolveStartState(self.object, position, input_bidi_level);
        if state.fragment.IsNull() {
            return VisualCaretMoveResult::default();
        }
        let fragment = state.fragment;
        let offset = state.offset;

        if ((right && state.is_after_atomic) || (!right && state.is_before_atomic))
            && fragment.IsAtomicInline()
        {
            let adjacent = if right {
                fragment.NextLeafOnLine()
            } else {
                fragment.PrevLeafOnLine()
            };
            if adjacent.IsNull() {
                return MoveToAdjacentVisualLine(&fragment.cursor, right);
            }
            if adjacent.IsText() {
                return MakeTextResult(
                    &adjacent,
                    if right {
                        VisualStartOffset(&adjacent.cursor)
                    } else {
                        VisualEndOffset(&adjacent.cursor)
                    },
                    if right {
                        TextAffinity::Downstream
                    } else {
                        TextAffinity::Upstream
                    },
                );
            }
            if adjacent.IsAtomicInline() {
                return MakeAtomicResult(
                    &adjacent,
                    if right {
                        AtomicInlineEdge::Before
                    } else {
                        AtomicInlineEdge::After
                    },
                    if right {
                        TextAffinity::Downstream
                    } else {
                        TextAffinity::Upstream
                    },
                );
            }
            return VisualCaretMoveResult::default();
        }

        if fragment.IsText() {
            let ltr = fragment.Direction() == TextDirection::kLtr;
            let advance_dom = right == ltr;
            let in_fragment = if advance_dom {
                offset < fragment.TextEndOffset()
            } else {
                offset > fragment.TextStartOffset()
            };
            if in_fragment {
                let moved = GraphemeOffset(&fragment, offset, advance_dom);
                let valid = if advance_dom {
                    moved > offset && moved <= fragment.TextEndOffset()
                } else {
                    moved < offset && moved >= fragment.TextStartOffset()
                };
                if valid {
                    let mut result = MakeTextResult(
                        &fragment,
                        moved,
                        if advance_dom {
                            TextAffinity::Downstream
                        } else {
                            TextAffinity::Upstream
                        },
                    );
                    result.entered_bidi_run = entered_bidi_run;
                    return result;
                }
            }
        }

        let adjacent = if right {
            fragment.NextLeafOnLine()
        } else {
            fragment.PrevLeafOnLine()
        };
        if !adjacent.IsNull() {
            if adjacent.IsText() {
                let entry = if right {
                    VisualStartOffset(&adjacent.cursor)
                } else {
                    VisualEndOffset(&adjacent.cursor)
                };
                let ltr = adjacent.Direction() == TextDirection::kLtr;
                let affinity = if right == ltr {
                    TextAffinity::Downstream
                } else {
                    TextAffinity::Upstream
                };
                let bidi_boundary =
                    fragment.IsText() && fragment.BidiLevel() != adjacent.BidiLevel();
                if bidi_boundary {
                    let advance_dom = right == ltr;
                    let moved = GraphemeOffset(&adjacent, entry, advance_dom);
                    if moved != entry {
                        let mut result = MakeTextResult(&adjacent, moved, affinity);
                        result.entered_bidi_run = !entered_bidi_run;
                        return result;
                    }
                    let beyond = if right {
                        adjacent.NextLeafOnLine()
                    } else {
                        adjacent.PrevLeafOnLine()
                    };
                    if beyond.IsText() {
                        let mut result = MakeTextResult(
                            &beyond,
                            if right {
                                VisualStartOffset(&beyond.cursor)
                            } else {
                                VisualEndOffset(&beyond.cursor)
                            },
                            if right == (beyond.Direction() == TextDirection::kLtr) {
                                TextAffinity::Downstream
                            } else {
                                TextAffinity::Upstream
                            },
                        );
                        result.entered_bidi_run = false;
                        return result;
                    }
                } else {
                    let mut result = MakeTextResult(&adjacent, entry, affinity);
                    if result.position.position == position.position {
                        let advance_dom = right == ltr;
                        let moved = GraphemeOffset(&adjacent, entry, advance_dom);
                        if moved != entry {
                            result = MakeTextResult(&adjacent, moved, affinity);
                        }
                    }
                    result.entered_bidi_run = false;
                    return result;
                }
            }
            if adjacent.IsAtomicInline() {
                return MakeAtomicResult(
                    &adjacent,
                    if right {
                        AtomicInlineEdge::Before
                    } else {
                        AtomicInlineEdge::After
                    },
                    if right {
                        TextAffinity::Downstream
                    } else {
                        TextAffinity::Upstream
                    },
                );
            }
        }

        let cross_line = MoveToAdjacentVisualLine(&fragment.cursor, right);
        if !cross_line.position.position.IsNull() {
            return cross_line;
        }
        VisualCaretMoveResult {
            position: PositionWithAffinity {
                position: position.position,
                affinity: if right {
                    TextAffinity::Downstream
                } else {
                    TextAffinity::Upstream
                },
            },
            bidi_level: input_bidi_level,
            entered_bidi_run: false,
        }
    }
}
