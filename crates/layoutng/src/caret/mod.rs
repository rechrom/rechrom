//! Caret presentation state and geometry share the layout package, but are
//! stored on the resident LayoutObjectTree. Page drives each instance through
//! the shared layout/paint lifecycle; invalidation targets the owning control.
//! Blink keeps the corresponding controller in core/editing/FrameCaret.
pub mod display_item_client;
pub mod geometry;
use display_item_client::CaretDisplayItemClient;

/// Editing focus and marked ranges expressed in layout's UTF-16 offset space.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextCaret {
    pub node_id: u64,
    pub utf16_offset: u32,
    pub empty: bool,
    pub composition: Option<(u32, u32)>,
    pub selection: Option<(u32, u32)>,
}
/// Current presentation state consumed by paint without DOM or window dependencies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretPaintState {
    pub node_id: u64,
    pub offset: u32,
    pub empty: bool,
    pub composition: Option<(u32, u32)>,
    pub selection: Option<(u32, u32)>,
    pub visible: bool,
    pub display_item_client_id: u64,
    pub display_item_client_is_cacheable: bool,
    pub display_item_client_is_just_created: bool,
}
use std::time::{Duration, Instant};

pub const BLINK_INTERVAL: Duration = Duration::from_millis(500);
pub struct FrameCaret {
    position: Option<TextCaret>,
    visible: bool,
    next_blink: Option<Instant>,
    display_item_client: Box<CaretDisplayItemClient>,
}
impl Default for FrameCaret {
    fn default() -> Self {
        Self {
            position: None,
            visible: false,
            next_blink: None,
            display_item_client: Box::default(),
        }
    }
}
impl FrameCaret {
    pub fn update(&mut self, position: Option<TextCaret>, now: Instant, restart: bool) -> bool {
        let before = self.paint_state();
        if position != self.position || restart {
            self.position = position;
            self.visible = position.is_some();
            self.next_blink = position.map(|_| now + BLINK_INTERVAL);
        } else if let Some(deadline) = self.next_blink {
            if now >= deadline {
                // Preserve phase across delayed host turns without replaying missed frames.
                let phases =
                    now.duration_since(deadline).as_nanos() / BLINK_INTERVAL.as_nanos() + 1;
                if phases % 2 != 0 {
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
    pub fn paint_state(&self) -> Option<CaretPaintState> {
        self.position.map(|position| CaretPaintState {
            node_id: position.node_id,
            offset: position.utf16_offset,
            empty: position.empty,
            composition: position.composition,
            selection: position.selection,
            visible: self.visible,
            display_item_client_id: self.display_item_client.Id(),
            display_item_client_is_cacheable: self.display_item_client.IsCacheable(),
            display_item_client_is_just_created: self.display_item_client.IsJustCreated(),
        })
    }

    pub fn display_item_client(&self) -> &CaretDisplayItemClient {
        &self.display_item_client
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
