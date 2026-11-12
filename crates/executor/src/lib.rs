//! Injectable execution of finite, owned work. No engine or Page dependencies.
use std::{io, sync::Arc, time::Duration};
pub type Task = Box<dyn FnOnce() + Send + 'static>;

#[cfg(target_os = "macos")]
const DELAYED_WAKE_IDENT: u64 = 1;
#[cfg(target_os = "macos")]
const DELAYED_TIMER_IDENT: u64 = 2;

#[cfg(target_os = "macos")]
fn timer_event(ident: u64, filter: i16, flags: u16, fflags: u32, data: i64) -> libc::kevent64_s {
    libc::kevent64_s {
        ident,
        filter,
        flags,
        fflags,
        data,
        udata: 0,
        ext: [0; 2],
    }
}

#[cfg(target_os = "macos")]
fn change_timer_event(queue: std::os::fd::RawFd, event: &libc::kevent64_s) -> io::Result<()> {
    let result = unsafe {
        libc::kevent64(
            queue,
            event,
            1,
            std::ptr::null_mut(),
            0,
            libc::KEVENT_FLAG_NONE,
            std::ptr::null(),
        )
    };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
fn create_delayed_timer_queue() -> io::Result<std::os::fd::RawFd> {
    let queue = unsafe { libc::kqueue() };
    if queue < 0 {
        return Err(io::Error::last_os_error());
    }
    let wake = timer_event(
        DELAYED_WAKE_IDENT,
        libc::EVFILT_USER,
        libc::EV_ADD | libc::EV_CLEAR,
        0,
        0,
    );
    if let Err(error) = change_timer_event(queue, &wake) {
        unsafe { libc::close(queue) };
        return Err(error);
    }
    Ok(queue)
}

#[cfg(target_os = "macos")]
fn wake_delayed_timer(queue: std::os::fd::RawFd) {
    let wake = timer_event(
        DELAYED_WAKE_IDENT,
        libc::EVFILT_USER,
        0,
        libc::NOTE_TRIGGER,
        0,
    );
    let _ = change_timer_event(queue, &wake);
}

#[cfg(target_os = "macos")]
fn arm_delayed_timer(queue: std::os::fd::RawFd, deadline: std::time::Instant) -> io::Result<()> {
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    let microseconds = remaining
        .as_nanos()
        .div_ceil(1_000)
        .max(1)
        .min(i64::MAX as u128) as i64;
    change_timer_event(
        queue,
        &timer_event(
            DELAYED_TIMER_IDENT,
            libc::EVFILT_TIMER,
            libc::EV_ADD | libc::EV_ONESHOT,
            libc::NOTE_USECONDS | libc::NOTE_CRITICAL,
            microseconds,
        ),
    )
}

#[cfg(target_os = "macos")]
fn wait_for_delayed_timer(queue: std::os::fd::RawFd) -> io::Result<libc::kevent64_s> {
    loop {
        let mut event = timer_event(0, 0, 0, 0, 0);
        let result = unsafe {
            libc::kevent64(
                queue,
                std::ptr::null(),
                0,
                &mut event,
                1,
                libc::KEVENT_FLAG_NONE,
                std::ptr::null(),
            )
        };
        if result > 0 {
            if event.flags & libc::EV_ERROR != 0 && event.data != 0 {
                return Err(io::Error::from_raw_os_error(event.data as i32));
            }
            return Ok(event);
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

#[cfg(target_os = "macos")]
fn run_delayed_task_worker(
    receiver: std::sync::mpsc::Receiver<(std::time::Instant, Task)>,
    ready: std::sync::mpsc::Sender<Task>,
    stopped: Arc<std::sync::atomic::AtomicBool>,
    queue: std::os::fd::RawFd,
) {
    let result = (|| -> io::Result<()> {
        let mut pending: Vec<(std::time::Instant, Task)> = Vec::new();
        loop {
            while let Ok(task) = receiver.try_recv() {
                pending.push(task);
            }
            if stopped.load(std::sync::atomic::Ordering::Acquire) {
                return Ok(());
            }
            let Some((index, deadline)) = pending
                .iter()
                .enumerate()
                .min_by_key(|(_, (deadline, _))| *deadline)
                .map(|(index, (deadline, _))| (index, *deadline))
            else {
                let _ = wait_for_delayed_timer(queue)?;
                continue;
            };
            arm_delayed_timer(queue, deadline)?;
            loop {
                let event = wait_for_delayed_timer(queue)?;
                if stopped.load(std::sync::atomic::Ordering::Acquire) {
                    return Ok(());
                }
                if event.ident == DELAYED_WAKE_IDENT {
                    break;
                }
                if event.ident == DELAYED_TIMER_IDENT {
                    let (_, task) = pending.swap_remove(index);
                    if ready.send(task).is_err() {
                        return Ok(());
                    }
                    break;
                }
            }
        }
    })();
    unsafe { libc::close(queue) };
    if let Err(error) = result {
        eprintln!("executor-delayed-timer: {error}");
    }
}

#[cfg(not(target_os = "macos"))]
fn run_delayed_task_worker(
    receiver: std::sync::mpsc::Receiver<(std::time::Instant, Task)>,
    ready: std::sync::mpsc::Sender<Task>,
) {
    let mut pending: Vec<(std::time::Instant, Task)> = Vec::new();
    loop {
        if pending.is_empty() {
            match receiver.recv() {
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
        match receiver.recv_timeout(wait) {
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
}
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
    #[cfg(target_os = "macos")]
    delayed_timer_queue: std::os::fd::RawFd,
    #[cfg(target_os = "macos")]
    delayed_stopped: Arc<std::sync::atomic::AtomicBool>,
}
impl PoolExecutor {
    pub fn new(name: &str, stack_size: usize, lanes: usize) -> io::Result<Self> {
        Self::new_with_thread_initializer(name, stack_size, lanes, Arc::new(|| {}))
    }

    /// Constructs the same reusable pool while letting the embedding apply
    /// platform thread policy before either worker or timer work begins.
    /// Engines still see only `Executor`; thread placement and QoS remain an
    /// application responsibility.
    pub fn new_with_thread_initializer(
        name: &str,
        stack_size: usize,
        lanes: usize,
        initialize_thread: Arc<dyn Fn() + Send + Sync>,
    ) -> io::Result<Self> {
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
            let initialize_thread = initialize_thread.clone();
            std::thread::Builder::new()
                .name(format!("{name}-{lane}"))
                .stack_size(stack_size)
                .spawn(move || {
                    initialize_thread();
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
        let initialize_timer_thread = initialize_thread;
        #[cfg(target_os = "macos")]
        let delayed_timer_queue = create_delayed_timer_queue()?;
        #[cfg(target_os = "macos")]
        let worker_timer_queue = delayed_timer_queue;
        #[cfg(target_os = "macos")]
        let delayed_stopped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        #[cfg(target_os = "macos")]
        let worker_stopped = delayed_stopped.clone();
        let timer_worker = std::thread::Builder::new()
            .name(format!("{name}-timer"))
            .stack_size(stack_size)
            .spawn(move || {
                initialize_timer_thread();
                #[cfg(target_os = "macos")]
                run_delayed_task_worker(
                    delayed_receiver,
                    ready,
                    worker_stopped,
                    worker_timer_queue,
                );
                #[cfg(not(target_os = "macos"))]
                run_delayed_task_worker(delayed_receiver, ready);
            });
        if let Err(error) = timer_worker {
            #[cfg(target_os = "macos")]
            unsafe {
                libc::close(delayed_timer_queue);
            }
            return Err(error);
        }
        Ok(Self {
            id,
            lanes,
            busy,
            sender,
            delayed_sender,
            #[cfg(target_os = "macos")]
            delayed_timer_queue,
            #[cfg(target_os = "macos")]
            delayed_stopped,
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
            .map_err(|_| io::Error::other("executor timer stopped"))?;
        #[cfg(target_os = "macos")]
        wake_delayed_timer(self.delayed_timer_queue);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Drop for PoolExecutor {
    fn drop(&mut self) {
        self.delayed_stopped
            .store(true, std::sync::atomic::Ordering::Release);
        wake_delayed_timer(self.delayed_timer_queue);
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
