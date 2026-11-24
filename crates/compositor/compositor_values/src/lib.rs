//! cc shared values named by both the main side (`layerize`) and the impl
//! side (`compositor` and its packages).
//!
//! The modules keep their Chromium source paths: cc/input values carried by
//! the commit (compositor_commit_data.h, layer_selection_bound.h,
//! touch_action_region.h, ...), the LayerTreeSettings fields
//! (scheduler_settings.h, tile_manager_settings.h, managed_memory_policy.h),
//! the viz values in BeginMainFrameAndCommitState and the commit
//! (begin_frame_args.h, local_surface_id.h, shared_image_format.h), base
//! values and the gfx/Skia values cc layers carry. Impl packages re-export
//! them under their old paths. This package owns no lifecycle, thread or
//! engine state.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// cc/input values.
pub mod browser_controls_params;
pub mod compositor_commit_data;
pub mod hit_test_opaqueness;
pub mod input_types;
pub mod layer_selection_bound;
pub mod main_thread_scrolling_reason;
pub mod page_scale_animation;
pub mod scroll_decision;
pub mod scroll_hit_test_rect;
pub mod scroll_snap_data;
pub mod scroll_source_type;
pub mod scrollbar;
pub mod snap_selection_strategy;
pub mod touch_action;
pub mod touch_action_region;

// LayerTreeSettings fields, proxy and scheduler values named by both proxies.
pub mod commit_earlyout_reason;
pub mod gpu_memory_allocation;
pub mod layer_tree_settings_features;
pub mod managed_memory_policy;
pub mod scheduler_settings;
pub mod tile_manager_settings;
pub mod tile_priority;

// viz values.
pub mod begin_frame;
pub mod local_surface_id;
pub mod shared_image_format;
pub mod surface_id;

// base, gfx, Skia and cc/raster values.
pub mod color4f;
pub mod location;
pub mod prioritized_tile;
pub mod raster_source;
pub mod render_surface_flags;
pub mod sk_matrix;
pub mod traced_value;

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
