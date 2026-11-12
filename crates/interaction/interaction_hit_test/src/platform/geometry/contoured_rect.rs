// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/geometry/contoured_rect.h:262
// Source: third_party/blink/renderer/core/layout/hit_test_location.cc:246-248
// The ContouredRect owner supplies its existing geometry query directly.
pub trait ContouredRectHitTest {
    fn IntersectsQuad(&self, quad: &foundation::gfx::QuadF) -> bool;
}
