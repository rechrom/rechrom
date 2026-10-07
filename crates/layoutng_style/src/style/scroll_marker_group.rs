// cpp: layoutng_style/style/scroll_marker_group.h:15
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ScrollMarkerPosition {
    kAfter,
    kBefore,
}

// cpp: layoutng_style/style/scroll_marker_group.h:16
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ScrollMarkerMode {
    kTabs,
    kLinks,
}

// cpp: layoutng_style/style/scroll_marker_group.h:13-46
// No Clone or Copy: the C++ copy constructor and assignment are deleted.
#[derive(Debug)]
pub struct ScrollMarkerGroup {
    mode_: ScrollMarkerMode,
    position_: ScrollMarkerPosition,
}

#[allow(non_snake_case)]
impl ScrollMarkerGroup {
    // cpp: layoutng_style/style/scroll_marker_group.h:18-20
    pub const fn new(position: ScrollMarkerPosition, mode: ScrollMarkerMode) -> Self {
        Self {
            mode_: mode,
            position_: position,
        }
    }
    pub const fn new_links(position: ScrollMarkerPosition) -> Self {
        Self::new(position, ScrollMarkerMode::kLinks)
    }

    // cpp: layoutng_style/style/scroll_marker_group.h:29-30
    pub const fn Mode(&self) -> ScrollMarkerMode {
        self.mode_
    }
    pub const fn Position(&self) -> ScrollMarkerPosition {
        self.position_
    }

    // cpp: layoutng_style/style/scroll_marker_group.h:32-39
    pub fn IsInTabsMode(&self) -> bool {
        self.mode_ == ScrollMarkerMode::kTabs
    }
    pub fn IsInLinksMode(&self) -> bool {
        self.mode_ == ScrollMarkerMode::kLinks
    }
    pub fn PositionAfter(&self) -> bool {
        self.position_ == ScrollMarkerPosition::kAfter
    }
    pub fn PositionBefore(&self) -> bool {
        self.position_ == ScrollMarkerPosition::kBefore
    }

    // cpp: layoutng_style/style/scroll_marker_group.h:41
    pub fn Trace(&self, _visitor: Option<&mut foundation::Visitor>) {}
}

// cpp: layoutng_style/style/scroll_marker_group.h:25-27
impl PartialEq for ScrollMarkerGroup {
    fn eq(&self, other: &Self) -> bool {
        self.position_ == other.position_ && self.mode_ == other.mode_
    }
}
impl Eq for ScrollMarkerGroup {}
