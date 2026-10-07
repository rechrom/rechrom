use layoutng_assembly::internal::layout_input::Offset;

// cpp: interaction/input_event.h:14-110
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventModifiers {
    pub alt: bool,
    pub control: bool,
    pub meta: bool,
    pub shift: bool,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseEventType {
    #[default]
    kMove,
    kDown,
    kUp,
    kClick,
    kDoubleClick,
    kContextMenu,
    kEnter,
    kLeave,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    kNone,
    kPrimary,
    kMiddle,
    kSecondary,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerEventType {
    #[default]
    kMove,
    kDown,
    kUp,
    kCancel,
    kEnter,
    kLeave,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
    #[default]
    kMouse,
    kPen,
    kTouch,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum KeyEventType {
    #[default]
    kDown,
    kUp,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositionEventType {
    kStart,
    #[default]
    kUpdate,
    kEnd,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FocusEventType {
    #[default]
    kFocus,
    kBlur,
}
#[derive(Clone, Debug, Default)]
pub struct MouseEvent {
    pub r#type: MouseEventType,
    pub position: Offset,
    pub button: MouseButton,
    pub modifiers: EventModifiers,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct PointerEvent {
    pub r#type: PointerEventType,
    pub pointer_type: PointerType,
    pub pointer_id: u32,
    pub position: Offset,
    pub button: MouseButton,
    pub modifiers: EventModifiers,
    pub pressure: f64,
    pub primary: bool,
    pub target_node_id: Option<u64>,
}
impl Default for PointerEvent {
    fn default() -> Self {
        Self {
            r#type: PointerEventType::kMove,
            pointer_type: PointerType::kMouse,
            pointer_id: 1,
            position: Offset::default(),
            button: MouseButton::kNone,
            modifiers: EventModifiers::default(),
            pressure: 0.0,
            primary: true,
            target_node_id: None,
        }
    }
}
/// Native wheel gesture boundary retained for safe host input coalescing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelPhase {
    #[default]
    kNone,
    kBegan,
    kChanged,
    kEnded,
    kCancelled,
}
/// WebInputEvent::RailsMode. On macOS Chromium derives this once at the
/// native event boundary and keeps the original wheel deltas for DOM event
/// dispatch; default scrolling consumes only the selected axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WheelRailsMode {
    #[default]
    kFree,
    kHorizontal,
    kVertical,
}
/// Original independent NSEvent phases (public NSEventPhase bits), plus the
/// boot-relative event timestamp. The effective WheelEvent.phase still drives
/// host dispatch; momentum identity must also survive coalescing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeWheelMetadata {
    pub timestamp_seconds: f64,
    pub phase: u64,
    pub momentum_phase: u64,
}
#[derive(Clone, Debug, Default)]
pub struct WheelEvent {
    pub phase: WheelPhase,
    pub rails_mode: WheelRailsMode,
    pub native: Option<NativeWheelMetadata>,
    pub delta_units: ScrollGranularity,
    pub position: Offset,
    pub delta: Offset,
    pub modifiers: EventModifiers,
    pub target_node_id: Option<u64>,
}

// ui/events/types/scroll_types.h. Native wheel deltas are already expressed
// in pixels; precise pixels retain the device distinction for input coalescing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollGranularity {
    kScrollByPrecisePixel,
    #[default]
    kScrollByPixel,
    kScrollByLine,
    kScrollByPage,
    kScrollByDocument,
}

impl WheelEvent {
    /// WebMouseWheelEvent::CanCoalesce / HaveConsistentPhase: compare both
    /// original native phases, but not timestamps; reversal is not a boundary.
    pub fn CanCoalesce(&self, next: &Self) -> bool {
        self.modifiers == next.modifiers
            && self.delta_units == next.delta_units
            && self.phase == next.phase
            && self.rails_mode == next.rails_mode
            && match (self.native, next.native) {
                (Some(a), Some(b)) => a.phase == b.phase && a.momentum_phase == b.momentum_phase,
                (None, None) => true,
                _ => false,
            }
    }

    /// Platform-specific default scroll action after macOS wheel railing.
    /// Keep `delta` untouched so JavaScript observes the native event.
    pub fn DefaultScrollDelta(&self) -> Offset {
        match self.rails_mode {
            WheelRailsMode::kFree => self.delta,
            WheelRailsMode::kHorizontal => Offset {
                x: self.delta.x,
                y: 0.0,
            },
            WheelRailsMode::kVertical => Offset {
                x: 0.0,
                y: self.delta.y,
            },
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct KeyEvent {
    pub r#type: KeyEventType,
    pub key: String,
    pub text: String,
    pub modifiers: EventModifiers,
    pub repeat: bool,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct CompositionEvent {
    /// UTF-8 byte offsets within preedit, as supplied by the native IME.
    pub selection: Option<(usize, usize)>,
    pub r#type: CompositionEventType,
    pub data: String,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct TextInputEvent {
    pub text: String,
    pub target_node_id: Option<u64>,
}
#[derive(Clone, Debug, Default)]
pub struct FocusEvent {
    pub r#type: FocusEventType,
    pub target_node_id: u64,
    pub related_target_node_id: Option<u64>,
}
#[derive(Clone, Debug)]
pub enum InputEvent {
    Mouse(MouseEvent),
    Pointer(PointerEvent),
    Wheel(WheelEvent),
    Key(KeyEvent),
    Composition(CompositionEvent),
    TextInput(TextInputEvent),
    Focus(FocusEvent),
}
