//! Caret presentation state and geometry share the layout package, but are
//! stored on the resident LayoutObjectTree. Page drives each instance through
//! the shared layout/paint lifecycle; invalidation targets the owning control.
//! Blink keeps the corresponding controller in core/editing/FrameCaret.
pub mod display_item_client;
pub mod geometry;
use display_item_client::CaretDisplayItemClient;

/// Editing focus and marked ranges expressed in layout's UTF-16 offset space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextCaret {
    /// The native CaretDisplayItemClient painter owner. This may be an
    /// anonymous inner-editor block rather than the author text control.
    pub node_id: u64,
    /// Author text-control identity for inner-editor selection and block-text
    /// paint. Kept distinct from the caret painter just as Blink keeps the
    /// enclosing TextControlElement separate from CaretDisplayItemClient's
    /// LayoutBlock.
    pub text_control_host: Option<u64>,
    pub utf16_offset: u32,
    pub empty: bool,
    pub composition: Option<(u32, u32)>,
    pub selection: Option<(u32, u32)>,
    /// Derived native painter geometry for a DOM selection. Text controls keep
    /// their existing value/inner-editor query until their layout update.
    pub native: Option<NativeCaretGeometry>,
    /// The control adapter admits this only from the host's canonical style,
    /// before its generated inner-editor text is traversed by Paint.
    pub control_block_text: bool,
}
/// CaretDisplayItemClient's derived block-local rectangle and painter identity.
/// LayoutObject and fragment storage remain private to the resident result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeCaretGeometry {
    pub(crate) painter: usize,
    pub(crate) fragment: Option<usize>,
    pub rect: foundation::PhysicalRect,
    pub shape: crate::internal::caret_rect::CaretShape,
    /// Resolved from the selection anchor's canonical ComputedStyle; the
    /// painter block can have a different inherited color and caret shape.
    pub color: crate::internal::layout_input_types::Color,
    /// Source CaretDisplayItemClient::block_caret_anchor_; the canonical DOM
    /// anchor and its OffsetMapping text-content offset remain distinct from
    /// the painter block and the bidi-adjusted inline caret cursor.
    pub block_text: Option<BlockCaretText>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockCaretText {
    pub anchor_id: u64,
    pub text_content_offset: u32,
}
impl NativeCaretGeometry {
    /// FrameSelection::ComputeBlockCaretCharacterOffset (cc:1662-1712)
    /// and text_fragment_painter.cc:863-869. Resolve ::first-letter through
    /// its associated text node without copying a fragment or reparsing text.
    pub fn BlockCaretTextPaint(
        self,
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
    ) -> Option<(u32, crate::internal::layout_input_types::Color)> {
        if self.shape != crate::internal::caret_rect::CaretShape::kBlock
            || !foundation::RuntimeEnabledFeatures::CSSCaretColorWithOptionalSecondValueEnabled()
        {
            return None;
        }
        let block = self.block_text?;
        let item = cursor.Cursor().InlineItem()?;
        if !item.IsText()
            || block.text_content_offset < item.StartOffset()
            || block.text_content_offset >= item.EndOffset()
        {
            return None;
        }
        let object = unsafe { cursor.Cursor().GetLayoutObject().as_ref() }?;
        let text_fragment =
            foundation::DynamicTo::<crate::layout_text_fragment::LayoutTextFragment>(object);
        let node = if let Some(fragment) = unsafe { text_fragment.as_ref() } {
            unsafe { fragment.AssociatedTextNode().as_ref() }.map(|text| &**text)
        } else {
            unsafe { object.GetNode().as_ref() }
        }?;
        if node.InputId() != block.anchor_id {
            return None;
        }
        let color = object.StyleRef().ResolvedCaretTextColor()?;
        Some((
            block.text_content_offset,
            crate::internal::boundary::paint_style_export::ColorValue(color),
        ))
    }
    #[allow(non_snake_case)]
    pub fn Matches(self, cursor: crate::layout_engine::PaintFragmentCursor<'_>) -> bool {
        self.painter == cursor.Cursor().GetLayoutObject() as usize
            && self.fragment.is_none_or(|fragment| {
                cursor
                    .Cursor()
                    .PhysicalFragment()
                    .is_some_and(|current| fragment == current as *const _ as usize)
            })
    }
    pub fn local_rect(self) -> geometry::LocalCaretRect {
        geometry::LocalCaretRect {
            offset: crate::internal::layout_input::Offset {
                x: self.rect.offset.left.ToDouble(),
                y: self.rect.offset.top.ToDouble(),
            },
            size: crate::internal::layout_input::Size {
                width: self.rect.size.width.ToDouble(),
                height: self.rect.size.height.ToDouble(),
            },
        }
    }
}
/// Current presentation state consumed by paint without DOM or window dependencies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaretPaintState {
    pub display_item_client_invalidation_reason: foundation::PaintInvalidationReason,
    pub node_id: u64,
    pub text_control_host: Option<u64>,
    pub offset: u32,
    pub empty: bool,
    pub composition: Option<(u32, u32)>,
    pub selection: Option<(u32, u32)>,
    pub native: Option<NativeCaretGeometry>,
    pub control_block_text: bool,
    pub visible: bool,
    pub display_item_client_id: u64,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: bool,
}
impl CaretPaintState {
    pub fn BlockCaretTextPaint(
        self,
        cursor: crate::layout_engine::PaintFragmentCursor<'_>,
    ) -> Option<(u32, crate::internal::layout_input_types::Color)> {
        if let Some(native) = self.native {
            return native.BlockCaretTextPaint(cursor);
        }
        if !self.control_block_text
            || self.composition.is_some()
            || self.selection.is_some()
            || cursor.paint.text_control_host != self.text_control_host
            || !foundation::RuntimeEnabledFeatures::CSSCaretColorWithOptionalSecondValueEnabled()
        {
            return None;
        }
        let item = cursor.Cursor().InlineItem()?;
        if !item.IsText() || self.offset < item.StartOffset() || self.offset >= item.EndOffset() {
            return None;
        }
        let object = unsafe { cursor.Cursor().GetLayoutObject().as_ref() }?;
        Some((
            self.offset,
            crate::internal::boundary::paint_style_export::ColorValue(
                object.StyleRef().ResolvedCaretTextColor()?,
            ),
        ))
    }
}
use std::time::{Duration, Instant};

