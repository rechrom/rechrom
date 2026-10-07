//! Native display demand, corresponding to CVDisplayLinkMac and
//! ExternalBeginFrameSourceMac; the timer is a DelayBasedBeginFrameSource fallback.
use foundation::begin_frame::{BeginFrameArgs, BeginFrameSource};
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct NativeBeginFrame {
    pub args: BeginFrameArgs,
    /// CoreVideo's estimated target display time, not a scanout confirmation.
    /// Timer fallback and synthetic callers have no native display target.
    pub display_time: Option<Instant>,
}

impl From<BeginFrameArgs> for NativeBeginFrame {
    fn from(args: BeginFrameArgs) -> Self {
        Self {
            args,
            display_time: None,
        }
    }
}

impl std::ops::Deref for NativeBeginFrame {
    type Target = BeginFrameArgs;
    fn deref(&self) -> &Self::Target {
        &self.args
    }
}

/// Correlate a diagnostic external WindowServer timestamp with tracing's
/// Instant clock. Used only by the opt-in optical presentation probe.
#[cfg(target_os = "macos")]
pub(crate) fn sample_host_clock() -> (Instant, f64) {
    #[repr(C)]
    #[derive(Default)]
    struct Timebase {
        numer: u32,
        denom: u32,
    }
    extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut Timebase) -> i32;
    }
    static TIMEBASE: std::sync::OnceLock<Timebase> = std::sync::OnceLock::new();
    let timebase = TIMEBASE.get_or_init(|| {
        let mut value = Timebase::default();
        // Darwin's system timebase is fixed; this does not read display state.
        assert_eq!(unsafe { mach_timebase_info(&mut value) }, 0);
        assert_ne!(value.denom, 0);
        value
    });
    let before = Instant::now();
    let ticks = unsafe { mach_absolute_time() };
    let after = Instant::now();
    let nanos = u128::from(ticks) * u128::from(timebase.numer) / u128::from(timebase.denom);
    (before + after.duration_since(before) / 2, nanos as f64)
}

/// An infallible enqueue-only callback into the owner's command queue. It must
/// not execute Page, invoke client code, or block waiting for the owner thread.
/// This is CVDisplayLinkMac's PostTask-to-register-thread boundary.
type QueuedFrameNotify = Arc<dyn Fn(NativeBeginFrame) + Send + Sync>;
static NEXT_SOURCE_ID: AtomicU64 = AtomicU64::new(1);

struct BeginFrameArgsGenerator {
    next_sequence_number: u64,
    next_expected_frame_time: Option<Instant>,
}

impl Default for BeginFrameArgsGenerator {
    fn default() -> Self {
        Self {
            next_sequence_number: 1,
            next_expected_frame_time: None,
        }
    }
}

impl BeginFrameArgsGenerator {
    fn generate(
        &mut self,
        source_id: u64,
        frame_time: Instant,
        deadline: Instant,
        interval: Duration,
    ) -> BeginFrameArgs {
        // viz::BeginFrameSource::BeginFrameArgsGenerator counts elapsed ticks
        // even when demand was idle. Its 5% margin tolerates callback jitter.
        let skipped = self.next_expected_frame_time.map_or(0, |expected| {
            let adjusted = frame_time.checked_add(interval / 20).unwrap_or(frame_time);
            let elapsed = adjusted.saturating_duration_since(expected);
            u64::try_from(elapsed.as_nanos() / interval.as_nanos()).unwrap_or(u64::MAX)
        });
        let sequence_number = self.next_sequence_number.saturating_add(skipped);
        self.next_expected_frame_time = Some(deadline);
        self.next_sequence_number = sequence_number.saturating_add(1);
        BeginFrameArgs {
            source_id,
            sequence_number,
            frame_time,
            deadline,
            interval,
        }
    }
}

