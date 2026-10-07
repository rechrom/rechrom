// Owner-isolated compile-only actor. No JS value, realm or callback state crosses.
use quickjs::{quickjs::*, quickjs_header::*};
use std::{
    ffi::CString,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
        Arc, Mutex,
    },
    thread,
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
    cancelled: Arc<AtomicBool>,
    reply: mpsc::Sender<Outcome>,
}
pub(crate) struct Worker {
    sender: SyncSender<Request>,
    stopped: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<Wake>>>,
}
pub(crate) struct Job {
    pub source: Arc<String>,
    pub name: String,
    receiver: Receiver<Outcome>,
    pending_request: Option<Request>,
    sender: SyncSender<Request>,
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
    pub fn new(native_stack_budget: usize) -> Option<Self> {
        // Preserve the configured QuickJS limit; reserve extra stack for Rust
        // allocator/serializer/host frames rather than lowering page semantics.
        let thread_stack = native_stack_budget
            .checked_add(4 * 1024 * 1024)?
            .max(4 * 1024 * 1024);
        let (sender, receiver) = mpsc::sync_channel::<Request>(2);
        let stopped = Arc::new(AtomicBool::new(false));
        let actor_stopped = stopped.clone();
        let wake: Arc<Mutex<Option<Wake>>> = Arc::new(Mutex::new(None));
        let actor_wake = wake.clone();
        thread::Builder::new().name("quickjs-module-compiler".into()).stack_size(thread_stack).spawn(move||{
            while let Ok(request)=receiver.recv() {
                if actor_stopped.load(Ordering::Acquire) { break; }
                // A dequeued request frees capacity for other pending jobs in
                // this same runtime/Page; wake its owner to enqueue them.
                notify(&actor_wake);
                if request.cancelled.load(Ordering::Acquire) { continue; }
                let started=std::env::var_os("BROWSER_PROFILE_INPUT").is_some()
                    .then(std::time::Instant::now);
                let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||unsafe{compile(&request)}))
                    .unwrap_or(Outcome::MainFallback);
                if let Some(started)=started {
                    eprintln!("javascript-module-worker-profile source={:?} bytes={} native_stack={} worker_thread={:?} success={} total_ms={:.3}",
                        request.name,request.source.len(),request.stack,thread::current().id(),
                        matches!(&outcome,Outcome::Compiled{..}),started.elapsed().as_secs_f64()*1000.0);
                }
                if !actor_stopped.load(Ordering::Acquire)&&!request.cancelled.load(Ordering::Acquire) {
                    if request.reply.send(outcome).is_ok() { notify(&actor_wake); }
                }
            }
        }).ok()?;
        Some(Self {
            sender,
            stopped,
            wake,
        })
    }
    pub fn set_wake(&mut self, wake: Option<Wake>) {
        *self.wake.lock().unwrap_or_else(|e| e.into_inner()) = wake;
    }
    pub fn begin(&self, source: &str, name: &str, stack: usize) -> Job {
        let source = Arc::new(source.to_owned());
        let (reply, receiver) = mpsc::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let request = Request {
            source: source.clone(),
            name: name.to_owned(),
            stack,
            cancelled: cancelled.clone(),
            reply,
        };
        let mut job = Job {
            source,
            name: name.to_owned(),
            receiver,
            pending_request: Some(request),
            sender: self.sender.clone(),
            cancelled,
        };
        job.enqueue();
        job
    }
}
impl Job {
    fn enqueue(&mut self) {
        if let Some(request) = self.pending_request.take() {
            match self.sender.try_send(request) {
                Ok(()) => {}
                Err(TrySendError::Full(request)) => self.pending_request = Some(request),
                Err(TrySendError::Disconnected(request)) => {
                    let _ = request.reply.send(Outcome::MainFallback);
                }
            }
        }
    }
    pub fn poll(&mut self) -> Option<Outcome> {
        self.enqueue();
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
        JS_EVAL_TYPE_MODULE | JS_EVAL_FLAG_COMPILE_ONLY | JS_EVAL_FLAG_HOST_NO_RESOLVE,
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
    #[test]
    fn background_module_worker_wakes_full_queue_and_discards_cancelled_jobs() {
        let mut worker = Worker::new(1024 * 1024).expect("worker thread");
        let gate = Arc::new((Mutex::new(false), std::sync::Condvar::new()));
        struct Release(Arc<(Mutex<bool>, std::sync::Condvar)>);
        impl Drop for Release {
            fn drop(&mut self) {
                *self.0 .0.lock().unwrap() = true;
                self.0 .1.notify_all();
            }
        }
        let release = Release(gate.clone());
        let (send, receive) = mpsc::channel();
        let actor_gate = gate.clone();
        worker.set_wake(Some(Arc::new(move || {
            let _ = send.send(());
            let mut open = actor_gate.0.lock().unwrap();
            while !*open {
                open = actor_gate.1.wait(open).unwrap();
            }
        })));
        let mut jobs: Vec<_> = (0..8)
            .map(|i| worker.begin("export const x=1;", &format!("queue-{i}.js"), 1024 * 1024))
            .collect();
        receive
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("dequeue must wake owner");
        assert!(
            jobs.iter().filter(|j| j.pending_request.is_some()).count() >= 4,
            "bounded channel must leave excess jobs actually pending"
        );
        let cancelled = jobs[1].cancelled.clone();
        jobs.remove(1);
        assert!(cancelled.load(Ordering::Acquire));
        drop(release);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        for mut job in jobs {
            loop {
                match job.poll() {
                    Some(Outcome::Compiled { bytes, .. }) => {
                        assert!(!bytes.is_empty());
                        break;
                    }
                    Some(Outcome::MainFallback) => {
                        panic!("valid queued compile unexpectedly fell back")
                    }
                    None => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "queued compiler work did not finish"
                        );
                        let _ = receive.recv_timeout(std::time::Duration::from_millis(10));
                    }
                }
            }
        }
    }
}