pub const BLINK_INTERVAL: Duration = Duration::from_millis(500);
pub struct FrameCaret {
    position: Option<TextCaret>,
    visible: bool,
    enabled: bool,
    blinking_suspended: bool,
    next_blink: Option<Instant>,
    display_item_client: Box<CaretDisplayItemClient>,
    pending_block_text_invalidations: Vec<u64>,
}
impl Default for FrameCaret {
    fn default() -> Self {
        Self {
            position: None,
            visible: false,
            enabled: false,
            blinking_suspended: false,
            next_blink: None,
            display_item_client: Box::default(),
            pending_block_text_invalidations: Vec::new(),
        }
    }
}
impl FrameCaret {
    /// Source frame_caret.cc:154-160. Stop the same resident blink timer and
    /// hide the caret's existing paint client, without discarding its position.
    #[allow(non_snake_case)]
    pub fn StopCaretBlinkTimer(&mut self) -> bool {
        let needs_visual_update = self.next_blink.is_some() || self.visible;
        self.next_blink = None;
        self.visible = false;
        if needs_visual_update {
            self.display_item_client.InvalidateForCaretPaint();
        }
        needs_visual_update
    }
    pub fn update(&mut self, position: Option<TextCaret>, now: Instant, restart: bool) -> bool {
        let before = self.paint_state();
        // caret_display_item_client.cc:272-299. Invalidate the previous anchor
        // when its character cell moves, and both owners when anchors change.
        let old = self.position.and_then(|state| state.native);
        let new = position.and_then(|state| state.native);
        let old_anchor = old
            .and_then(|native| native.block_text)
            .map(|text| text.anchor_id)
            .or_else(|| {
                self.position
                    .filter(|state| state.control_block_text)
                    .and_then(|state| state.text_control_host)
            });
        let new_anchor = new
            .and_then(|native| native.block_text)
            .map(|text| text.anchor_id)
            .or_else(|| {
                position
                    .filter(|state| state.control_block_text)
                    .and_then(|state| state.text_control_host)
            });
        if old_anchor != new_anchor
            || old.map(|native| native.rect) != new.map(|native| native.rect)
            || self
                .position
                .filter(|state| state.control_block_text)
                .map(|state| state.utf16_offset)
                != position
                    .filter(|state| state.control_block_text)
                    .map(|state| state.utf16_offset)
        {
            self.pending_block_text_invalidations.extend(old_anchor);
            if new_anchor != old_anchor {
                self.pending_block_text_invalidations.extend(new_anchor);
            }
        }
        if position != self.position
            || restart
            || (self.enabled && position.is_some() && self.next_blink.is_none())
        {
            self.position = position;
            self.visible = self.enabled && position.is_some();
            self.next_blink = position
                .filter(|_| self.enabled)
                .map(|_| now + BLINK_INTERVAL);
        } else if let Some(deadline) = self.next_blink {
            if now >= deadline {
                // Preserve phase across delayed host turns without replaying missed frames.
                let phases =
                    now.duration_since(deadline).as_nanos() / BLINK_INTERVAL.as_nanos() + 1;
                // FrameCaret::CaretBlinkTimerFired keeps an already visible
                // caret visible while the mouse is down. A hidden caret may
                // become visible once, then stays visible for later phases.
                if self.blinking_suspended {
                    self.visible = true;
                } else if phases % 2 != 0 {
                    self.visible = !self.visible;
                }
                let remainder = now.duration_since(deadline).as_nanos() % BLINK_INTERVAL.as_nanos();
                self.next_blink =
                    Some(now + BLINK_INTERVAL - Duration::from_nanos(remainder as u64));
            }
        }
        let changed = before != self.paint_state();
        if changed {
            self.display_item_client.InvalidateForCaretPaint();
        }
        changed
    }
    /// Source: core/editing/frame_caret.cc:187-199. Keep focus enablement on
    /// this same resident caret owner, and invalidate its existing paint client.
    #[allow(non_snake_case)]
    pub fn SetCaretEnabled(&mut self, enabled: bool) -> bool {
        if self.enabled == enabled {
            return false;
        }
        self.enabled = enabled;
        if !enabled {
            self.next_blink = None;
            self.visible = false;
        }
        self.display_item_client.InvalidateForCaretPaint();
        true
    }
    #[allow(non_snake_case)]
    pub fn IsCaretEnabled(&self) -> bool {
        self.enabled
    }

