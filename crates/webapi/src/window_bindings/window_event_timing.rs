//! WindowPerformance's typed outputs enter the owning Window task source.
//! Entries retain their source timing/interaction identity; no EventRef or
//! native performance model is copied into the JavaScript timeline.
use super::*;
use interaction::default::core::timing::window_performance::WindowPerformanceEffect;

pub(super) fn ConsumeWithoutRealm(effect: WindowPerformanceEffect) {
    match effect {
        WindowPerformanceEffect::InputDelay {
            creation_time,
            processing_start,
            ..
        } => {
            browser_tracing::instant(
                "event_timing",
                "InputDelay",
                &[(
                    "delay_us",
                    processing_start
                        .ToInternalValue()
                        .saturating_sub(creation_time.ToInternalValue()) as f64,
                )],
            );
        }
        WindowPerformanceEffect::UserInteraction(metric) => {
            browser_tracing::instant(
                "event_timing",
                "UserInteraction",
                &[
                    (
                        "interaction_id",
                        metric.interaction_id.web_exposed_id as f64,
                    ),
                    (
                        "latency_us",
                        metric
                            .end_time
                            .ToInternalValue()
                            .saturating_sub(metric.creation_time.ToInternalValue())
                            as f64,
                    ),
                    ("frame_index", metric.frame_index as f64),
                ],
            );
        }
        WindowPerformanceEffect::ArtificialPointerupOrClick => {
            browser_tracing::instant("event_timing", "ArtificialPointerupOrClick", &[]);
        }
        WindowPerformanceEffect::EventTimingReady { .. } => {}
    }
}

impl WindowJavaScriptBindings {
    pub(crate) fn BindPerformanceEffects(window: &Rc<RefCell<Self>>) {
        let Some(performance) = window
            .borrow()
            .performance_timing
            .ExistingEventTimingSource()
        else {
            return;
        };
        let source = Rc::downgrade(&performance);
        let window = Rc::downgrade(window);
        performance.BindEffectSink(Rc::new(move |effect| {
            let WindowPerformanceEffect::EventTimingReady { entry, .. } = effect else {
                ConsumeWithoutRealm(effect);
                return;
            };
            let (Some(window), Some(performance)) = (window.upgrade(), source.upgrade()) else {
                return;
            };
            let info = *entry.reporting_info.borrow();
            let stamp =
                |time| HostValue::Number(performance.MonotonicTimeToDOMHighResTimeStamp(time));
            let record = HostValue::Record(vec![
                ("name".into(), HostValue::String(entry.name.clone())),
                ("entryType".into(), HostValue::String("event".into())),
                ("startTime".into(), stamp(info.creation_time)),
                (
                    "duration".into(),
                    HostValue::Number(entry.DurationMilliseconds()),
                ),
                ("processingStart".into(), stamp(info.processing_start_time)),
                ("processingEnd".into(), stamp(info.processing_end_time)),
                ("cancelable".into(), HostValue::Boolean(entry.cancelable)),
                (
                    "interactionId".into(),
                    HostValue::Number(
                        entry
                            .GetInteractionIdInfo()
                            .map_or(0, |id| id.web_exposed_id) as f64,
                    ),
                ),
                (
                    "target".into(),
                    entry
                        .Target()
                        .filter(|target| target.IsConnected())
                        .and_then(|target| target.NodeId())
                        .map_or(HostValue::Null(JavaScriptNull), |id| {
                            HostValue::Object(HostObjectRef { id })
                        }),
                ),
            ]);
            let (callback, errors) = {
                let window = window.borrow();
                (
                    window.event_timing_callback.clone(),
                    window.posted_message_errors.clone(),
                )
            };
            let Some(callback) = callback else {
                return;
            };
            window.borrow_mut().EnqueueTaskWithRuntime(
                Box::new(move |runtime, realm| {
                    let result = runtime.Call(realm, &callback, &HostValue::default(), &[record]);
                    if let Some(error) = result.exception {
                        errors.borrow_mut().push(error);
                    }
                }),
                None,
            );
        }));
    }
}
