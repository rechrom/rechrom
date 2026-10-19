pub mod surfaces {
    pub mod surface;
    pub mod surface_allocation_group;
    pub mod surface_dependency_deadline;
    pub mod surface_manager;
    pub mod surface_reference;
}
pub mod display {
    pub mod aggregated_frame;
    pub mod display_damage_tracker;
    pub mod display_scheduler;
    pub mod display_scheduler_base;
    pub mod frame_deadline_decider;
    pub mod resolved_frame_data;
    pub mod surface_aggregator;
}
