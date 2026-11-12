// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/editing/commands/edit_command.h:43-52
#[derive(Clone, Copy, Debug)]
pub enum PasswordEchoBehavior {
    kEchoIfPasswordEchoPhysicalEnabled,
    kEchoIfPasswordEchoTouchEnabled,
    kDoNotEcho,
}
