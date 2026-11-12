//! Editing queries borrow resident LayoutObject, FragmentItems, inline caret,
//! line and OffsetMapping owners. The DOM editing owner retains canonicalization.
use super::*;
use crate::internal::editing::forward::{Position, TextAffinity};
use crate::offset_mapping::OffsetMapping;

impl LayoutNode<'_> {
    pub fn CaretBarWidth(&self) -> f64 {
        self.object.BarCaretWidth().ToDouble()
    }
    pub fn HasBlockCaretTextOverride(&self) -> bool {
        foundation::RuntimeEnabledFeatures::CSSCaretColorWithOptionalSecondValueEnabled()
            && self.object.StyleRef().CaretShape() == foundation::ECaretShape::kBlock
            && !self.object.StyleRef().IsCaretTextColorAuto()
    }

    /// simulated_event_util.cc:48-58. Borrow this exact LayoutBox border box,
    /// transform its center through the resident coordinate mapper, and retain
    /// the same ComputedStyle's absolute zoom. No axis-aligned bbox substitute.
    pub fn SimulatedClickRootFrameCenter(&self) -> Option<(foundation::gfx::Point, f32)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let box_ = unsafe {
            foundation::DynamicTo::<crate::internal::layout_box::LayoutBox>(self.object).as_ref()
        }?;
        let center = box_.PhysicalBorderBoxRect().Center();
        let root=self.object.LocalToAncestorPointPhysical(
            &center,std::ptr::null(),
            crate::internal::map_coordinates_flags::MapCoordinatesFlags::from_mode(
                crate::internal::map_coordinates_flags::MapCoordinatesMode::kTraverseDocumentBoundaries,
            ),
        );
        Some((
            foundation::gfx::Point::new(root.left.Round(), root.top.Round()),
            self.object.StyleRef().EffectiveZoom(),
        ))
    }
    /// Source: inline_cursor.cc:897-920; visible_units_line.cc:49-84,109-152.
    /// Preserve the first collapsible wrap-space and upstream affinity on the
    /// resident line/mapping, rather than deriving a boundary from DOM offsets.
    pub fn EndOfLine(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let line = crate::layout_algo_inline_editing_caret::ContainingLine(&caret.cursor, options);
        if line.IsNull() {
            return None;
        }
        let original = line.PositionForEndOfLine(options.bidi_caret_affinity);
        if original.position.IsNull() {
            return None;
        }
        if !line.Current().Style().NeedsTrailingSpace() || !line.Current().HasSoftWrapToNextLine() {
            return Some(original);
        }
        let Some(anchor) = original
            .position
            .node_id
            .and_then(|id| self.output.Node(id))
        else {
            return Some(original);
        };
        let Some(node) = (unsafe { anchor.object.NonPseudoNode().as_ref() }) else {
            return Some(original);
        };
        let context = OffsetMapping::GetInlineFormattingContextForNode(node, &original.position);
        if context.is_null() {
            return Some(original);
        }
        let Some(mapping) = (unsafe {
            crate::internal::inline_node::InlineNode::GetOffsetMapping(context).as_ref()
        }) else {
            return Some(original);
        };
        let Some(offset) = mapping.GetTextContentOffsetForNode(node, &original.position) else {
            return Some(original);
        };
        if offset == mapping.GetText().length() {
            return Some(original);
        }
        let adjusted = mapping.GetFirstPosition(offset + 1);
        let Some(adjusted_anchor) = adjusted.node_id.and_then(|id| self.output.Node(id)) else {
            return Some(original);
        };
        if !unsafe { adjusted_anchor.object.NonPseudoNode().as_ref() }
            .is_some_and(|node| node.IsTextNode())
        {
            return Some(original);
        }
        let Some(&ch) = mapping
            .GetText()
            .Span16()
            .and_then(|text| text.get(offset as usize))
        else {
            return Some(original);
        };
        if !adjusted_anchor
            .object
            .StyleRef()
            .IsCollapsibleWhiteSpace(ch)
        {
            return Some(original);
        }
        Some(crate::EditingPositionWithAffinity {
            position: adjusted,
            affinity: crate::EditingTextAffinity::Upstream,
        })
    }
    /// Source: core/layout/inline/offset_mapping.cc:47-61,278-299,405-413.
    /// The caller preserves anchor kind until this owning conversion point.
    pub fn EditingTextContentOffset(&self, position: &Position) -> Option<u32> {
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        if position.node_id != Some(node.InputId()) {
            return None;
        }
        let mapping = unsafe { OffsetMapping::GetForLayoutObject(self.object).as_ref() }?;
        mapping.GetTextContentOffsetForNode(node, position)
    }

    /// Source: core/layout/inline/inline_caret_position.cc:527-537. This
    /// converts an already resolved text-content boundary; resolution and bidi
    /// adjustment remain the editing owner's preceding lifecycle stage.
    pub fn EditingPositionForTextContentOffset(
        &self,
        offset: u32,
        affinity: TextAffinity,
    ) -> Option<crate::EditingPositionWithAffinity> {
        let mapping = unsafe { OffsetMapping::GetForLayoutObject(self.object).as_ref() }?;
        let position = match affinity {
            TextAffinity::Downstream => mapping.GetLastPosition(offset),
            TextAffinity::Upstream => mapping.GetFirstPosition(offset),
        };
        (!position.IsNull()).then_some(crate::EditingPositionWithAffinity { position, affinity })
    }

    /// Identity of the resident inline formatting context. No inline owner or
    /// object pointer crosses the Layout boundary.
    pub fn EditingInlineFormattingContext(&self) -> Option<u64> {
        let container = OffsetMapping::GetInlineFormattingContextOfObject(self.object);
        let container = unsafe { container.as_ref() }?;
        Some(container as *const _ as usize as u64)
    }
    pub fn EditingInlineFormattingContextForPosition(
        &self,
        position: &crate::EditingPosition,
    ) -> Option<(u64, bool)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        let container = OffsetMapping::GetInlineFormattingContextForNode(node, position);
        let container = unsafe { container.as_ref() }?;
        Some((
            container as *const _ as usize as u64,
            container.StyleRef().Display() == foundation::EDisplay::kInlineBlock,
        ))
    }

    /// selection_modifier_line.cc:240-251. Return the original DOM identity
    /// only when the position's native inline formatting context is itself an
    /// atomic inline LayoutBlockFlow; the editing owner constructs the outer
    /// before/after position and performs the adjacent-line query.
    pub fn EditingAtomicInlineFormattingContextNode(
        &self,
        position: &crate::EditingPosition,
    ) -> Option<u64> {
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        let context = OffsetMapping::GetInlineFormattingContextForNode(node, position);
        let context = unsafe { context.as_ref() }?;
        if !context.IsInline() {
            return None;
        }
        let node = unsafe { context.NonPseudoNode().as_ref() }?;
        Some(node.InputId())
    }
}

