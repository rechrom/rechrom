/*
 * This file is part of the DOM implementation for WebCore.
 *
 * Copyright (C) 2006 Apple Computer, Inc.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// Copyright 2006 Apple Computer, Inc. Copyright 2017-2025 The Chromium Authors.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source editing/markers/{document_marker_controller,document_marker_group,
// document_marker,spell_check_marker_list_impl,sorted_document_marker_list_editor,
// overlapping_document_marker_list_editor}.{cc,h}. Context-menu marker subset.
use super::styleable_marker::{
    StyleableMarker, SuggestionMarker, SuggestionMarkerProperties,
    SuggestionMarkerReplacementScope, SuggestionType,
};
use crate::core::editing::visible_position::DownstreamVisiblePositionContext;
use crate::core::editing::{
    compare_positions::CompareDOMPositions,
    editing_boundaries::EditableBoundaryContext,
    iterators::{
        text_iterator::{TextIterator, TextIteratorContext},
        text_iterator_behavior::TextIteratorBehavior,
    },
    position::{Position, PositionDOM, PositionTree},
    position_conversion::PositionTreeScope,
    position_units_word::{self, WordBreakContext},
    selection_adjuster::WordSide,
    text_offset_mapping::{self, InlineContentsContext},
    visible_position::VisiblePositionContext,
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    hash::Hash,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkerType {
    Spelling,
    Grammar,
    TextFragment,
    Glic,
    Composition,
    Suggestion,
    PreviewStylusGesture,
}
impl MarkerType {
    fn index(self) -> usize {
        match self {
            Self::Spelling => 0,
            Self::Grammar => 1,
            Self::TextFragment => 2,
            Self::Glic => 3,
            Self::Composition => 4,
            Self::Suggestion => 5,
            Self::PreviewStylusGesture => 6,
        }
    }
    fn bit(self) -> u16 {
        match self {
            Self::Spelling => 1,
            Self::Grammar => 2,
            Self::TextFragment => 1 << 6,
            Self::Glic => 1 << 8,
            Self::Composition => 1 << 3,
            Self::Suggestion => 1 << 5,
            Self::PreviewStylusGesture => 1 << 9,
        }
    }
    fn spelling(self) -> bool {
        matches!(self, Self::Spelling | Self::Grammar)
    }
}
#[derive(Clone, Copy)]
pub struct MarkerTypes(u16);
impl MarkerTypes {
    pub fn Single(kind: MarkerType) -> Self {
        Self(kind.bit())
    }
    pub fn Union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub fn Misspelling() -> Self {
        Self(3)
    }
    pub fn All() -> Self {
        Self(1 | 2 | 1 << 3 | 1 << 5 | 1 << 6 | 1 << 8 | 1 << 9)
    }
    fn types(self) -> impl Iterator<Item = MarkerType> {
        [
            MarkerType::Spelling,
            MarkerType::Grammar,
            MarkerType::Composition,
            MarkerType::Suggestion,
            MarkerType::TextFragment,
            MarkerType::Glic,
            MarkerType::PreviewStylusGesture,
        ]
        .into_iter()
        .filter(move |kind| self.0 & kind.bit() != 0)
    }
}
pub struct DocumentMarker {
    style: Option<StyleableMarker>,
    suggestion: Option<SuggestionMarker>,
    kind: MarkerType,
    start: Cell<usize>,
    end: Cell<usize>,
    description: Rc<[u16]>,
    hide_suggestion_menu: bool,
    release_group: Box<dyn Fn(usize)>,
}
impl DocumentMarker {
    pub fn Styleable(&self) -> Option<&StyleableMarker> {
        self.style.as_ref()
    }
    pub fn Suggestion(&self) -> Option<&SuggestionMarker> {
        self.suggestion.as_ref()
    }
    pub fn GetType(&self) -> MarkerType {
        self.kind
    }
    pub fn StartOffset(&self) -> usize {
        self.start.get()
    }
    pub fn EndOffset(&self) -> usize {
        self.end.get()
    }
    pub fn Description(&self) -> Rc<[u16]> {
        self.description.clone()
    }
    pub fn ShouldHideSuggestionMenu(&self) -> bool {
        self.hide_suggestion_menu
    }
    fn ShiftOffsets(&self, old: usize, new: usize) {
        self.start.set(self.start.get() + new - old);
        self.end.set(self.end.get() + new - old);
    }
    /// Source document_marker.cc:47-89.
    fn ComputeOffsetsAfterShift(
        &self,
        offset: usize,
        old: usize,
        new: usize,
    ) -> Option<(usize, usize)> {
        let (mut start, mut end) = (self.StartOffset(), self.EndOffset());
        if start > offset || (start == offset && old == 0) {
            start = if start <= offset + old {
                offset + new
            } else {
                start + new - old
            };
        }
        if end > offset {
            end = if end < offset + old {
                offset
            } else {
                end + new - old
            };
        }
        (start < end).then_some((start, end))
    }
}
// Rust's last strong owner is the source Oilpan weak-key clearing boundary.
impl Drop for DocumentMarker {
    fn drop(&mut self) {
        (self.release_group)(self as *const Self as usize);
    }
}
fn identity(marker: &Rc<DocumentMarker>) -> usize {
    Rc::as_ptr(marker) as usize
}
#[derive(Default)]
struct MarkerList {
    markers: Vec<Rc<DocumentMarker>>,
}
impl MarkerList {
    /// Source SpellCheckMarkerListImpl::Add:18-75; overlapping editor:30-47.
    fn Add(&mut self, marker: Rc<DocumentMarker>) {
        if !marker.kind.spelling() {
            let index = self
                .markers
                .partition_point(|m| m.StartOffset() <= marker.StartOffset());
            self.markers.insert(index, marker);
            return;
        }
        if self
            .markers
            .last()
            .is_none_or(|m| m.EndOffset() < marker.StartOffset())
        {
            self.markers.push(marker);
            return;
        }
        let first = self
            .markers
            .partition_point(|m| m.EndOffset() < marker.StartOffset());
        if marker.EndOffset() < self.markers[first].StartOffset() {
            self.markers.insert(first, marker);
            return;
        }
        let last = first
            + self.markers[first..].partition_point(|m| m.StartOffset() <= marker.EndOffset());
        marker
            .start
            .set(marker.StartOffset().min(self.markers[first].StartOffset()));
        marker
            .end
            .set(marker.EndOffset().max(self.markers[last - 1].EndOffset()));
        self.markers.splice(first..last, [marker]);
    }
    fn FirstIntersecting(&self, start: usize, end: usize) -> Option<Rc<DocumentMarker>> {
        let index = self.markers.partition_point(|m| m.EndOffset() <= start);
        self.markers
            .get(index)
            .filter(|m| m.StartOffset() < end)
            .cloned()
    }
    fn Intersecting(&self, start: usize, end: usize, kind: MarkerType) -> Vec<Rc<DocumentMarker>> {
        if kind.spelling() {
            let first = self.markers.partition_point(|m| m.EndOffset() <= start);
            let last = self.markers.partition_point(|m| m.StartOffset() < end);
            return self.markers[first..last].to_vec();
        }
        let last = self.markers.partition_point(|m| m.StartOffset() < end);
        self.markers[..last]
            .iter()
            .filter(|m| m.EndOffset() > start)
            .cloned()
            .collect()
    }
    fn Shift(
        &mut self,
        kind: MarkerType,
        text: &[u16],
        marker_mutation: foundation::editing_marker_scope::MarkerMutationKind,
        offset: usize,
        old: usize,
        new: usize,
    ) -> bool {
        let mut changed = false;
        if kind.spelling() {
            let first = self.markers.partition_point(|m| m.EndOffset() <= offset);
            let mut removed = 0;
            for marker in &self.markers[first..] {
                if marker.StartOffset() < offset + old {
                    removed += 1;
                    changed = true;
                    continue;
                }
                marker.ShiftOffsets(old, new);
                changed = true;
            }
            self.markers.drain(first..first + removed);
        } else {
            self.markers.retain(|marker| {
                // suggestion_marker_list_impl.cc:17-60,112-196.
                if kind==MarkerType::Suggestion {
                    let (a,b)=(marker.StartOffset(),marker.EndOffset());
                    let remove=if marker_mutation==foundation::editing_marker_scope::MarkerMutationKind::SuggestionReplacement{
                        a<offset+old && b>offset && !(a<=offset && b>=offset+old)
                    }else if offset<b && offset+old>a {true}
                    else if new>0 && offset==b {foundation::unicode::IsAlphanumeric(CodePointAt(text,offset) as i32)}
                    else if new>0 && offset==a {foundation::unicode::IsAlphanumeric(CodePointAt(text,offset+new-1) as i32)}
                    else {false};
                    if remove {changed=true;return false;}
                }
                let Some((start, end)) = marker.ComputeOffsetsAfterShift(offset, old, new) else {
                    changed = true;
                    return false;
                };
                if start != marker.StartOffset() || end != marker.EndOffset() {
                    marker.start.set(start);
                    marker.end.set(end);
                    changed = true;
                }
                true
            });
        }
        changed
    }
}
pub struct DocumentMarkerGroup<N> {
    members: RefCell<HashMap<usize, (Weak<DocumentMarker>, N)>>,
}
impl<N: Clone + Eq> DocumentMarkerGroup<N> {
    fn new() -> Self {
        Self {
            members: RefCell::new(HashMap::new()),
        }
    }
    fn Set(&self, marker: &Rc<DocumentMarker>, node: N) {
        self.members
            .borrow_mut()
            .insert(identity(marker), (Rc::downgrade(marker), node));
    }
    fn Erase(&self, marker: &Rc<DocumentMarker>) {
        self.members.borrow_mut().remove(&identity(marker));
    }
    /// Source document_marker_group.cc:34-41. Keep the original weak marker
    /// membership; a query never recreates a marker or its cross-node group.
    pub fn GetMarkerForText(&self, node: &N) -> Option<Rc<DocumentMarker>> {
        self.members
            .borrow()
            .values()
            .find_map(|(marker, text)| (text == node).then(|| marker.upgrade()).flatten())
    }
    /// Source document_marker_group.cc:13-39. Source DOM order, never node ID.
    pub fn Bounds<C: PositionDOM<Node = N> + PositionTreeScope>(
        &self,
        ctx: &C,
    ) -> Option<(Position<N>, Position<N>)> {
        let mut start = None;
        let mut end = None;
        for (marker, node) in self.members.borrow().values() {
            let Some(marker) = marker.upgrade() else {
                continue;
            };
            let a = Position::new(node.clone(), marker.StartOffset());
            let b = Position::new(node.clone(), marker.EndOffset());
            if start
                .as_ref()
                .is_none_or(|s| CompareDOMPositions(ctx, &a, s) == Some(std::cmp::Ordering::Less))
            {
                start = Some(a)
            }
            if end.as_ref().is_none_or(|e| {
                CompareDOMPositions(ctx, &b, e) == Some(std::cmp::Ordering::Greater)
            }) {
                end = Some(b)
            }
        }
        Some((start?, end?))
    }
}
/// These typed source effects are consumed in order by the document embedding.
/// Native marker paint includes the first-letter LayoutText and the existing AX
/// cache; visual overflow is tested by the original HighlightStyleUtils owner.
pub enum MarkerEffect<N> {
    InvalidatePaint(N),
    InvalidateVisualOverflow { node: N, kind: MarkerType },
}
pub struct DocumentMarkerController<N: Clone + Eq + Hash> {
    document: u128,
    maps: RefCell<[HashMap<N, MarkerList>; 7]>,
    groups: Rc<RefCell<HashMap<usize, (Weak<DocumentMarker>, Rc<DocumentMarkerGroup<N>>)>>>,
    possibly_existing: Cell<u16>,
}
impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    /// Source MarkersFor/MarkersForNode exposes the resident per-Text lists to
    /// Paint. Export only immutable composition offsets, without another DOM
    /// iterator or copying the controller's marker/group ownership.
    pub fn CompositionMarkerRanges(&self) -> Vec<(N, usize, usize)> {
        self.maps.borrow()[MarkerType::Composition.index()]
            .iter()
            .flat_map(|(node, list)| {
                list.markers
                    .iter()
                    .map(|marker| (node.clone(), marker.StartOffset(), marker.EndOffset()))
            })
            .collect()
    }
    /// Source document_marker_controller.cc:382-410. Walk the same DOM text
    /// iterator and remove from the original per-node marker lists.
    pub fn RemoveMarkersInRange<
        C: crate::core::editing::iterators::text_iterator::TextIteratorContext<Node = N>,
    >(
        &self,
        context: &C,
        start: &Position<N>,
        end: &Position<N>,
        types: MarkerTypes,
    ) -> Vec<MarkerEffect<N>> {
        use crate::core::editing::iterators::{
            text_iterator::TextIterator, text_iterator_behavior::TextIteratorBehavior,
        };
        let mut iterator = TextIterator::new(context, start, end, TextIteratorBehavior::default());
        let mut effects = Vec::new();
        while !iterator.AtEnd() {
            if !self.PossiblyHasMarkers(types) {
                break;
            }
            let node = iterator.CurrentContainer();
            if context.IsText(&node) {
                let start = iterator.StartOffsetInCurrentContainer();
                let end = iterator.EndOffsetInCurrentContainer();
                effects.extend(self.RemoveMarkers(&node, start, end - start, types));
            }
            iterator.Advance();
        }
        effects
    }
    pub fn new(document: u128) -> Self {
        Self {
            document,
            maps: RefCell::new(std::array::from_fn(|_| HashMap::new())),
            groups: Rc::new(RefCell::new(HashMap::new())),
            possibly_existing: Cell::new(0),
        }
    }
    pub fn PossiblyHasMarkers(&self, types: MarkerTypes) -> bool {
        self.possibly_existing.get() & types.0 != 0
    }
    fn Check<C: PositionDOM<Node = N>>(&self, ctx: &C, node: &N) {
        assert_eq!(
            ctx.DocumentIdentity(node),
            self.document,
            "marker belongs to its original Document"
        );
    }
    /// Source controller.cc:412-486. The original DOM TextIterator creates one
    /// marker per rendered Text run and one shared cross-node group.
    pub fn AddMarkerInternal<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        kind: MarkerType,
        description: Rc<[u16]>,
        hide: bool,
    ) -> Vec<MarkerEffect<N>> {
        self.AddMarkerWithPayload(ctx, start, end, kind, description, hide, || (None, None))
    }
    fn AddMarkerWithPayload<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        kind: MarkerType,
        description: Rc<[u16]>,
        hide: bool,
        mut payload: impl FnMut() -> (Option<StyleableMarker>, Option<SuggestionMarker>),
    ) -> Vec<MarkerEffect<N>> {
        assert!(
            !ctx.IsFlatTreeStrategy(),
            "DocumentMarkerController adds markers through source DOM TextIterator"
        );
        if let Some(node) = start.AnchorNode() {
            self.Check(ctx, node)
        }
        let group = Rc::new(DocumentMarkerGroup::new());
        let mut effects = Vec::new();
        let mut it = TextIterator::new(ctx, start, end, TextIteratorBehavior::default());
        while !it.AtEnd() {
            let (a, b) = (
                it.StartOffsetInCurrentContainer(),
                it.EndOffsetInCurrentContainer(),
            );
            let node = it.CurrentContainer();
            if a != b && ctx.IsText(&node) {
                assert!(b <= ctx.CharacterDataLength(&node).expect("Text"));
                let groups = Rc::downgrade(&self.groups);
                let (style, suggestion) = payload();
                let marker = Rc::new(DocumentMarker {
                    style,
                    suggestion,
                    kind,
                    start: Cell::new(a),
                    end: Cell::new(b),
                    description: description.clone(),
                    hide_suggestion_menu: hide,
                    release_group: Box::new(move |id| {
                        if let Some(groups) = groups.upgrade() {
                            groups.borrow_mut().remove(&id);
                        }
                    }),
                });
                self.maps.borrow_mut()[kind.index()]
                    .entry(node.clone())
                    .or_default()
                    .Add(marker.clone());
                self.possibly_existing
                    .set(self.possibly_existing.get() | kind.bit());
                group.Set(&marker, node.clone());
                self.groups
                    .borrow_mut()
                    .insert(identity(&marker), (Rc::downgrade(&marker), group.clone()));
                effects.push(MarkerEffect::InvalidatePaint(node.clone()));
                effects.push(MarkerEffect::InvalidateVisualOverflow { node, kind });
            }
            it.Advance();
        }
        effects
    }
    pub fn DidUpdateCharacterData(
        &self,
        node: &N,
        text: &[u16],
        marker_mutation: foundation::editing_marker_scope::MarkerMutationKind,
        offset: usize,
        old: usize,
        new: usize,
    ) -> Vec<MarkerEffect<N>> {
        if !self.PossiblyHasMarkers(MarkerTypes::All()) {
            return Vec::new();
        }
        let mut effects = Vec::new();
        let mut changed = false;
        let mut maps = self.maps.borrow_mut();
        for kind in MarkerTypes::All().types() {
            let map = &mut maps[kind.index()];
            if let Some(list) = map.get_mut(node) {
                changed |= list.Shift(kind, text, marker_mutation, offset, old, new);
                if list.markers.is_empty() {
                    effects.push(MarkerEffect::InvalidateVisualOverflow {
                        node: node.clone(),
                        kind,
                    });
                    map.remove(node);
                }
            }
            if map.is_empty() {
                self.possibly_existing
                    .set(self.possibly_existing.get() & !kind.bit());
            }
        }
        if changed {
            effects.push(MarkerEffect::InvalidatePaint(node.clone()));
        }
        effects
    }
    /// Source controller.cc:490-564 and sorted/overlapping list MoveMarkers.
    pub fn MoveMarkers(&self, src: &N, length: usize, dst: &N) -> Vec<MarkerEffect<N>> {
        if length == 0 || !self.PossiblyHasMarkers(MarkerTypes::All()) {
            return Vec::new();
        }
        assert!(src != dst, "source and destination marker lists differ");
        let mut effects = Vec::new();
        let mut dirty = false;
        let mut maps = self.maps.borrow_mut();
        for kind in MarkerTypes::All().types() {
            let map = &mut maps[kind.index()];
            if map.is_empty() {
                continue;
            }
            let Some(mut source) = map.remove(src) else {
                return effects;
            };
            let mut destination = map.remove(dst).unwrap_or_default();
            let end_offset = length - 1;
            let mut moved = false;
            if kind.spelling() {
                let count = source
                    .markers
                    .partition_point(|marker| marker.StartOffset() <= end_offset);
                for marker in &source.markers[..count] {
                    if marker.EndOffset() > end_offset {
                        marker.end.set(end_offset);
                    }
                    destination.Add(marker.clone());
                }
                source.markers.drain(..count);
                moved = count != 0;
            } else {
                source.markers.retain(|marker| {
                    if marker.StartOffset() > end_offset {
                        return true;
                    }
                    if marker.EndOffset() <= end_offset {
                        destination.Add(marker.clone());
                        moved = true;
                    }
                    false
                });
            }
            if moved {
                dirty = true;
                effects.push(MarkerEffect::InvalidateVisualOverflow {
                    node: dst.clone(),
                    kind,
                });
                for marker in &destination.markers {
                    if let Some((_, group)) = self.groups.borrow().get(&identity(marker)) {
                        group.Set(marker, dst.clone());
                    }
                }
            }
            if source.markers.is_empty() {
                effects.push(MarkerEffect::InvalidateVisualOverflow {
                    node: src.clone(),
                    kind,
                });
            } else {
                map.insert(src.clone(), source);
            }
            if !destination.markers.is_empty() {
                map.insert(dst.clone(), destination);
            }
            if map.is_empty() {
                self.possibly_existing
                    .set(self.possibly_existing.get() & !kind.bit());
            }
        }
        if dirty {
            effects.push(MarkerEffect::InvalidatePaint(dst.clone()));
        }
        effects
    }
    pub fn RemoveMarkers(
        &self,
        node: &N,
        start: usize,
        length: usize,
        types: MarkerTypes,
    ) -> Vec<MarkerEffect<N>> {
        if length == 0 {
            return Vec::new();
        }
        let end = start + length;
        let mut effects = Vec::new();
        let mut maps = self.maps.borrow_mut();
        for kind in types.types() {
            let map = &mut maps[kind.index()];
            if let Some(list) = map.get_mut(node) {
                let old = list.markers.len();
                list.markers.retain(|marker| {
                    if marker.EndOffset() <= start || marker.StartOffset() >= end {
                        return true;
                    }
                    if let Some((_, group)) = self.groups.borrow_mut().remove(&identity(marker)) {
                        group.Erase(marker)
                    }
                    false
                });
                if list.markers.len() != old {
                    effects.push(MarkerEffect::InvalidateVisualOverflow {
                        node: node.clone(),
                        kind,
                    });
                    effects.push(MarkerEffect::InvalidatePaint(node.clone()));
                }
                if list.markers.is_empty() {
                    map.remove(node);
                }
            }
            if map.is_empty() {
                self.possibly_existing
                    .set(self.possibly_existing.get() & !kind.bit());
            }
        }
        effects
    }
    pub fn FirstMarkerIntersectingOffsetRange<C: PositionDOM<Node = N>>(
        &self,
        ctx: &C,
        node: &N,
        start: usize,
        end: usize,
        types: MarkerTypes,
    ) -> Option<Rc<DocumentMarker>> {
        self.Check(ctx, node);
        if !self.PossiblyHasMarkers(types)
            || start == end && (start == 0 || Some(start) == ctx.CharacterDataLength(node))
        {
            return None;
        }
        let maps = self.maps.borrow();
        types
            .types()
            .find_map(|kind| maps[kind.index()].get(node)?.FirstIntersecting(start, end))
    }
    pub fn GetMarkerGroupForMarker(
        &self,
        marker: &Rc<DocumentMarker>,
    ) -> Option<Rc<DocumentMarkerGroup<N>>> {
        self.groups
            .borrow()
            .get(&identity(marker))
            .filter(|(weak, _)| weak.upgrade().is_some_and(|m| Rc::ptr_eq(&m, marker)))
            .map(|(_, group)| group.clone())
    }
    pub fn FirstMarkerGroupAroundPosition<C>(
        &self,
        ctx: &C,
        position: &Position<N>,
        types: MarkerTypes,
    ) -> Option<Rc<DocumentMarkerGroup<N>>>
    where
        C: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        self.FirstMarkerAroundPosition(ctx, position, types)
            .and_then(|marker| self.GetMarkerGroupForMarker(&marker))
    }
    pub fn FirstMarkerAroundPosition<C>(
        &self,
        ctx: &C,
        position: &Position<N>,
        types: MarkerTypes,
    ) -> Option<Rc<DocumentMarker>>
    where
        C: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        let mut found = None;
        self.Around(ctx, position, |node, start, end| {
            found = self.FirstMarkerIntersectingOffsetRange(ctx, &node, start, end, types);
            found.is_none()
        });
        found
    }
    pub fn MarkersAroundPosition<C>(
        &self,
        ctx: &C,
        position: &Position<N>,
        types: MarkerTypes,
    ) -> Vec<(N, Rc<DocumentMarker>)>
    where
        C: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        let mut result = Vec::new();
        if !self.PossiblyHasMarkers(types) {
            return result;
        }
        self.Around(ctx, position, |node, start, end| {
            if start == end && (start == 0 || Some(start) == ctx.CharacterDataLength(&node)) {
                return true;
            }
            let maps = self.maps.borrow();
            for kind in types.types() {
                if let Some(list) = maps[kind.index()].get(&node) {
                    for marker in list.Intersecting(start, end, kind) {
                        result.push((node.clone(), marker));
                    }
                }
            }
            true
        });
        result
    }
    /// Source controller.cc:177-187,628-664,759-819. Word boundaries and flat
    /// range traversal come from existing Editing owners, never DOM text concat.
    fn Around<C>(
        &self,
        ctx: &C,
        position: &Position<N>,
        mut visit: impl FnMut(N, usize, usize) -> bool,
    ) where
        C: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        if position.IsNull() {
            return;
        }
        let mut start = position_units_word::StartOfWordPosition(
            ctx,
            position,
            WordSide::PreviousWordIfOnBoundary,
        );
        let mut end =
            position_units_word::EndOfWordPosition(ctx, position, WordSide::NextWordIfOnBoundary);
        if start.IsNull() {
            start = position.clone()
        }
        if end.IsNull() {
            end = position.clone()
        }
        if ctx.VisibleContext().CompareFlatPositions(&start, &end)
            == Some(std::cmp::Ordering::Greater)
        {
            return;
        }
        let start_node = start.ComputeContainerNode(ctx);
        let end_node = end.ComputeContainerNode(ctx);
        let start_offset = start.ComputeOffsetInContainerNode(ctx);
        let end_offset = end.ComputeOffsetInContainerNode(ctx);
        let past = end.NodeAsRangePastLastNode(ctx);
        let mut node = text_offset_mapping::NodeAsRangeFirstNode(ctx, &start);
        while node != past {
            let Some(current) = node else { break };
            node = text_offset_mapping::NextNode(ctx, &current, None);
            if !ctx.IsText(&current) {
                continue;
            }
            let a = if Some(&current) == start_node.as_ref() {
                start_offset
            } else {
                0
            };
            let b = if Some(&current) == end_node.as_ref() {
                end_offset
            } else {
                ctx.CharacterDataLength(&current).expect("Text")
            };
            if !visit(current, a, b) {
                return;
            }
        }
    }
}

// Source document_marker_controller.cc:667-697,739-745. A noncollapsed
// ephemeral range checks its first Text container only; the caret overload uses
// the original flat-tree word query. Do not replace this with a tree walk.
impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    pub fn FirstMarkerIntersectingEphemeralRange<D, F>(
        &self,
        dom: &D,
        flat: &F,
        start: &Position<N>,
        end: &Position<N>,
        types: MarkerTypes,
    ) -> Option<Rc<DocumentMarker>>
    where
        D: PositionDOM<Node = N> + PositionTreeScope,
        F: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        if start.IsNull() || end.IsNull() {
            return None;
        }
        if start == end {
            let position =
                super::super::position_conversion::ToPositionInFlatTree(dom, flat, start);
            return self.FirstMarkerAroundPosition(flat, &position, types);
        }
        let node = start.ComputeContainerNode(dom)?;
        if !dom.IsText(&node) {
            return None;
        }
        let a = start.ComputeOffsetInContainerNode(dom);
        let b = if end.ComputeContainerNode(dom).as_ref() == Some(&node) {
            end.ComputeOffsetInContainerNode(dom)
        } else {
            dom.CharacterDataLength(&node).expect("Text length")
        };
        self.FirstMarkerIntersectingOffsetRange(dom, &node, a, b, types)
    }
    pub fn FirstMarkerGroupIntersectingEphemeralRange<D, F>(
        &self,
        dom: &D,
        flat: &F,
        start: &Position<N>,
        end: &Position<N>,
        types: MarkerTypes,
    ) -> Option<Rc<DocumentMarkerGroup<N>>>
    where
        D: PositionDOM<Node = N> + PositionTreeScope,
        F: TextIteratorContext<Node = N>
            + InlineContentsContext
            + WordBreakContext
            + EditableBoundaryContext,
    {
        self.FirstMarkerIntersectingEphemeralRange(dom, flat, start, end, types)
            .and_then(|marker| self.GetMarkerGroupForMarker(&marker))
    }
}

// ICU U16_GET, suggestion_marker_list_impl.cc:17-21: decode either surrogate.
fn CodePointAt(text: &[u16], index: usize) -> u32 {
    let c = text[index];
    if (0xdc00..=0xdfff).contains(&c) && index > 0 && (0xd800..=0xdbff).contains(&text[index - 1]) {
        return 0x10000 + ((u32::from(text[index - 1]) - 0xd800) << 10) + (u32::from(c) - 0xdc00);
    }
    if (0xd800..=0xdbff).contains(&c)
        && text
            .get(index + 1)
            .is_some_and(|next| (0xdc00..=0xdfff).contains(next))
    {
        return 0x10000 + ((u32::from(c) - 0xd800) << 10) + (u32::from(text[index + 1]) - 0xdc00);
    }
    u32::from(c)
}

// The same controller/maps own all styleable markers. Source controller
// cc:281-340,823-876,1189-1282; no separate suggestion interval store exists.
impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    pub fn AddCompositionMarker<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        style: &StyleableMarker,
    ) -> Vec<MarkerEffect<N>> {
        self.AddMarkerWithPayload(
            ctx,
            start,
            end,
            MarkerType::Composition,
            Rc::from([]),
            false,
            || (Some(style.clone()), None),
        )
    }
    pub fn AddPreviewStylusGestureMarker<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        background: foundation::Color,
    ) -> Vec<MarkerEffect<N>> {
        use super::styleable_marker::{ImeTextSpanThickness, ImeTextSpanUnderlineStyle};
        let style = StyleableMarker {
            thickness: ImeTextSpanThickness::None,
            underline_style: ImeTextSpanUnderlineStyle::None,
            background_color: background,
            ..Default::default()
        };
        self.AddMarkerWithPayload(
            ctx,
            start,
            end,
            MarkerType::PreviewStylusGesture,
            Rc::from([]),
            false,
            || (Some(style.clone()), None),
        )
    }
    pub fn AddSuggestionMarker<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        properties: &SuggestionMarkerProperties,
    ) -> Vec<MarkerEffect<N>> {
        self.AddMarkerWithPayload(
            ctx,
            start,
            end,
            MarkerType::Suggestion,
            Rc::from([]),
            properties.hide_suggestion_menu,
            || {
                (
                    Some(properties.style.clone()),
                    Some(SuggestionMarker::new(properties)),
                )
            },
        )
    }
    pub fn MarkersIntersectingRange<C: PositionDOM<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        types: MarkerTypes,
    ) -> Vec<(N, Rc<DocumentMarker>)> {
        let mut result = Vec::new();
        if !self.PossiblyHasMarkers(types) {
            return result;
        }
        let (start_node, end_node) = (
            start.ComputeContainerNode(ctx),
            end.ComputeContainerNode(ctx),
        );
        let (start_offset, end_offset) = (
            start.ComputeOffsetInContainerNode(ctx),
            end.ComputeOffsetInContainerNode(ctx),
        );
        let past = end.NodeAsRangePastLastNode(ctx);
        let mut node = text_offset_mapping::NodeAsRangeFirstNode(ctx, start);
        while node != past {
            let Some(current) = node else {
                break;
            };
            node = text_offset_mapping::NextNode(ctx, &current, None);
            if !ctx.IsText(&current) {
                continue;
            }
            let max = ctx
                .CharacterDataLength(&current)
                .expect("source Text length");
            let a = if Some(&current) == start_node.as_ref() {
                start_offset
            } else {
                0
            };
            let b = if Some(&current) == end_node.as_ref() {
                end_offset
            } else {
                max
            };
            // Preserve the exact source's second (max,0) boundary guard.
            if a == 0 && b == 0 || a == max && b == 0 {
                continue;
            }
            let maps = self.maps.borrow();
            for kind in types.types() {
                if let Some(list) = maps[kind.index()].get(&current) {
                    for marker in list.Intersecting(a, b, kind) {
                        result.push((current.clone(), marker));
                    }
                }
            }
        }
        result
    }
    pub fn RemoveSuggestionMarkerInRangeOnFinish<C: PositionDOM<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
    ) -> Vec<MarkerEffect<N>> {
        let pairs = self.MarkersIntersectingRange(
            ctx,
            start,
            end,
            MarkerTypes::Single(MarkerType::Suggestion),
        );
        let mut effects = Vec::new();
        for (node, marker) in pairs {
            let suggestion = marker.Suggestion().expect("source suggestion marker");
            if suggestion.NeedsRemovalOnFinishComposing() {
                self.RemoveSuggestionTag(&node, suggestion.Tag(), false);
                effects.push(MarkerEffect::InvalidatePaint(node));
            }
        }
        effects
    }
    fn RemoveSuggestionTag(&self, node: &N, tag: i32, require_found: bool) -> bool {
        let mut maps = self.maps.borrow_mut();
        let map = &mut maps[MarkerType::Suggestion.index()];
        let Some(list) = map.get_mut(node) else {
            assert!(!require_found, "source marker node map entry");
            return false;
        };
        let index = list
            .markers
            .iter()
            .position(|marker| marker.Suggestion().expect("source suggestion").Tag() == tag);
        if let Some(index) = index {
            list.markers.remove(index);
        } else if require_found {
            return false;
        }
        if list.markers.is_empty() {
            map.remove(node);
        }
        if map.is_empty() {
            self.possibly_existing
                .set(self.possibly_existing.get() & !MarkerType::Suggestion.bit());
        }
        index.is_some()
    }
    pub fn RemoveSuggestionMarkerByTag(&self, node: &N, tag: i32) -> Vec<MarkerEffect<N>> {
        if !self.PossiblyHasMarkers(MarkerTypes::Single(MarkerType::Suggestion)) {
            return Vec::new();
        }
        if self.RemoveSuggestionTag(node, tag, true) {
            vec![MarkerEffect::InvalidatePaint(node.clone())]
        } else {
            Vec::new()
        }
    }
    fn RemoveFirstSuggestionTypeInNode(&self, node: &N, kind: SuggestionType) -> bool {
        let mut maps = self.maps.borrow_mut();
        let map = &mut maps[MarkerType::Suggestion.index()];
        let Some(list) = map.get_mut(node) else {
            return false;
        };
        let Some(index) = list.markers.iter().position(|marker| {
            marker
                .Suggestion()
                .expect("source suggestion")
                .GetSuggestionType()
                == kind
        }) else {
            return false;
        };
        list.markers.remove(index);
        if list.markers.is_empty() {
            map.remove(node);
        }
        if map.is_empty() {
            self.possibly_existing
                .set(self.possibly_existing.get() & !MarkerType::Suggestion.bit());
        }
        true
    }
    pub fn RemoveSuggestionMarkerByTypeInRange<C: PositionDOM<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        kind: SuggestionType,
    ) -> Vec<MarkerEffect<N>> {
        let pairs = self.MarkersIntersectingRange(
            ctx,
            start,
            end,
            MarkerTypes::Single(MarkerType::Suggestion),
        );
        let mut effects = Vec::new();
        for (node, _) in pairs {
            // The source calls RemoveMarkerByType once per intersecting pair,
            // deliberately removing the first such marker in that node list.
            self.RemoveFirstSuggestionTypeInNode(&node, kind);
            effects.push(MarkerEffect::InvalidatePaint(node));
        }
        effects
    }
    pub fn RemoveSuggestionMarkerByType(&self, kind: SuggestionType) -> Vec<MarkerEffect<N>> {
        if !self.PossiblyHasMarkers(MarkerTypes::Single(MarkerType::Suggestion)) {
            return Vec::new();
        }
        let nodes: Vec<N> = self.maps.borrow()[MarkerType::Suggestion.index()]
            .keys()
            .cloned()
            .collect();
        for node in nodes {
            if self.RemoveFirstSuggestionTypeInNode(&node, kind) {
                return vec![MarkerEffect::InvalidatePaint(node)];
            }
        }
        Vec::new()
    }
    pub fn RemoveMarkersOfTypes(&self, types: MarkerTypes) -> Vec<MarkerEffect<N>> {
        if !self.PossiblyHasMarkers(types) {
            return Vec::new();
        }
        let mut effects = Vec::new();
        for kind in types.types() {
            let map = std::mem::take(&mut self.maps.borrow_mut()[kind.index()]);
            for node in map.into_keys() {
                effects.push(MarkerEffect::InvalidateVisualOverflow {
                    node: node.clone(),
                    kind,
                });
                effects.push(MarkerEffect::InvalidatePaint(node));
            }
            self.possibly_existing
                .set(self.possibly_existing.get() & !kind.bit());
        }
        effects
    }
}

impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    /// TextSuggestionController.cc564-595; source returns immediately at the
    /// first matching Text/type, without materializing the rest of the range.
    pub fn FirstMarkerIntersectingFlatRange<C: PositionDOM<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
        types: MarkerTypes,
    ) -> Option<(N, Rc<DocumentMarker>)> {
        let (start_node, end_node) = (
            start.ComputeContainerNode(ctx),
            end.ComputeContainerNode(ctx),
        );
        let (a, b) = (
            start.ComputeOffsetInContainerNode(ctx),
            end.ComputeOffsetInContainerNode(ctx),
        );
        let past = end.NodeAsRangePastLastNode(ctx);
        let mut node = text_offset_mapping::NodeAsRangeFirstNode(ctx, start);
        while node != past {
            let Some(current) = node else {
                break;
            };
            node = text_offset_mapping::NextNode(ctx, &current, None);
            if !ctx.IsText(&current) {
                continue;
            }
            let start = if Some(&current) == start_node.as_ref() {
                a
            } else {
                0
            };
            let end = if Some(&current) == end_node.as_ref() {
                b
            } else {
                ctx.CharacterDataLength(&current).expect("source Text")
            };
            if let Some(marker) =
                self.FirstMarkerIntersectingOffsetRange(ctx, &current, start, end, types)
            {
                return Some((current, marker));
            }
        }
        None
    }
}

impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    // document_marker_controller.cc:1143-1161,1304-1320.
    pub fn RemoveMarkersForNode(&self, node: &N, types: MarkerTypes) -> Vec<MarkerEffect<N>> {
        if !self.PossiblyHasMarkers(types) {
            return Vec::new();
        }
        let mut effects = Vec::new();
        for kind in types.types() {
            let removed = self.maps.borrow_mut()[kind.index()].remove(node);
            if removed.is_some() {
                effects.push(MarkerEffect::InvalidateVisualOverflow {
                    node: node.clone(),
                    kind,
                });
                effects.push(MarkerEffect::InvalidatePaint(node.clone()));
            }
            if self.maps.borrow()[kind.index()].is_empty() {
                self.possibly_existing
                    .set(self.possibly_existing.get() & !kind.bit());
            }
        }
        effects
    }
}

impl<N: Clone + Eq + Hash + 'static> DocumentMarkerController<N> {
    /// text_fragment_marker_list_impl.cc:13-37; controller.cc:957-967.
    /// Preserve the first original marker owner while expanding its endpoint.
    pub fn MergeOverlappingTextFragmentMarkers(&self) {
        for list in self.maps.borrow_mut()[MarkerType::TextFragment.index()].values_mut() {
            let mut merged: Vec<Rc<DocumentMarker>> = Vec::new();
            for marker in &list.markers {
                if let Some(active) = merged
                    .last()
                    .filter(|active| marker.StartOffset() < active.EndOffset())
                {
                    active.end.set(active.EndOffset().max(marker.EndOffset()));
                } else {
                    merged.push(marker.clone());
                }
            }
            list.markers = merged;
        }
    }
    /// DocumentMarkerController.cc:336-343. TextFragmentMarker adds no extra
    /// payload to DocumentMarker's source range/type fields.
    pub fn AddTextFragmentMarker<C: TextIteratorContext<Node = N>>(
        &self,
        ctx: &C,
        start: &Position<N>,
        end: &Position<N>,
    ) -> Vec<MarkerEffect<N>> {
        self.AddMarkerInternal(
            ctx,
            start,
            end,
            MarkerType::TextFragment,
            Rc::from([]),
            false,
        )
    }
}
