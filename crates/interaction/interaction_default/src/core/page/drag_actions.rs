//! Chromium core/page/drag_actions.h:41-47.

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum DragSourceAction {
    #[default]
    None,
    DHTML,
    Image,
    Link,
    Selection,
}
