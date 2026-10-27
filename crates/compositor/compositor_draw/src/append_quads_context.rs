use crate::{draw_mode::DrawMode, ViewTransitionToken};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default)]
pub struct AppendQuadsContext {
    pub draw_mode: DrawMode,
    pub capture_view_transition_tokens: BTreeSet<ViewTransitionToken>,
    pub for_view_transition_capture: bool,
}

impl AppendQuadsContext {
    pub fn new(
        draw_mode: DrawMode,
        capture_view_transition_tokens: BTreeSet<ViewTransitionToken>,
        for_view_transition_capture: bool,
    ) -> Self {
        Self {
            draw_mode,
            capture_view_transition_tokens,
            for_view_transition_capture,
        }
    }
}
