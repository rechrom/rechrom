//! Chromium performance_timeline_entry_id_generator.h:19-109, .cc:16-37.
use std::{cell::Cell, rc::Rc};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PerformanceTimelineEntryIdInfo {
    pub web_exposed_id: u64,
    pub non_web_exposed_id: u64,
}
impl PerformanceTimelineEntryIdInfo {
    pub const NONE: Self = Self {
        web_exposed_id: 0,
        non_web_exposed_id: 0,
    };
    pub const MIN_ID: u64 = 100;
    pub const MAX_ID: u64 = (1 << 53) - 1;
    pub const MAX_ID_FOR_RESET: u64 = 10000;
    pub const ID_INCREMENT: u64 = 7;
}
pub trait PerformanceRandom {
    fn RandIntInclusive(&self, min: u64, max: u64) -> u64;
}
pub struct PerformanceTimelineEntryIdGenerator {
    current: Cell<PerformanceTimelineEntryIdInfo>,
    random: Rc<dyn PerformanceRandom>,
}
impl PerformanceTimelineEntryIdGenerator {
    pub fn new(random: Rc<dyn PerformanceRandom>) -> Self {
        let id = random.RandIntInclusive(
            PerformanceTimelineEntryIdInfo::MIN_ID,
            PerformanceTimelineEntryIdInfo::MAX_ID_FOR_RESET,
        );
        Self {
            current: Cell::new(PerformanceTimelineEntryIdInfo {
                web_exposed_id: id,
                non_web_exposed_id: 1,
            }),
            random,
        }
    }
    pub fn GetValue(&self) -> PerformanceTimelineEntryIdInfo {
        self.current.get()
    }
    pub fn IncrementId(&self) -> PerformanceTimelineEntryIdInfo {
        let mut id = self.current.get();
        id.non_web_exposed_id = id.non_web_exposed_id.wrapping_add(1);
        id.web_exposed_id += PerformanceTimelineEntryIdInfo::ID_INCREMENT;
        if id.web_exposed_id > PerformanceTimelineEntryIdInfo::MAX_ID {
            id.web_exposed_id = self.random.RandIntInclusive(
                PerformanceTimelineEntryIdInfo::MIN_ID,
                PerformanceTimelineEntryIdInfo::MAX_ID_FOR_RESET,
            );
        }
        self.current.set(id);
        id
    }
}
