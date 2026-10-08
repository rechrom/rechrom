#![allow(non_snake_case)]

//! Platform-independent browser event-loop scheduling.
//!
//! The embedding communication pump owns blocking and actual wake delivery.
//! `EventLoopEngine` owns task selection and rendering opportunities. One
//! `OnWake` call executes at most one complete turn and returns the next wake
//! request to the embedder. The injected executor owns each task's internal
//! lifecycle, including any required microtask checkpoint.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExecutionContextId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TaskId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TaskSource {
    UserInteraction = 0,
    Networking = 1,
    Timer = 2,
    PostedMessage = 3,
    DomManipulation = 4,
    Parser = 5,
    Internal = 6,
}

impl TaskSource {
    const COUNT: usize = 7;
    const REGULAR: [Self; 6] = [
        Self::Networking,
        Self::Timer,
        Self::PostedMessage,
        Self::DomManipulation,
        Self::Parser,
        Self::Internal,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug)]
pub struct ScheduledTask<T> {
    pub id: TaskId,
    pub context: ExecutionContextId,
    pub source: TaskSource,
    pub ready_at: Option<Instant>,
    pub payload: T,
}

#[derive(Debug)]
pub enum EventLoopMutation<T, F> {
    PostTask(ScheduledTask<T>),
    CancelTask(TaskId),
    RequestRendering(ExecutionContextId),
    BeginMainFrame {
        context: ExecutionContextId,
        frame: F,
    },
    DestroyContext(ExecutionContextId),
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventLoopEffect {
    RequestBeginMainFrame(ExecutionContextId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WakeRequest {
    Now,
    At(Instant),
    None,
    Stop,
}

#[derive(Debug)]
pub struct ApplyResult {
    pub wake: WakeRequest,
    pub effects: Vec<EventLoopEffect>,
}

impl ApplyResult {
    fn Wake(wake: WakeRequest) -> Self {
        Self {
            wake,
            effects: Vec::new(),
        }
    }
}

pub trait EventLoopExecutor<T, F> {
    type Error;

    /// Execute one complete task turn. The client owns the task's domain
    /// semantics, including the microtask checkpoint required after a web
    /// task; the scheduler never reaches into Page, DOM or a script runtime.
    fn RunTask(
        &mut self,
        context: ExecutionContextId,
        source: TaskSource,
        task: T,
    ) -> Result<(), Self::Error>;

    fn UpdateRendering(&mut self, context: ExecutionContextId, frame: F)
        -> Result<(), Self::Error>;
}

struct ReadyTask<T> {
    id: TaskId,
    context: ExecutionContextId,
    payload: T,
}

struct DelayedTask<T> {
    id: TaskId,
    context: ExecutionContextId,
    source: TaskSource,
    payload: T,
}

/// Scheduling state for one browser event loop.
///
/// It contains no message pump, channel, thread, platform clock or OS API.
/// The embedding pump applies mutations and calls `OnWake` when a previously
/// returned `WakeRequest` is delivered.
pub struct EventLoopEngine<T, F> {
    ready: [VecDeque<ReadyTask<T>>; TaskSource::COUNT],
    delayed: BTreeMap<(Instant, u64), DelayedTask<T>>,
    rendering_requested: BTreeSet<ExecutionContextId>,
    pending_rendering: BTreeMap<ExecutionContextId, F>,
    next_delayed_order: u64,
    next_regular_source: usize,
    stopped: bool,
    executing: bool,
}

impl<T, F> Default for EventLoopEngine<T, F> {
    fn default() -> Self {
        Self {
            ready: std::array::from_fn(|_| VecDeque::new()),
            delayed: BTreeMap::new(),
            rendering_requested: BTreeSet::new(),
            pending_rendering: BTreeMap::new(),
            next_delayed_order: 0,
            next_regular_source: 0,
            stopped: false,
            executing: false,
        }
    }
}

impl<T, F> EventLoopEngine<T, F> {
    pub fn New() -> Self {
        Self::default()
    }

    pub fn Apply(&mut self, mutation: EventLoopMutation<T, F>, now: Instant) -> ApplyResult {
        if self.stopped {
            return ApplyResult::Wake(WakeRequest::Stop);
        }
        let mut effects = Vec::new();
        match mutation {
            EventLoopMutation::PostTask(task) => {
                if task.ready_at.is_some_and(|ready_at| ready_at > now) {
                    let order = self.next_delayed_order;
                    self.next_delayed_order = self.next_delayed_order.wrapping_add(1);
                    self.delayed.insert(
                        (task.ready_at.unwrap(), order),
                        DelayedTask {
                            id: task.id,
                            context: task.context,
                            source: task.source,
                            payload: task.payload,
                        },
                    );
                } else {
                    self.ready[task.source.index()].push_back(ReadyTask {
                        id: task.id,
                        context: task.context,
                        payload: task.payload,
                    });
                }
            }
            EventLoopMutation::CancelTask(id) => {
                for queue in &mut self.ready {
                    queue.retain(|task| task.id != id);
                }
                self.delayed.retain(|_, task| task.id != id);
            }
            EventLoopMutation::RequestRendering(context) => {
                if self.rendering_requested.insert(context) {
                    effects.push(EventLoopEffect::RequestBeginMainFrame(context));
                }
            }
            EventLoopMutation::BeginMainFrame { context, frame } => {
                self.rendering_requested.remove(&context);
                // A not-yet-started rendering opportunity may be superseded by
                // the newest compositor request. In-flight work is never
                // cancelled or rejected here based on a display deadline.
                self.pending_rendering.insert(context, frame);
            }
            EventLoopMutation::DestroyContext(context) => {
                self.rendering_requested.remove(&context);
                self.pending_rendering.remove(&context);
                for queue in &mut self.ready {
                    queue.retain(|task| task.context != context);
                }
                self.delayed.retain(|_, task| task.context != context);
            }
            EventLoopMutation::Stop => {
                self.stopped = true;
                return ApplyResult {
                    wake: WakeRequest::Stop,
                    effects,
                };
            }
        }
        self.PromoteReady(now);
        ApplyResult {
            wake: self.NextWake(),
            effects,
        }
    }

    /// Execute at most one complete browser turn.
    pub fn OnWake<E>(&mut self, now: Instant, executor: &mut E) -> Result<WakeRequest, E::Error>
    where
        E: EventLoopExecutor<T, F>,
    {
        if self.stopped {
            return Ok(WakeRequest::Stop);
        }
        assert!(!self.executing, "EventLoopEngine::OnWake must not reenter");
        self.executing = true;
        self.PromoteReady(now);

        let result = if let Some(task) = self.TakeReady(TaskSource::UserInteraction) {
            Self::RunCompleteTask(executor, TaskSource::UserInteraction, task)
        } else if let Some((context, frame)) = self.pending_rendering.pop_first() {
            executor.UpdateRendering(context, frame)
        } else if let Some((source, task)) = self.TakeRegularReady() {
            Self::RunCompleteTask(executor, source, task)
        } else {
            Ok(())
        };

        self.executing = false;
        result?;
        // The engine never reads a clock. A long-running turn may make a
        // returned `At` deadline immediately due; the embedding pump then
        // calls `OnWake` again with its fresh monotonic time.
        self.PromoteReady(now);
        Ok(self.NextWake())
    }

    pub fn NextWakeRequest(&mut self, now: Instant) -> WakeRequest {
        if self.stopped {
            return WakeRequest::Stop;
        }
        self.PromoteReady(now);
        self.NextWake()
    }

    fn RunCompleteTask<E>(
        executor: &mut E,
        source: TaskSource,
        task: ReadyTask<T>,
    ) -> Result<(), E::Error>
    where
        E: EventLoopExecutor<T, F>,
    {
        executor.RunTask(task.context, source, task.payload)
    }

    fn PromoteReady(&mut self, now: Instant) {
        while let Some((&(ready_at, order), _)) = self.delayed.first_key_value() {
            if ready_at > now {
                break;
            }
            let task = self.delayed.remove(&(ready_at, order)).unwrap();
            self.ready[task.source.index()].push_back(ReadyTask {
                id: task.id,
                context: task.context,
                payload: task.payload,
            });
        }
    }

    fn TakeReady(&mut self, source: TaskSource) -> Option<ReadyTask<T>> {
        let queue = &mut self.ready[source.index()];
        queue.pop_front()
    }

    fn TakeRegularReady(&mut self) -> Option<(TaskSource, ReadyTask<T>)> {
        for offset in 0..TaskSource::REGULAR.len() {
            let index = (self.next_regular_source + offset) % TaskSource::REGULAR.len();
            let source = TaskSource::REGULAR[index];
            if let Some(task) = self.TakeReady(source) {
                self.next_regular_source = (index + 1) % TaskSource::REGULAR.len();
                return Some((source, task));
            }
        }
        None
    }

    fn NextWake(&self) -> WakeRequest {
        if self.stopped {
            return WakeRequest::Stop;
        }
        if !self.pending_rendering.is_empty() || self.ready.iter().any(|queue| !queue.is_empty()) {
            return WakeRequest::Now;
        }
        self.delayed
            .first_key_value()
            .map_or(WakeRequest::None, |(&(ready_at, _), _)| {
                WakeRequest::At(ready_at)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::Infallible;
    use std::time::Duration;

    #[derive(Default)]
    struct TestExecutor {
        log: Vec<String>,
    }

    impl EventLoopExecutor<&'static str, u64> for TestExecutor {
        type Error = Infallible;

        fn RunTask(
            &mut self,
            context: ExecutionContextId,
            source: TaskSource,
            task: &'static str,
        ) -> Result<(), Self::Error> {
            self.log
                .push(format!("task:{}:{source:?}:{task}", context.0));
            Ok(())
        }

        fn UpdateRendering(
            &mut self,
            context: ExecutionContextId,
            frame: u64,
        ) -> Result<(), Self::Error> {
            self.log.push(format!("render:{}:{frame}", context.0));
            Ok(())
        }
    }

    fn task(
        id: u64,
        context: u64,
        source: TaskSource,
        payload: &'static str,
    ) -> EventLoopMutation<&'static str, u64> {
        EventLoopMutation::PostTask(ScheduledTask {
            id: TaskId(id),
            context: ExecutionContextId(context),
            source,
            ready_at: None,
            payload,
        })
    }

    #[test]
    fn one_wake_runs_one_complete_task_turn() {
        let now = Instant::now();
        let mut engine = EventLoopEngine::New();
        engine.Apply(task(1, 7, TaskSource::Timer, "a"), now);
        engine.Apply(task(2, 7, TaskSource::Timer, "b"), now);
        let mut client = TestExecutor::default();

        assert_eq!(engine.OnWake(now, &mut client), Ok(WakeRequest::Now));
        assert_eq!(client.log, ["task:7:Timer:a"]);
        assert_eq!(engine.OnWake(now, &mut client), Ok(WakeRequest::None));
        assert_eq!(
            client.log,
            [
                "task:7:Timer:a",
                "task:7:Timer:b"
            ]
        );
    }

    #[test]
    fn input_precedes_rendering_and_regular_tasks() {
        let now = Instant::now();
        let mut engine = EventLoopEngine::New();
        engine.Apply(task(1, 1, TaskSource::Networking, "network"), now);
        engine.Apply(
            EventLoopMutation::BeginMainFrame {
                context: ExecutionContextId(1),
                frame: 10,
            },
            now,
        );
        engine.Apply(task(2, 1, TaskSource::UserInteraction, "input"), now);
        let mut client = TestExecutor::default();

        engine.OnWake(now, &mut client).unwrap();
        engine.OnWake(now, &mut client).unwrap();
        engine.OnWake(now, &mut client).unwrap();
        assert_eq!(
            client.log,
            [
                "task:1:UserInteraction:input",
                "render:1:10",
                "task:1:Networking:network"
            ]
        );
    }

    #[test]
    fn request_and_not_started_frames_are_coalesced() {
        let now = Instant::now();
        let mut engine: EventLoopEngine<&'static str, u64> = EventLoopEngine::New();
        let first = engine.Apply(
            EventLoopMutation::RequestRendering(ExecutionContextId(4)),
            now,
        );
        let duplicate = engine.Apply(
            EventLoopMutation::RequestRendering(ExecutionContextId(4)),
            now,
        );
        assert_eq!(
            first.effects,
            [EventLoopEffect::RequestBeginMainFrame(ExecutionContextId(
                4
            ))]
        );
        assert!(duplicate.effects.is_empty());

        engine.Apply(
            EventLoopMutation::BeginMainFrame {
                context: ExecutionContextId(4),
                frame: 20,
            },
            now,
        );
        engine.Apply(
            EventLoopMutation::BeginMainFrame {
                context: ExecutionContextId(4),
                frame: 21,
            },
            now,
        );
        let mut client = TestExecutor::default();
        assert_eq!(engine.OnWake(now, &mut client), Ok(WakeRequest::None));
        assert_eq!(client.log, ["render:4:21"]);
    }

    #[test]
    fn delayed_and_cancelled_tasks_do_not_poll() {
        let now = Instant::now();
        let due = now + Duration::from_millis(25);
        let mut engine = EventLoopEngine::New();
        let applied = engine.Apply(
            EventLoopMutation::PostTask(ScheduledTask {
                id: TaskId(9),
                context: ExecutionContextId(1),
                source: TaskSource::Timer,
                ready_at: Some(due),
                payload: "timer",
            }),
            now,
        );
        assert_eq!(applied.wake, WakeRequest::At(due));
        engine.Apply(EventLoopMutation::CancelTask(TaskId(9)), now);
        let mut client = TestExecutor::default();
        assert_eq!(engine.OnWake(due, &mut client), Ok(WakeRequest::None));
        assert!(client.log.is_empty());
    }
}
