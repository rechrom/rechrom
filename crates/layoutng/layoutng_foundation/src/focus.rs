// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: blink/public/mojom/input/focus_type.mojom:7-23;
// core/html/focus_options.idl:1-4.
// Shared values let Document retain the same FocusOptions owner used by the
// focus transaction, without a dependency from DOM/Style back into Interaction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum FocusType {
    #[default]
    kNone = 0,
    kScript,
    kForward,
    kBackward,
    kSpatialNavigation,
    kMouse,
    kAccessKey,
    kPage,
}
#[derive(Clone, Debug, Default)]
pub struct FocusOptions {
    pub prevent_scroll: bool,
    pub focus_visible: Option<bool>,
}

/// Identity-bearing focus capability of the actual Element EditContext.
/// Source EditContext::Focus/Blur (edit_context.cc:193-220). This is a narrow
/// source-owner alias; it does not create an EditContext or emulate its IME.
pub trait ElementEditContextFocus {
    fn Focus(&self);
    fn Blur(&self);
}
