//! MacScrollbarImplV2 and its original ui::OverlayScrollbarAnimatorMac owner.
//! Source mac_scrollbar_animator_impl.mm:35-61,71-97;
//! ui/native_theme/scrollbar_animator_mac.cc:17-286.
pub use scroll_animation::overlay_scrollbar_animator_mac::Timer;
use scroll_animation::overlay_scrollbar_animator_mac::{
    Effect as NativeEffect, OverlayScrollbarAnimatorMac, ScrollbarAnimationTimerMac,
};
pub use scroll_animation::TimeTicks;
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub enum Activity {
    Entered,
    Exited,
    Scrolled(bool),
    FadeInDeferred(bool),
    FadeOutIfNeeded,
}
pub enum Effect {
    Schedule {
        timer: Timer,
        generation: u64,
        delay_us: u64,
    },
    Cancel(Timer),
    Publish {
        thumb: f32,
        track: f32,
        width: i32,
        thumb_dirty: bool,
        track_dirty: bool,
        hidden: Option<bool>,
    },
}
pub struct MacScrollbarAnimation {
    animator: OverlayScrollbarAnimatorMac,
    timers: HashMap<Timer, (u64, ScrollbarAnimationTimerMac)>,
    generation: u64,
}
impl MacScrollbarAnimation {
    pub fn new(expanded: i32, unexpanded: i32, enabled: bool) -> Self {
        Self {
            animator: OverlayScrollbarAnimatorMac::new(expanded, unexpanded, enabled),
            timers: HashMap::new(),
            generation: 0,
        }
    }
    pub fn Activity(&mut self, activity: Activity, now: TimeTicks) -> Vec<Effect> {
        let effects = match activity {
            Activity::Entered => self.animator.MouseDidEnter(),
            Activity::Exited => self.animator.MouseDidExit(),
            Activity::Scrolled(mouse) => self.animator.DidScroll(mouse),
            Activity::FadeInDeferred(mouse) => self.animator.FadeInScrollbar(true, mouse),
            Activity::FadeOutIfNeeded => self.animator.FadeOutScrollbarIfNeeded(),
        };
        self.Apply(effects, now)
    }
    pub fn TimerFired(&mut self, timer: Timer, generation: u64, now: TimeTicks) -> Vec<Effect> {
        let Some((stored, clock)) = self.timers.get(&timer) else {
            return Vec::new();
        };
        if *stored != generation {
            return Vec::new();
        }
        let (progress, running) = clock.Sample(now);
        let effects = self.animator.TimerFired(timer, progress);
        let mut output = self.Apply(effects, now);
        if self
            .timers
            .get(&timer)
            .is_some_and(|(stored, _)| *stored == generation)
        {
            if running {
                output.push(Effect::Schedule {
                    timer,
                    generation,
                    delay_us: 16_666,
                });
            } else {
                self.timers.remove(&timer);
            }
        }
        output
    }
    fn Apply(&mut self, effects: Vec<NativeEffect>, now: TimeTicks) -> Vec<Effect> {
        let (mut thumb_dirty, mut track_dirty, mut hidden) = (false, false, None);
        let mut output = Vec::new();
        for effect in effects {
            match effect {
                NativeEffect::Start(timer, duration) => {
                    self.generation = self
                        .generation
                        .checked_add(1)
                        .expect("Mac scrollbar timer generation");
                    self.timers.insert(
                        timer,
                        (
                            self.generation,
                            ScrollbarAnimationTimerMac::new(now, duration),
                        ),
                    );
                    output.push(Effect::Cancel(timer));
                    output.push(Effect::Schedule {
                        timer,
                        generation: self.generation,
                        delay_us: if timer == Timer::FadeDelay {
                            duration
                        } else {
                            duration.min(16_666)
                        },
                    });
                }
                NativeEffect::Stop(timer) => {
                    self.timers.remove(&timer);
                    output.push(Effect::Cancel(timer));
                }
                NativeEffect::Thumb => thumb_dirty = true,
                NativeEffect::Track => track_dirty = true,
                NativeEffect::Hidden(value) => hidden = Some(value),
            }
        }
        if thumb_dirty || track_dirty || hidden.is_some() {
            output.push(Effect::Publish {
                thumb: self.animator.ThumbAlpha(),
                track: self.animator.TrackAlpha(),
                width: self.animator.ThumbWidth(),
                thumb_dirty,
                track_dirty,
                hidden,
            });
        }
        output
    }
}
