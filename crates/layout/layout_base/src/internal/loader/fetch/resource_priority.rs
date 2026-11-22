// cpp: layoutng/internal/loader/fetch/resource_priority.h:33-38
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum VisibilityStatus {
    kNotVisible,
    kVisible,
}

// cpp: layoutng/internal/loader/fetch/resource_priority.h:55-61
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ResourcePrioritySource {
    kImageLoader,
    kOther,
}

// cpp: layoutng/internal/loader/fetch/resource_priority.h:33-62
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourcePriority {
    pub visibility: VisibilityStatus,
    pub intra_priority_value: i32,
    pub is_lcp_resource: bool,
    pub source: ResourcePrioritySource,
}

impl Default for ResourcePriority {
    // cpp: layoutng/internal/loader/fetch/resource_priority.h:40-40
    fn default() -> Self {
        Self::new(VisibilityStatus::kNotVisible, 0)
    }
}

impl ResourcePriority {
    // cpp: layoutng/internal/loader/fetch/resource_priority.h:41-42
    pub fn new(status: VisibilityStatus, intra_value: i32) -> Self {
        Self {
            visibility: status,
            intra_priority_value: intra_value,
            is_lcp_resource: false,
            source: ResourcePrioritySource::kOther,
        }
    }
}
