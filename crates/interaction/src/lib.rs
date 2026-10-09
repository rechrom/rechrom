#![allow(non_snake_case, non_camel_case_types)]
pub mod cursor;
pub mod default_event_handler;
pub mod event;
pub mod event_dispatcher;
pub mod event_path;
pub mod focus_controller;
pub mod frame_aligned_input_queue;
pub mod input_event;
pub mod input_type;
pub mod interaction_engine;
pub mod ownership;
pub mod state_invalidation;
pub mod text_editor;
pub mod wheel_event_regions;
pub use interaction_engine::{
    Interaction, InteractionEffect, InteractionOutput, InteractionOutputEmitter, InteractionResult,
    InteractionState,
};
pub use state_invalidation::{InteractionStateNeedsPaint, ValidateInteractionState};
pub use wheel_event_regions::{BlockingWheelEventRegions, WheelEventRegionResolver};
