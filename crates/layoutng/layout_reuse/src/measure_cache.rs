use foundation::{Member, Visitor};
use layout_fragment::layout_result::LayoutResult;
use layout_reuse::RecentEntries;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;

use crate::internal::block_node::BlockNode;
use crate::internal::constraint_space::ConstraintSpace;
use crate::internal::layout_utils::{CalculateSizeBasedLayoutCacheStatus, LayoutCacheStatus};

// cpp: layoutng/internal/measure_cache.h:21-55
pub struct MeasureCache {
    cache_: RecentEntries<Member<LayoutResult>, 8>,
}

impl Default for MeasureCache {
    fn default() -> Self {
        Self {
            cache_: RecentEntries::default(),
        }
    }
}

#[allow(non_snake_case)]
impl MeasureCache {
    // cpp: layoutng/internal/measure_cache.h:29-33
    pub const MAX_CACHE_ENTRIES: usize = 8;

    // cpp: layoutng/internal/measure_cache.h:35-35
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(self.cache_.storage());
    }

    // cpp: layoutng/internal/measure_cache.h:37-41
    // cpp: layoutng/internal/measure_cache.cc:13-37
    #[inline(never)]
    pub fn Find(
        &mut self,
        node: &BlockNode,
        new_space: &ConstraintSpace,
        fragment_geometry: &mut Option<FragmentGeometry>,
    ) -> *const LayoutResult {
        self.cache_
            .touch_where(|entry| {
                CalculateSizeBasedLayoutCacheStatus(
                    node,
                    std::ptr::null(),
                    unsafe { &*entry.Get() },
                    new_space,
                    fragment_geometry,
                ) == LayoutCacheStatus::kHit
            })
            .map_or(std::ptr::null(), |entry| entry.Get().cast_const())
    }

    // cpp: layoutng/internal/measure_cache.h:43-43
    // cpp: layoutng/internal/measure_cache.cc:39-44
    pub fn Add(&mut self, result: *const LayoutResult) {
        self.cache_
            .push(Member::from_ptr(result as *mut LayoutResult));
    }

    // cpp: layoutng/internal/measure_cache.h:45-45
    // cpp: layoutng/internal/measure_cache.cc:46-48
    pub fn Clear(&mut self) {
        self.cache_.clear();
    }

    // cpp: layoutng/internal/measure_cache.h:47-47
    // InvalidateItems has no definition in the supplied source tree.

    // cpp: layoutng/internal/measure_cache.h:48-48
    // cpp: layoutng/internal/measure_cache.cc:50-54
    pub fn LayoutObjectWillBeDestroyed(&self) {
        for entry in self.cache_.iter() {
            unsafe { &*entry.Get() }
                .GetPhysicalFragment()
                .LayoutObjectWillBeDestroyed();
        }
    }

    // cpp: layoutng/internal/measure_cache.h:49-49
    // cpp: layoutng/internal/measure_cache.cc:56-62
    pub fn SetFragmentChildrenInvalid(&self, except: *const LayoutResult) {
        for entry in self.cache_.iter() {
            let result = entry.Get();
            if result != except as *mut LayoutResult {
                unsafe { &*result }
                    .GetMutableForLayoutBoxCachedResults()
                    .SetFragmentChildrenInvalid();
            }
        }
    }

    // cpp: layoutng/internal/measure_cache.h:51-51
    // cpp: layoutng/internal/measure_cache.cc:64-66
    pub fn GetLastForTesting(&self) -> *const LayoutResult {
        self.cache_
            .last()
            .map_or(std::ptr::null(), |entry| entry.Get())
    }
}
