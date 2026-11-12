//! The retained Window clock and performance time origin shared by its bindings.
use interaction::{
    default::core::timing::{
        performance_timeline_entry_id_generator::PerformanceRandom,
        window_performance::{EventTimingEnvironment, WindowPerformance},
    },
    dispatch::core::dom::events::event::EventClock,
    input::base::time::time::TimeTicks,
};
use std::{
    cell::OnceCell,
    rc::Rc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct SystemPerformanceRandom;
impl PerformanceRandom for SystemPerformanceRandom {
    fn RandIntInclusive(&self, min: u64, max: u64) -> u64 {
        assert!(min <= max);
        let range = max - min + 1;
        let limit = u64::MAX - (u64::MAX % range);
        loop {
            let mut bytes = [0; 8];
            getrandom::fill(&mut bytes).expect("WindowPerformance entropy");
            let value = u64::from_ne_bytes(bytes);
            if value < limit {
                return min + value % range;
            }
        }
    }
}

pub struct WindowPerformanceTiming {
    start: Instant,
    time_origin: f64,
    event_timing: OnceCell<Rc<WindowPerformance>>,
}
impl WindowPerformanceTiming {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            time_origin: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system performance epoch")
                .as_secs_f64()
                * 1000.0,
            event_timing: OnceCell::new(),
        }
    }
    pub fn BindEventTimingEnvironment(
        &self,
        clock: Rc<dyn EventClock>,
        environment: Rc<dyn EventTimingEnvironment>,
        time_origin: TimeTicks,
        cross_origin_isolated: bool,
    ) {
        let performance = WindowPerformance::new(
            clock,
            environment,
            Rc::new(SystemPerformanceRandom),
            time_origin,
            cross_origin_isolated,
        );
        performance.BindEffectSink(Rc::new(super::window_event_timing::ConsumeWithoutRealm));
        assert!(
            self.event_timing.set(performance).is_ok(),
            "bind one WindowPerformance to a Window"
        );
    }
    pub fn EventTimingSource(&self) -> Rc<WindowPerformance> {
        self.event_timing
            .get()
            .expect("WindowPerformance environment is bound before event dispatch")
            .clone()
    }
    pub fn ExistingEventTimingSource(&self) -> Option<Rc<WindowPerformance>> {
        self.event_timing.get().cloned()
    }
    pub fn TimeOrigin(&self) -> f64 {
        self.time_origin
    }
    pub fn MonotonicOrigin(&self) -> Instant {
        self.start
    }
    pub fn MonotonicElapsed(&self) -> Duration {
        self.start.elapsed()
    }
    pub fn MonotonicTimeToDOMHighResTimeStamp(&self, time: Instant) -> f64 {
        if let Some(performance) = self.event_timing.get() {
            let origin = performance.GetTimeOriginInternal().ToInternalValue();
            let delta = if let Some(delta) = time.checked_duration_since(self.start) {
                delta.as_micros().min(i64::MAX as u128) as i64
            } else {
                -(self
                    .start
                    .duration_since(time)
                    .as_micros()
                    .min(i64::MAX as u128) as i64)
            };
            return performance.MonotonicTimeToDOMHighResTimeStamp(TimeTicks::FromInternalValue(
                origin.saturating_add(delta),
            ));
        }
        if let Some(delta) = time.checked_duration_since(self.start) {
            delta.as_secs_f64() * 1000.0
        } else {
            -self.start.duration_since(time).as_secs_f64() * 1000.0
        }
    }
    pub fn Now(&self) -> f64 {
        self.MonotonicTimeToDOMHighResTimeStamp(Instant::now())
    }
}