#[cfg(any(target_os = "macos", test))]
#[derive(Default)]
struct NativeFrameDemand {
    generator: BeginFrameArgsGenerator,
    latest: Option<NativeBeginFrame>,
    last_delivered: Option<u64>,
    requested: bool,
}

#[cfg(any(target_os = "macos", test))]
impl NativeFrameDemand {
    fn request(&mut self, now: Instant) -> Option<NativeBeginFrame> {
        self.requested = true;
        // ExternalBeginFrameSourceMac::GetMissedBeginFrameArgs offers the
        // current cycle on renewed demand instead of always waiting for vsync.
        let frame = self.latest?;
        if now >= frame.deadline || self.last_delivered == Some(frame.sequence_number) {
            return None;
        }
        self.deliver(frame)
    }

    fn on_tick(
        &mut self,
        source_id: u64,
        frame_time: Instant,
        deadline: Instant,
        interval: Duration,
        display_time: Option<Instant>,
    ) -> Option<NativeBeginFrame> {
        let args = self
            .generator
            .generate(source_id, frame_time, deadline, interval);
        let frame = NativeBeginFrame { args, display_time };
        trace_vsync(frame, false, self.requested);
        // Keep actual native phase even while idle; never synthesize a new tick.
        self.latest = Some(frame);
        if self.requested {
            self.deliver(frame)
        } else {
            None
        }
    }

    fn deliver(&mut self, frame: NativeBeginFrame) -> Option<NativeBeginFrame> {
        self.requested = false;
        self.last_delivered = Some(frame.sequence_number);
        Some(frame)
    }
}

// Record source pulses independently of Page demand and owner-thread delays.
fn trace_vsync(frame: NativeBeginFrame, timer_fallback: bool, requested: bool) {
    if !browser_tracing::enabled() {
        return;
    }
    let args = frame.args;
    let _scope = browser_tracing::scope(browser_tracing::Context {
        target_id: 1,
        source_id: args.source_id,
        frame_id: args.sequence_number,
        ..Default::default()
    });
    browser_tracing::instant_at(
        "frame",
        "VSync",
        args.frame_time,
        &[
            ("interval_ms", args.interval.as_secs_f64() * 1000.0),
            ("timer_fallback", if timer_fallback { 1.0 } else { 0.0 }),
            ("requested", if requested { 1.0 } else { 0.0 }),
            (
                "display_target_offset_ms",
                frame.display_time.map_or(f64::NAN, |t| {
                    t.saturating_duration_since(args.frame_time).as_secs_f64() * 1000.0
                }),
            ),
        ],
    );
}

