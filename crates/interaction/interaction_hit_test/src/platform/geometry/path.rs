// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/geometry/path.h:89-93
// Source: third_party/blink/renderer/core/layout/hit_test_location.cc:123-132,267-280
// A read-only projection of the existing Path owner; no copied path model or
// alternate path algorithm is retained by Interaction.
use foundation::gfx::{PointF, QuadF};
pub use foundation::WindRule;

pub trait PathHitTest {
    fn CountVerbs(&self) -> i32;
    fn Intersects(&self, quad: &QuadF) -> bool;
    fn Contains(&self, point: PointF) -> bool;
    fn IntersectsWithWindRule(&self, quad: &QuadF, rule: WindRule) -> bool;
    fn ContainsWithWindRule(&self, point: PointF, rule: WindRule) -> bool;
}