impl LayoutNode<'_> {
    /// Resolve a source position from the node's native coordinate space.
    /// The editing owner projects the anchor and applies AdjustForEditingBoundary.
    pub fn PositionForPoint(
        &self,
        point: &PhysicalOffset,
        options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let result =
            crate::layout_algo_inline_editing_caret::PositionForPoint(self.object, point, options);
        (!result.position.IsNull()).then_some(result)
    }
    pub fn FirstPositionInOrBefore(&self) -> Option<crate::EditingPositionWithAffinity> {
        let result = self.object.FirstPositionInOrBeforeThis();
        (!result.position.IsNull()).then_some(result)
    }
    pub fn LastPositionInOrAfter(&self) -> Option<crate::EditingPositionWithAffinity> {
        let result = self.object.LastPositionInOrAfterThis();
        (!result.position.IsNull()).then_some(result)
    }
}

impl LayoutNode<'_> {
    /// Identity is scoped to the currently retained layout result. Source:
    /// core/layout/inline/line_utils.cc:12-24 (NGContainingLineBoxOf).
    pub fn EditingLine(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
    ) -> Option<u64> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let line = crate::layout_algo_inline_editing_caret::ContainingLine(&caret.cursor, options);
        line.IsNotNull().then(|| line.CurrentItem() as usize as u64)
    }
    /// Source: core/layout/inline/inline_cursor.cc:869-895.
    pub fn StartOfLine(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let line = crate::layout_algo_inline_editing_caret::ContainingLine(&caret.cursor, options);
        if line.IsNull() {
            return None;
        }
        let position = line.PositionForStartOfLine(options.bidi_caret_affinity);
        (!position.position.IsNull()).then_some(position)
    }

    /// Source visible_units_line.cc:287-348. NeedsCaretNormalization,
    /// editable-root containment and boundary adjustment stay with the DOM
    /// editing owner; this is StartPositionForLine<LogicalOrdering> on the
    /// resident InlineCursor. Logical ordering is defined only with bidi caret
    /// affinity disabled in the source implementation.
    pub fn LogicalStartOfLine(
        &self,
        position: &crate::EditingPositionWithAffinity,
        mut options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        options.bidi_caret_affinity = false;
        self.StartOfLine(position, options)
    }

    /// Source visible_units_line.cc:383-431. The shared EndPositionForLine
    /// keeps its soft-wrap-space correction; DOM owns normalization and the
    /// editable-root clamp around this native logical endpoint.
    pub fn LogicalEndOfLine(
        &self,
        position: &crate::EditingPositionWithAffinity,
        mut options: crate::EditingPositionOptions,
    ) -> Option<crate::EditingPositionWithAffinity> {
        options.bidi_caret_affinity = false;
        self.EndOfLine(position, options)
    }
}

