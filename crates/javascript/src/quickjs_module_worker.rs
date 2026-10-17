// Owner-isolated compile-only actor. No JS value, realm or callback state crosses.
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
};
type Wake = Arc<dyn Fn() + Send + Sync>;
pub(crate) enum Outcome {
    Compiled {
        bytes: Vec<u8>,
        metadata: JSCompiledSourceMetadata,
    },
    MainFallback,
}
struct Request {
    source: Arc<String>,
    name: String,
    stack: usize,
    module: bool,
    trace_context: browser_tracing::Context,
    cancelled: Arc<AtomicBool>,
    reply: mpsc::Sender<Outcome>,
}
pub(crate) struct Worker {
    executor: Arc<dyn executor::Executor>,
    stopped: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<Wake>>>,
}
pub(crate) struct Job {
    pub source: Arc<String>,
    pub name: String,
    receiver: Receiver<Outcome>,
    cancelled: Arc<AtomicBool>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
    }
}
fn notify(wake: &Mutex<Option<Wake>>) {
    let callback = wake.lock().unwrap_or_else(|e| e.into_inner()).clone();
    if let Some(callback) = callback {
        callback();
    }
}
impl Worker {
    pub fn with_executor(executor: Arc<dyn executor::Executor>) -> Self {
        Self {
            executor,
            stopped: Arc::new(AtomicBool::new(false)),
            wake: Default::default(),
        }
    }
    pub fn set_wake(&mut self, wake: Option<Wake>) {
        *self.wake.lock().unwrap_or_else(|e| e.into_inner()) = wake;
    }
    pub fn begin(&self, source: &str, name: &str, stack: usize) -> Job {
        self.begin_kind(source, name, stack, true)
    }
    pub fn begin_classic(&self, source: &str, name: &str, stack: usize) -> Job {
        self.begin_kind(source, name, stack, false)
    }
    fn begin_kind(&self, source: &str, name: &str, stack: usize, module: bool) -> Job {
        let source = Arc::new(source.to_owned());
        let (reply, receiver) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            source: source.clone(),
            name: name.to_owned(),
            stack,
            module,
            trace_context: browser_tracing::context(),
            cancelled: cancelled.clone(),
            reply,
        };
        let stopped = self.stopped.clone();
        let wake = self.wake.clone();
        let failed = request.reply.clone();
        if self
            .executor
            .execute(Box::new(move || {
                if stopped.load(Ordering::Acquire) || request.cancelled.load(Ordering::Acquire) {
                    return;
                }
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
                    compile(&request)
                }))
                .unwrap_or(Outcome::MainFallback);
                if !stopped.load(Ordering::Acquire)
                    && !request.cancelled.load(Ordering::Acquire)
                    && request.reply.send(outcome).is_ok()
                {
                    notify(&wake);
                }
            }))
            .is_err()
        {
            let _ = failed.send(Outcome::MainFallback);
        }
        Job {
            source,
            name: name.to_owned(),
            receiver,
            cancelled,
        }
    }
}
impl Job {
    pub fn poll(&mut self) -> Option<Outcome> {
        match self.receiver.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Outcome::MainFallback),
        }
    }
}

struct Runtime {
    rt: *mut JSRuntime,
    ctx: *mut JSContext,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                JS_FreeContext(self.ctx);
            }
            JS_FreeRuntime(self.rt);
        }
    }
}
struct Module {
    ctx: *mut JSContext,
    value: JSValue,
}
impl Drop for Module {
    fn drop(&mut self) {
        unsafe {
            JS_FreeValue(self.ctx, self.value);
        }
    }
}
struct Buffer {
    ctx: *mut JSContext,
    ptr: *mut u8,
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            js_free(self.ctx, self.ptr.cast());
        }
    }
}
unsafe fn compile(request: &Request) -> Outcome {
    // A fresh context avoids retained module names/realm records across jobs.
    let _scope = browser_tracing::scope(request.trace_context);
    let mut trace = browser_tracing::span("javascript", "BackgroundScriptCompile");
    trace.set("source_bytes", request.source.len() as f64);
    trace.set("module", request.module as u8 as f64);
    let rt = JS_NewRuntime();
    if rt.is_null() {
        return Outcome::MainFallback;
    }
    let mut runtime = Runtime {
        rt,
        ctx: std::ptr::null_mut(),
    };
    JS_SetMaxStackSize(rt, request.stack);
    JS_EnableExceptionMetadata(rt, true);
    JS_SetExceptionMetadataStackReadPolicy(rt, true);
    runtime.ctx = JS_NewContext(rt);
    if runtime.ctx.is_null() {
        return Outcome::MainFallback;
    }
    let ctx = runtime.ctx;
    let mut source = request.source.as_bytes().to_vec();
    source.push(0);
    let Ok(name) = CString::new(request.name.as_bytes()) else {
        return Outcome::MainFallback;
    };
    let module = JS_Eval(
        ctx,
        source.as_ptr().cast(),
        request.source.len(),
        name.as_ptr(),
        JS_EVAL_FLAG_COMPILE_ONLY
            | if request.module {
                JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_HOST_NO_RESOLVE
            } else {
                JS_EVAL_TYPE_GLOBAL
            },
    );
    if JS_IsException(module) != 0 {
        return Outcome::MainFallback;
    }
    let _module_owner = Module { ctx, value: module };
    (|| {
        let Some(metadata) = JS_ExportCompiledSourceMetadata(ctx, module) else {
            return Outcome::MainFallback;
        };
        let mut len = 0;
        let buffer = JS_WriteObject(ctx, &mut len, module, JS_WRITE_OBJ_BYTECODE);
        if buffer.is_null() {
            return Outcome::MainFallback;
        }
        let _buffer_owner = Buffer { ctx, ptr: buffer };
        let bytes = std::slice::from_raw_parts(buffer, len).to_vec();
        Outcome::Compiled { bytes, metadata }
    })()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Queued(Mutex<Vec<executor::Task>>);
    impl executor::Executor for Queued {
        fn execute(&self, task: executor::Task) -> std::io::Result<()> {
            self.0.lock().unwrap().push(task);
            Ok(())
        }
    }
    #[test]
    fn background_module_worker_uses_injected_executor_and_discards_cancelled_jobs() {
        let executor = Arc::new(Queued::default());
        let mut worker = Worker::with_executor(executor.clone());
        let (send, receive) = mpsc::channel();
        worker.set_wake(Some(Arc::new(move || {
            let _ = send.send(());
        })));
        let mut jobs: Vec<_> = (0..8)
            .map(|i| worker.begin("export const x=1;", &format!("queue-{i}.js"), 1024 * 1024))
            .collect();
        assert!(jobs.iter_mut().all(|job| job.poll().is_none()));
        let cancelled = jobs[1].cancelled.clone();
        jobs.remove(1);
        assert!(cancelled.load(Ordering::Acquire));
        let tasks = std::mem::take(&mut *executor.0.lock().unwrap());
        for task in tasks {
            task();
        }
        assert_eq!(receive.try_iter().count(), 7);
        for mut job in jobs {
            match job.poll() {
                Some(Outcome::Compiled { bytes, .. }) => assert!(!bytes.is_empty()),
                _ => panic!("valid injected compile did not finish"),
            }
        }
        let cancelled_owner = worker.begin("export const y=1;", "closed.js", 1024 * 1024);
        drop(worker);
        executor.0.lock().unwrap().pop().unwrap()();
        assert_eq!(receive.try_iter().count(), 0);
        drop(cancelled_owner);
    }
}
