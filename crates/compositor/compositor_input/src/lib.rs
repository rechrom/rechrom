//! Platform-neutral compositor input vocabulary and impl-input algorithms.
//!
//! Host events are adapted by the `compositor` facade. This package owns only
//! compositor input values and decisions, never a Page or native event loop.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod browser_controls;
pub mod browser_controls_offset_manager;
pub mod browser_controls_offset_manager_client;
pub mod browser_controls_params;
pub mod compositor_commit_data;
pub mod hit_test_list;
pub mod hit_test_opaqueness;
pub mod input_handler_types;
pub mod input_types;
pub mod interaction_state;
pub mod layer_selection_bound;
pub mod main_thread_scrolling_reason;
pub mod mobile_optimized_viewport;
pub mod offset_tag;
pub mod page_scale_animation;
pub mod performance;
pub mod scroll_begin;
pub mod scroll_decision;
pub mod scroll_end;
pub mod scroll_hit_test_rect;
pub mod scroll_snap_data;
pub mod scroll_source_type;
pub mod scroll_state;
pub mod scroll_state_data;
pub mod scroll_update;
pub mod scroll_utils;
pub mod scroll_velocity_tracker;
pub mod scrollbar;
pub mod scrollbar_animation;
pub mod scrollbar_animation_controller;
pub mod scrollbar_controller;
pub mod scrollbar_controller_boundary;
pub mod scrollbar_flash;
pub mod single_scrollbar_animation_controller_thinning;
pub mod snap_geometry;
pub mod snap_selection_strategy;
pub mod touch_action;
pub mod touch_action_region;
pub mod viewport;

pub mod actively_scrolling_type {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum ActivelyScrollingType {
        kNone,
        kPrecise,
        kAnimated,
    }
}

pub mod browser_controls_state {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum BrowserControlsState {
        kShown = 1,
        kHidden = 2,
        kBoth = 3,
    }

    impl BrowserControlsState {
        pub const kMaxValue: Self = Self::kBoth;
    }
}

pub mod event_listener_properties {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum EventListenerClass {
        kPointerRawUpdate = 0,
        kTouchStartOrMove,
        kMouseWheel,
        kTouchEndOrCancel,
    }

    impl EventListenerClass {
        pub const kLast: Self = Self::kTouchEndOrCancel;
    }

    pub const kEventListenerClassCount: usize = EventListenerClass::kLast as usize + 1;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum EventListenerProperties {
        kNone,
        kPassive,
        kBlocking,
        kBlockingAndPassive,
    }

    impl EventListenerProperties {
        pub const kLast: Self = Self::kBlockingAndPassive;
    }
}

pub mod overscroll_behavior {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Type {
        kNone,
        kAuto,
        kContain,
        kChain,
    }

    impl Type {
        pub const kMax: Self = Self::kChain;
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct OverscrollBehavior {
        pub x: Type,
        pub y: Type,
    }

    impl OverscrollBehavior {
        pub const fn new() -> Self {
            Self {
                x: Type::kAuto,
                y: Type::kAuto,
            }
        }

        pub const fn FromType(r#type: Type) -> Self {
            Self {
                x: r#type,
                y: r#type,
            }
        }

        pub const fn FromAxes(x_type: Type, y_type: Type) -> Self {
            Self {
                x: x_type,
                y: y_type,
            }
        }

        pub fn HasXLocalBorderEffects(&self) -> bool {
            self.x == Type::kAuto || self.x == Type::kContain
        }

        pub fn HasYLocalBorderEffects(&self) -> bool {
            self.y == Type::kAuto || self.y == Type::kContain
        }

        pub fn PropagatesXScroll(&self) -> bool {
            self.x == Type::kAuto || self.x == Type::kChain
        }

        pub fn PropagatesYScroll(&self) -> bool {
            self.y == Type::kAuto || self.y == Type::kChain
        }
    }

    impl Default for OverscrollBehavior {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ScrollGranularity {
    kScrollByPrecisePixel = 0,
    kScrollByPixel,
    kScrollByLine,
    kScrollByPage,
    kScrollByDocument,
}

impl ScrollGranularity {
    pub const kFirstScrollGranularity: Self = Self::kScrollByPrecisePixel;
    pub const kMaxValue: Self = Self::kScrollByDocument;
}
