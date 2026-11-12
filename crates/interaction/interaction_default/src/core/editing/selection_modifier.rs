// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source selection_modifier.h:43-156; selection_modifier.cc:54-1165.
// This is a call-stack owner over original DOM/flat positions. It never
// publishes another retained selection model or approximates a layout query.
pub use super::commands::editor_command_input_type::SelectionModifyDirection as ModifyDirection;
use super::{
    commands::editor_command_input_type::SelectionModifyDirection,
    editing_behavior::EditingBehaviorType,
    editing_boundaries::EditableBoundaryContext,
    editing_strategy::{self, SelectionStyleContext},
    inline_box_position::{self, InlineAdjustmentContext},
    position::{Position, PositionWithAffinity, TextAffinity},
    position_units_paragraph::{self, ParagraphContext},
    position_units_word::PlatformWordBehavior,
    visible_navigation,
    visible_position::{VisiblePosition, VisiblePositionContext},
    visible_selection::VisibleSelection,
    visible_units::EditingBoundaryCrossingRule as Rule,
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use foundation::{LayoutUnit, TextDirection};
use std::cmp::Ordering;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionModifyAlteration {
    Move,
    Extend,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionModifyVerticalDirection {
    Up,
    Down,
}
use super::text_granularity::TextGranularity as G;
#[derive(Clone, Debug)]
pub struct VisualCaretMoveResult<N> {
    pub position: PositionWithAffinity<N>,
    pub bidi_level: Option<u8>,
    pub entered_bidi_run: bool,
}
/// Source helpers return original strategy positions. Text segmentation and
/// line movement are the existing owning modules; native leaves borrow the
/// resident caret/line cursor, with no copied fragment or DOM model.
pub trait SelectionModifierContext:
    VisiblePositionContext
    + EditableBoundaryContext
    + InlineAdjustmentContext
    + ParagraphContext
    + SelectionStyleContext
{
    type LifecycleGuard;
    fn AssertLayoutPrepared(&self);
    fn UpdateAllLifecyclePhasesExceptPaint(&self);
    fn DisallowLifecycleTransitions(&self) -> Self::LifecycleGuard;
    fn Behavior(&self) -> EditingBehaviorType;
    fn IsSpatialNavigationEnabled(&self) -> bool;
    fn CreateVisibleSelectionInFlatTree(
        &self,
        selection: &VisibleSelection<Self::Node>,
    ) -> VisibleSelection<Self::Node>;
    fn CreateVisibleSelectionInDOMTree(
        &self,
        flat: &VisibleSelection<Self::Node>,
    ) -> VisibleSelection<Self::Node>;
    fn LayoutTextDirection(&self, node: &Self::Node) -> Option<TextDirection>;
    fn NativeCaretDirections(
        &self,
        adjusted: &PositionWithAffinity<Self::Node>,
    ) -> Option<(TextDirection, TextDirection)>;
    fn NativeMoveCaretVisually(
        &self,
        position: &PositionWithAffinity<Self::Node>,
        level: Option<u8>,
        entered: bool,
        right: bool,
    ) -> VisualCaretMoveResult<Self::Node>;
    fn CaretLineDirectionCoordinateIgnoringTransforms(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> Option<LayoutUnit>;
    fn AbsoluteCaretBounds(
        &self,
        position: &PositionWithAffinity<Self::Node>,
    ) -> foundation::gfx::Rect;
    fn WordPosition(
        &self,
        position: &Position<Self::Node>,
        backward: bool,
        behavior: PlatformWordBehavior,
    ) -> PositionWithAffinity<Self::Node>;
    fn SentencePosition(
        &self,
        position: &Position<Self::Node>,
        backward: bool,
    ) -> Position<Self::Node>;
    fn StartOfSentencePosition(&self, position: &Position<Self::Node>) -> Position<Self::Node>;
    fn EndOfSentence(&self, position: &Position<Self::Node>) -> PositionWithAffinity<Self::Node>;
    fn LogicalLineBoundary(
        &self,
        position: &PositionWithAffinity<Self::Node>,
        end: bool,
    ) -> PositionWithAffinity<Self::Node>;
    fn LinePosition(
        &self,
        position: &PositionWithAffinity<Self::Node>,
        coordinate: LayoutUnit,
        next: bool,
    ) -> PositionWithAffinity<Self::Node>;
    fn BidiVisualOrderCaretMovementEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::BidiVisualOrderCaretMovementEnabled()
    }
    fn SelectionFocusAffinityEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::SelectionFocusAffinityEnabled()
    }
    fn ModifyParagraphCrossEditingBoundaryEnabled(&self) -> bool {
        foundation::RuntimeEnabledFeatures::ModifyParagraphCrossEditingoundaryEnabled()
    }
}
fn visible<C: VisiblePositionContext>(
    ctx: &C,
    position: Position<C::Node>,
    affinity: TextAffinity,
) -> VisiblePosition<C::Node> {
    VisiblePosition::Create(ctx, PositionWithAffinity { position, affinity })
}
fn none<N: Clone + PartialEq>(selection: &VisibleSelection<N>) -> bool {
    selection.anchor.IsNull()
}
fn range<N: Clone + PartialEq>(selection: &VisibleSelection<N>) -> bool {
    !none(selection) && selection.anchor != selection.focus
}
fn anchor_first<C: EditableBoundaryContext>(ctx: &C, s: &VisibleSelection<C::Node>) -> bool {
    EditableBoundaryContext::ComparePositions(ctx, &s.anchor, &s.focus)
        .expect("visible selection retains comparable flat-tree positions")
        != Ordering::Greater
}
fn start<C: EditableBoundaryContext>(ctx: &C, s: &VisibleSelection<C::Node>) -> Position<C::Node> {
    if none(s) {
        return Position::default();
    }
    if anchor_first(ctx, s) {
        s.anchor.clone()
    } else {
        s.focus.clone()
    }
}
fn end<C: EditableBoundaryContext>(ctx: &C, s: &VisibleSelection<C::Node>) -> Position<C::Node> {
    if none(s) {
        return Position::default();
    }
    if anchor_first(ctx, s) {
        s.focus.clone()
    } else {
        s.anchor.clone()
    }
}
// VisibleSelection.cc:317-345; ranges use endpoint-specific affinities.
fn visible_start<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    visible(
        ctx,
        start(ctx, s),
        if range(s) {
            TextAffinity::Downstream
        } else {
            s.affinity
        },
    )
}
fn visible_end<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    visible(
        ctx,
        end(ctx, s),
        if range(s) {
            TextAffinity::Upstream
        } else {
            s.affinity
        },
    )
}
fn visible_anchor<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    visible(
        ctx,
        s.anchor.clone(),
        if range(s) {
            if anchor_first(ctx, s) {
                TextAffinity::Upstream
            } else {
                TextAffinity::Downstream
            }
        } else {
            s.affinity
        },
    )
}
fn visible_focus<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    visible(
        ctx,
        s.focus.clone(),
        if range(s) {
            if anchor_first(ctx, s) {
                TextAffinity::Downstream
            } else {
                TextAffinity::Upstream
            }
        } else {
            s.affinity
        },
    )
}
fn compute_visible_focus<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> VisiblePosition<C::Node> {
    let mut affinity = s.affinity;
    if ctx.SelectionFocusAffinityEnabled()
        && range(s)
        && s.focus
            .AnchorNode()
            .is_some_and(|node| DownstreamVisiblePositionContext::HasLayoutObject(ctx, node))
    {
        let downstream = PositionWithAffinity {
            position: s.focus.clone(),
            affinity: TextAffinity::Downstream,
        };
        let upstream = PositionWithAffinity {
            position: s.focus.clone(),
            affinity: TextAffinity::Upstream,
        };
        if !DownstreamVisiblePositionContext::InSameLine(ctx, &downstream, &upstream) {
            affinity = if anchor_first(ctx, s) {
                TextAffinity::Downstream
            } else {
                TextAffinity::Upstream
            };
        }
    }
    visible(ctx, s.focus.clone(), affinity)
}
fn block_direction<C: SelectionModifierContext>(
    ctx: &C,
    position: &Position<C::Node>,
) -> TextDirection {
    if position.IsNull() {
        return TextDirection::kLtr;
    }
    let node = position
        .ComputeContainerNode(ctx)
        .expect("non-null visible position has a container");
    let position = Position::FirstPositionInOrBeforeNode(ctx, &node);
    ParagraphContext::EnclosingBlock(ctx, &position, Rule::CannotCross)
        .and_then(|node| ctx.LayoutTextDirection(&node))
        .unwrap_or(TextDirection::kLtr)
}
fn caret_directions<C: SelectionModifierContext>(
    ctx: &C,
    position: &PositionWithAffinity<C::Node>,
) -> Option<(TextDirection, TextDirection)> {
    if position.position.IsNull() {
        return None;
    }
    let adjusted =
        inline_box_position::ComputeInlineAdjustedPosition(ctx, position.clone(), Rule::CanCross);
    if adjusted.position.IsNull()
        || DownstreamVisiblePositionContext::InlineFormattingContext(ctx, &adjusted.position)
            .is_none()
    {
        return None;
    }
    ctx.NativeCaretDirections(&adjusted)
}
fn direction_of<C: SelectionModifierContext>(
    ctx: &C,
    s: &VisibleSelection<C::Node>,
) -> TextDirection {
    let a = caret_directions(ctx, visible_start(ctx, s).ToPositionWithAffinity()).map(|d| d.0);
    let b = caret_directions(ctx, visible_end(ctx, s).ToPositionWithAffinity()).map(|d| d.0);
    if let (Some(a), Some(b)) = (a, b) {
        if a == b {
            return a;
        }
    }
    block_direction(ctx, &s.focus)
}
fn paragraph_boundary<C: SelectionModifierContext>(
    ctx: &C,
    p: &VisiblePosition<C::Node>,
    end: bool,
    rule: Rule,
) -> VisiblePosition<C::Node> {
    let position = if end {
        position_units_paragraph::EndOfParagraph(ctx, p.DeepEquivalent(), rule)
    } else {
        position_units_paragraph::StartOfParagraph(ctx, p.DeepEquivalent(), rule)
    };
    visible(ctx, position, TextAffinity::Downstream)
}
pub fn NoXPosForVerticalArrowNavigation() -> LayoutUnit {
    LayoutUnit::Min()
}
pub struct SelectionModifier<N> {
    selection: VisibleSelection<N>,
    current_selection: VisibleSelection<N>,
    x_pos: LayoutUnit,
    caret_bidi_level: Option<u8>,
    entered_bidi_run: bool,
    selection_is_directional: bool,
    raw_visual_position: PositionWithAffinity<N>,
}
impl<N: Clone + PartialEq> SelectionModifier<N> {
    /// Input is the original DOM selection converted by the original flat-tree
    /// strategy. The value lives only for this source call stack.
    pub fn new(selection_in_flat_tree: VisibleSelection<N>, x_pos: LayoutUnit) -> Self {
        Self {
            selection: VisibleSelection::default(),
            current_selection: selection_in_flat_tree,
            x_pos,
            caret_bidi_level: None,
            entered_bidi_run: false,
            selection_is_directional: false,
            raw_visual_position: PositionWithAffinity::default(),
        }
    }
    pub fn XPosForVerticalArrowNavigation(&self) -> LayoutUnit {
        self.x_pos
    }
    pub fn CaretBidiLevel(&self) -> Option<u8> {
        self.caret_bidi_level
    }
    pub fn SetCaretBidiLevel(&mut self, level: Option<u8>) {
        self.caret_bidi_level = level;
    }
    pub fn EnteredBidiRun(&self) -> bool {
        self.entered_bidi_run
    }
    pub fn SetEnteredBidiRun(&mut self, value: bool) {
        self.entered_bidi_run = value;
    }
    pub fn SetSelectionIsDirectional(&mut self, value: bool) {
        self.selection_is_directional = value;
    }
    pub fn Selection<C: SelectionModifierContext<Node = N>>(&self, ctx: &C) -> VisibleSelection<N> {
        ctx.CreateVisibleSelectionInDOMTree(&self.current_selection)
    }
    pub fn CurrentSelectionInFlatTree(&self) -> &VisibleSelection<N> {
        &self.current_selection
    }
    pub fn DirectionOfSelection<C: SelectionModifierContext<Node = N>>(
        &self,
        ctx: &C,
    ) -> TextDirection {
        direction_of(ctx, &self.selection)
    }
    fn PrepareToModifySelection<C: SelectionModifierContext<Node = N>>(
        &self,
        ctx: &C,
        alter: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
    ) -> VisibleSelection<N> {
        let selection = ctx.CreateVisibleSelectionInFlatTree(&self.current_selection);
        if alter != SelectionModifyAlteration::Extend || none(&selection) || !range(&selection) {
            return selection;
        }
        let anchor_is_start = if self.selection_is_directional {
            anchor_first(ctx, &selection)
        } else {
            match direction {
                SelectionModifyDirection::Right => {
                    direction_of(ctx, &selection) == TextDirection::kLtr
                }
                SelectionModifyDirection::Forward => true,
                SelectionModifyDirection::Left => {
                    direction_of(ctx, &selection) != TextDirection::kLtr
                }
                SelectionModifyDirection::Backward => false,
            }
        };
        let (first, last) = (start(ctx, &selection), end(ctx, &selection));
        ctx.CreateVisibleSelectionInFlatTree(&VisibleSelection {
            anchor: if anchor_is_start {
                first.clone()
            } else {
                last.clone()
            },
            focus: if anchor_is_start { last } else { first },
            affinity: TextAffinity::Downstream,
        })
    }
    fn PositionForPlatform<C: SelectionModifierContext<Node = N>>(
        &self,
        ctx: &C,
        get_start: bool,
    ) -> VisiblePosition<N> {
        if ctx.Behavior() == EditingBehaviorType::Mac {
            if get_start {
                visible_start(ctx, &self.selection)
            } else {
                visible_end(ctx, &self.selection)
            }
        } else if anchor_first(ctx, &self.selection) {
            visible_end(ctx, &self.selection)
        } else {
            visible_start(ctx, &self.selection)
        }
    }
    fn LineDirectionPoint<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        p: &Position<N>,
    ) -> LayoutUnit {
        if none(&self.selection) {
            return LayoutUnit::default();
        }
        if self.x_pos == NoXPosForVerticalArrowNavigation() {
            let position = visible(ctx, p.clone(), self.selection.affinity);
            self.x_pos = if position.IsNull() {
                LayoutUnit::default()
            } else {
                ctx.CaretLineDirectionCoordinateIgnoringTransforms(
                    position.ToPositionWithAffinity(),
                )
                .unwrap_or_default()
            };
        }
        self.x_pos
    }
    fn ParagraphPosition<C: SelectionModifierContext<Node = N>>(
        &self,
        ctx: &C,
        passed: &VisiblePosition<N>,
        x: LayoutUnit,
        next: bool,
    ) -> VisiblePosition<N> {
        let mut position = passed.clone();
        loop {
            let new = VisiblePosition::Create(
                ctx,
                ctx.LinePosition(position.ToPositionWithAffinity(), x, next),
            );
            if new.IsNull() || new.DeepEquivalent() == position.DeepEquivalent() {
                break;
            }
            position = new;
            let rule = if ctx.ModifyParagraphCrossEditingBoundaryEnabled() {
                Rule::CanCross
            } else {
                Rule::CannotCross
            };
            if passed.IsNull()
                || paragraph_boundary(ctx, passed, false, rule).DeepEquivalent()
                    != paragraph_boundary(ctx, &position, false, rule).DeepEquivalent()
            {
                break;
            }
        }
        position
    }
    fn TryVisual<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        right: bool,
    ) -> Option<VisiblePosition<N>> {
        if !ctx.BidiVisualOrderCaretMovementEnabled() {
            return None;
        }
        let result = ctx.NativeMoveCaretVisually(
            compute_visible_focus(ctx, &self.selection).ToPositionWithAffinity(),
            self.caret_bidi_level,
            self.entered_bidi_run,
            right,
        );
        if result.position.position.IsNull() {
            return None;
        }
        self.caret_bidi_level = result.bidi_level;
        self.entered_bidi_run = result.entered_bidi_run;
        self.raw_visual_position = result.position.clone();
        Some(VisiblePosition::Create(ctx, result.position))
    }
    fn LogicalPosition<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        forward: bool,
        granularity: G,
        extend: bool,
    ) -> VisiblePosition<N> {
        let focus = compute_visible_focus(ctx, &self.selection);
        let platform = || self.PositionForPlatform(ctx, !forward);
        match granularity {
            G::kCharacter => {
                if !extend && range(&self.selection) {
                    return visible(
                        ctx,
                        if forward {
                            end(ctx, &self.selection)
                        } else {
                            start(ctx, &self.selection)
                        },
                        self.selection.affinity,
                    );
                }
                if forward {
                    visible_navigation::NextPositionOf(
                        ctx,
                        focus.ToPositionWithAffinity(),
                        Rule::CanSkipOver,
                    )
                } else {
                    visible_navigation::PreviousPositionOf(
                        ctx,
                        focus.DeepEquivalent(),
                        Rule::CanSkipOver,
                    )
                }
            }
            G::kWord => {
                let behavior = if ctx.Behavior().ShouldSkipSpaceWhenMovingRight() {
                    PlatformWordBehavior::SkipSpaces
                } else {
                    PlatformWordBehavior::DontSkipSpaces
                };
                VisiblePosition::Create(
                    ctx,
                    ctx.WordPosition(focus.DeepEquivalent(), !forward, behavior),
                )
            }
            G::kSentence => visible(
                ctx,
                ctx.SentencePosition(focus.DeepEquivalent(), !forward),
                if forward {
                    TextAffinity::Upstream
                } else {
                    TextAffinity::Downstream
                },
            ),
            G::kLine | G::kParagraph => {
                let position = if extend { focus } else { platform() };
                let selection_focus = self.selection.focus.clone();
                let x = self.LineDirectionPoint(ctx, &selection_focus);
                if granularity == G::kLine {
                    VisiblePosition::Create(
                        ctx,
                        ctx.LinePosition(position.ToPositionWithAffinity(), x, forward),
                    )
                } else {
                    self.ParagraphPosition(ctx, &position, x, forward)
                }
            }
            G::kSentenceBoundary => {
                let position = platform();
                if forward {
                    VisiblePosition::Create(ctx, ctx.EndOfSentence(position.DeepEquivalent()))
                } else {
                    visible(
                        ctx,
                        ctx.StartOfSentencePosition(position.DeepEquivalent()),
                        TextAffinity::Downstream,
                    )
                }
            }
            G::kLineBoundary => VisiblePosition::Create(
                ctx,
                ctx.LogicalLineBoundary(platform().ToPositionWithAffinity(), forward),
            ),
            G::kParagraphBoundary => {
                let position = platform();
                let rule = if !extend && ctx.IsEditablePosition(position.DeepEquivalent()) {
                    Rule::CanSkipOver
                } else {
                    Rule::CannotCross
                };
                paragraph_boundary(ctx, &position, forward, rule)
            }
            G::kDocumentBoundary => {
                let position = platform();
                let editable = ctx.IsEditablePosition(position.DeepEquivalent());
                visible(
                    ctx,
                    if forward {
                        if editable {
                            visible_navigation::EndOfEditableContent(ctx, position.DeepEquivalent())
                        } else {
                            visible_navigation::EndOfDocument(ctx, position.DeepEquivalent())
                        }
                    } else if editable {
                        visible_navigation::StartOfEditableContent(ctx, position.DeepEquivalent())
                    } else {
                        visible_navigation::StartOfDocument(ctx, position.DeepEquivalent())
                    },
                    TextAffinity::Downstream,
                )
            }
        }
    }
    fn AdjustForUserSelectAll<C: SelectionModifierContext<Node = N>>(
        &self,
        ctx: &C,
        position: VisiblePosition<N>,
        forward: bool,
    ) -> VisiblePosition<N> {
        let Some(root) = position
            .DeepEquivalent()
            .AnchorNode()
            .and_then(|node| editing_strategy::RootUserSelectAllForNode(ctx, node))
        else {
            return position;
        };
        let boundary = if forward {
            Position::AfterNode(root)
        } else {
            Position::BeforeNode(root)
        };
        let position = ctx.BoundaryCaretPosition(&boundary, Rule::CanCross, !forward);
        visible(ctx, position, TextAffinity::Downstream)
    }
    fn ComputeModifyPosition<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        alter: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
        granularity: G,
    ) -> VisiblePosition<N> {
        let extend = alter == SelectionModifyAlteration::Extend;
        let block_ltr = block_direction(ctx, &self.selection.focus) == TextDirection::kLtr;
        match direction {
            SelectionModifyDirection::Forward | SelectionModifyDirection::Backward => {
                let forward = direction == SelectionModifyDirection::Forward;
                let position = self.LogicalPosition(ctx, forward, granularity, extend);
                if extend {
                    self.AdjustForUserSelectAll(
                        ctx,
                        position,
                        if forward { block_ltr } else { !block_ltr },
                    )
                } else {
                    position
                }
            }
            SelectionModifyDirection::Left | SelectionModifyDirection::Right => {
                let right = direction == SelectionModifyDirection::Right;
                if extend {
                    let forward = match granularity {
                        G::kCharacter | G::kWord | G::kLineBoundary => right == block_ltr,
                        _ => right,
                    };
                    let position = if granularity == G::kCharacter {
                        self.TryVisual(ctx, right).unwrap_or_else(|| {
                            self.LogicalPosition(ctx, forward, granularity, true)
                        })
                    } else {
                        self.LogicalPosition(ctx, forward, granularity, true)
                    };
                    return self.AdjustForUserSelectAll(ctx, position, right == block_ltr);
                }
                if granularity == G::kLineBoundary {
                    let position = self.PositionForPlatform(ctx, true);
                    return VisiblePosition::Create(
                        ctx,
                        ctx.LogicalLineBoundary(
                            position.ToPositionWithAffinity(),
                            right == block_ltr,
                        ),
                    );
                }
                if granularity == G::kCharacter && range(&self.selection) {
                    return visible(
                        ctx,
                        if right == (self.DirectionOfSelection(ctx) == TextDirection::kLtr) {
                            end(ctx, &self.selection)
                        } else {
                            start(ctx, &self.selection)
                        },
                        self.selection.affinity,
                    );
                }
                if granularity == G::kCharacter {
                    if let Some(position) = self.TryVisual(ctx, right) {
                        return position;
                    }
                }
                let directional = granularity == G::kCharacter
                    || granularity == G::kWord
                    || ctx.BidiVisualOrderCaretMovementEnabled();
                let ltr = if directional {
                    let directions = caret_directions(
                        ctx,
                        visible_focus(ctx, &self.selection).ToPositionWithAffinity(),
                    );
                    if granularity == G::kCharacter || !ctx.BidiVisualOrderCaretMovementEnabled() {
                        directions.map(|d| d.1)
                    } else {
                        directions.map(|d| d.0)
                    }
                    .unwrap_or_else(|| block_direction(ctx, &self.selection.focus))
                        == TextDirection::kLtr
                } else {
                    true
                };
                self.LogicalPosition(ctx, right == ltr, granularity, false)
            }
        }
    }
    pub fn Modify<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        alter: SelectionModifyAlteration,
        direction: SelectionModifyDirection,
        granularity: G,
    ) -> bool {
        ctx.AssertLayoutPrepared();
        if matches!(granularity, G::kLine | G::kParagraph) {
            ctx.UpdateAllLifecyclePhasesExceptPaint();
        }
        let _scope = ctx.DisallowLifecycleTransitions();
        self.selection = self.PrepareToModifySelection(ctx, alter, direction);
        if none(&self.selection) {
            return false;
        }
        let was_range = range(&self.selection);
        let original_start = visible_start(ctx, &self.selection);
        let mut position = self.ComputeModifyPosition(ctx, alter, direction, granularity);
        if position.IsNull() {
            return false;
        }
        if ctx.IsSpatialNavigationEnabled()
            && !was_range
            && alter == SelectionModifyAlteration::Move
            && position.DeepEquivalent() == original_start.DeepEquivalent()
        {
            return false;
        }
        let old_start = start(ctx, &self.selection);
        let x = self.LineDirectionPoint(ctx, &old_start);
        match alter {
            SelectionModifyAlteration::Move => {
                let pwa = if ctx.BidiVisualOrderCaretMovementEnabled()
                    && !self.raw_visual_position.position.IsNull()
                {
                    std::mem::take(&mut self.raw_visual_position)
                } else {
                    position.ToPositionWithAffinity().clone()
                };
                self.current_selection = VisibleSelection {
                    anchor: pwa.position.clone(),
                    focus: pwa.position,
                    affinity: pwa.affinity,
                };
            }
            SelectionModifyAlteration::Extend => {
                if ctx.BidiVisualOrderCaretMovementEnabled()
                    && !self.raw_visual_position.position.IsNull()
                {
                    let raw = std::mem::take(&mut self.raw_visual_position);
                    self.current_selection = VisibleSelection {
                        anchor: self.selection.anchor.clone(),
                        focus: raw.position,
                        affinity: TextAffinity::Downstream,
                    };
                } else {
                    if range(&self.selection)
                        && matches!(granularity, G::kWord | G::kParagraph | G::kLine)
                        && !ctx
                            .Behavior()
                            .ShouldExtendSelectionByWordOrLineAcrossCaret()
                    {
                        let new = ctx.CreateVisibleSelectionInFlatTree(&VisibleSelection {
                            anchor: self.selection.anchor.clone(),
                            focus: position.DeepEquivalent().clone(),
                            affinity: self.selection.affinity,
                        });
                        if anchor_first(ctx, &self.selection) != anchor_first(ctx, &new) {
                            position = visible_anchor(ctx, &self.selection);
                        }
                    }
                    if !ctx
                        .Behavior()
                        .ShouldAlwaysGrowSelectionWhenExtendingToBoundary()
                        || !range(&self.selection)
                        || !matches!(
                            granularity,
                            G::kLineBoundary | G::kParagraphBoundary | G::kDocumentBoundary
                        )
                    {
                        self.current_selection = VisibleSelection {
                            anchor: self.selection.anchor.clone(),
                            focus: position.DeepEquivalent().clone(),
                            affinity: TextAffinity::Downstream,
                        };
                    } else {
                        let ltr =
                            block_direction(ctx, &self.selection.focus) == TextDirection::kLtr;
                        let forward = direction == SelectionModifyDirection::Forward
                            || (ltr && direction == SelectionModifyDirection::Right)
                            || (!ltr && direction == SelectionModifyDirection::Left);
                        let anchor_is_start = anchor_first(ctx, &self.selection);
                        self.current_selection = if forward {
                            VisibleSelection {
                                anchor: if anchor_is_start {
                                    self.selection.anchor.clone()
                                } else {
                                    position.DeepEquivalent().clone()
                                },
                                focus: if anchor_is_start {
                                    position.DeepEquivalent().clone()
                                } else {
                                    self.selection.focus.clone()
                                },
                                affinity: TextAffinity::Downstream,
                            }
                        } else {
                            VisibleSelection {
                                anchor: if anchor_is_start {
                                    position.DeepEquivalent().clone()
                                } else {
                                    self.selection.anchor.clone()
                                },
                                focus: if anchor_is_start {
                                    self.selection.focus.clone()
                                } else {
                                    position.DeepEquivalent().clone()
                                },
                                affinity: TextAffinity::Downstream,
                            }
                        };
                    }
                }
            }
        }
        if matches!(granularity, G::kLine | G::kParagraph) {
            self.x_pos = x;
        }
        true
    }
    pub fn ModifyWithPageGranularity<C: SelectionModifierContext<Node = N>>(
        &mut self,
        ctx: &C,
        alter: SelectionModifyAlteration,
        vertical_distance: i32,
        direction: SelectionModifyVerticalDirection,
    ) -> bool {
        if vertical_distance == 0 {
            return false;
        }
        ctx.AssertLayoutPrepared();
        ctx.UpdateAllLifecyclePhasesExceptPaint();
        let _scope = ctx.DisallowLifecycleTransitions();
        let up = direction == SelectionModifyVerticalDirection::Up;
        self.selection = self.PrepareToModifySelection(
            ctx,
            alter,
            if up {
                SelectionModifyDirection::Backward
            } else {
                SelectionModifyDirection::Forward
            },
        );
        let (position, x) = if alter == SelectionModifyAlteration::Move {
            let p = if up {
                start(ctx, &self.selection)
            } else {
                end(ctx, &self.selection)
            };
            let position = visible(ctx, p.clone(), self.selection.affinity);
            let x = self.LineDirectionPoint(ctx, &p);
            (position, x)
        } else {
            let focus = self.selection.focus.clone();
            (
                compute_visible_focus(ctx, &self.selection),
                self.LineDirectionPoint(ctx, &focus),
            )
        };
        let caret_y = |position: &VisiblePosition<N>| {
            let rect = ctx.AbsoluteCaretBounds(position.ToPositionWithAffinity());
            if rect.IsEmpty() {
                None
            } else {
                let y = rect.y() + rect.height() / 2;
                Some(if up { -y } else { y })
            }
        };
        let Some(start_y) = caret_y(&position) else {
            return false;
        };
        let mut last_y = start_y;
        let mut result = VisiblePosition::default();
        let mut p = position;
        for _ in 0..1024 {
            let next =
                VisiblePosition::Create(ctx, ctx.LinePosition(p.ToPositionWithAffinity(), x, !up));
            if next.IsNull() || next.DeepEquivalent() == p.DeepEquivalent() {
                break;
            }
            let Some(next_y) = caret_y(&next) else {
                break;
            };
            if i64::from(next_y) - i64::from(start_y) > i64::from(vertical_distance) {
                break;
            }
            if next_y >= last_y {
                last_y = next_y;
                result = next.clone();
            }
            p = next;
        }
        if result.IsNull() {
            return false;
        }
        self.current_selection = if alter == SelectionModifyAlteration::Move {
            VisibleSelection {
                anchor: result.DeepEquivalent().clone(),
                focus: result.DeepEquivalent().clone(),
                affinity: if up {
                    TextAffinity::Upstream
                } else {
                    TextAffinity::Downstream
                },
            }
        } else {
            VisibleSelection {
                anchor: self.selection.anchor.clone(),
                focus: result.DeepEquivalent().clone(),
                affinity: TextAffinity::Downstream,
            }
        };
        true
    }
}
