//! Native visible-position leaves for SelectionController. A query borrows
//! the current DOM and resident Layout result only until it returns.
use super::*;
use font_engine::text::implementation::word_break_iterator::SentenceBreakIterator;
use font_engine::text::implementation::word_break_iterator::{kTextBreakDone, WordBreakIterator};
use interaction_default::core::editing::{
    editing_strategy::{self, SelectionStyleContext},
    position_units_word::{self, WordBreakContext},
    selection_adjuster::{WordGranularityContext, WordInclusion, WordSide},
    selection_boundary_adjuster::FlatSelectionBoundaryContext,
    text_offset_mapping::InlineContentsContext,
    visible_selection, visible_units_commands,
};
use std::ffi::CString;

pub enum SelectionGeometryQuery {
    IsValid(EditingPositionWithAffinity),
    Visible(EditingPositionWithAffinity),
    VisibleSelection {
        selection: DOMSelection,
        word: bool,
    },
    PlainText(DOMSelection),
    FirstPosition(u64),
    UserSelectAll(u64),
    LastPositionInEditableRoot {
        position: EditingPosition,
        selection: DOMSelection,
    },
    SkipWhitespace(EditingPositionWithAffinity),
    Compare(EditingPosition, EditingPosition),
    NodeSelectable(u64),
    NodeCanStartSelection(u64),
    NodeSelectionLocked(u64),
}
pub enum SelectionGeometryResult {
    Position(Option<EditingPositionWithAffinity>),
    Selection(Option<DOMSelection>),
    Bounds(Option<(EditingPositionWithAffinity, EditingPositionWithAffinity)>),
    Text(Vec<u16>),
    Compare(Option<Ordering>),
    Boolean(bool),
}
impl InlineContentsContext for EditingPositionContext<'_> {
    fn InlineContentsRange(
        &self,
        c: &Self::InlineContents,
    ) -> Option<(Position<u64>, Position<u64>)> {
        let (a, b) = c.GetRange()?;
        let source = |p| {
            SourcePosition(EditingPositionWithAffinity {
                position: p,
                affinity: EditingTextAffinity::Downstream,
            })
            .position
        };
        Some((source(a), source(b)))
    }
    fn FirstPositionAfterBlockFlow(&self, c: &Self::InlineContents) -> Position<u64> {
        SourcePosition(EditingPositionWithAffinity {
            position: c.FirstPositionAfterBlockFlow(),
            affinity: EditingTextAffinity::Downstream,
        })
        .position
    }
    fn LastPositionBeforeBlockFlow(&self, c: &Self::InlineContents) -> Position<u64> {
        SourcePosition(EditingPositionWithAffinity {
            position: c.LastPositionBeforeBlockFlow(),
            affinity: EditingTextAffinity::Downstream,
        })
        .position
    }
}
impl WordBreakContext for EditingPositionContext<'_> {
    fn FollowingWordBoundary(&self, text: &[u16], offset: usize) -> Option<usize> {
        WordBreakIterator(text, self.ui_locale?, |iterator| {
            Some(iterator.Following(i32::try_from(offset).ok()?))
        })
        .flatten()
        .filter(|boundary| *boundary != kTextBreakDone)
        .map(|boundary| boundary as usize)
    }
    fn PrecedingWordBoundary(&self, text: &[u16], offset: usize) -> Option<usize> {
        WordBreakIterator(text, self.ui_locale?, |iterator| {
            Some(iterator.Preceding(i32::try_from(offset).ok()?))
        })
        .flatten()
        .filter(|boundary| *boundary != kTextBreakDone)
        .map(|boundary| boundary as usize)
    }
}
impl interaction_default::core::editing::position_units_sentence::SentenceBreakContext
    for EditingPositionContext<'_>
{
    fn FollowingSentenceBoundary(&self, text: &[u16], offset: usize) -> Option<usize> {
        SentenceBreakIterator(text, self.ui_locale?, |iterator| {
            Some(iterator.Following(i32::try_from(offset).ok()?))
        })
        .flatten()
        .filter(|boundary| *boundary != kTextBreakDone)
        .map(|boundary| boundary as usize)
    }
    fn PrecedingSentenceBoundary(&self, text: &[u16], offset: usize) -> Option<usize> {
        SentenceBreakIterator(text, self.ui_locale?, |iterator| {
            Some(iterator.Preceding(i32::try_from(offset).ok()?))
        })
        .flatten()
        .filter(|boundary| *boundary != kTextBreakDone)
        .map(|boundary| boundary as usize)
    }
}
impl interaction_default::core::editing::selection_adjuster::GranularityContext
    for EditingPositionContext<'_>
{
    fn ComparePositions(&self, a: &Position<u64>, b: &Position<u64>) -> Option<Ordering> {
        EditableBoundaryContext::ComparePositions(self, a, b)
    }
    fn StartOfSentence(&self, p: &Position<u64>) -> Position<u64> {
        interaction_default::core::editing::position_units_sentence::StartOfSentencePosition(
            self, p,
        )
    }
    fn EndOfSentence(&self, p: &VisiblePosition<u64>) -> VisiblePosition<u64> {
        interaction_default::core::editing::position_units_sentence::EndOfVisibleSentence(self,p,interaction_default::core::editing::position_units_sentence::SentenceTrailingSpaceBehavior::IncludeSpace)
    }
    fn EndOfLine(&self, p: &PositionWithAffinity<u64>) -> PositionWithAffinity<u64> {
        interaction_default::core::editing::visible_units_line::EndOfLine(self, p)
    }
    fn StartOfParagraph(&self, p: &VisiblePosition<u64>) -> VisiblePosition<u64> {
        visible_units_commands::StartOfParagraph(self, p)
    }
    fn EndOfParagraph(&self, p: &VisiblePosition<u64>) -> VisiblePosition<u64> {
        visible_units_commands::EndOfParagraph(self, p)
    }
    fn StartOfDocument(&self, p: &Position<u64>) -> Position<u64> {
        visible_navigation::StartOfDocument(self, p)
    }
    fn EndOfDocument(&self, p: &VisiblePosition<u64>) -> VisiblePosition<u64> {
        VisiblePosition::Create(
            self,
            PositionWithAffinity {
                position: visible_navigation::EndOfDocument(self, p.DeepEquivalent()),
                affinity: TextAffinity::Downstream,
            },
        )
    }
    fn PreviousPosition(&self, p: &VisiblePosition<u64>) -> VisiblePosition<u64> {
        visible_navigation::PreviousPositionOf(self, p.DeepEquivalent(), Rule::CanCross)
    }
}
impl FlatSelectionBoundaryContext for EditingPositionContext<'_> {
    fn SelectionEditingBoundarySlottedContentEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::SelectionEditingBoundarySlottedContentEnabled()
    }
    fn AssignedSlotWithoutRecalc(&self, n: &u64) -> Option<u64> {
        self.document
            .AssignedSlotWithoutRecalc(self.Index(n)?)
            .map(|i| self.document.Node(i).Id())
    }
    fn OwnerShadowHost(&self, n: &u64) -> Option<u64> {
        self.document
            .OwnerShadowHost(self.Index(n)?)
            .map(|i| self.document.Node(i).Id())
    }
    fn IsSlot(&self, n: &u64) -> bool {
        self.Index(n)
            .is_some_and(|i| self.document.Node(i).IsHTMLElement("slot"))
    }
    fn HasAssignedNodesNoRecalc(&self, n: &u64) -> bool {
        self.Index(n)
            .is_some_and(|i| self.document.HasAssignedNodesNoRecalc(i))
    }
}
impl SelectionStyleContext for EditingPositionContext<'_> {
    fn HasLayoutObject(&self, n: &u64) -> bool {
        self.Layout(n).is_some()
    }
    fn IsTextControlForUserSelect(&self, n: &u64) -> bool {
        TextOffsetMappingContext::IsTextControl(self, n)
    }
    fn LayoutUsedValueOfUserSelect(&self, n: &u64) -> Option<foundation::EUserSelect> {
        self.Layout(n)?;
        let mut index = self.Index(n)?;
        loop {
            if let Some(style) = self
                .document
                .ResolvedStyleFor(index)
                .and_then(|style| style.ComputedStyle())
            {
                return Some(style.UsedUserSelect());
            }
            index = self.document.FlatTreeParent(index)?;
        }
    }
}
impl WordGranularityContext for EditingPositionContext<'_> {
    fn CompareWordPositions(&self, a: &Position<u64>, b: &Position<u64>) -> Option<Ordering> {
        EditableBoundaryContext::ComparePositions(self, a, b)
    }
    fn StartOfWord(&self, p: &Position<u64>, s: WordSide) -> Position<u64> {
        position_units_word::StartOfWordPosition(self, p, s)
    }
    fn EndOfWord(&self, p: &Position<u64>, s: WordSide) -> Position<u64> {
        position_units_word::EndOfWordPosition(self, p, s)
    }
    fn MiddleOfWord(&self, a: &Position<u64>, b: &Position<u64>) -> Position<u64> {
        position_units_word::MiddleOfWordPosition(self, a, b)
    }
    fn IsStartOfLine(&self, p: &VisiblePosition<u64>) -> bool {
        visible_units_commands::IsStartOfLine(self, p)
    }
    fn IsEndOfLine(&self, p: &VisiblePosition<u64>) -> bool {
        visible_units_commands::IsEndOfLine(self, p)
    }
    fn IsEndOfParagraph(&self, p: &VisiblePosition<u64>) -> bool {
        visible_units_commands::IsEndOfParagraph(self, p)
    }
    fn IsEndOfEditableOrNonEditableContent(&self, p: &VisiblePosition<u64>) -> bool {
        visible_navigation::IsEndOfEditableOrNonEditableContent(self, p.DeepEquivalent())
    }
    fn NextPosition(&self, p: &VisiblePosition<u64>, cannot_cross: bool) -> VisiblePosition<u64> {
        visible_navigation::NextPositionOf(
            self,
            p.ToPositionWithAffinity(),
            if cannot_cross {
                Rule::CannotCross
            } else {
                Rule::CanCross
            },
        )
    }
    fn IsEmptyTableCell(&self, n: Option<&u64>) -> bool {
        let mut current = n.copied();
        while let Some(node) = current {
            if let Some(layout) = self.Layout(&node) {
                return layout.EditingIsEmptyTableCell();
            }
            // The source first finds a rendered DOM ancestor, independently
            // of the flat strategy used for the selection endpoints.
            current = self.Dom().Parent(&node);
        }
        false
    }
    fn EnclosingTableCell(&self, p: &Position<u64>) -> Option<u64> {
        let mut node = p.ComputeContainerNode(self);
        while let Some(n) = node {
            if self
                .Layout(&n)
                .is_some_and(|n| n.EditingParagraphLayoutFacts().table_cell)
            {
                return Some(n);
            }
            node = PositionTree::Parent(self, &n);
        }
        None
    }
    fn TableElementJustBefore(&self, p: &VisiblePosition<u64>) -> Option<u64> {
        let upstream = visible_units::MostBackwardCaretPosition(
            self,
            p.DeepEquivalent(),
            Rule::CannotCross,
            SnapToClient::Editing,
        );
        let n = *upstream.AnchorNode()?;
        (PositionDOM::IsDisplayInsideTable(self, &n) && upstream.AtLastEditingPositionForNode(self))
            .then_some(n)
    }
    fn IsEnclosingBlock(&self, n: &u64) -> bool {
        self.Layout(n)
            .is_some_and(|n| n.EditingParagraphLayoutFacts().is_enclosing_block)
    }
}
pub(super) fn FlatSelection(
    ctx: &EditingPositionContext<'_>,
    s: DOMSelection,
) -> (Position<u64>, Position<u64>, TextAffinity) {
    let a = SourcePosition(EditingPositionWithAffinity {
        position: s.anchor,
        affinity: s.affinity,
    });
    let f = SourcePosition(EditingPositionWithAffinity {
        position: s.focus,
        affinity: s.affinity,
    });
    (
        position_conversion::ToPositionInFlatTree(&ctx.Dom(), &ctx.Flat(), &a.position),
        position_conversion::ToPositionInFlatTree(&ctx.Dom(), &ctx.Flat(), &f.position),
        a.affinity,
    )
}
pub(super) fn DomSelection(
    ctx: &EditingPositionContext<'_>,
    s: visible_selection::VisibleSelection<u64>,
) -> Option<DOMSelection> {
    NativeDOMSelection(
        &position_conversion::ToPositionInDOMTree(&ctx.Dom(), &ctx.Flat(), &s.anchor),
        &position_conversion::ToPositionInDOMTree(&ctx.Dom(), &ctx.Flat(), &s.focus),
        s.affinity,
    )
}
pub(super) fn QuerySelectionGeometry(
    document: &Document,
    layout: LayoutOutput<'_>,
    query: SelectionGeometryQuery,
    locale: &CString,
) -> Option<SelectionGeometryResult> {
    let _lifecycle = document.Lifecycle().DisallowTransition();
    let ctx = EditingPositionContext {
        document,
        layout: Some(layout),
        flat: true,
        ui_locale: Some(locale.as_c_str()),
        behavior: None,
    };
    let flat = |p: EditingPositionWithAffinity| {
        let p = SourcePosition(p);
        PositionWithAffinity {
            position: position_conversion::ToPositionInFlatTree(&ctx.Dom(), &ctx, &p.position),
            affinity: p.affinity,
        }
    };
    let native = |p: &Position<u64>, a: TextAffinity| {
        NativePosition(
            &position_conversion::ToPositionInDOMTree(&ctx.Dom(), &ctx, p),
            a,
        )
    };
    Some(match query {
        SelectionGeometryQuery::IsValid(p) => SelectionGeometryResult::Boolean(
            SourcePosition(p)
                .position
                .IsValidFor(&ctx.Dom(), document.RootHandle().Identity()),
        ),
        SelectionGeometryQuery::Visible(p) => {
            let p = VisiblePosition::Create(&ctx, flat(p));
            SelectionGeometryResult::Position(native(
                p.DeepEquivalent(),
                p.ToPositionWithAffinity().affinity,
            ))
        }
        SelectionGeometryQuery::VisibleSelection { selection, word } => {
            let (a, f, affinity) = FlatSelection(&ctx, selection);
            let s = if word {
                visible_selection::CreateVisibleWordSelectionInFlatTree(
                    &ctx,
                    &a,
                    &f,
                    affinity,
                    WordInclusion::Default,
                )
            } else {
                visible_selection::CreateVisibleCharacterSelectionInFlatTree(&ctx, &a, &f, affinity)
            };
            SelectionGeometryResult::Selection(DomSelection(&ctx, s))
        }
        SelectionGeometryQuery::PlainText(selection) => {
            let (a, f, _) = FlatSelection(&ctx, selection);
            let (start, end) = if EditableBoundaryContext::ComparePositions(&ctx, &a, &f)
                == Some(Ordering::Greater)
            {
                (f, a)
            } else {
                (a, f)
            };
            let mut behavior=interaction_default::core::editing::iterators::text_iterator_behavior::TextIteratorBehavior::default();
            behavior.SetEmitsObjectReplacementCharacter(
                start
                    .AnchorNode()
                    .is_some_and(|n| ctx.IsEditableNodeValue(n)),
            );
            SelectionGeometryResult::Text(
                interaction_default::core::editing::iterators::text_iterator::PlainText(
                    &ctx, &start, &end, behavior,
                ),
            )
        }
        SelectionGeometryQuery::FirstPosition(node) => {
            // SelectionController::SetCaretAtHitTestResult canonicalizes the
            // fallback just like the hit position. A raw before/inside-node
            // boundary can have no visible caret (or canonicalize elsewhere).
            let position = VisiblePosition::Create(
                &ctx,
                PositionWithAffinity {
                    position: Position::FirstPositionInOrBeforeNode(&ctx, &node),
                    affinity: TextAffinity::Downstream,
                },
            );
            SelectionGeometryResult::Position(native(
                position.DeepEquivalent(),
                position.ToPositionWithAffinity().affinity,
            ))
        }
        SelectionGeometryQuery::UserSelectAll(node) => {
            let bounds = editing_strategy::RootUserSelectAllForNode(&ctx, &node).and_then(|node| {
                let start = visible_units::MostBackwardCaretPosition(
                    &ctx,
                    &Position::BeforeNode(node),
                    Rule::CanCross,
                    SnapToClient::Editing,
                );
                let end = visible_units::MostForwardCaretPosition(
                    &ctx,
                    &Position::AfterNode(node),
                    Rule::CanCross,
                    SnapToClient::Editing,
                );
                Some((
                    native(&start, TextAffinity::Downstream)?,
                    native(&end, TextAffinity::Downstream)?,
                ))
            });
            SelectionGeometryResult::Bounds(bounds)
        }
        SelectionGeometryQuery::LastPositionInEditableRoot {
            position,
            selection,
        } => {
            let p = flat(EditingPositionWithAffinity {
                position,
                affinity: selection.affinity,
            });
            let (a, f, _) = FlatSelection(&ctx, selection);
            let start = if EditableBoundaryContext::ComparePositions(&ctx, &a, &f)
                == Some(Ordering::Greater)
            {
                f
            } else {
                a
            };
            let last = EditableBoundaryContext::RootEditableElementOf(&ctx, &start).map(|root| {
                VisiblePosition::Create(
                    &ctx,
                    PositionWithAffinity {
                        position: Position::LastPositionInNode(&ctx, &root),
                        affinity: TextAffinity::Downstream,
                    },
                )
            });
            SelectionGeometryResult::Boolean(
                last.is_some_and(|last| p.position == *last.DeepEquivalent()),
            )
        }
        SelectionGeometryQuery::SkipWhitespace(p) => {
            let p = flat(p);
            SelectionGeometryResult::Position(native(
                &visible_navigation::SkipWhitespace(&ctx, &p.position),
                p.affinity,
            ))
        }
        SelectionGeometryQuery::Compare(a, b) => {
            let a = flat(EditingPositionWithAffinity {
                position: a,
                affinity: EditingTextAffinity::Downstream,
            });
            let b = flat(EditingPositionWithAffinity {
                position: b,
                affinity: EditingTextAffinity::Downstream,
            });
            SelectionGeometryResult::Compare(EditableBoundaryContext::ComparePositions(
                &ctx,
                &a.position,
                &b.position,
            ))
        }
        SelectionGeometryQuery::NodeSelectable(n) => SelectionGeometryResult::Boolean(
            ctx.Layout(&n).is_some_and(|n| n.EditingIsSelectable()),
        ),
        SelectionGeometryQuery::NodeSelectionLocked(n) => SelectionGeometryResult::Boolean(
            ctx.Layout(&n)
                .is_some_and(|n| n.EditingSelectionStartFacts().0),
        ),
        SelectionGeometryQuery::NodeCanStartSelection(n) => {
            use interaction_hit_test::core::dom::node_hit_test::{self, SelectionStartNode};
            struct Node<'a, 'b> {
                context: &'a EditingPositionContext<'b>,
                node: u64,
            }
            impl SelectionStartNode for Node<'_, '_> {
                fn LockedAncestorPreventingPaint(&self) -> bool {
                    self.context
                        .Layout(&self.node)
                        .is_some_and(|n| n.EditingSelectionStartFacts().0)
                }
                fn IsEditable(&self) -> bool {
                    self.context.IsEditableNodeValue(&self.node)
                }
                fn LayoutSelectionStyle(
                    &self,
                ) -> Option<(foundation::EUserSelect, foundation::EUserDrag)> {
                    let (_, select, drag) = self
                        .context
                        .Layout(&self.node)?
                        .EditingSelectionStartFacts();
                    Some((select, drag))
                }
                fn FlatTreeParentForSelection(&self) -> Option<Box<dyn SelectionStartNode + '_>> {
                    Some(Box::new(Node {
                        context: self.context,
                        node: PositionTree::Parent(self.context, &self.node)?,
                    }))
                }
            }
            SelectionGeometryResult::Boolean(node_hit_test::CanStartSelectionAfterLayoutTreeUpdate(
                &Node {
                    context: &ctx,
                    node: n,
                },
            ))
        }
    })
}
