//! Chromium replace_selection_command.{h,cc}, pinned 6c1d401fcca5.
//! ReplacementFragment preparation and the general/trivial command paths retain
//! canonical native node, style, selection and CompositeEditCommand undo owners.
//! Position and smart replacement helpers retain their Chromium source owners;
//! markup and style helpers remain in replacement_{markup,style}.
use super::{
    paragraph_editing_context::EditResult,
    structural_editing_context::{StructuralEditingContext, StructuralPositionQuery},
};
use crate::core::editing::position::{Position, PositionDOM, PositionTree};
use interaction_dispatch::core::events::input_event::InputType;

#[derive(Clone, Copy, Default)]
pub struct ReplaceSelectionOptions {
    pub select_replacement: bool,
    pub smart_replace: bool,
    pub match_style: bool,
    pub prevent_nesting: bool,
    pub moving_paragraph: bool,
    pub sanitize_fragment: bool,
}
#[derive(Clone)]
pub struct ReplacementRange {
    pub start: Position<u64>,
    pub end: Position<u64>,
}
/// A stack fragment view retains only the original detached root identity and
/// source interchange flags. It never projects the incoming DOM into a tree.
pub struct ReplacementFragment {
    pub root: u64,
    pub interchange_start: bool,
    pub interchange_end: bool,
    trivial_text: Vec<u16>,
}
pub trait ReplacementFragmentDOM: PositionTree<Node = u64> {
    fn IsInterchangeNewline(&self, node: u64) -> bool;
    /// Node::RemoveChild, including the temporary connected test holder.
    fn RemoveFragmentNode(&self, node: u64) -> EditResult;
}
impl ReplacementFragment {
    pub fn new(root: u64) -> Self {
        Self {
            root,
            interchange_start: false,
            interchange_end: false,
            trivial_text: Vec::new(),
        }
    }
    /// replace_selection_command.cc:426-454. Nodes remain in their native
    /// destination Document; only the two source interchange flags persist.
    pub fn RemoveInterchangeNodes<C: ReplacementFragmentDOM>(&mut self, c: &C) -> EditResult {
        self.interchange_start = false;
        self.interchange_end = false;
        let mut node = c.ChildAt(&self.root, 0);
        while let Some(n) = node {
            if c.IsInterchangeNewline(n) {
                self.interchange_start = true;
                c.RemoveFragmentNode(n)?;
                break;
            }
            node = c.ChildAt(&n, 0);
        }
        if c.ChildCount(&self.root) == 0 {
            return Ok(());
        }
        let mut node = self.LastChild(c);
        while let Some(n) = node {
            if c.IsInterchangeNewline(n) {
                self.interchange_end = true;
                c.RemoveFragmentNode(n)?;
                break;
            }
            node = c
                .ChildCount(&n)
                .checked_sub(1)
                .and_then(|i| c.ChildAt(&n, i));
        }
        Ok(())
    }
    pub fn FirstChild<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64> {
        c.ChildAt(&self.root, 0)
    }
    pub fn LastChild<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64> {
        c.ChildCount(&self.root)
            .checked_sub(1)
            .and_then(|i| c.ChildAt(&self.root, i))
    }
    pub fn IsTrivialTextOnlyFragment<C: PositionTree<Node = u64>>(&self, c: &C) -> bool {
        self.FirstChild(c)
            .is_some_and(|n| Some(n) == self.LastChild(c) && c.IsText(&n))
    }
    pub fn IsEmpty<C: PositionTree<Node = u64>>(&self, c: &C) -> bool {
        self.FirstChild(c).is_none() && !self.interchange_start && !self.interchange_end
    }
}
/// Scalar/live-DOM leaves used by ReplacementFragment's constructor. Fragment
/// preparation does not enter CompositeEditCommand's undo history.
pub trait ReplacementPreparationContext: ReplacementFragmentDOM {
    fn UpdateStyleAndLayout(&self);
    fn IsRichlyEditable(&self, node: u64) -> bool;
    fn IsNodeRendered(&self, node: u64) -> bool;
    fn IsTableStructure(&self, node: u64) -> bool;
    fn FragmentTextContent(&self, node: u64) -> Vec<u16>;
    fn HasBeforeTextInsertedListener(&self, root: u64) -> bool;
    fn EnclosingTextControl(&self, node: u64) -> Option<(u64, bool)>;
    fn BeforeTextInserted(&self, root: u64, text: &[u16]) -> EditResult<Vec<u16>>;
    fn CreateFragmentFromText(&self, text: &[u16]) -> EditResult<u64>;
    fn CreateEmptyFragment(&self) -> EditResult<u64>;
    fn CreateFragmentElement(&self, tag: &str) -> EditResult<u64>;
    fn AppendFragmentChild(&self, node: u64, parent: u64) -> EditResult;
    fn AppendTextOrBr(&self, text: &[u16], fragment: u64) -> EditResult;
    fn TestRenderingText(&self, holder: u64) -> EditResult<Vec<u16>>;
}
impl ReplacementFragment {
    /// replace_selection_command.cc:193-312. Native fragment identities remain
    /// in the destination Document throughout test rendering and reconstruction.
    pub fn Prepare<C: ReplacementPreparationContext>(&mut self, c: &C, root: u64) -> EditResult {
        if self.FirstChild(c).is_none() {
            return Ok(());
        }
        c.UpdateStyleAndLayout();
        let control = c.EnclosingTextControl(root);
        let rich = c.IsRichlyEditable(root);
        if !c.HasBeforeTextInsertedListener(root) && control.is_none() && rich {
            return self.RemoveInterchangeNodes(c);
        }
        if !rich {
            let plain = (0..c.ChildCount(&self.root))
                .filter_map(|i| c.ChildAt(&self.root, i))
                .all(|n| c.IsText(&n) || Some(n) == self.LastChild(c) && c.IsInterchangeNewline(n));
            if plain {
                self.RemoveInterchangeNodes(c)?;
                let original = c.FragmentTextContent(self.root);
                let text = c.BeforeTextInserted(root, &original)?;
                if original != text {
                    self.root = c.CreateFragmentFromText(&text)?;
                    self.RemoveInterchangeNodes(c)?;
                }
                self.UpdateTrivialReplacementText(c);
                if control.is_some_and(|(_, textarea)| textarea) {
                    self.UpdateFragmentForTextArea(c)?;
                }
                return Ok(());
            }
        }
        self.UpdateTrivialReplacementText(c);
        let holder = self.InsertFragmentForTestRendering(c, root)?;
        let text_result = c.TestRenderingText(holder);
        // Always restore the original native nodes if lifecycle/text extraction
        // aborts; a failed query must not leave the holder in the editable root.
        let text = match text_result {
            Ok(text) => text,
            Err(error) => {
                self.RestoreAndRemoveTestRenderingNodesToFragment(c, holder)?;
                return Err(error);
            }
        };
        let prepared = self
            .RemoveInterchangeNodesIn(c, holder)
            .and_then(|_| self.RemoveUnrenderedNodes(c, holder));
        self.RestoreAndRemoveTestRenderingNodesToFragment(c, holder)?;
        prepared?;
        let inserted = c.BeforeTextInserted(root, &text)?;
        if text != inserted || !c.IsRichlyEditable(root) {
            c.UpdateStyleAndLayout();
            self.root = c.CreateFragmentFromText(&inserted)?;
            self.UpdateTrivialReplacementText(c);
            if self.FirstChild(c).is_none() {
                return Ok(());
            }
            let holder = self.InsertFragmentForTestRendering(c, root)?;
            let prepared = self
                .RemoveInterchangeNodesIn(c, holder)
                .and_then(|_| self.RemoveUnrenderedNodes(c, holder));
            self.RestoreAndRemoveTestRenderingNodesToFragment(c, holder)?;
            prepared?;
        }
        Ok(())
    }
    fn UpdateTrivialReplacementText<C: ReplacementPreparationContext>(&mut self, c: &C) {
        if self.IsTrivialTextOnlyFragment(c) {
            self.trivial_text = c.FragmentTextContent(self.FirstChild(c).unwrap());
        }
    }
    /// Source :314-353. Only LF-bearing text nodes are recreated; all other
    /// nodes are moved, retaining their native identity and attributes.
    fn UpdateFragmentForTextArea<C: ReplacementPreparationContext>(&mut self, c: &C) -> EditResult {
        let mut replacement = None;
        let mut node = self.FirstChild(c);
        while let Some(current) = node {
            node = c.NextSibling(&current);
            if !c.IsText(&current) || !c.FragmentTextContent(current).contains(&10) {
                if let Some(fragment) = replacement {
                    c.AppendFragmentChild(current, fragment)?;
                }
                continue;
            }
            let fragment = match replacement {
                Some(fragment) => fragment,
                None => {
                    let fragment = c.CreateEmptyFragment()?;
                    while let Some(first) = self.FirstChild(c).filter(|n| *n != current) {
                        c.AppendFragmentChild(first, fragment)?;
                    }
                    replacement = Some(fragment);
                    fragment
                }
            };
            c.AppendTextOrBr(&c.FragmentTextContent(current), fragment)?;
        }
        if let Some(fragment) = replacement {
            self.root = fragment;
        }
        Ok(())
    }
    /// Source :386-425. These are DOM operations, deliberately outside undo.
    fn InsertFragmentForTestRendering<C: ReplacementPreparationContext>(
        &self,
        c: &C,
        root: u64,
    ) -> EditResult<u64> {
        let holder = c.CreateFragmentElement("div")?;
        while let Some(node) = self.FirstChild(c) {
            c.AppendFragmentChild(node, holder)?;
        }
        c.AppendFragmentChild(holder, root)?;
        c.UpdateStyleAndLayout();
        Ok(holder)
    }
    fn RestoreAndRemoveTestRenderingNodesToFragment<C: ReplacementPreparationContext>(
        &self,
        c: &C,
        holder: u64,
    ) -> EditResult {
        while let Some(node) = c.ChildAt(&holder, 0) {
            c.AppendFragmentChild(node, self.root)?;
        }
        c.RemoveFragmentNode(holder)
    }
    fn RemoveUnrenderedNodes<C: ReplacementPreparationContext>(
        &self,
        c: &C,
        holder: u64,
    ) -> EditResult {
        let mut unrendered = Vec::new();
        let mut node = c.ChildAt(&holder, 0);
        while let Some(current) = node {
            if !c.IsNodeRendered(current) && !c.IsTableStructure(current) {
                unrendered.push(current);
            }
            node = crate::core::editing::text_offset_mapping::NextNode(c, &current, Some(&holder));
        }
        for node in unrendered {
            c.RemoveFragmentNode(node)?;
        }
        Ok(())
    }
    fn RemoveInterchangeNodesIn<C: ReplacementFragmentDOM>(
        &mut self,
        c: &C,
        container: u64,
    ) -> EditResult {
        let root = self.root;
        self.root = container;
        let result = self.RemoveInterchangeNodes(c);
        self.root = root;
        result
    }
}
pub trait TrivialReplacementContext: StructuralEditingContext {
    fn Text(&self, node: u64) -> Vec<u16>;
    fn ElementToSplitToAvoidPastingIntoInlineElementsWithStyle(
        &self,
        p: &Position<u64>,
    ) -> EditResult<Option<u64>>;
    fn IsTabSpanElementTextNode(&self, node: u64) -> bool;
    fn ShouldRemoveEndBR(&self, node: u64) -> bool;
    fn RemoveNodeAndPruneAncestors(&self, node: u64) -> EditResult;
    fn NormalizeNbspInInsertedRange(&self, range: &ReplacementRange) -> EditResult;
    fn NormalizeNbspForPasteEnabled(&self) -> bool;
    fn SetEndingReplacementSelection(&self, start: &Position<u64>, end: &Position<u64>);
}
pub trait ReplacementFacts: StructuralEditingContext {
    fn IsStyledInlineHTMLElement(&self, node: u64) -> EditResult<bool>;
    fn LayoutCanHaveChildren(&self, node: u64) -> bool;
    fn InNoQuirksMode(&self) -> bool;
    fn TextData(&self, node: u64) -> Vec<u16>;
    fn CharacterAtVisibleBoundary(&self, position: &Position<u64>, before: bool) -> u16;
}
/// editing_utilities.cc:1227-1244, used by replace_selection_command.cc:1314-1336.
pub fn ElementToSplitToAvoidPastingIntoInlineElementsWithStyle<C: ReplacementFacts>(
    ctx: &C,
    p: &Position<u64>,
) -> EditResult<Option<u64>> {
    let stop = ctx.EnclosingBlock(p);
    let root = p
        .ComputeContainerNode(ctx)
        .and_then(|n| ctx.RootEditable(n));
    let mut node = p.ComputeContainerNode(ctx);
    let mut highest = None;
    while let Some(n) = node.filter(|n| Some(*n) != stop) {
        if (root.is_none() || ctx.Editable(n)) && ctx.IsStyledInlineHTMLElement(n)? {
            highest = Some(n);
        }
        if Some(n) == root {
            break;
        }
        node = ctx.Parent(&n);
    }
    Ok(highest)
}
pub(in crate::core::editing) fn AtEdge<C: StructuralEditingContext>(
    ctx: &C,
    p: &Position<u64>,
    query: StructuralPositionQuery,
) -> bool {
    let visible = ctx.QueryPosition(p, StructuralPositionQuery::Visible);
    !visible.IsNull()
        && visible
            == ctx.QueryPosition(
                &ctx.QueryPosition(p, query),
                StructuralPositionQuery::Visible,
            )
}
/// replace_selection_command.cc:2071-2094.
pub fn ShouldRemoveEndBR<C: ReplacementFacts>(
    ctx: &C,
    node: u64,
    original: &Position<u64>,
) -> bool {
    if !ctx.IsConnected(&node) {
        return false;
    }
    let visible = ctx.QueryPosition(
        &Position::BeforeNode(node),
        StructuralPositionQuery::Visible,
    );
    if ctx.QueryPosition(&visible, StructuralPositionQuery::Previous) == *original {
        return false;
    }
    if !ctx.InNoQuirksMode()
        && AtEdge(ctx, &visible, StructuralPositionQuery::BlockEnd)
        && !AtEdge(ctx, &visible, StructuralPositionQuery::ParagraphStart)
    {
        return true;
    }
    AtEdge(ctx, &visible, StructuralPositionQuery::ParagraphStart)
        && AtEdge(ctx, &visible, StructuralPositionQuery::ParagraphEnd)
}
/// editing_commands_utilities.cc:65-91 and composite_edit_command.cc:498-507.
pub fn RemoveNodeAndPruneAncestors<C: ReplacementFacts>(ctx: &C, node: u64) -> EditResult {
    let parent = ctx.Parent(&node);
    ctx.Remove(node)?;
    PruneAncestors(ctx, parent)
}
/// Shared CompositeEditCommand::Prune after an existing child/text mutation.
/// The source algorithm below is also used by RemoveNodeAndPruneAncestors.
pub fn PruneAncestors<C: ReplacementFacts>(ctx: &C, parent: Option<u64>) -> EditResult {
    let root = parent.and_then(|n| ctx.RootEditable(n));
    let mut current = parent;
    let mut previous = None;
    while let Some(n) = current {
        let facts = ctx.NodeFacts(n);
        if facts.has_layout {
            let mut child = ctx.ChildAt(&n, 0);
            let mut has_rendered = false;
            while let Some(c) = child {
                if Some(c) == previous {
                    child = crate::core::editing::text_offset_mapping::NextSkippingChildren(
                        ctx,
                        &c,
                        Some(&n),
                    );
                    continue;
                }
                if ctx.NodeFacts(c).has_layout {
                    has_rendered = true;
                    break;
                }
                child = crate::core::editing::text_offset_mapping::NextNode(ctx, &c, Some(&n));
            }
            if !ctx.LayoutCanHaveChildren(n) || has_rendered || Some(n) == root {
                if let Some(remove) = previous {
                    ctx.Remove(remove)?;
                }
                return Ok(());
            }
        }
        previous = Some(n);
        current = ctx.Parent(&n);
    }
    Ok(())
}

