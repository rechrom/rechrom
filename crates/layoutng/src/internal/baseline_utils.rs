#![allow(non_snake_case, non_camel_case_types)]

use foundation::{IsParallelWritingMode, WritingDirectionMode, WritingMode};

// cpp: layoutng/internal/baseline_utils.h:12-12
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaselineGroup {
    kMajor,
    kMinor,
}

// cpp: layoutng/internal/baseline_utils.h:15-44
pub fn DetermineBaselineWritingMode(
    container_writing_direction: WritingDirectionMode,
    child_writing_mode: WritingMode,
    is_parallel_context: bool,
) -> WritingMode {
    let orthogonal_writing_mode = if is_parallel_context {
        container_writing_direction.GetWritingMode()
    } else if child_writing_mode == WritingMode::kHorizontalTb {
        if container_writing_direction.IsLtr() {
            WritingMode::kVerticalLr
        } else {
            WritingMode::kVerticalRl
        }
    } else {
        WritingMode::kHorizontalTb
    };
    let is_parallel = IsParallelWritingMode(
        container_writing_direction.GetWritingMode(),
        child_writing_mode,
    );

    if is_parallel_context {
        if is_parallel {
            child_writing_mode
        } else {
            orthogonal_writing_mode
        }
    } else if is_parallel {
        orthogonal_writing_mode
    } else {
        child_writing_mode
    }
}

// cpp: layoutng/internal/baseline_utils.h:51-89
pub fn DetermineBaselineGroup(
    container_writing_direction: WritingDirectionMode,
    baseline_writing_mode: WritingMode,
    is_parallel_context: bool,
    is_last_baseline: bool,
    is_flipped: bool,
) -> BaselineGroup {
    let container_writing_mode = container_writing_direction.GetWritingMode();

    let mut start_group = BaselineGroup::kMajor;
    let mut end_group = BaselineGroup::kMinor;
    if is_last_baseline {
        std::mem::swap(&mut start_group, &mut end_group);
    }
    if is_flipped {
        std::mem::swap(&mut start_group, &mut end_group);
    }

    if is_parallel_context {
        debug_assert!(IsParallelWritingMode(
            container_writing_mode,
            baseline_writing_mode
        ));
        return if baseline_writing_mode == container_writing_mode {
            start_group
        } else {
            end_group
        };
    }

    let is_ltr = container_writing_direction.IsLtr();
    match baseline_writing_mode {
        WritingMode::kHorizontalTb | WritingMode::kVerticalLr | WritingMode::kSidewaysLr => {
            if is_ltr {
                start_group
            } else {
                end_group
            }
        }
        WritingMode::kVerticalRl | WritingMode::kSidewaysRl => {
            if is_ltr {
                end_group
            } else {
                start_group
            }
        }
        _ => panic!("unreachable baseline writing mode"),
    }
}

// cpp: layoutng/internal/baseline_utils.h:56-56
pub fn DetermineBaselineGroupDefault(
    container_writing_direction: WritingDirectionMode,
    baseline_writing_mode: WritingMode,
    is_parallel_context: bool,
    is_last_baseline: bool,
) -> BaselineGroup {
    DetermineBaselineGroup(
        container_writing_direction,
        baseline_writing_mode,
        is_parallel_context,
        is_last_baseline,
        false,
    )
}
