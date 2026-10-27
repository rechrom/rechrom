//! Default-off durable mirrors of structural trace spans. A killed owner leaves
//! its actual entry record behind; a compositor request is never an entry.
//! Allocation labels describe the birth phase, not a semantic retained owner.
use std::{
    cell::Cell,
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
    time::Instant,
};

pub const PHASE_NAMES: [&str; 18] = [
    "unscoped",
    "host_task",
    "main_frame",
    "page",
    "loading",
    "style",
    "layout_tree",
    "layout_native",
    "layout_export",
    "javascript",
    "paint",
    "raster",
    "render",
    "resource",
    "font",
    "native_gc",
    "diagnostic_trace",
    "diagnostic_report",
];
thread_local! {
    static ALLOCATION_PHASE: Cell<usize> = const { Cell::new(0) };
    static PARENT: Cell<u64> = const { Cell::new(0) };
}
pub(crate) fn allocation_phase() -> usize {
    ALLOCATION_PHASE.try_with(Cell::get).unwrap_or(0)
}
pub struct AllocationScope(usize, std::marker::PhantomData<std::rc::Rc<()>>);
impl AllocationScope {
    pub fn new(phase: usize) -> Self {
        #[cfg(feature = "allocation_diagnostics")]
        {
            Self(
                ALLOCATION_PHASE
                    .try_with(|p| p.replace(if phase < PHASE_NAMES.len() { phase } else { 0 }))
                    .unwrap_or(0),
                std::marker::PhantomData,
            )
        }
        #[cfg(not(feature = "allocation_diagnostics"))]
        {
            let _ = phase;
            Self(0, std::marker::PhantomData)
        }
    }
    pub(crate) fn trace() -> Self {
        Self::new(16)
    }
}
impl Drop for AllocationScope {
    fn drop(&mut self) {
        #[cfg(feature = "allocation_diagnostics")]
        let _ = ALLOCATION_PHASE.try_with(|p| p.set(self.0));
    }
}
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("RECHROM_TASK_DIAGNOSTICS").is_some())
}
fn phase(category: &str, name: &str) -> Option<usize> {
    Some(match name {
        "Page.ResolveStyles" => 5,
        "LayoutEngine.TreeUpdate" | "Page.SynchronizeLayoutTree" => 6,
        "Host.MemoryCensus" => 17,
        "LayoutEngine.PopulatePaintData" | "LayoutEngine.RefreshTargetedPaintData" => 8,
        "NativeHeapCollection" => 15,
        "Page.PumpLoading"
        | "Page.ResolveStylesAndLoadResources"
        | "Page.DiscoverImages"
        | "Page.DiscoverFonts"
        | "Page.PollImages"
        | "Page.PollFonts"
        | "Page.FlushScriptTasks" => 4,
        "RasterWorkerWait" | "RasterTile" => return None,
        _ if name.starts_with("layout_invalidation.") => 6,
        _ if name.starts_with("layout_fragment.") || name.starts_with("layout_commit.") => 8,
        _ => match category {
            "owner_task" | "task" => 1,
            "owner_frame" | "frame" => 2,
            "lifecycle" | "navigation" => 3,
            "layout" => 7,
            "javascript" => 9,
            "paint" | "prepaint" => 10,
            "raster" => 11,
            "render" | "compose" => 12,
            "resource" | "loading" => 13,
            "font" => 14,
            _ => return None,
        },
    })
}
pub struct PhaseGuard {
    owner: AllocationScope,
    id: u64,
    parent: u64,
    name: &'static str,
    category: &'static str,
    phase: usize,
    started: Instant,
    context: crate::Context,
    durable: bool,
}
impl PhaseGuard {
    pub(crate) fn enter(category: &'static str, name: &'static str) -> Option<Self> {
        let durable = enabled();
        if !durable && !cfg!(feature = "allocation_diagnostics") {
            return None;
        }
        let phase = phase(category, name)?;
        static IDS: AtomicU64 = AtomicU64::new(1);
        let id = IDS.fetch_add(1, Ordering::Relaxed);
        let parent = PARENT.with(|p| p.replace(id));
        let guard = Self {
            owner: AllocationScope::new(phase),
            id,
            parent,
            category,
            name,
            phase,
            started: Instant::now(),
            context: crate::context(),
            durable,
        };
        if durable {
            guard.report("begin");
        }
        Some(guard)
    }
    fn report(&self, edge: &str) {
        let _allocation = AllocationScope::new(17);
        let time_ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let _ = writeln!(std::io::stderr().lock(), "owner-phase time_ns={} edge={} id={} parent={} thread={:?} category={} name={} phase={} elapsed_us={} context={:?}",
            time_ns, edge, self.id, self.parent, std::thread::current().id(), self.category, self.name,
            PHASE_NAMES[self.phase], self.started.elapsed().as_micros(), self.context);
    }
}
impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if self.durable {
            self.report("end");
        }
        let _ = PARENT.try_with(|p| p.set(self.parent));
        // owner restores the nearest enclosing structural phase after this drop.
        let _ = &self.owner;
    }
}

#[cfg(test)]
mod layout_package_phase_tests {
    use super::phase;

    #[test]
    fn layout_package_prefixes_keep_stable_diagnostic_owners() {
        assert_eq!(phase("layout", "layout_invalidation.PrepareTree"), Some(6));
        assert_eq!(phase("layout", "layout_reuse.SlowLookup"), Some(7));
        assert_eq!(phase("layout", "layout_algo_grid.Layout"), Some(7));
        assert_eq!(phase("layout", "layout_fragment.ExportTree"), Some(8));
        assert_eq!(phase("layout", "layout_commit.CommitEnvironment"), Some(8));
        assert_eq!(
            phase("prepaint", "layout_fragment.CertifyPaintInputs"),
            Some(8)
        );
    }
}

#[cfg(all(test, feature = "allocation_diagnostics"))]
mod tests {
    use super::*;
    #[test]
    fn structural_phases_restore_with_tracing_off_and_unwind() {
        let original = allocation_phase();
        let outer = PhaseGuard::enter("owner_task", "Host.Foreground").unwrap();
        assert_eq!(allocation_phase(), 1);
        {
            let _style = PhaseGuard::enter("lifecycle", "Page.ResolveStyles").unwrap();
            assert_eq!(allocation_phase(), 5);
            assert!(PhaseGuard::enter("test", "NotAStructuralPhase").is_none());
            assert_eq!(allocation_phase(), 5);
            let _ = std::panic::catch_unwind(|| {
                let _js = PhaseGuard::enter("javascript", "Callback").unwrap();
                assert_eq!(allocation_phase(), 9);
                panic!("controlled unwind");
            });
            assert_eq!(allocation_phase(), 5);
        }
        assert_eq!(allocation_phase(), 1);
        drop(outer);
        assert_eq!(allocation_phase(), original);
    }
}
