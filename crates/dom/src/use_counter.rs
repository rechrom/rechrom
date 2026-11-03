#![allow(non_snake_case)]

use std::collections::HashSet;

/// The document-scoped features currently emitted by translated LayoutImage
/// code. Keep the feature identity at the DOM boundary, matching Blink's
/// `WebFeature`/`UseCounter` ownership rather than leaking telemetry flags into
/// Layout or Paint inputs.
///
/// cpp: public/mojom/use_counter/metrics/web_feature.mojom
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebFeature {
    kClientHintsContentDPR,
    kAnimatedImageUsedMoreThanOnce,
}

/// Sticky, document-owned feature usage. Blink's UseCounter records a feature
/// once per Document; callers may therefore report from repeated notifications
/// and paint phases without manufacturing an event count.
///
/// cpp: core/frame/use_counter.h
#[derive(Default)]
pub struct UseCounter {
    counted_features: HashSet<WebFeature>,
}

impl UseCounter {
    pub fn Count(&mut self, feature: WebFeature) {
        self.counted_features.insert(feature);
    }

    pub fn IsCounted(&self, feature: WebFeature) -> bool {
        self.counted_features.contains(&feature)
    }
}