    /// Source: core/editing/frame_caret.h:64-71. This mutates the same
    /// resident owner consumed by Layout and Paint.
    #[allow(non_snake_case)]
    pub fn SetCaretBlinkingSuspended(&mut self, suspended: bool) {
        self.blinking_suspended = suspended;
    }
    #[allow(non_snake_case)]
    pub fn IsCaretBlinkingSuspended(&self) -> bool {
        self.blinking_suspended
    }
    /// FrameCaret's Blink timer deadline. The embedding's ordinary task
    /// scheduler wakes the owning Page at this instant; `update` then advances
    /// the phase and schedules Paint through the normal lifecycle.
    #[allow(non_snake_case)]
    pub fn NextBlinkDeadline(&self) -> Option<Instant> {
        self.next_blink
    }
    pub fn paint_state(&self) -> Option<CaretPaintState> {
        self.position.map(|position| CaretPaintState {
            display_item_client_invalidation_reason: self
                .display_item_client
                .GetPaintInvalidationReason(),
            node_id: position.node_id,
            text_control_host: position.text_control_host,
            offset: position.utf16_offset,
            empty: position.empty,
            composition: position.composition,
            selection: position.selection,
            native: position.native,
            control_block_text: position.control_block_text,
            visible: self.visible,
            display_item_client_id: self.display_item_client.Id(),
            display_item_client_is_cacheable: self.display_item_client.IsCacheable(),
            display_item_client_is_just_created: self.display_item_client.IsJustCreated(),
        })
    }

    pub fn display_item_client(&self) -> &CaretDisplayItemClient {
        &self.display_item_client
    }

    pub fn TakeBlockCaretTextInvalidations(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.pending_block_text_invalidations)
    }

    /// Commit validates this resident client only when the artifact actually
    /// referenced it. DOM/control client IDs cannot validate the caret.
    #[allow(non_snake_case)]
    pub fn ValidateForCommittedPaint(&self, client_id: u64) -> bool {
        if client_id != self.display_item_client.Id() {
            return false;
        }
        self.display_item_client.ValidateForCommittedPaint();
        true
    }
}

/// DragCaret owns an independent CaretDisplayItemClient. Its position is
/// updated by DragController's retained native range and never participates in
/// FrameCaret blinking or the platform input-method focus rectangle.
#[derive(Default)]
pub struct DragCaret {
    position: Option<TextCaret>,
    display_item_client: Box<CaretDisplayItemClient>,
}
impl DragCaret {
    pub fn update(&mut self, position: Option<TextCaret>) -> bool {
        if self.position == position {
            return false;
        }
        self.position = position;
        self.display_item_client.InvalidateForCaretPaint();
        true
    }
    pub fn paint_state(&self) -> Option<CaretPaintState> {
        self.position.map(|position| CaretPaintState {
            display_item_client_invalidation_reason: self
                .display_item_client
                .GetPaintInvalidationReason(),
            node_id: position.node_id,
            text_control_host: position.text_control_host,
            offset: position.utf16_offset,
            empty: position.empty,
            composition: None,
            selection: None,
            native: position.native,
            control_block_text: false,
            visible: true,
            display_item_client_id: self.display_item_client.Id(),
            display_item_client_is_cacheable: self.display_item_client.IsCacheable(),
            display_item_client_is_just_created: self.display_item_client.IsJustCreated(),
        })
    }
    pub fn display_item_client(&self) -> &CaretDisplayItemClient {
        &self.display_item_client
    }
    pub fn ValidateForCommittedPaint(&self, client_id: u64) -> bool {
        if client_id != self.display_item_client.Id() {
            return false;
        }
        self.display_item_client.ValidateForCommittedPaint();
        true
    }
}
