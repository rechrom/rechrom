//! Benchmark implementations receive the resident owner as a per-call borrow.
//! The controller never stores a host handle or copies an owner facade.
pub trait HostHandle {
    fn IsNull(&self) -> bool;
}

impl HostHandle for compositor_tree::LayerTreeHostId {
    fn IsNull(&self) -> bool {
        self.0 == 0
    }
}
pub trait MicroBenchmarkImpl<H: HostHandle + ?Sized> {
    fn IsDone(&self) -> bool;
    fn DidCompleteCommit(&mut self, host: &mut H);
}
