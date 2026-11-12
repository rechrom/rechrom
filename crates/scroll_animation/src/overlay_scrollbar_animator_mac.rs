//! ui/native_theme/scrollbar_animator_mac.cc:17-286; .h:55-147.
//! Pure source state. The embedding routes timer requests and decided native
//! paint mutations; it never computes opacity, width or fade policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Timer {
    Expand,
    FadeInTrack,
    FadeOut,
    FadeDelay,
}
#[derive(Clone, Copy, Debug)]
pub enum Effect {
    Start(Timer, u64),
    Stop(Timer),
    Thumb,
    Track,
    Hidden(bool),
}
pub struct OverlayScrollbarAnimatorMac {
    expanded: i32,
    unexpanded: i32,
    width: i32,
    thumb: f32,
    track: f32,
    mouse_in_track: bool,
    enabled: bool,
    deferred: bool,
    expand: bool,
    fade_in: bool,
    fade_out: bool,
    delay: bool,
}
impl OverlayScrollbarAnimatorMac {
    pub fn new(expanded: i32, unexpanded: i32, enabled: bool) -> Self {
        Self {
            expanded,
            unexpanded,
            width: unexpanded,
            thumb: 0.,
            track: 0.,
            mouse_in_track: false,
            enabled,
            deferred: false,
            expand: false,
            fade_in: false,
            fade_out: false,
            delay: false,
        }
    }
    pub fn ThumbAlpha(&self) -> f32 {
        self.thumb
    }
    pub fn TrackAlpha(&self) -> f32 {
        self.track
    }
    pub fn ThumbWidth(&self) -> i32 {
        self.width
    }
    fn Duration(&self) -> u64 {
        if self.enabled {
            250_000
        } else {
            0
        }
    }
    fn DelayUpdate(&mut self, e: &mut Vec<Effect>) {
        if self.mouse_in_track || self.deferred {
            self.delay = false;
            e.push(Effect::Stop(Timer::FadeDelay));
            return;
        }
        if !self.enabled && self.width != self.expanded {
            return;
        }
        self.delay = true;
        e.push(Effect::Start(
            Timer::FadeDelay,
            if self.enabled { 500_000 } else { 0 },
        ));
    }
    fn CancelFade(&mut self, e: &mut Vec<Effect>) {
        assert!(self.fade_out);
        self.fade_out = false;
        e.push(Effect::Stop(Timer::FadeOut));
        self.thumb = 1.;
        e.push(Effect::Thumb);
        if self.track > 0. {
            self.track = 1.;
            e.push(Effect::Track);
        }
    }
    pub fn MouseDidEnter(&mut self) -> Vec<Effect> {
        let mut e = Vec::new();
        if self.thumb == 0. || self.mouse_in_track {
            return e;
        }
        self.mouse_in_track = true;
        if self.fade_out {
            self.CancelFade(&mut e);
        }
        self.DelayUpdate(&mut e);
        if !self.fade_in && self.track != 1. {
            self.fade_in = true;
            e.push(Effect::Start(Timer::FadeInTrack, self.Duration()));
        }
        if !self.expand && self.width != self.expanded {
            self.expand = true;
            e.push(Effect::Start(Timer::Expand, self.Duration()));
        }
        e
    }
    pub fn MouseDidExit(&mut self) -> Vec<Effect> {
        self.mouse_in_track = false;
        let mut e = Vec::new();
        self.DelayUpdate(&mut e);
        e
    }
    pub fn DidScroll(&mut self, mouse_in_frame: bool) -> Vec<Effect> {
        let mut e = Vec::new();
        if self.fade_out {
            self.CancelFade(&mut e);
            self.DelayUpdate(&mut e);
            return e;
        }
        if self.thumb == 1. {
            self.DelayUpdate(&mut e);
            return e;
        }
        debug_assert_eq!(self.width, self.unexpanded);
        debug_assert_eq!(self.thumb, 0.);
        debug_assert!(!self.fade_in);
        self.width = self.unexpanded;
        self.thumb = 1.;
        e.push(Effect::Thumb);
        e.push(Effect::Hidden(false));
        if mouse_in_frame {
            self.mouse_in_track = true;
            self.width = self.expanded;
            self.track = 1.;
            e.push(Effect::Track);
        }
        self.DelayUpdate(&mut e);
        e
    }
    pub fn FadeInScrollbar(&mut self, defer: bool, mouse_in_frame: bool) -> Vec<Effect> {
        self.deferred = defer;
        self.DidScroll(mouse_in_frame)
    }
    pub fn FadeOutScrollbarIfNeeded(&mut self) -> Vec<Effect> {
        if !self.deferred {
            return Vec::new();
        }
        self.deferred = false;
        let mut e = Vec::new();
        self.DelayUpdate(&mut e);
        e
    }
    pub fn TimerFired(&mut self, timer: Timer, progress: f64) -> Vec<Effect> {
        let mut e = Vec::new();
        match timer {
            Timer::FadeDelay => {
                self.delay = false;
                self.fade_in = false;
                self.fade_out = true;
                e.push(Effect::Stop(Timer::FadeInTrack));
                e.push(Effect::Stop(Timer::FadeOut));
                e.push(Effect::Start(Timer::FadeOut, self.Duration()));
            }
            Timer::Expand => {
                if !self.expand {
                    return e;
                }
                self.width = ((1. - progress) * f64::from(self.unexpanded)
                    + progress * f64::from(self.expanded)) as i32;
                e.push(Effect::Thumb);
                if progress == 1. {
                    self.expand = false;
                }
            }
            Timer::FadeInTrack => {
                if !self.fade_in {
                    return e;
                }
                debug_assert!(!self.fade_out);
                self.track = progress as f32;
                e.push(Effect::Track);
                if progress == 1. {
                    self.fade_in = false;
                }
            }
            Timer::FadeOut => {
                if !self.fade_out {
                    return e;
                }
                debug_assert!(!self.fade_in);
                self.thumb = (1. - progress) as f32;
                e.push(Effect::Thumb);
                if self.track != 0. {
                    self.track = (1. - progress) as f32;
                    e.push(Effect::Track);
                }
                if progress == 1. {
                    self.expand = false;
                    self.fade_out = false;
                    e.push(Effect::Stop(Timer::Expand));
                    self.width = self.unexpanded;
                    e.push(Effect::Hidden(true));
                }
            }
        }
        e
    }
}
/// Source timer clock and ease-in-out. A 60 Hz task services an animation;
/// sampling elapsed time never decides completion of a different scroll owner.
pub struct ScrollbarAnimationTimerMac {
    start: crate::TimeTicks,
    duration: u64,
}
impl ScrollbarAnimationTimerMac {
    pub fn new(now: crate::TimeTicks, duration: u64) -> Self {
        Self {
            start: now,
            duration,
        }
    }
    pub fn Sample(&self, now: crate::TimeTicks) -> (f64, bool) {
        let elapsed = (now - self.start).InMicroseconds().max(0) as u64;
        let done = elapsed >= self.duration;
        let fraction = if self.duration == 0 {
            1.
        } else {
            (elapsed as f64 / self.duration as f64).clamp(0., 1.)
        };
        let curve = crate::cubic_bezier::CubicBezier::new(0.42, 0., 0.58, 1.);
        (curve.Solve(fraction), !done)
    }
}
