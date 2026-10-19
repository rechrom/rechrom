//! Injectable execution of finite, owned work. No engine or Page dependencies.
use std::{io, sync::Arc, time::Duration};
pub type Task = Box<dyn FnOnce() + Send + 'static>;
/// An accepted task runs exactly once. Implementations may execute inline or
/// on another thread. Err means it was not executed. Panics are not IO errors.
pub trait Executor: Send + Sync {
    fn execute(&self, task: Task) -> io::Result<()>;
    fn execute_after(&self, delay: Duration, task: Task) -> io::Result<()>;
}
#[derive(Default)]
pub struct InlineExecutor;
impl Executor for InlineExecutor {
    fn execute(&self, task: Task) -> io::Result<()> {
        task();
        Ok(())
    }

    fn execute_after(&self, delay: Duration, task: Task) -> io::Result<()> {
        std::thread::sleep(delay);
        task();
        Ok(())
    }
}
/// Platform implementation. Embedders can replace this with a pool or scheduler.
/// Engines never create OS threads or depend on a JoinHandle.
pub struct ThreadExecutor {
    name: String,
    stack_size: usize,
}
impl ThreadExecutor {
    pub fn new(name: impl Into<String>, stack_size: usize) -> Self {
        Self {
            name: name.into(),
            stack_size,
        }
    }
}
impl Executor for ThreadExecutor {
    fn execute(&self, task: Task) -> io::Result<()> {
        std::thread::Builder::new()
            .name(self.name.clone())
            .stack_size(self.stack_size)
            .spawn(task)
            .map(|_| ())
    }

    fn execute_after(&self, delay: Duration, task: Task) -> io::Result<()> {
        std::thread::Builder::new()
            .name(format!("{}-timer", self.name))
            .stack_size(self.stack_size)
            .spawn(move || {
                std::thread::sleep(delay);
                task();
            })
            .map(|_| ())
    }
}
pub fn inline() -> Arc<dyn Executor> {
    Arc::new(InlineExecutor)
}

/// A reusable platform pool with inline progress for nested work when saturated.
thread_local! { static CURRENT_POOL: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) }; }
static NEXT_POOL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
pub struct PoolExecutor {
    id: u64,
    lanes: usize,
    busy: Arc<std::sync::atomic::AtomicUsize>,
    sender: std::sync::mpsc::Sender<Task>,
    delayed_sender: std::sync::mpsc::Sender<(std::time::Instant, Task)>,
}
impl PoolExecutor {
    pub fn new(name: &str, stack_size: usize, lanes: usize) -> io::Result<Self> {
        if lanes == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "executor requires a lane",
            ));
        }
        let id = NEXT_POOL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let busy = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (sender, receiver) = std::sync::mpsc::channel::<Task>();
        let receiver = Arc::new(std::sync::Mutex::new(receiver));
        for lane in 0..lanes {
            let receiver = receiver.clone();
            let busy = busy.clone();
            std::thread::Builder::new()
                .name(format!("{name}-{lane}"))
                .stack_size(stack_size)
                .spawn(move || {
                    CURRENT_POOL.set(Some(id));
                    loop {
                        let task = receiver.lock().unwrap_or_else(|e| e.into_inner()).recv();
                        let Ok(task) = task else {
                            break;
                        };
                        // The task owns result/error delivery. Preserve the worker
                        // after a task panic; the normal panic hook remains visible.
                        busy.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
                        busy.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                    }
                })?;
        }
        // A delayed task waits on this timer thread and enters the worker queue
        // only when due. Sleeping in a worker would starve the pool when callers
        // repeatedly replace deadlines, as compositor schedulers routinely do.
        let (delayed_sender, delayed_receiver) =
            std::sync::mpsc::channel::<(std::time::Instant, Task)>();
        let ready = sender.clone();
        std::thread::Builder::new()
            .name(format!("{name}-timer"))
            .stack_size(stack_size)
            .spawn(move || {
                let mut pending: Vec<(std::time::Instant, Task)> = Vec::new();
                loop {
                    if pending.is_empty() {
                        match delayed_receiver.recv() {
                            Ok(task) => pending.push(task),
                            Err(_) => break,
                        }
                        continue;
                    }
                    let next = pending
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, (deadline, _))| *deadline)
                        .map(|(index, (deadline, _))| (index, *deadline))
                        .unwrap();
                    let wait = next.1.saturating_duration_since(std::time::Instant::now());
                    match delayed_receiver.recv_timeout(wait) {
                        Ok(task) => pending.push(task),
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                            let (_, task) = pending.swap_remove(next.0);
                            if ready.send(task).is_err() {
                                break;
                            }
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })?;
        Ok(Self {
            id,
            lanes,
            busy,
            sender,
            delayed_sender,
        })
    }
}
impl Executor for PoolExecutor {
    fn execute(&self, task: Task) -> io::Result<()> {
        // A raster coordinator may submit child tiles and wait for them.
        // Execute a child directly when this pool has no free lane, including
        // a one-lane pool. This avoids worker starvation and also permits a
        // synchronous executor without changing engine control flow.
        if CURRENT_POOL.get() == Some(self.id)
            && self.busy.load(std::sync::atomic::Ordering::SeqCst) >= self.lanes
        {
            task();
            return Ok(());
        }
        self.sender
            .send(task)
            .map_err(|_| io::Error::other("executor stopped"))
    }

    fn execute_after(&self, delay: Duration, task: Task) -> io::Result<()> {
        self.delayed_sender
            .send((std::time::Instant::now() + delay, task))
            .map_err(|_| io::Error::other("executor timer stopped"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn implementations_execute_owned_tasks_and_pool_accepts_nested_work() {
        let local = std::thread::current().id();
        let value = Arc::new(std::sync::Mutex::new(None));
        let result = value.clone();
        InlineExecutor
            .execute(Box::new(move || {
                *result.lock().unwrap() = Some(std::thread::current().id())
            }))
            .unwrap();
        assert_eq!(*value.lock().unwrap(), Some(local));
        let pool = Arc::new(PoolExecutor::new("executor-test", 1024 * 1024, 1).unwrap());
        let nested = pool.clone();
        let (done, result) = std::sync::mpsc::channel();
        pool.execute(Box::new(move || {
            let (child, child_result) = std::sync::mpsc::channel();
            nested
                .execute(Box::new(move || {
                    child.send(std::thread::current().id()).unwrap()
                }))
                .unwrap();
            done.send(
                child_result
                    .recv_timeout(std::time::Duration::from_secs(2))
                    .unwrap(),
            )
            .unwrap();
        }))
        .unwrap();
        assert_ne!(
            result
                .recv_timeout(std::time::Duration::from_secs(3))
                .unwrap(),
            local
        );
    }
}