pub fn create(
    notify: QueuedFrameNotify,
    fallback_interval: Duration,
    display_id: Option<u32>,
) -> io::Result<Arc<dyn BeginFrameSource>> {
    if fallback_interval.is_zero() || Instant::now().checked_add(fallback_interval).is_none() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid fallback frame interval",
        ));
    }
    #[cfg(target_os = "macos")]
    match mac::create(notify.clone(), fallback_interval, display_id) {
        Ok(source) => return Ok(source),
        Err(error) => {
            eprintln!("begin_frame_source: CVDisplayLink unavailable; timer fallback: {error}")
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = display_id;
    TimerSource::create(notify, fallback_interval)
}

fn finish_worker(worker: &mut Option<JoinHandle<()>>) {
    if let Some(worker) = worker.take() {
        // A notification may release its own source. In that case shutdown is
        // already signalled and the worker exits after this notification.
        if worker.thread().id() != thread::current().id() {
            let _ = worker.join();
        }
    }
}

#[derive(Default)]
struct TimerDemand {
    requested: bool,
    stopped: bool,
}

struct TimerSource {
    shared: Arc<(Mutex<TimerDemand>, Condvar)>,
    worker: Option<JoinHandle<()>>,
}

impl TimerSource {
    fn create(
        notify: QueuedFrameNotify,
        interval: Duration,
    ) -> io::Result<Arc<dyn BeginFrameSource>> {
        let shared = Arc::new((Mutex::new(TimerDemand::default()), Condvar::new()));
        let state = shared.clone();
        let source_id = NEXT_SOURCE_ID.fetch_add(1, Ordering::Relaxed);
        let worker = thread::Builder::new()
            .name("begin-frame-timer-fallback".into())
            .spawn(move || {
                let timebase = Instant::now();
                let mut generator = BeginFrameArgsGenerator::default();
                let (lock, wake) = &*state;
                let mut demand = lock.lock().unwrap();
                loop {
                    while !demand.requested && !demand.stopped {
                        demand = wake.wait(demand).unwrap();
                    }
                    if demand.stopped {
                        break;
                    }
                    // Snap to the next tick of the supplied interval, rather than
                    // starting a new delay after each callback (DelayBasedTimeSource).
                    let now = Instant::now();
                    let remainder = now.duration_since(timebase).as_nanos() % interval.as_nanos();
                    let elapsed_in_tick = Duration::new(
                        (remainder / 1_000_000_000) as u64,
                        (remainder % 1_000_000_000) as u32,
                    );
                    let delay = interval - elapsed_in_tick;
                    let frame_time = now + delay;
                    while !demand.stopped && Instant::now() < frame_time {
                        let remaining = frame_time.saturating_duration_since(Instant::now());
                        demand = wake.wait_timeout(demand, remaining).unwrap().0;
                    }
                    if demand.stopped {
                        break;
                    }
                    demand.requested = false;
                    drop(demand);
                    let frame = generator
                        .generate(source_id, frame_time, frame_time + interval, interval)
                        .into();
                    trace_vsync(frame, true, true);
                    notify(frame);
                    demand = lock.lock().unwrap();
                }
            })?;
        Ok(Arc::new(Self {
            shared,
            worker: Some(worker),
        }))
    }
}

impl BeginFrameSource for TimerSource {
    fn request_begin_frame(&self) {
        let (lock, wake) = &*self.shared;
        let mut demand = lock.lock().unwrap();
        if !demand.stopped && !demand.requested {
            demand.requested = true;
            wake.notify_one();
        }
    }
}

impl Drop for TimerSource {
    fn drop(&mut self) {
        let (lock, wake) = &*self.shared;
        lock.lock().unwrap().stopped = true;
        wake.notify_one();
        finish_worker(&mut self.worker);
    }
}

#[cfg(target_os = "macos")]
mod mac {
    use super::*;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    // Exact CVBase.h ABI, including the embedded CVSMPTETime.
    #[repr(C)]
    struct SmpteTime {
        subframes: i16,
        subframe_divisor: i16,
        counter: u32,
        kind: u32,
        flags: u32,
        hours: i16,
        minutes: i16,
        seconds: i16,
        frames: i16,
    }
    #[repr(C)]
    struct TimeStamp {
        version: u32,
        video_time_scale: i32,
        video_time: i64,
        host_time: u64,
        rate_scalar: f64,
        video_refresh_period: i64,
        smpte_time: SmpteTime,
        flags: u64,
        reserved: u64,
    }
    #[repr(C)]
    struct CvTime {
        value: i64,
        scale: i32,
        flags: i32,
    }
    #[repr(C)]
    #[derive(Default)]
    struct MachTimebase {
        numer: u32,
        denom: u32,
    }
    type Link = *mut c_void;
    type Callback = unsafe extern "C" fn(
        Link,
        *const TimeStamp,
        *const TimeStamp,
        u64,
        *mut u64,
        *mut c_void,
    ) -> i32;
    #[link(name = "CoreVideo", kind = "framework")]
    extern "C" {
        fn CVDisplayLinkCreateWithCGDisplay(display: u32, link: *mut Link) -> i32;
        fn CVDisplayLinkGetCurrentCGDisplay(link: Link) -> u32;
        fn CVDisplayLinkSetOutputCallback(
            link: Link,
            callback: Callback,
            context: *mut c_void,
        ) -> i32;
        fn CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link: Link) -> CvTime;
        fn CVDisplayLinkStart(link: Link) -> i32;
        fn CVDisplayLinkStop(link: Link) -> i32;
        fn CVDisplayLinkIsRunning(link: Link) -> u8;
        fn CVDisplayLinkRelease(link: Link);
    }
    extern "C" {
        fn mach_absolute_time() -> u64;
        fn mach_timebase_info(info: *mut MachTimebase) -> i32;
    }

    struct CallbackState {
        switching_display: AtomicBool,
        active_link: AtomicUsize,
        frames: Mutex<NativeFrameDemand>,
        source_id: u64,
        enqueue: QueuedFrameNotify,
        timebase: MachTimebase,
        nominal_interval: Mutex<Duration>,
    }

    impl CallbackState {
        fn host_duration(&self, ticks: u64) -> Option<Duration> {
            let nanos = u128::from(ticks) * u128::from(self.timebase.numer)
                / u128::from(self.timebase.denom);
            Some(Duration::from_nanos(u64::try_from(nanos).ok()?))
        }
        fn host_clock_sample(&self) -> (u64, Instant) {
            // The host clock is Mach absolute time (CVBase.h). Map its actual
            // timestamp into Instant, with a bracketed clock read so this also
            // remains valid across system sleep.
            let before = Instant::now();
            let host_now = unsafe { mach_absolute_time() };
            let after = Instant::now();
            (host_now, before + after.duration_since(before) / 2)
        }
        fn frame_time(
            &self,
            stamp: &TimeStamp,
            host_now: u64,
            instant: Instant,
        ) -> Option<Instant> {
            if stamp.flags & (1 << 1) == 0 {
                return None;
            }
            if stamp.host_time >= host_now {
                instant.checked_add(self.host_duration(stamp.host_time - host_now)?)
            } else {
                instant.checked_sub(self.host_duration(host_now - stamp.host_time)?)
            }
        }
    }

    fn interval(stamp: &TimeStamp) -> Option<Duration> {
        if stamp.flags & (1 << 3) == 0
            || stamp.video_refresh_period <= 0
            || stamp.video_time_scale <= 0
        {
            return None;
        }
        let nanos = u128::try_from(stamp.video_refresh_period).ok()? * 1_000_000_000
            / u128::try_from(stamp.video_time_scale).ok()?;
        let duration = Duration::from_nanos(u64::try_from(nanos).ok()?);
        if duration.is_zero() {
            None
        } else {
            Some(duration)
        }
    }

    unsafe extern "C" fn callback(
        link: Link,
        now: *const TimeStamp,
        output: *const TimeStamp,
        _flags: u64,
        _flags_out: *mut u64,
        context: *mut c_void,
    ) -> i32 {
        // CVDisplayLinkStop synchronizes callbacks before the context is freed.
        // Only enqueue on the owner thread, as Chromium's PostTask does here.
        let state = unsafe { &*context.cast::<CallbackState>() };
        if state.switching_display.load(Ordering::Acquire)
            || state.active_link.load(Ordering::Acquire) != link as usize
        {
            return 0;
        }
        let now = unsafe { now.as_ref() };
        let output = unsafe { output.as_ref() };
        // Use one clock mapping for both callback and output timestamps so
        // their difference retains the native target-display phase.
        let (host_now, instant) = state.host_clock_sample();
        let frame_time = now
            .and_then(|stamp| state.frame_time(stamp, host_now, instant))
            .unwrap_or_else(Instant::now);
        let display_time = output.and_then(|stamp| state.frame_time(stamp, host_now, instant));
        let interval = now
            .and_then(interval)
            .or_else(|| output.and_then(interval))
            .unwrap_or_else(|| *state.nominal_interval.lock().unwrap());
        if let Some(deadline) = frame_time.checked_add(interval) {
            let mut frames = state.frames.lock().unwrap();
            if state.switching_display.load(Ordering::Acquire)
                || state.active_link.load(Ordering::Acquire) != link as usize
            {
                return 0;
            }
            if let Some(frame) = frames.on_tick(
                state.source_id,
                frame_time,
                deadline,
                interval,
                display_time,
            ) {
                // Both producers enqueue under this lock, preserving native
                // sequence order. enqueue only sends the owner's command.
                (state.enqueue)(frame);
            }
        }
        0
    }

    struct NativeLink {
        link: usize,
        display_id: u32,
    }

    struct DisplaySource {
        // Opaque CoreVideo handle: used only through the thread-safe display-link
        // API and released after stopping. No unsafe Send/Sync implementations.
        native: Mutex<NativeLink>,
        context: usize,
        state: Arc<CallbackState>,
    }
    impl BeginFrameSource for DisplaySource {
        fn request_begin_frame(&self) {
            let mut frames = self.state.frames.lock().unwrap();
            if self.state.switching_display.load(Ordering::Acquire) {
                frames.requested = true;
            } else if let Some(frame) = frames.request(Instant::now()) {
                (self.state.enqueue)(frame);
            }
        }
        fn set_display(&self, display_id: u32) {
            if let Err(error) = self.bind_display(display_id) {
                eprintln!("begin_frame_source: display {display_id} binding failed; retaining previous display: {error}");
            }
        }
    }

    impl DisplaySource {
        fn bind_display(&self, display_id: u32) -> io::Result<()> {
            let mut native = self.native.lock().unwrap();
            if native.display_id == display_id {
                return Ok(());
            }
            let link = open_link(display_id)?;
            let result = unsafe {
                CVDisplayLinkSetOutputCallback(link, callback, self.context as *mut c_void)
            };
            if result != 0 {
                unsafe {
                    CVDisplayLinkRelease(link);
                }
                return Err(io::Error::other(format!(
                    "CVDisplayLinkSetOutputCallback: {result}"
                )));
            }
            // Verify the new source before stopping the old one. Inactive links
            // cannot consume demand; a failed switch keeps the old valid source.
            self.state.switching_display.store(true, Ordering::Release);
            let result = (|| {
                if let Err(error) = start_link(link) {
                    discard_link(link, &self.state);
                    return Err(error);
                }
                let previous = native.link as Link;
                let stopped = unsafe { CVDisplayLinkStop(previous) };
                if unsafe { CVDisplayLinkIsRunning(previous) } != 0 {
                    discard_link(link, &self.state);
                    return Err(io::Error::other(format!(
                        "CVDisplayLinkStop previous display: {stopped}"
                    )));
                }
                let fallback = *self.state.nominal_interval.lock().unwrap();
                *self.state.nominal_interval.lock().unwrap() = nominal_interval(link, fallback);
                native.link = link as usize;
                native.display_id = display_id;
                // The previous monitor's unconsumed target is not a tick of
                // the new monitor. Pending demand waits for its first callback.
                self.state.frames.lock().unwrap().latest = None;
                self.state
                    .active_link
                    .store(link as usize, Ordering::Release);
                unsafe {
                    CVDisplayLinkRelease(previous);
                }
                if std::env::var_os("BROWSER_PROFILE_FRAMES").is_some() {
                    let interval = *self.state.nominal_interval.lock().unwrap();
                    eprintln!("begin-frame-source mode=cv-display-link event=rebind display_id={display_id} nominal_interval_ms={:.3}", interval.as_secs_f64() * 1000.0);
                }
                Ok(())
            })();
            self.state.switching_display.store(false, Ordering::Release);
            result
        }
    }

    impl Drop for DisplaySource {
        fn drop(&mut self) {
            self.state.switching_display.store(true, Ordering::Release);
            self.state.frames.lock().unwrap().requested = false;
            unsafe {
                let link = self.native.get_mut().unwrap().link as Link;
                CVDisplayLinkStop(link);
                if CVDisplayLinkIsRunning(link) == 0 {
                    CVDisplayLinkRelease(link);
                    drop(Arc::from_raw(self.context as *const CallbackState));
                } else {
                    // An unexpected OS stop failure must not free callback memory.
                    // Keep the native handle/context alive rather than permit UAF.
                    eprintln!(
                        "begin_frame_source: CVDisplayLinkStop failed; retaining callback context"
                    );
                }
            }
        }
    }

    fn open_link(display_id: u32) -> io::Result<Link> {
        if display_id == 0 {
            return Err(io::Error::other("window has no valid display ID"));
        }
        let mut link: Link = std::ptr::null_mut();
        let result = unsafe { CVDisplayLinkCreateWithCGDisplay(display_id, &mut link) };
        if result != 0 || link.is_null() {
            return Err(io::Error::other(format!(
                "CVDisplayLinkCreateWithCGDisplay: {result}"
            )));
        }
        if unsafe { CVDisplayLinkGetCurrentCGDisplay(link) } == 0 {
            unsafe {
                CVDisplayLinkRelease(link);
            }
            return Err(io::Error::other("CVDisplayLink has no current display"));
        }
        Ok(link)
    }

    fn nominal_interval(link: Link, fallback_interval: Duration) -> Duration {
        let nominal = unsafe { CVDisplayLinkGetNominalOutputVideoRefreshPeriod(link) };
        let interval = if nominal.flags & 1 == 0 && nominal.value > 0 && nominal.scale > 0 {
            Duration::from_secs_f64(nominal.value as f64 / nominal.scale as f64)
        } else {
            fallback_interval
        };
        if interval.is_zero() {
            fallback_interval
        } else {
            interval
        }
    }

    fn start_link(link: Link) -> io::Result<()> {
        let result = unsafe { CVDisplayLinkStart(link) };
        if result != 0 && unsafe { CVDisplayLinkIsRunning(link) } == 0 {
            Err(io::Error::other(format!("CVDisplayLinkStart: {result}")))
        } else {
            Ok(())
        }
    }

    fn discard_link(link: Link, state: &Arc<CallbackState>) {
        unsafe {
            if CVDisplayLinkIsRunning(link) != 0 {
                CVDisplayLinkStop(link);
            }
            if CVDisplayLinkIsRunning(link) == 0 {
                CVDisplayLinkRelease(link);
            } else {
                // An OS stop failure leaves an inactive callback alive. Its
                // active_link check suppresses demand, and this extra strong
                // reference protects the original raw callback context.
                let _ = Arc::into_raw(state.clone());
                eprintln!("begin_frame_source: inactive CVDisplayLinkStop failed; retaining callback context");
            }
        }
    }

    pub(super) fn create(
        enqueue: QueuedFrameNotify,
        fallback_interval: Duration,
        display_id: Option<u32>,
    ) -> io::Result<Arc<dyn BeginFrameSource>> {
        let mut timebase = MachTimebase::default();
        if unsafe { mach_timebase_info(&mut timebase) } != 0
            || timebase.denom == 0
            || timebase.numer == 0
        {
            return Err(io::Error::other("mach_timebase_info failed"));
        }
        let display_id =
            display_id.ok_or_else(|| io::Error::other("window has no current display"))?;
        let link = open_link(display_id)?;
        let state = Arc::new(CallbackState {
            switching_display: AtomicBool::new(false),
            active_link: AtomicUsize::new(link as usize),
            frames: Mutex::new(NativeFrameDemand::default()),
            source_id: NEXT_SOURCE_ID.fetch_add(1, Ordering::Relaxed),
            enqueue,
            timebase,
            nominal_interval: Mutex::new(nominal_interval(link, fallback_interval)),
        });
        let context = Arc::into_raw(state.clone()) as usize;
        let source = DisplaySource {
            native: Mutex::new(NativeLink {
                link: link as usize,
                display_id,
            }),
            context,
            state,
        };
        let result =
            unsafe { CVDisplayLinkSetOutputCallback(link, callback, context as *mut c_void) };
        if result != 0 {
            return Err(io::Error::other(format!(
                "CVDisplayLinkSetOutputCallback: {result}"
            )));
        }
        start_link(link)?;
        if std::env::var_os("BROWSER_PROFILE_FRAMES").is_some() {
            let interval = *source.state.nominal_interval.lock().unwrap();
            eprintln!("begin-frame-source mode=cv-display-link event=create display_id={display_id} nominal_interval_ms={:.3}", interval.as_secs_f64() * 1000.0);
        }
        Ok(Arc::new(source))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_frame_generator_counts_elapsed_ticks_with_jitter() {
        let mut generator = BeginFrameArgsGenerator::default();
        let base = Instant::now();
        let interval = Duration::from_millis(10);
        let first = generator.generate(7, base, base + interval, interval);
        assert_eq!(first.sequence_number, 1);
        let after_idle = base + interval * 3;
        let fourth = generator.generate(7, after_idle, after_idle + interval, interval);
        assert_eq!(fourth.sequence_number, 4);
        // A callback slightly before its nominal tick must not lose a tick.
        let jittered = base + interval * 4 - Duration::from_micros(400);
        let fifth = generator.generate(7, jittered, jittered + interval, interval);
        assert_eq!(fifth.sequence_number, 5);
        assert_eq!(fifth.frame_time, jittered);
        assert_eq!(fifth.deadline, jittered + interval);
        let native: NativeBeginFrame = fifth.into();
        assert_eq!(native.sequence_number, 5);
        assert_eq!(native.display_time, None);
    }

    #[test]
    fn current_native_cycle_is_delivered_once_or_waits_for_next_tick() {
        let base = Instant::now();
        let interval = Duration::from_millis(10);
        let mut frames = NativeFrameDemand::default();
        // No snapshot: demand must wait for the first actual callback.
        assert!(frames.request(base).is_none());
        let first = frames
            .on_tick(7, base, base + interval, interval, Some(base + interval))
            .unwrap();
        assert_eq!(first.sequence_number, 1);
        // Already consumed: repeated requests coalesce into the next tick.
        assert!(frames.request(base + interval / 2).is_none());
        assert!(frames.request(base + interval / 2).is_none());
        let second = frames
            .on_tick(
                7,
                base + interval,
                base + interval * 2,
                interval,
                Some(base + interval * 2),
            )
            .unwrap();
        assert_eq!(second.sequence_number, 2);
        // Idle tick: renewed demand receives that unconsumed native cycle.
        assert!(frames
            .on_tick(
                7,
                base + interval * 2,
                base + interval * 3,
                interval,
                Some(base + interval * 3)
            )
            .is_none());
        let current = frames.request(base + interval * 2 + interval / 2).unwrap();
        assert_eq!(current.sequence_number, 3);
        assert_eq!(current.frame_time, base + interval * 2);
        assert_eq!(current.display_time, Some(base + interval * 3));
        assert!(frames.request(base + interval * 2 + interval / 2).is_none());
        let fourth = frames
            .on_tick(
                7,
                base + interval * 3,
                base + interval * 4,
                interval,
                Some(base + interval * 4),
            )
            .unwrap();
        assert_eq!(fourth.sequence_number, 4);
        assert!(frames
            .on_tick(
                7,
                base + interval * 4,
                base + interval * 5,
                interval,
                Some(base + interval * 5)
            )
            .is_none());
        // At or after deadline, the stale unconsumed cycle is never delivered.
        assert!(frames.request(base + interval * 5).is_none());
        let next = frames
            .on_tick(
                7,
                base + interval * 5,
                base + interval * 6,
                interval,
                Some(base + interval * 6),
            )
            .unwrap();
        assert_eq!(next.sequence_number, 6);
    }
}