impl LayoutNode<'_> {
    pub fn IsBeforeNonCollapsedContent(&self, position: &crate::EditingPosition) -> bool {
        self.AdjacentToNonCollapsedContent(position, true)
    }
    pub fn IsAfterNonCollapsedContent(&self, position: &crate::EditingPosition) -> bool {
        self.AdjacentToNonCollapsedContent(position, false)
    }
    fn AdjacentToNonCollapsedContent(
        &self,
        position: &crate::EditingPosition,
        before: bool,
    ) -> bool {
        let Some(node) = (unsafe { self.object.NonPseudoNode().as_ref() }) else {
            return false;
        };
        let Some(mapping) = (unsafe { OffsetMapping::GetForLayoutObject(self.object).as_ref() })
        else {
            return false;
        };
        mapping.IsAdjacentToNonCollapsedContentForNode(node, position, before)
    }
    pub fn StartOfNextNonCollapsedContent(
        &self,
        position: &crate::EditingPosition,
    ) -> Option<crate::EditingPosition> {
        self.NonCollapsedBoundary(position, true)
    }
    pub fn EndOfLastNonCollapsedContent(
        &self,
        position: &crate::EditingPosition,
    ) -> Option<crate::EditingPosition> {
        self.NonCollapsedBoundary(position, false)
    }
    fn NonCollapsedBoundary(
        &self,
        position: &crate::EditingPosition,
        forward: bool,
    ) -> Option<crate::EditingPosition> {
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        let mapping = unsafe { OffsetMapping::GetForLayoutObject(self.object).as_ref() }?;
        let position = mapping.NonCollapsedBoundaryForNode(node, position, forward);
        (!position.IsNull()).then_some(position)
    }
}

