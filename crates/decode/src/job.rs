use executor::Executor;
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
};
pub type Wake = Arc<dyn Fn() + Send + Sync>;
pub(crate) type WakeSlot = Arc<Mutex<Option<Wake>>>;
pub struct DecodeJob<T> {
    receiver: mpsc::Receiver<std::thread::Result<io::Result<T>>>,
    cancelled: Arc<AtomicBool>,
}
impl<T> Drop for DecodeJob<T> {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
impl<T: Send + 'static> DecodeJob<T> {
    pub(crate) fn start(
        executor: &dyn Executor,
        wake: WakeSlot,
        name: &'static str,
        work: impl FnOnce() -> io::Result<T> + Send + 'static,
    ) -> Self {
        let (sender, receiver) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancel = cancelled.clone();
        let failed = sender.clone();
        let context = browser_tracing::context();
        let submit = executor.execute(Box::new(move || {
            if cancel.load(Ordering::Acquire) {
                return;
            }
            let _context = browser_tracing::scope(context);
            let _trace = browser_tracing::span("decode", name);
            // Preserve invariant failures across the execution boundary. The
            // Page owner reports them through its ordinary terminal error path;
            // they must not be relabeled as a broken network image/font.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
            if !cancel.load(Ordering::Acquire) && sender.send(result).is_ok() {
                let callback = wake.lock().unwrap_or_else(|e| e.into_inner()).clone();
                if let Some(callback) = callback {
                    callback();
                }
            }
        }));
        if let Err(error) = submit {
            let _ = failed.send(Ok(Err(error)));
        }
        Self {
            receiver,
            cancelled,
        }
    }
    /// A consumed job must be removed by its owner. Empty never blocks.
    pub fn poll(&mut self) -> Option<io::Result<T>> {
        match self.receiver.try_recv() {
            Ok(Ok(result)) => Some(result),
            Ok(Err(panic)) => std::panic::resume_unwind(panic),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Err(io::Error::other(
                "decode executor stopped before completion",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Queued(Mutex<Vec<executor::Task>>);
    impl Executor for Queued {
        fn execute(&self, task: executor::Task) -> io::Result<()> {
            self.0.lock().unwrap().push(task);
            Ok(())
        }
    }
    #[test]
    fn injected_execution_handles_inline_completion_pending_cancellation_and_failure() {
        let inline = executor::InlineExecutor;
        let wake: WakeSlot = Default::default();
        let mut immediate = DecodeJob::start(&inline, wake.clone(), "inline", || Ok(7));
        assert_eq!(immediate.poll().unwrap().unwrap(), 7);
        let queued = Queued::default();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let signal = calls.clone();
        *wake.lock().unwrap() = Some(Arc::new(move || {
            signal.fetch_add(1, Ordering::SeqCst);
        }));
        let mut pending = DecodeJob::start(&queued, wake.clone(), "queued", || Ok(9));
        assert!(pending.poll().is_none());
        queued.0.lock().unwrap().pop().unwrap()();
        assert_eq!(pending.poll().unwrap().unwrap(), 9);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let cancelled =
            DecodeJob::start(&queued, wake.clone(), "cancelled", || -> io::Result<()> {
                panic!("cancelled work executed")
            });
        drop(cancelled);
        queued.0.lock().unwrap().pop().unwrap()();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let mut panicked =
            DecodeJob::start(&inline, wake.clone(), "panic", || -> io::Result<()> {
                panic!("decoder invariant")
            });
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| panicked.poll())).is_err()
        );
        struct Reject;
        impl Executor for Reject {
            fn execute(&self, _: executor::Task) -> io::Result<()> {
                Err(io::Error::other("rejected"))
            }
        }
        let mut failed = DecodeJob::start(&Reject, wake, "failed", || Ok(1));
        assert_eq!(failed.poll().unwrap().unwrap_err().to_string(), "rejected");
    }
}