// Source replace_selection_command.cc:154-191.
use super::structural_editing_context::StructuralPositionQuery as Q;
pub fn PositionAvoidingPrecedingNodes<C: GeneralReplacementContext>(
    c: &C,
    p: &Position<u64>,
) -> Position<u64> {
    let mut p = p.clone();
    if p.AnchorNode().is_none_or(|n| c.EditingIgnoresContent(n)) {
        return p;
    }
    let block = c.EnclosingBlock(&p);
    while p.ComputeContainerNode(c) != block {
        if c.RawLineBreakExists(&p) {
            break;
        }
        let Some(container) = p.ComputeContainerNode(c).filter(|n| c.Parent(n).is_some()) else {
            break;
        };
        let next = Position::AfterNode(container).ToOffsetInAnchor(c);
        if next == p
            || c.EnclosingBlock(&next) != block
            || c.QueryPosition(&p, Q::Visible) != c.QueryPosition(&next, Q::Visible)
        {
            break;
        }
        p = next;
    }
    p
}

// Source replace_selection_command.cc:2097-2212.
use super::{composite_edit_command::InsertNodeAt, smart_replace::IsCharacterSmartReplaceExempt};
pub trait SmartReplacementContext: ReplacementFacts {
    fn CharacterCodePointAtBoundary(&self, position: &Position<u64>, before: bool) -> u32;
    fn CreateReplacementText(&self, text: &[u16]) -> EditResult<u64>;
    fn SmartReplaceUsesMacCharacterSets(&self) -> bool;
}
fn SmartReplaceAtEdge<C: SmartReplacementContext>(c: &C, p: &Position<u64>, start: bool) -> bool {
    let visible = c.QueryPosition(p, StructuralPositionQuery::Visible);
    !visible.IsNull()
        && visible
            == c.QueryPosition(
                &c.QueryPosition(
                    p,
                    if start {
                        StructuralPositionQuery::ParagraphStart
                    } else {
                        StructuralPositionQuery::ParagraphEnd
                    },
                ),
                StructuralPositionQuery::Visible,
            )
}
pub fn AddSpacesForSmartReplace<C: SmartReplacementContext>(
    c: &C,
    range: &mut ReplacementRange,
) -> EditResult {
    c.UpdateStyleAndLayout();
    let visible_end = c.QueryPosition(&range.end, StructuralPositionQuery::Visible);
    let upstream = c.QueryPosition(&visible_end, StructuralPositionQuery::Backward);
    let mut end_node = upstream.ComputeNodeBeforePosition(c);
    let mut end_offset = end_node
        .filter(|n| c.IsText(n))
        .and_then(|n| c.CharacterDataLength(&n))
        .unwrap_or(0);
    if upstream.IsOffsetInAnchor() {
        end_node = upstream.ComputeContainerNode(c);
        end_offset = upstream.ComputeOffsetInContainerNode(c);
    }
    let needs_trailing = !SmartReplaceAtEdge(c, &visible_end, false)
        && !IsCharacterSmartReplaceExempt(
            c.CharacterCodePointAtBoundary(&visible_end, false),
            false,
            c.SmartReplaceUsesMacCharacterSets(),
        );
    if let Some(node) = end_node.filter(|_| needs_trailing) {
        let facts = c.NodeFacts(node);
        let space = if !facts.has_layout || !facts.preserve_spaces {
            0xA0
        } else {
            0x20
        };
        if c.IsText(&node) {
            c.InsertText(node, end_offset, &[space])?;
            if range.end.ComputeContainerNode(c) == Some(node) {
                range.end = Position::new(node, range.end.ComputeOffsetInContainerNode(c) + 1);
            }
        } else {
            let text = c.CreateReplacementText(&[space])?;
            InsertNodeAt(c, text, &Position::AfterNode(node))?;
            range.end = Position::LastPositionInNode(c, &text);
        }
    }
    c.UpdateStyleAndLayout();
    let visible_start = c.QueryPosition(&range.start, StructuralPositionQuery::Visible);
    let downstream = c.QueryPosition(&visible_start, StructuralPositionQuery::Forward);
    let mut start_node = downstream.ComputeNodeAfterPosition(c);
    let mut start_offset = 0;
    if downstream.IsOffsetInAnchor() {
        start_node = downstream.ComputeContainerNode(c);
        start_offset = downstream.ComputeOffsetInContainerNode(c);
    }
    let needs_leading = !SmartReplaceAtEdge(c, &visible_start, true)
        && !IsCharacterSmartReplaceExempt(
            c.CharacterCodePointAtBoundary(&visible_start, true),
            true,
            c.SmartReplaceUsesMacCharacterSets(),
        );
    if let Some(node) = start_node.filter(|_| needs_leading) {
        let facts = c.NodeFacts(node);
        let space = if !facts.has_layout || !facts.preserve_spaces {
            0xA0
        } else {
            0x20
        };
        if c.IsText(&node) {
            c.InsertText(node, start_offset, &[space])?;
            if range.end.ComputeContainerNode(c) == Some(node)
                && range.end.ComputeOffsetInContainerNode(c) != 0
            {
                range.end = Position::new(node, range.end.ComputeOffsetInContainerNode(c) + 1);
            }
        } else {
            let text = c.CreateReplacementText(&[space])?;
            c.InsertBefore(text, node)?;
            range.start = Position::FirstPositionInNode(text);
        }
    }
    Ok(())
}
/// replace_selection_command.cc:2411-2492. As in source, collect changed text
/// identities and replacement strings before synchronous mutation reentry.
pub fn NormalizeNbspInInsertedRange<C: ReplacementFacts>(
    ctx: &C,
    range: &ReplacementRange,
) -> EditResult {
    let source = crate::core::editing::ephemeral_range::EphemeralRange::new(
        range.start.clone(),
        range.end.clone(),
    );
    if source.IsNull() {
        return Ok(());
    }
    ctx.UpdateStyleAndLayout();
    let text_nodes: Vec<_> = source.Nodes(ctx).filter(|n| ctx.IsText(n)).collect();
    let mut replacements = Vec::new();
    for node in text_nodes {
        let facts = ctx.NodeFacts(node);
        if !facts.has_layout || (facts.preserve_spaces && facts.richly_editable) {
            continue;
        }
        let data = ctx.TextData(node);
        if !data.contains(&0xA0) {
            continue;
        }
        let a = if range.start.AnchorNode() == Some(&node) {
            range.start.ComputeOffsetInContainerNode(ctx)
        } else {
            0
        };
        let b = if range.end.AnchorNode() == Some(&node) {
            range.end.ComputeOffsetInContainerNode(ctx)
        } else {
            data.len()
        };
        if a >= b {
            continue;
        }
        let mut replacement = data.clone();
        let mut changed = false;
        for i in a..b.min(data.len()) {
            if data[i] != 0xA0 {
                continue;
            }
            let previous = if i > 0 {
                data[i - 1]
            } else {
                ctx.CharacterAtVisibleBoundary(&Position::new(node, 0), true)
            };
            let next = if i + 1 < data.len() {
                data[i + 1]
            } else {
                ctx.CharacterAtVisibleBoundary(&Position::new(node, data.len()), false)
            };
            if previous != 0
                && !crate::core::editing::editing_utilities::IsWhitespace(previous)
                && next != 0
                && !crate::core::editing::editing_utilities::IsWhitespace(next)
            {
                replacement[i] = 0x20;
                changed = true;
            }
        }
        if changed {
            replacements.push((node, data.len(), replacement));
        }
    }
    for (node, length, text) in replacements {
        ctx.ReplaceText(node, 0, length, &text)?;
    }
    Ok(())
}
pub struct ReplaceSelectionCommand {
    pub options: ReplaceSelectionOptions,
    pub input_type: InputType,
    pub input_event_data: Vec<u16>,
    pub inserted_range: Option<ReplacementRange>,
}
impl ReplaceSelectionCommand {
    pub fn new(options: ReplaceSelectionOptions) -> Self {
        Self {
            options,
            input_type: InputType::kNone,
            input_event_data: Vec::new(),
            inserted_range: None,
        }
    }
    /// `false` is Chromium's ordinary decline into DoApply's general fragment
    /// path. The caller must continue that source path, never report success.
    pub fn PerformTrivialReplace<C: TrivialReplacementContext>(
        &mut self,
        ctx: &C,
        fragment: &ReplacementFragment,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult<bool> {
        self.input_event_data = fragment.trivial_text.clone();
        if !fragment.IsTrivialTextOnlyFragment(ctx) {
            return Ok(false);
        }
        let replacement_text = ctx.Text(fragment.FirstChild(ctx).unwrap());
        if self.options.smart_replace || fragment.interchange_start || fragment.interchange_end {
            return Ok(false);
        }
        if ctx
            .ElementToSplitToAvoidPastingIntoInlineElementsWithStyle(start)?
            .is_some()
        {
            return Ok(false);
        }
        ctx.UpdateStyleAndLayout();
        let node_after = ctx
            .QueryPosition(end, StructuralPositionQuery::Forward)
            .AnchorNode()
            .copied();
        let Some(text_node) = start.ComputeContainerNode(ctx).filter(|n| ctx.IsText(n)) else {
            return Ok(false);
        };
        if Some(text_node) != end.ComputeContainerNode(ctx)
            || ctx.IsTabSpanElementTextNode(text_node)
        {
            return Ok(false);
        }
        let start_offset = start.ComputeOffsetInContainerNode(ctx);
        let end_offset = end.ComputeOffsetInContainerNode(ctx);
        ctx.ReplaceText(
            text_node,
            start_offset,
            end_offset - start_offset,
            &replacement_text,
        )?;
        let inserted_end = Position::new(text_node, start_offset + replacement_text.len());
        ctx.UpdateStyleAndLayout();
        if let Some(node) = node_after.filter(|n| {
            ctx.Parent(n).is_some() && ctx.NodeFacts(*n).br && ctx.ShouldRemoveEndBR(*n)
        }) {
            ctx.RemoveNodeAndPruneAncestors(node)?;
        }
        let range = ReplacementRange {
            start: start.clone(),
            end: inserted_end,
        };
        if matches!(
            self.input_type,
            InputType::kInsertFromPaste | InputType::kInsertFromDrop
        ) && ctx.NormalizeNbspForPasteEnabled()
        {
            ctx.NormalizeNbspInInsertedRange(&range)?;
        }
        ctx.SetEndingReplacementSelection(
            if self.options.select_replacement {
                &range.start
            } else {
                &range.end
            },
            &range.end,
        );
        self.inserted_range = Some(range);
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ReplacementApplyError {
    Aborted,
}
impl From<super::paragraph_editing_context::EditingAborted> for ReplacementApplyError {
    fn from(_: super::paragraph_editing_context::EditingAborted) -> Self {
        Self::Aborted
    }
}
pub trait GeneralReplacementContext:
    TrivialReplacementContext
    + ReplacementFacts
    + ReplacementFragmentDOM
    + super::replacement_markup::ReplacementMarkupContext
    + super::replacement_style::ReplacementStyleContext
    + super::replacement_smart::SmartReplacementContext
    + super::simplify_markup_command::SimplifyMarkupContext
    + super::break_blockquote_command::BreakBlockquoteContext
{
    fn IsTextControlPlaceholderBreak(&self, node: u64) -> bool;
    fn IsInTextControl(&self, node: u64) -> bool;
    fn IsPasswordField(&self, position: &Position<u64>) -> bool;
    fn RebalanceReplacementWhitespace(&self, position: &Position<u64>) -> EditResult;
    fn ApplyReplacementStyleRange(
        &self,
        style: &Self::Style,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult;
    fn ApplyReplacementStyledElement(
        &self,
        element: u64,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult;
    fn HasLinkHref(&self, node: u64) -> bool;
    fn WrapReplacementContentsInDummySpan(&self, node: u64) -> EditResult<u64>;
    fn TabSpanCheckSibling(&self) -> bool;
    fn EditingUseDomPositionApi(&self) -> bool;
    fn RawLineBreakExists(&self, p: &Position<u64>) -> bool;
    fn RemoveCollapsedPlaceholderGate(&self) -> bool;
    fn CreateTextControlPlaceholderBreak(&self, root: u64) -> EditResult<u64>;
}
use super::replacement_markup::ReplacementInsertedNodes;
use super::{
    delete_selection_command::DeleteSelectionCommand,
    insert_paragraph_separator_command::InsertParagraphSeparatorCommand,
};
use super::{
    replacement_markup as markup, replacement_position as positions, replacement_style as styles,
};
use crate::core::editing::text_offset_mapping;
fn Ancestor<C: StructuralEditingContext>(
    c: &C,
    p: &Position<u64>,
    predicate: impl Fn(u64) -> bool,
) -> Option<u64> {
    let mut node = p.AnchorNode().copied();
    while let Some(n) = node {
        if predicate(n) {
            return Some(n);
        }
        node = c.Parent(&n);
    }
    None
}
fn BlockAtAnchor<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    p.AnchorNode()
        .and_then(|n| c.EnclosingBlock(&Position::FirstPositionInNode(*n)))
}
fn IsMailQuote<C: StructuralEditingContext>(c: &C, node: u64) -> bool {
    c.Tag(node, "blockquote") && c.AttributeEquals(node, "type", "cite")
}
fn MailQuote<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    Ancestor(c, p, |n| IsMailQuote(c, n))
}
fn EnclosingListChild<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    Ancestor(c, p, |n| {
        c.Tag(n, "li")
            || c.Parent(&n)
                .is_some_and(|p| markup::IsHtmlListElement(c, p))
    })
}
fn EnclosingCell<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    Ancestor(c, p, |n| c.NodeFacts(n).table_cell)
}
fn EnclosingList<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Option<u64> {
    let container = p.ComputeContainerNode(c)?;
    Ancestor(c, &Position::FirstPositionInNode(container), |n| {
        markup::IsHtmlListElement(c, n)
    })
}
fn First<C: PositionDOM<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::FirstPositionInOrBeforeNode(c, &n)
}
fn Last<C: PositionDOM<Node = u64>>(c: &C, n: u64) -> Position<u64> {
    Position::LastPositionInOrAfterNode(c, &n)
}
fn Visible<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Position<u64> {
    c.UpdateStyleAndLayout();
    c.QueryPosition(p, StructuralPositionQuery::Visible)
}
fn EndVisible<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> Position<u64> {
    c.UpdateStyleAndLayout();
    if let Some(select) = Ancestor(c, p, |n| c.Tag(n, "select")) {
        c.QueryPosition(&Last(c, select), StructuralPositionQuery::Visible)
    } else {
        c.QueryPosition(p, StructuralPositionQuery::Visible)
    }
}
fn IsPlainTextMarkup<C: GeneralReplacementContext>(c: &C, node: u64) -> bool {
    if !c.Tag(node, "div") || !c.HasAttributes(node) {
        return false;
    }
    let first = c.ChildAt(&node, 0);
    if c.ChildCount(&node) == 1 {
        return first.is_some_and(|n| c.IsText(&n) || c.ChildCount(&n) > 0);
    }
    c.ChildCount(&node) == 2
        && first
            .and_then(|n| c.ChildAt(&n, 0))
            .is_some_and(|n| c.IsTabSpanElementTextNode(n))
        && c.ChildAt(&node, 1).is_some_and(|n| c.IsText(&n))
}
fn EnclosingInline<C: StructuralEditingContext>(c: &C, mut node: u64) -> u64 {
    while let Some(parent) = c.Parent(&node) {
        if c.NodeFacts(parent).block_flow || c.Tag(parent, "body") {
            return node;
        }
        let mut sibling = c.PreviousSibling(&node);
        while let Some(n) = sibling {
            if c.NodeFacts(n).block_flow {
                return node;
            }
            sibling = c.PreviousSibling(&n);
        }
        node = parent;
    }
    node
}
fn ShouldMerge<C: StructuralEditingContext>(
    c: &C,
    source: &Position<u64>,
    destination: &Position<u64>,
) -> bool {
    if source.IsNull() || destination.IsNull() {
        return false;
    }
    let (Some(source_node), Some(destination_node), Some(block)) = (
        source.AnchorNode(),
        destination.AnchorNode(),
        BlockAtAnchor(c, source),
    ) else {
        return false;
    };
    (!c.Tag(block, "blockquote") || IsMailQuote(c, block))
        && EnclosingListChild(c, source) == EnclosingListChild(c, destination)
        && EnclosingCell(c, source) == EnclosingCell(c, destination)
        && (!markup::IsHtmlHeaderElement(c, block)
            || BlockAtAnchor(c, destination).is_some_and(|b| {
                ["h1", "h2", "h3", "h4", "h5", "h6"]
                    .iter()
                    .any(|t| c.Tag(block, t) && c.Tag(b, t))
            }))
        && !c.NodeFacts(*source_node).block
        && !c.NodeFacts(*destination_node).block
}
fn QuoteLevel<C: StructuralEditingContext>(c: &C, p: &Position<u64>) -> usize {
    let mut count = 0;
    let mut n = p.AnchorNode().copied();
    while let Some(node) = n {
        if IsMailQuote(c, node) {
            count += 1;
        }
        n = c.Parent(&node);
    }
    count
}
fn ShouldMergeStart<C: GeneralReplacementContext>(
    c: &C,
    range: &ReplacementRange,
    started_at_start: bool,
    newline: bool,
    in_quote: bool,
    moving: bool,
) -> bool {
    if moving {
        return false;
    }
    let start = Visible(c, &range.start);
    let prev = c.QueryPosition(&start, StructuralPositionQuery::PreviousWithinEditable);
    if prev.IsNull() {
        return false;
    }
    if AtEdge(c, &start, StructuralPositionQuery::ParagraphStart)
        && in_quote
        && MailQuote(c, &range.end).is_some()
        && QuoteLevel(c, &prev) == QuoteLevel(c, &range.end)
    {
        return true;
    }
    !started_at_start
        && !newline
        && AtEdge(c, &start, StructuralPositionQuery::ParagraphStart)
        && !start.AnchorNode().is_some_and(|n| c.NodeFacts(*n).br)
        && ShouldMerge(c, &start, &prev)
}
fn ShouldMergeEnd<C: GeneralReplacementContext>(
    c: &C,
    range: &ReplacementRange,
    ended_at_end: bool,
) -> bool {
    let end = EndVisible(c, &range.end);
    let next = c.QueryPosition(&end, StructuralPositionQuery::NextWithinEditable);
    !next.IsNull()
        && !ended_at_end
        && AtEdge(c, &end, StructuralPositionQuery::ParagraphEnd)
        && !end.AnchorNode().is_some_and(|n| c.NodeFacts(*n).br)
        && ShouldMerge(c, &end, &next)
}
impl ReplaceSelectionCommand {
    /// replace_selection_command.cc:1372-1491. Selection deletion and the
    /// paragraph split remain source-owned and happen before fragment insertion.
    fn InsertParagraphSeparatorIfNeeds<C: GeneralReplacementContext>(
        &mut self,
        c: &C,
        fragment: &ReplacementFragment,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> EditResult<Position<u64>> {
        let visible_start = Visible(c, start);
        let visible_end = Visible(c, end);
        let start_edge = AtEdge(c, &visible_start, StructuralPositionQuery::ParagraphStart);
        let end_edge = AtEdge(c, &visible_end, StructuralPositionQuery::ParagraphEnd);
        let root = start.AnchorNode().and_then(|n| c.RootEditable(*n));
        let block = BlockAtAnchor(c, &visible_start);
        let quote = MailQuote(c, start).is_some();
        let plain = start
            .AnchorNode()
            .is_some_and(|n| !c.NodeFacts(*n).richly_editable);
        if start_edge && end_edge && !quote
            || block == root
            || block.is_some_and(|n| c.Tag(n, "li"))
            || plain
        {
            self.options.prevent_nesting = false;
        }
        let mut insertion = if start != end {
            let merge =
                quote || end_edge || AtEdge(c, &visible_start, StructuralPositionQuery::BlockStart);
            DeleteSelectionCommand::ApplyWithOptions(c, (start.clone(), end.clone()), false, merge)?
        } else {
            start.clone()
        };
        if fragment.interchange_start {
            c.UpdateStyleAndLayout();
            let visible = Visible(c, &insertion);
            let next = c.QueryPosition(&visible, StructuralPositionQuery::NextWithinEditable);
            insertion = if AtEdge(c, &visible, StructuralPositionQuery::ParagraphEnd)
                && !AtEdge(c, &visible, StructuralPositionQuery::ParagraphStart)
                && !next.IsNull()
            {
                next
            } else {
                InsertParagraphSeparatorCommand::Apply(c, (insertion.clone(), insertion))?
            };
            c.SetEndingPosition(&insertion);
            c.UpdateStyleAndLayout();
        }
        if start == end
            && self.options.prevent_nesting
            && !quote
            && !AtEdge(c, &insertion, StructuralPositionQuery::ParagraphEnd)
            && !AtEdge(c, &insertion, StructuralPositionQuery::ParagraphStart)
            && (markup::HasBlockLevelContent(c, fragment.root)
                || matches!(
                    self.input_type,
                    InputType::kInsertFromPaste
                        | InputType::kInsertFromDrop
                        | InputType::kInsertReplacementText
                ))
        {
            insertion = InsertParagraphSeparatorCommand::Apply(c, (insertion.clone(), insertion))?;
            c.UpdateStyleAndLayout();
            insertion = c.QueryPosition(&Visible(c, &insertion), StructuralPositionQuery::Previous);
            c.SetEndingPosition(&insertion);
        }
        Ok(insertion)
    }
    /// Source :1493-2068. Helpers mutate one live DOM and one InsertedNodes
    /// owner; all source cleanup/paragraph/smart-replacement phases are reached.
    pub fn DoApply<C: GeneralReplacementContext>(
        &mut self,
        c: &C,
        fragment: &ReplacementFragment,
        start: &Position<u64>,
        end: &Position<u64>,
    ) -> Result<(), ReplacementApplyError> {
        if self.PerformTrivialReplace(c, fragment, start, end)? {
            return Ok(());
        }
        c.UpdateStyleAndLayout();
        let root = start
            .ComputeContainerNode(c)
            .and_then(|n| c.RootEditable(n))
            .ok_or(ReplacementApplyError::Aborted)?;
        if start
            .AnchorNode()
            .is_some_and(|n| !c.NodeFacts(*n).richly_editable)
            && end
                .AnchorNode()
                .is_some_and(|n| !c.NodeFacts(*n).richly_editable)
        {
            self.options.match_style = false;
        }
        let insertion_style = if self.options.match_style {
            Some(c.MergeTypingStyle(c.CaptureStyle(start, false)?)?)
        } else {
            None
        };
        let in_quote = MailQuote(c, start).is_some();
        let plain = start
            .AnchorNode()
            .is_some_and(|n| !c.NodeFacts(*n).richly_editable);
        let ended_at_end = AtEdge(c, end, StructuralPositionQuery::ParagraphEnd);
        let started_at_start = AtEdge(c, start, StructuralPositionQuery::ParagraphStart);
        let mut insertion = self.InsertParagraphSeparatorIfNeeds(c, fragment, start, end)?;
        if MailQuote(c, &insertion).is_some()
            && self.options.prevent_nesting
            && Ancestor(c, &insertion, |n| c.NodeFacts(n).table_structure).is_none()
        {
            let br = super::break_blockquote_command::BreakBlockquoteCommand::ApplyAtCaret(
                c, &insertion,
            )?;
            let node = *br.AnchorNode().ok_or(ReplacementApplyError::Aborted)?;
            insertion = Position::BeforeNode(node).ToOffsetInAnchor(c);
            c.Remove(node)?;
        }
        positions::PrepareWhitespaceAtPositionForSplit(c, &mut insertion)?;
        c.UpdateStyleAndLayout();
        let placeholder = if !c.RemoveCollapsedPlaceholderGate()
            || insertion
                .AnchorNode()
                .is_some_and(|n| c.Editable(*n) && c.NodeFacts(*n).richly_editable)
        {
            positions::ComputePlaceholderToCollapseAt(c, &insertion)
        } else {
            Position::default()
        };
        let forward = c.QueryPosition(&insertion, StructuralPositionQuery::Forward);
        if forward.AnchorNode().is_none() {
            return Ok(());
        }
        let end_br = forward
            .AnchorNode()
            .copied()
            .filter(|n| c.NodeFacts(*n).br)
            .filter(|n| !c.IsInTextControl(*n) || c.IsTextControlPlaceholderBreak(*n));
        let before_br = end_br.map(|n| {
            c.QueryPosition(
                &Visible(c, &Position::BeforeNode(n)),
                StructuralPositionQuery::Previous,
            )
        });
        let mut insertion_block = BlockAtAnchor(c, &insertion);
        if self.options.prevent_nesting
            && insertion_block.is_some_and(|n| n != root && !c.NodeFacts(n).table_cell)
            && !in_quote
        {
            let block = insertion_block.unwrap();
            if AtEdge(c, &insertion, StructuralPositionQuery::BlockEnd)
                && !(AtEdge(c, &insertion, StructuralPositionQuery::BlockStart)
                    && fragment.interchange_end)
            {
                insertion = Position::AfterNode(block).ToOffsetInAnchor(c);
            } else if AtEdge(c, &insertion, StructuralPositionQuery::BlockStart) {
                insertion = Position::BeforeNode(block).ToOffsetInAnchor(c);
            }
        }
        insertion = positions::PositionAvoidingSpecialElementBoundary(c, &insertion)?;
        c.SetTypingStyle(None);
        markup::RemoveHeadContents(c, fragment.root)?;
        c.UpdateStyleAndLayout();
        insertion = positions::PositionAvoidingPrecedingNodes(c, &insertion);
        insertion = positions::PositionOutsideTabSpan(c, &insertion)?;
        styles::HandleStyleSpansBeforeInsertion(
            c,
            &insertion,
            fragment.root,
            self.options.moving_paragraph,
            self.input_type == InputType::kInsertFromPaste,
        )?;
        if fragment.IsEmpty(c) || fragment.FirstChild(c).is_none() {
            return Ok(());
        }
        if !self.options.match_style && EnclosingList(c, &insertion).is_none() {
            if let Some(text) = insertion.ComputeContainerNode(c).filter(|n| c.IsText(n)) {
                let offset = insertion.ComputeOffsetInContainerNode(c);
                if offset > 0 && offset < c.CharacterDataLength(&text).unwrap_or(0) {
                    c.SplitText(text, offset)?;
                    insertion = Position::FirstPositionInNode(text);
                }
            }
            if let Some(split) =
                ElementToSplitToAvoidPastingIntoInlineElementsWithStyle(c, &insertion)?
            {
                if insertion.ComputeContainerNode(c) != c.Parent(&split) {
                    let start = insertion
                        .ComputeNodeAfterPosition(c)
                        .or_else(|| insertion.ComputeContainerNode(c))
                        .ok_or(ReplacementApplyError::Aborted)?;
                    let node = super::composite_edit_command::SplitTreeToNode(
                        c,
                        start,
                        c.Parent(&split).ok_or(ReplacementApplyError::Aborted)?,
                    )?;
                    insertion = if insertion.IsAfterAnchorOrAfterChildren() {
                        Position::AfterNode(node)
                    } else {
                        Position::BeforeNode(node)
                    }
                    .ToOffsetInAnchor(c);
                }
            }
        }
        let first = fragment
            .FirstChild(c)
            .ok_or(ReplacementApplyError::Aborted)?;
        let mut inserted = InsertedNodes::default();
        inserted.ref_node = Some(first);
        let mut node = c.NextSibling(&first);
        c.RemoveFragmentNode(first)?;
        let block_start = BlockAtAnchor(c, &insertion);
        if (markup::IsHtmlListElement(c, first)
            || c.Tag(first, "li")
            || c.ChildAt(&first, 0)
                .is_some_and(|n| markup::IsHtmlListElement(c, n)))
            && block_start.is_some_and(|n| {
                c.NodeFacts(n).list_item && c.Parent(&n).is_some_and(|p| c.Editable(p))
            })
        {
            inserted.ref_node = Some(markup::InsertAsListItems(
                c,
                first,
                block_start.unwrap(),
                &insertion,
                &mut inserted,
            )?);
            if let Some(block) = block_start.filter(|n| c.Tag(*n, "li") && c.ChildCount(n) == 0) {
                c.Remove(block)?;
            }
        } else {
            super::composite_edit_command::InsertNodeAt(c, first, &insertion)?;
            inserted.RespondToNodeInsertion(first);
        }
        let Some(reference) = inserted.ref_node.filter(|n| c.IsConnected(n)) else {
            return Ok(());
        };
        let mut plain_fragment = IsPlainTextMarkup(c, reference);
        let mut scan = node;
        while let Some(n) = scan.filter(|_| plain_fragment) {
            plain_fragment = IsPlainTextMarkup(c, n);
            scan = c.NextSibling(&n);
        }
        // CompositeEditCommand::InsertNodeListAfter captures the original source
        // list before mutation reentry; retain only its native node identities.
        let mut nodes = Vec::new();
        while let Some(n) = node {
            nodes.push(n);
            node = c.NextSibling(&n);
        }
        let mut reference = reference;
        for n in nodes {
            c.RemoveFragmentNode(n)?;
            super::composite_edit_command::InsertNodeAt(c, n, &Position::AfterNode(reference))?;
            inserted.RespondToNodeInsertion(n);
            reference = n;
        }
        inserted.ref_node = Some(reference);
        if insertion
            .AnchorNode()
            .is_some_and(|n| c.NodeFacts(*n).richly_editable)
        {
            markup::RemoveUnrenderedTextNodesAtEnds(c, &mut inserted)?;
            if inserted.ref_node.is_none() {
                return Err(ReplacementApplyError::Aborted);
            }
        }
        c.UpdateStyleAndLayout();
        let Some(first) = inserted.first.filter(|n| c.IsConnected(n)) else {
            return Ok(());
        };
        insertion_block = insertion_block.filter(|n| c.IsConnected(n));
        let start_visible = Visible(c, &First(c, first));
        if !start_visible.IsNull()
            && insertion_block.is_some_and(|block| {
                insertion.AnchorNode() == c.Parent(&block).as_ref()
                    && insertion.ComputeEditingOffset(c) < c.NodeIndex(&block)
            })
            && !AtEdge(c, &start_visible, StructuralPositionQuery::ParagraphStart)
        {
            let br = c.CreateElement("br")?;
            super::composite_edit_command::InsertNodeAt(c, br, &start_visible)?;
        }
        c.UpdateStyleAndLayout();
        if let (Some(br), Some(original)) = (end_br, before_br.as_ref()) {
            if plain_fragment
                || c.NodeFacts(root).inline
                || ShouldRemoveEndBR(c, br, original) && !(fragment.interchange_end && plain)
            {
                inserted.WillRemoveNode(c, br);
                if inserted.ref_node.is_none() {
                    return Err(ReplacementApplyError::Aborted);
                }
                c.RemoveNodeAndPruneAncestors(br)?;
            }
        }
        markup::MakeInsertedContentRoundTrippableWithHTMLTreeBuilder(c, &inserted)?;
        styles::RemoveRedundantStylesAndKeepStyleSpanInline(
            c,
            &mut inserted,
            self.options.moving_paragraph,
        )?;
        if self.options.sanitize_fragment {
            if let Some(first) = inserted.first {
                super::simplify_markup_command::SimplifyMarkupCommand::Apply(
                    c,
                    first,
                    inserted.PastLastLeaf(c),
                )?;
            }
        }
        let mut range = ReplacementRange {
            start: inserted.first.map(|n| First(c, n)).unwrap_or_default(),
            end: inserted
                .LastLeafInserted(c)
                .map(|n| Last(c, n))
                .unwrap_or_default(),
        };
        let merge_end = ShouldMergeEnd(c, &range, ended_at_end);
        if ShouldMergeStart(
            c,
            &range,
            started_at_start,
            fragment.interchange_start,
            in_quote,
            self.options.moving_paragraph,
        ) {
            let moving_start = Visible(c, &range.start);
            let destination = c.QueryPosition(&moving_start, StructuralPositionQuery::Previous);
            if let Some(destination_node) = destination.AnchorNode() {
                let inline = EnclosingInline(c, *destination_node);
                if merge_end && *destination_node != inline && c.NextSibling(&inline).is_some() {
                    let br = c.CreateElement("br")?;
                    c.InsertBefore(br, inserted.ref_node.ok_or(ReplacementApplyError::Aborted)?)?;
                }
            }
            let moving_end = EndVisible(c, &range.end);
            if c.QueryPosition(&moving_end, StructuralPositionQuery::ParagraphStart) == moving_start
            {
                let br = c.CreateElement("br")?;
                super::composite_edit_command::InsertNodeAt(c, br, &moving_end)?;
                if !moving_start.IsConnected(c) {
                    return Ok(());
                }
            }
            c.UpdateStyleAndLayout();
            let moving_start = Visible(c, &moving_start);
            let destination = Visible(c, &destination);
            let paragraph_end =
                c.QueryPosition(&moving_start, StructuralPositionQuery::ParagraphEnd);
            let block = c
                .EnclosingBlock(&moving_start)
                .ok_or(ReplacementApplyError::Aborted)?;
            let moved = super::composite_edit_command::MoveParagraphWithResult(
                c,
                &moving_start,
                &paragraph_end,
                &destination,
                block,
            )?;
            c.SetEndingReplacementSelection(&moved.0, &moved.1);
            c.UpdateStyleAndLayout();
            range.start = c.QueryPosition(&moved.0, StructuralPositionQuery::Forward);
            if !range.end.IsConnected(c) {
                range.end = c.QueryPosition(&moved.1, StructuralPositionQuery::Backward);
            }
        }
        let mut last_to_select = Position::default();
        if fragment.interchange_end {
            let end = EndVisible(c, &range.end);
            let next = c.QueryPosition(&end, StructuralPositionQuery::NextWithinEditable);
            if ended_at_end
                || !AtEdge(c, &end, StructuralPositionQuery::ParagraphEnd)
                || next.IsNull()
            {
                if c.IsInTextControl(root) {
                    if let Some(last) = inserted.LastLeafInserted(c) {
                        if c.NextSibling(&last).is_none() {
                            let br = c.CreateTextControlPlaceholderBreak(root)?;
                            super::composite_edit_command::InsertNodeAt(
                                c,
                                br,
                                &Position::AfterNode(last),
                            )?;
                        }
                        last_to_select = Position::AfterNode(last);
                        c.SetEndingPosition(&last_to_select);
                        last_to_select = Visible(c, &last_to_select);
                    }
                } else if !AtEdge(c, &end, StructuralPositionQuery::ParagraphStart) {
                    c.SetEndingPosition(&end);
                    if let Some(block) = c.EnclosingBlock(&end).filter(|n| c.Tag(*n, "li")) {
                        let li = c.CreateElement("li")?;
                        super::composite_edit_command::InsertNodeAt(
                            c,
                            li,
                            &Position::AfterNode(block),
                        )?;
                        last_to_select = Position::FirstPositionInNode(li);
                        c.SetEndingPosition(&last_to_select);
                    } else {
                        let stop = inserted.first.and_then(|n| c.Parent(&n));
                        let mut node = end.AnchorNode().copied();
                        let mut pasted_quote = false;
                        while let Some(n) = node.filter(|n| Some(*n) != stop) {
                            pasted_quote |= IsMailQuote(c, n);
                            node = c.Parent(&n);
                        }
                        last_to_select = InsertParagraphSeparatorCommand::ApplyForPaste(
                            c,
                            (end.clone(), end),
                            true,
                            !in_quote && pasted_quote,
                        )?;
                    }
                    c.UpdateStyleAndLayout();
                    last_to_select = Visible(c, &last_to_select);
                    if let Some(n) = last_to_select.AnchorNode() {
                        if range.start.IsNull() {
                            range.start = First(c, *n);
                        }
                        range.end = Last(c, *n);
                    }
                }
            } else {
                last_to_select = c.QueryPosition(&next, StructuralPositionQuery::Forward);
            }
        } else {
            self.MergeEndIfNeeded(c, &mut range, merge_end)?;
        }
        if self.options.smart_replace && !c.IsPasswordField(&range.start) {
            super::replacement_smart::AddSpacesForSmartReplace(c, &mut range)?;
        }
        if plain_fragment {
            self.options.match_style = false;
        }
        self.CompleteHtmlReplacement(c, range, insertion_style.as_ref(), &last_to_select)?;
        if !placeholder.IsNull() {
            positions::RemovePlaceholderAt(c, &placeholder)?;
        }
        if matches!(
            self.input_type,
            InputType::kInsertFromPaste | InputType::kInsertFromDrop
        ) && c.NormalizeNbspForPasteEnabled()
        {
            if let Some(range) = &self.inserted_range {
                c.NormalizeNbspInInsertedRange(range)?;
            }
        }
        Ok(())
    }
    fn MergeEndIfNeeded<C: GeneralReplacementContext>(
        &self,
        c: &C,
        range: &mut ReplacementRange,
        merge: bool,
    ) -> EditResult {
        if !merge || self.options.moving_paragraph {
            return Ok(());
        }
        let start = Visible(c, &range.start);
        let end = EndVisible(c, &range.end);
        let same = c.QueryPosition(&start, StructuralPositionQuery::ParagraphStart)
            == c.QueryPosition(&end, StructuralPositionQuery::ParagraphStart);
        let forward = !(same && !AtEdge(c, &start, StructuralPositionQuery::ParagraphStart));
        let mut destination = if forward {
            c.QueryPosition(&end, StructuralPositionQuery::Next)
        } else {
            end.clone()
        };
        let mut moving_start = if forward {
            c.QueryPosition(&end, StructuralPositionQuery::ParagraphStart)
        } else {
            c.QueryPosition(&end, StructuralPositionQuery::Next)
        };
        if c.QueryPosition(&moving_start, StructuralPositionQuery::ParagraphEnd) == destination {
            let br = c.CreateElement("br")?;
            c.InsertBefore(
                br,
                *moving_start
                    .AnchorNode()
                    .ok_or(super::paragraph_editing_context::EditingAborted)?,
            )?;
            c.UpdateStyleAndLayout();
            destination = Visible(c, &Position::BeforeNode(br));
            moving_start = Visible(c, &moving_start);
        }
        let moving_end = c.QueryPosition(&moving_start, StructuralPositionQuery::ParagraphEnd);
        let block = c
            .EnclosingBlock(&moving_start)
            .ok_or(super::paragraph_editing_context::EditingAborted)?;
        let moved = super::composite_edit_command::MoveParagraphWithResult(
            c,
            &moving_start,
            &moving_end,
            &destination,
            block,
        )?;
        c.SetEndingReplacementSelection(&moved.0, &moved.1);
        c.UpdateStyleAndLayout();
        if forward {
            if !range.start.IsConnected(c) {
                range.start = Visible(c, &moved.0);
            }
            range.end = Visible(c, &moved.1);
            if range.end.IsNull() {
                range.end = range.start.clone();
            }
        }
        Ok(())
    }
    /// Source :2214-2285. Range relocation is the DOM's registered live Range;
    /// ApplyStyle consumes the same captured compiled declaration owner.
    fn CompleteHtmlReplacement<C: GeneralReplacementContext>(
        &mut self,
        c: &C,
        range: ReplacementRange,
        style: Option<&C::Style>,
        last: &Position<u64>,
    ) -> EditResult {
        let mut start = Visible(c, &range.start);
        let mut end = EndVisible(c, &range.end);
        if !start.IsNull() && start.IsConnected(c) && !end.IsNull() && end.IsConnected(c) {
            c.RebalanceReplacementWhitespace(&start)?;
            c.RebalanceReplacementWhitespace(&end)?;
            if self.options.match_style {
                let a = c.Track(&start)?;
                let b = c.Track(&end)?;
                c.ApplyReplacementStyleRange(
                    style.ok_or(super::paragraph_editing_context::EditingAborted)?,
                    &start,
                    &end,
                )?;
                start = c.Tracked(&a);
                end = c.Tracked(&b);
            }
            if !last.IsNull() {
                end = last.clone();
            }
            MergeTextNodesAroundPosition(c, &mut start, &mut end)?;
        } else if !last.IsNull() {
            start = last.clone();
            end = last.clone();
        } else {
            return Ok(());
        }
        self.inserted_range = Some(ReplacementRange {
            start: start.clone(),
            end: end.clone(),
        });
        c.SetEndingReplacementSelection(
            if self.options.select_replacement {
                &start
            } else {
                &end
            },
            &end,
        );
        Ok(())
    }
}
#[derive(Default)]
struct InsertedNodes {
    first: Option<u64>,
    last: Option<u64>,
    ref_node: Option<u64>,
}
impl ReplacementInsertedNodes for InsertedNodes {
    fn FirstNodeInserted(&self) -> Option<u64> {
        self.first
    }
    fn RespondToNodeInsertion(&mut self, node: u64) {
        if self.first.is_none() {
            self.first = Some(node);
        }
        self.last = Some(node);
    }
    fn LastLeafInserted<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64> {
        let mut node = self.last?;
        while let Some(child) = c
            .ChildCount(&node)
            .checked_sub(1)
            .and_then(|i| c.ChildAt(&node, i))
        {
            node = child;
        }
        Some(node)
    }
    fn PastLastLeaf<C: PositionTree<Node = u64>>(&self, c: &C) -> Option<u64> {
        text_offset_mapping::NextNode(c, &self.LastLeafInserted(c)?, None)
    }
    fn WillRemoveNodePreservingChildren<C: PositionTree<Node = u64>>(&mut self, c: &C, node: u64) {
        if self.first == Some(node) {
            self.first = text_offset_mapping::NextNode(c, &node, None);
        }
        if self.last == Some(node) {
            self.last = c
                .ChildCount(&node)
                .checked_sub(1)
                .and_then(|i| c.ChildAt(&node, i))
                .or_else(|| text_offset_mapping::NextSkippingChildren(c, &node, None));
        }
        if self.ref_node == Some(node) {
            self.ref_node = text_offset_mapping::NextNode(c, &node, None);
        }
    }
    fn DidReplaceNode(&mut self, old: u64, new: u64) {
        if self.first == Some(old) {
            self.first = Some(new);
        }
        if self.last == Some(old) {
            self.last = Some(new);
        }
        if self.ref_node == Some(old) {
            self.ref_node = Some(new);
        }
    }
    fn WillRemoveNode<C: PositionTree<Node = u64>>(&mut self, c: &C, node: u64) {
        if self.first == Some(node) && self.last == Some(node) {
            self.first = None;
            self.last = None;
        } else if self.first == Some(node) {
            self.first = text_offset_mapping::NextSkippingChildren(c, &node, None);
        } else if self.last == Some(node) {
            let mut ancestor = Some(node);
            self.last = None;
            while let Some(current) = ancestor {
                if let Some(previous) = c.PreviousSibling(&current) {
                    self.last = Some(previous);
                    break;
                }
                ancestor = c.Parent(&current);
            }
        }
        if self
            .ref_node
            .is_some_and(|n| n == node || text_offset_mapping::IsDescendantOf(c, &n, &node))
        {
            self.ref_node = text_offset_mapping::NextSkippingChildren(c, &node, None);
        }
    }
}
use crate::core::editing::editing_utilities::ComputePositionForNodeRemoval;
/// Source :2287-2380, including the UTF-16 surrogate and 1024-unit guards.
fn MergeTextNodesAroundPosition<C: GeneralReplacementContext>(
    c: &C,
    p: &mut Position<u64>,
    other: &mut Position<u64>,
) -> EditResult {
    let offset = p.IsOffsetInAnchor();
    let other_offset = other.IsOffsetInAnchor();
    let text = p
        .ComputeContainerNode(c)
        .filter(|n| offset && c.IsText(n))
        .or_else(|| p.ComputeNodeBeforePosition(c).filter(|n| c.IsText(n)))
        .or_else(|| p.ComputeNodeAfterPosition(c).filter(|n| c.IsText(n)));
    let Some(text) = text else {
        return Ok(());
    };
    let data = c.TextData(text);
    let incomplete = data.first().is_some_and(|u| (0xDC00..=0xDFFF).contains(u))
        || data.last().is_some_and(|u| (0xD800..=0xDBFF).contains(u));
    if !incomplete && data.len() > 1024 {
        return Ok(());
    }
    if let Some(previous) = c.PreviousSibling(&text).filter(|n| c.IsText(n)) {
        let data = c.TextData(previous);
        if incomplete || data.len() <= 1024 {
            c.InsertText(text, 0, &data)?;
            *p = if offset {
                Position::new(
                    p.ComputeContainerNode(c).unwrap(),
                    data.len() + p.ComputeOffsetInContainerNode(c),
                )
            } else {
                ComputePositionForNodeRemoval(c, p, previous)
            };
            if other_offset {
                if other.ComputeContainerNode(c) == Some(text) {
                    *other =
                        Position::new(text, data.len() + other.ComputeOffsetInContainerNode(c));
                } else if other.ComputeContainerNode(c) == Some(previous) {
                    *other = Position::new(text, other.ComputeOffsetInContainerNode(c));
                }
            } else {
                *other = ComputePositionForNodeRemoval(c, other, previous);
            }
            c.Remove(previous)?;
        }
    }
    if let Some(next) = c.NextSibling(&text).filter(|n| c.IsText(n)) {
        let data = c.TextData(next);
        if !incomplete && data.len() > 1024 {
            return Ok(());
        }
        let length = c.CharacterDataLength(&text).unwrap_or(0);
        c.InsertText(text, length, &data)?;
        if !offset {
            *p = ComputePositionForNodeRemoval(c, p, next);
        }
        if other_offset && other.ComputeContainerNode(c) == Some(next) {
            *other = Position::new(text, length + other.ComputeOffsetInContainerNode(c));
        } else {
            *other = ComputePositionForNodeRemoval(c, other, next);
        }
        c.Remove(next)?;
    }
    Ok(())
}