impl LayoutNode<'_> {
    /// Source: core/editing/local_caret_rect.cc:179-193. The default bar shape
    /// uses the host's LocalFrameView::BarCaretWidth in CSS pixels.
    pub fn CaretBounds(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
        bar_width: f64,
    ) -> Option<LayoutRect> {
        let caret = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            options,
        )?;
        let rect = crate::layout_algo_inline_editing_caret_rect::BarCaretRect(
            &caret,
            foundation::LayoutUnit::from_f64(bar_width),
        )?;
        let transform = layout_fragment_fragment_client_rect_queries::transform_to_root(
            self.output,
            caret.cursor.ContainerFragment(),
            true,
        )?;
        let rect = layout_fragment_fragment_client_rect_queries::map_rect(&transform, rect);
        let left = rect.x.floor();
        let top = rect.y.floor();
        Some(LayoutRect {
            x: left,
            y: top,
            width: (rect.x + rect.width).ceil() - left,
            height: (rect.y + rect.height).ceil() - top,
        })
    }
}

fn IsEmptyInline(object: &LayoutObject) -> bool {
    let mut child = object.SlowFirstChild();
    while let Some(current) = unsafe { child.as_ref() } {
        if !current.IsFloatingOrOutOfFlowPositioned() {
            let text = DynamicTo::<crate::internal::layout_text::LayoutText>(current);
            if !((current.IsLayoutInline() && IsEmptyInline(current))
                || unsafe { text.as_ref() }.is_some_and(|text| text.IsAllCollapsibleWhitespace()))
            {
                return false;
            }
        }
        child = current.NextSibling();
    }
    true
}
/// Source: core/editing/visible_units.cc:439-489.
fn HasRenderedDescendants(object: &LayoutObject) -> bool {
    if object.ChildPaintBlockedByDisplayLock() || object.HasLockedAncestorPreventingPaint() {
        return false;
    }
    let block = DynamicTo::<crate::internal::layout_block_flow::LayoutBlockFlow>(object);
    if let Some(block) = unsafe { block.as_ref() } {
        if let Some(data) = unsafe { block.GetInlineNodeData().as_ref() } {
            if data.ItemsData(false).text_content.empty()
                && !data.HasFloatingOrOutOfFlowPositioned()
                && block.HasLineIfEmpty()
            {
                return false;
            }
        }
    }
    let stop = object.NextInPreOrderAfterChildrenDefault();
    let mut child = object.SlowFirstChild();
    while let Some(current) = unsafe { child.as_ref() } {
        if child == stop {
            break;
        }
        let blocked = current.ChildPaintBlockedByDisplayLock();
        if !blocked && !current.NonPseudoNode().is_null() {
            let text = DynamicTo::<crate::internal::layout_text::LayoutText>(current);
            if unsafe { text.as_ref() }.is_some_and(|text| text.HasNonCollapsedText()) {
                return true;
            }
            let box_ = DynamicTo::<LayoutBox>(current);
            if unsafe { box_.as_ref() }
                .is_some_and(|box_| box_.LogicalHeight() != foundation::LayoutUnit::default())
            {
                return true;
            }
            let inline = DynamicTo::<LayoutInline>(current);
            if unsafe { inline.as_ref() }.is_some_and(|inline| {
                IsEmptyInline(current) && !inline.PhysicalLinesBoundingBox().IsEmpty()
            }) {
                return true;
            }
        }
        child = if blocked {
            current.NextInPreOrderAfterChildrenDefault()
        } else {
            current.NextInPreOrderDefault()
        };
    }
    false
}
impl LayoutNode<'_> {
    /// Source VisibleUnits reads GetLayoutObject() before text-specific
    /// AssociatedLayoutObjectOf(), which may resolve another first-letter box.
    pub fn EditingCaretFacts(&self) -> crate::EditingCaretFacts {
        crate::EditingCaretFacts {
            direction: self.object.StyleRef().Direction(),
            used_user_modify: self.object.StyleRef().UsedUserModify(),
            visible: self.object.StyleRef().Visibility() == foundation::EVisibility::kVisible,
            blocked_by_display_lock: self.object.HasLockedAncestorPreventingPaint(),
            writing_mode: self.object.StyleRef().GetWritingMode(),
            is_text: self.object.IsText(),
        }
    }
    pub fn EditingInlineAdjustmentFacts(&self) -> crate::EditingInlineAdjustmentFacts {
        crate::EditingInlineAdjustmentFacts {
            is_text: self.object.IsText(),
            is_inline: self.object.IsInline(),
            is_atomic_inline: self.object.IsAtomicInline(),
            is_in_ng_inline_context: self.object.IsInLayoutNGInlineFormattingContext(),
            is_block_flow: self.object.IsLayoutBlockFlow(),
            is_layout_block: self.object.IsLayoutBlock(),
        }
    }
    pub fn EditingNodeFacts(&self) -> Option<crate::EditingNodeFacts> {
        unsafe { self.object.NonPseudoNode().as_ref() }.map(|node| node.InputEditingFacts())
    }
    pub fn EditingTextFacts(
        &self,
        offset_in_node: usize,
        first_letter_if_on_boundary: bool,
    ) -> Option<crate::EditingTextFacts> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        if !node.IsTextNode() {
            return None;
        }
        let object = crate::internal::layout_object::AssociatedLayoutObjectOf(
            node,
            i32::try_from(offset_in_node).ok()?,
            if first_letter_if_on_boundary {
                crate::internal::layout_object::LayoutObjectSide::kFirstLetterIfOnBoundary
            } else {
                crate::internal::layout_object::LayoutObjectSide::kRemainingTextIfOnBoundary
            },
        );
        let object = unsafe { object.as_ref() }?;
        let text =
            unsafe { DynamicTo::<crate::internal::layout_text::LayoutText>(object).as_ref() }?;
        let start_offset = text.TextStartOffset();
        let offset = u32::try_from(offset_in_node)
            .ok()?
            .checked_sub(start_offset);
        Some(crate::EditingTextFacts {
            start_offset,
            caret_min_offset: text.CaretMinOffset(),
            caret_max_offset: text.CaretMaxOffset(),
            has_non_collapsed_text: text.HasNonCollapsedText(),
            contains_caret_offset: offset
                .and_then(|offset| i32::try_from(offset).ok())
                .is_some_and(|offset| text.ContainsCaretOffset(offset)),
            is_before_non_collapsed_character: offset
                .is_some_and(|offset| text.IsBeforeNonCollapsedCharacter(offset)),
            is_after_non_collapsed_character: offset
                .is_some_and(|offset| text.IsAfterNonCollapsedCharacter(offset)),
            visible: object.StyleRef().Visibility() == foundation::EVisibility::kVisible,
            blocked_by_display_lock: object.HasLockedAncestorPreventingPaint(),
            writing_mode: object.StyleRef().GetWritingMode(),
        })
    }
    pub fn EditingIsRendered(&self) -> bool {
        self.object.StyleRef().Visibility() == foundation::EVisibility::kVisible
            && !self.object.HasLockedAncestorPreventingPaint()
    }
    pub fn EditingCanHaveChildren(&self) -> bool {self.object.CanHaveChildren()}
    /// Source node.cc:2038-2063: live LayoutObject selection-start facts.
    pub fn EditingSelectionStartFacts(
        &self,
    ) -> (bool, foundation::EUserSelect, foundation::EUserDrag) {
        (
            self.object.HasLockedAncestorPreventingPaint(),
            self.object.StyleRef().UsedUserSelect(),
            self.object.StyleRef().UserDrag(),
        )
    }
    /// Source selection_adjuster.cc:67-101, after the caller has found the
    /// rendered node. Inspect resident children, including generated pseudos.
    pub fn EditingIsEmptyTableCell(&self) -> bool {
        let mut object = self.object;
        if object.IsBR() {
            let Some(parent) = (unsafe { object.Parent().as_ref() }) else {
                return false;
            };
            object = parent;
        }
        if !object.IsTableCell() {
            return false;
        }
        let Some(child) = (unsafe { object.SlowFirstChild().as_ref() }) else {
            return true;
        };
        child.IsBR() && child.NextSibling().is_null()
    }
    pub fn EditingIsBR(&self) -> bool {
        self.object.IsBR()
    }
    pub fn EditingIsSVG(&self) -> bool {
        self.object.IsSVG()
    }
    pub fn EditingIsBlockContainer(&self) -> bool {
        self.object.IsLayoutBlockFlow()
            || self.object.IsFlexibleBox()
            || self.object.IsLayoutGridOrGridLanes()
    }
    pub fn EditingIsBlockFlow(&self) -> bool {
        self.object.IsLayoutBlockFlow()
    }
    pub fn EditingHasLogicalHeight(&self) -> bool {
        unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
            .is_some_and(|box_| box_.LogicalHeight() != foundation::LayoutUnit::default())
    }
    pub fn HasRenderedNonAnonymousDescendantsWithHeight(&self) -> bool {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        HasRenderedDescendants(self.object)
    }
    /// Source: core/editing/visible_units.cc:561-581. DOM identity facts stay
    /// with the caller; native geometry and rendered descendants stay here.
    pub fn EndsOfNodeAreVisuallyDistinctPositions(
        &self,
        can_have_children_for_editing: bool,
        is_html_table: bool,
        is_html_marquee: bool,
    ) -> bool {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        if !self.object.IsInline() {
            return true;
        }
        if is_html_table {
            return false;
        }
        if is_html_marquee {
            return true;
        }
        self.object.IsAtomicInline()
            && can_have_children_for_editing
            && unsafe { DynamicTo::<LayoutBox>(self.object).as_ref() }
                .is_some_and(|box_| !box_.StitchedSize().IsEmpty())
            && !HasRenderedDescendants(self.object)
    }
}

