//! Source LayoutText password-echo timer boundary. The native LayoutText owns
//! the timer; the embedding supplies only the document UserInteraction runner.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/layout/layout_text.cc:95-147; core/frame/settings.json5:77-96.
#![allow(non_snake_case)]
/// An opaque callback target. It does not expose or retain a LayoutObject.
/// Its serial distinguishes a restarted timer and a replacement object with
/// the same DOM node id; only the owning LayoutEngine interprets it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PasswordEchoTimerToken {
    pub(crate) node_id: u64,
    pub(crate) serial: u64,
}
/// Stop must cancel the queued one-shot. Delivery is asynchronous on the same
/// Page document sequence, through LayoutEngine::PasswordEchoTimerFired.
pub trait PasswordEchoTimerHandle {
    fn Stop(&self);
}
pub trait PasswordEchoTimerScheduler {
    fn StartUserInteractionOneShot(
        &self,
        delay_seconds: f64,
        token: PasswordEchoTimerToken,
    ) -> Box<dyn PasswordEchoTimerHandle>;
}
/// The source GetSettings() null check is represented by Option at the call
/// boundary. The default duration is source settings.json5's one second.
#[derive(Clone, Copy, Debug)]
pub struct PasswordEchoLayoutSettings {
    pub duration_seconds: f64,
}
impl Default for PasswordEchoLayoutSettings {
    fn default() -> Self {
        Self {
            duration_seconds: 1.0,
        }
    }
}
// Source: layout_text.cc:105-147. Moved from the weak LayoutText-keyed map
// into its canonical native LayoutText allocation; no parallel text is held.
pub(crate) struct SecureTextTimer {
    pub(crate) last_typed_character_offset: Option<u32>,
    pub(crate) token: Option<PasswordEchoTimerToken>,
    pub(crate) handle: Option<Box<dyn PasswordEchoTimerHandle>>,
}
impl SecureTextTimer {
    pub(crate) fn new() -> Self {
        Self {
            last_typed_character_offset: None,
            token: None,
            handle: None,
        }
    }
    pub(crate) fn Stop(&mut self) {
        self.token = None;
        if let Some(handle) = self.handle.take() {
            handle.Stop();
        }
    }
    pub(crate) fn Invalidate(&mut self) {
        self.last_typed_character_offset = None;
    }
}
impl Drop for SecureTextTimer {
    fn drop(&mut self) {
        self.Stop();
    }
}
