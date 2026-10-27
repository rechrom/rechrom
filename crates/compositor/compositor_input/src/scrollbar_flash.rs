//! Scrollbar flash fan-out and once-per-controller policy.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarFlashMode {
    EveryController,
    OncePerController,
}

pub trait ScrollbarFlashContext {
    type ElementId: Copy + PartialEq;

    fn controller_count(&self) -> usize;
    fn controller_element_id(&self, index: usize) -> Self::ElementId;
    fn was_flashed(&self, element: Self::ElementId) -> bool;
    fn notify_controller(&mut self, index: usize, did_scroll: bool);
    fn mark_flashed(&mut self, element: Self::ElementId);
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScrollbarFlashStats {
    pub controllers_visited: usize,
    pub controllers_notified: usize,
    pub already_flashed: usize,
    pub tracking_element_flashed: bool,
}

pub fn FlashScrollbars<C: ScrollbarFlashContext>(
    context: &mut C,
    mode: ScrollbarFlashMode,
    tracking_element: C::ElementId,
    did_scroll: bool,
) -> ScrollbarFlashStats {
    let mut trace = browser_tracing::span("compositor.input", "Compositor.Input.FlashScrollbars");
    let mut stats = ScrollbarFlashStats::default();
    for index in 0..context.controller_count() {
        stats.controllers_visited += 1;
        let element = context.controller_element_id(index);
        if mode == ScrollbarFlashMode::OncePerController && context.was_flashed(element) {
            stats.already_flashed += 1;
            continue;
        }
        context.notify_controller(index, did_scroll);
        stats.controllers_notified += 1;
        if mode == ScrollbarFlashMode::OncePerController {
            context.mark_flashed(element);
            // Preserve the source assignment semantics: the last newly
            // flashed controller determines this return value.
            stats.tracking_element_flashed = element == tracking_element;
        }
    }
    trace.set("controllers_visited", stats.controllers_visited as f64);
    trace.set("controllers_notified", stats.controllers_notified as f64);
    trace.set("already_flashed", stats.already_flashed as f64);
    trace.set(
        "tracking_element_flashed",
        stats.tracking_element_flashed as u8 as f64,
    );
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct Context {
        ids: Vec<u8>,
        flashed: BTreeSet<u8>,
        notifications: Vec<(u8, bool)>,
    }
    impl ScrollbarFlashContext for Context {
        type ElementId = u8;
        fn controller_count(&self) -> usize {
            self.ids.len()
        }
        fn controller_element_id(&self, index: usize) -> u8 {
            self.ids[index]
        }
        fn was_flashed(&self, element: u8) -> bool {
            self.flashed.contains(&element)
        }
        fn notify_controller(&mut self, index: usize, did_scroll: bool) {
            self.notifications.push((self.ids[index], did_scroll));
        }
        fn mark_flashed(&mut self, element: u8) {
            self.flashed.insert(element);
        }
    }

    #[test]
    fn every_controller_mode_notifies_without_mutating_once_set() {
        let mut context = Context {
            ids: vec![4, 7],
            flashed: BTreeSet::from([4]),
            notifications: Vec::new(),
        };
        let stats = FlashScrollbars(&mut context, ScrollbarFlashMode::EveryController, 7, true);
        assert_eq!(context.notifications, [(4, true), (7, true)]);
        assert_eq!(context.flashed, BTreeSet::from([4]));
        assert_eq!(stats.controllers_notified, 2);
        assert!(!stats.tracking_element_flashed);
    }

    #[test]
    fn once_mode_skips_seen_controllers_and_preserves_last_assignment() {
        let mut context = Context {
            ids: vec![4, 7, 9],
            flashed: BTreeSet::from([4]),
            notifications: Vec::new(),
        };
        let stats = FlashScrollbars(
            &mut context,
            ScrollbarFlashMode::OncePerController,
            7,
            false,
        );
        assert_eq!(context.notifications, [(7, false), (9, false)]);
        assert_eq!(stats.already_flashed, 1);
        assert!(!stats.tracking_element_flashed);
        assert_eq!(context.flashed, BTreeSet::from([4, 7, 9]));
    }
}