#[cfg(feature = "translation_in_progress")]
impl<'a> LayoutNode<'a> {
    pub fn EditingTextRuns(
        &self,
        start: u32,
        end: u32,
        behavior: super::EditingTextIteratorBehavior,
    ) -> Option<super::EditingTextRunCursor<'a>> {
        let node: &'a crate::internal::layout_node_metadata::Node =
            unsafe { self.object.NonPseudoNode().as_ref() }?;
        super::EditingTextRunCursor::New(node, start, end, behavior)
    }
    /// Borrow the current IFC's canonical immutable text; no paragraph/string
    /// projection is materialized at this boundary.
    pub fn EditingTextContent(&self) -> Option<&'a foundation::String> {
        let mapping: &'a OffsetMapping =
            unsafe { OffsetMapping::GetForLayoutObject(self.object).as_ref() }?;
        Some(mapping.GetText())
    }
}

impl LayoutNode<'_> {
    fn InsertionCaretShape(&self) -> crate::internal::caret_rect::CaretShape {
        use crate::internal::caret_rect::CaretShape;
        let style = self.object.StyleRef();
        if style.UsedUserModify() == foundation::EUserModify::kReadOnly {
            return CaretShape::kBar;
        }
        match style.CaretShape() {
            foundation::ECaretShape::kAuto | foundation::ECaretShape::kBar => CaretShape::kBar,
            foundation::ECaretShape::kBlock => CaretShape::kBlock,
            foundation::ECaretShape::kUnderscore => CaretShape::kUnderscore,
        }
    }

    /// Source: local_caret_rect.cc:89-131 and
    /// caret_display_item_client.cc:79-153. Keep the native IFC/fragment painter
    /// and map fallback rectangles through the source container chain.
    pub fn CaretPainterGeometry(
        &self,
        position: &crate::EditingPositionWithAffinity,
        editing_offset: i32,
        bar_width: f64,
    ) -> Option<(u64, crate::caret::NativeCaretGeometry)> {
        self.CaretPainterGeometryForComposition(position, editing_offset, bar_width, false)
    }

    pub fn CaretPainterGeometryForComposition(
        &self,
        position: &crate::EditingPositionWithAffinity,
        editing_offset: i32,
        bar_width: f64,
        composing: bool,
    ) -> Option<(u64, crate::caret::NativeCaretGeometry)> {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        // FrameCaret::GetCaretShape and CaretDisplayItemClient::UpdateStyleAndLayout
        // use the anchor's style, independently of the eventual painter block.
        let style = self.object.StyleRef();
        // caret_rect.cc:426-430: IME preedit always uses a bar, irrespective
        // of the anchor's CSS caret-shape.
        let shape = if composing {
            crate::internal::caret_rect::CaretShape::kBar
        } else {
            self.InsertionCaretShape()
        };
        let mut color = style.ResolvedCaretColor();
        if shape == crate::internal::caret_rect::CaretShape::kBlock
            && style.IsCaretColorAuto()
            && style.IsCaretTextColorAuto()
        {
            color.SetAlpha(0.5);
        }
        let color = crate::internal::boundary::paint_style_export::ColorValue(color);
        let block_text = if shape == crate::internal::caret_rect::CaretShape::kBlock
            && !style.IsCaretTextColorAuto()
            && foundation::RuntimeEnabledFeatures::CSSCaretColorWithOptionalSecondValueEnabled()
        {
            unsafe { self.object.NonPseudoNode().as_ref() }
                .filter(|node| node.IsTextNode())
                .and_then(|node| {
                    self.EditingTextContentOffset(&position.position)
                        .map(|text_content_offset| crate::caret::BlockCaretText {
                            anchor_id: node.InputId(),
                            text_content_offset,
                        })
                })
        } else {
            None
        };
        if let Some(caret) = crate::layout_algo_inline_editing_caret::ResolveForObject(
            self.object,
            position,
            Default::default(),
        ) {
            let shape = if caret.cursor.Current().Style().UsedUserModify()
                == foundation::EUserModify::kReadOnly
            {
                crate::internal::caret_rect::CaretShape::kBar
            } else {
                shape
            };
            if let Some(rect) = crate::layout_algo_inline_editing_caret_rect::CaretRect(
                &caret,
                shape,
                foundation::LayoutUnit::from_f64(bar_width),
            ) {
                let fragment = caret.cursor.ContainerFragment();
                let painter = unsafe { fragment.GetLayoutObject().as_ref() }?;
                let id =
                    unsafe { painter.NonPseudoNode().as_ref() }.map_or(0, |node| node.InputId());
                return Some((
                    id,
                    crate::caret::NativeCaretGeometry {
                        painter: painter as *const _ as usize,
                        fragment: Some(fragment as *const _ as usize),
                        rect,
                        shape,
                        color,
                        block_text,
                    },
                ));
            }
        }
        let mut rect = self.object.LocalCaretRect(editing_offset, shape);
        if rect.IsEmpty() {
            return None;
        }
        let painter = if self.object.IsLayoutBlock() {
            self.object
        } else {
            unsafe { self.object.ContainingBlock().as_ref() }?
        };
        let mut current = self.object;
        while !std::ptr::eq(current, painter) {
            let container = unsafe { current.Container().as_ref() }?;
            rect.offset += current.OffsetFromContainerDefault(container);
            current = container;
        }
        let id = unsafe { painter.NonPseudoNode().as_ref() }.map_or(0, |node| node.InputId());
        Some((
            id,
            crate::caret::NativeCaretGeometry {
                painter: painter as *const _ as usize,
                fragment: None,
                rect,
                shape,
                color,
                block_text,
            },
        ))
    }

    /// Source: local_caret_rect.cc:89-131 and visible_units.cc:546-559.
    /// Preserve the complete transformed quad, including empty local rectangles;
    /// integer-enclosing CaretBounds cannot express source caret equivalence.
    pub fn CaretQuadAndEmpty(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
        editing_offset: i32,
        bar_width: f64,
    ) -> (gfx::QuadF, bool) {
        let (quad, empty, _) =
            self.CaretQuadEmptyAndHorizontal(position, options, editing_offset, bar_width);
        (quad, empty)
    }
    pub fn CaretQuadEmptyAndHorizontal(
        &self,
        inline_position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
        editing_offset: i32,
        bar_width: f64,
    ) -> (gfx::QuadF, bool, bool) {
        self.CaretQuadEmptyAndHorizontalWithShape(
            inline_position,
            options,
            editing_offset,
            bar_width,
            crate::internal::caret_rect::CaretShape::kBar,
        )
    }
    /// FrameCaret::AbsoluteCaretBounds/GetCaretShape (frame_caret.cc:216-236).
    /// Visible-position equivalence and range extents continue to use bars.
    pub fn InsertionCaretQuadAndEmpty(
        &self,
        position: &crate::EditingPositionWithAffinity,
        editing_offset: i32,
        bar_width: f64,
    ) -> (gfx::QuadF, bool) {
        let (quad, empty, _) = self.CaretQuadEmptyAndHorizontalWithShape(
            position,
            Default::default(),
            editing_offset,
            bar_width,
            self.InsertionCaretShape(),
        );
        (quad, empty)
    }
    fn CaretQuadEmptyAndHorizontalWithShape(
        &self,
        inline_position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
        editing_offset: i32,
        bar_width: f64,
        shape: crate::internal::caret_rect::CaretShape,
    ) -> (gfx::QuadF, bool, bool) {
        let _scope = LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        if let Some(caret) = inline_position
            .position
            .node_id
            .and_then(|id| self.output.Node(id))
            .and_then(|node| {
                crate::layout_algo_inline_editing_caret::ResolveForObject(
                    node.object,
                    inline_position,
                    options,
                )
            })
        {
            let shape = if caret.cursor.Current().Style().UsedUserModify()
                == foundation::EUserModify::kReadOnly
            {
                crate::internal::caret_rect::CaretShape::kBar
            } else {
                shape
            };
            if let Some(mut rect) = crate::layout_algo_inline_editing_caret_rect::CaretRect(
                &caret,
                shape,
                foundation::LayoutUnit::from_f64(bar_width),
            ) {
                // CaretRect serves point queries in container coordinates.
                // Source ComputeLocalCaretRectByBoxSide instead returns atomic
                // boxes in their own local coordinates (caret_rect.cc:290-329).
                if caret.cursor.Current().IsAtomicInline() {
                    rect.offset -= caret.cursor.Current().OffsetInContainerFragment();
                }
                let object = unsafe { &*caret.cursor.Current().GetLayoutObject() };
                // Source editing_utilities.cc:1746-1748. Use the original
                // LayoutObject coordinate mapper; no fragment-tree path walk.
                return (
                    object.LocalRectToAbsoluteQuad(&rect, Default::default()),
                    rect.IsEmpty(),
                    object.IsHorizontalWritingMode(),
                );
            }
        }
        let rect = self.object.LocalCaretRect(editing_offset, shape);
        (
            self.object
                .LocalRectToAbsoluteQuad(&rect, Default::default()),
            rect.IsEmpty(),
            self.object.IsHorizontalWritingMode(),
        )
    }
    pub fn CaretQuad(
        &self,
        position: &crate::EditingPositionWithAffinity,
        options: crate::EditingPositionOptions,
        editing_offset: i32,
        bar_width: f64,
    ) -> gfx::QuadF {
        self.CaretQuadAndEmpty(position, options, editing_offset, bar_width)
            .0
    }
}

impl LayoutNode<'_> {
    pub fn EditingIsAtomicInline(&self) -> bool {
        self.object.IsAtomicInline()
    }
}
