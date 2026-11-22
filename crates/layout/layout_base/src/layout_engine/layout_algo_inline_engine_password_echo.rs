// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: insert_into_text_node_command.cc:73-80;
// set_character_data_command.cc:59-65; layout_text.cc:126-142,1352-1363.
// Boundary lookup only: the secure style gate, native timer and masking remain
// with the same resident LayoutText allocation. No LayoutObject crosses it.
use super::*;
use crate::password_echo::{
    PasswordEchoLayoutSettings, PasswordEchoTimerScheduler, PasswordEchoTimerToken,
};
impl LayoutEngine {
    pub fn MomentarilyRevealLastTypedCharacterIfSecure(
        &mut self,
        node_id: u64,
        offset: usize,
        settings: Option<PasswordEchoLayoutSettings>,
        scheduler: &dyn PasswordEchoTimerScheduler,
    ) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(object) = self
            .tree_
            .as_mut()
            .and_then(|tree| tree.OutputNodeMut(node_id))
        else {
            return false;
        };
        let text = DynamicTo::<LayoutText>(object as *mut LayoutObject);
        if text.is_null() {
            return false;
        }
        let text = unsafe { &mut *text };
        if !text.IsSecure() {
            return false;
        }
        // The sequence belongs to this LayoutEngine/Page, and survives tree
        // replacement so delayed callbacks cannot address a replacement text.
        self.next_password_echo_timer_serial_ = self
            .next_password_echo_timer_serial_
            .checked_add(1)
            .expect("password timer serial exhausted");
        let token = PasswordEchoTimerToken {
            node_id,
            serial: self.next_password_echo_timer_serial_,
        };
        text.MomentarilyRevealLastTypedCharacter(offset as u32, settings, token, scheduler);
        true
    }
    /// Called by the document UserInteraction one-shot with its original
    /// opaque target. Stopped/restarted/destroyed timers return false.
    pub fn PasswordEchoTimerFired(&mut self, token: PasswordEchoTimerToken) -> bool {
        self.ApplyMutation(LayoutMutation::PasswordEchoTimerFired(token))
    }
}
