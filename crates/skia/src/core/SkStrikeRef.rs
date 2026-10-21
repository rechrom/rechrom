// UNVERIFIED: no compilation, tests, syntax or source checks were run.
use crate::include::private::SkTemplates::AutoSTArray;
use crate::src::core::{SkGlyph::SkGlyph, SkRect::SkRect, SkStrike::SkStrike};
use core::ptr::NonNull;
use std::sync::Arc;
// Source: include/core/SkStrikeRef.h:32-43
// Source: include/core/SkStrikeRef.h:92-93
// Native atomic strong owner; copied references share exactly the same cached
// strike while retaining the scalar scale from their original font resolution.
#[derive(Clone)]
pub struct SkStrikeRef {
    fStrike: Option<Arc<SkStrike>>,
    fStrikeToSourceScale: f32,
}
// Source: src/core/SkStrikeRef.cpp:23-25
fn scale_rect(r: SkRect, s: f32) -> SkRect {
    SkRect {
        fLeft: r.fLeft * s,
        fTop: r.fTop * s,
        fRight: r.fRight * s,
        fBottom: r.fBottom * s,
    }
}
// Source: src/core/SkStrikeRef.cpp:29-34
impl Default for SkStrikeRef {
    fn default() -> Self {
        Self {
            fStrike: None,
            fStrikeToSourceScale: 0.0,
        }
    }
}
impl SkStrikeRef {
    pub fn new() -> Self {
        Self::default()
    }
    // Rust's ordinary by-value move transfers ownership without a refcount change.
    // This explicit projection additionally exposes the native observable moved-from
    // handle: null owner and its unchanged copied scalar member.
    // Source: src/core/SkStrikeRef.cpp:33
    pub fn move_from(source: &mut Self) -> Self {
        Self {
            fStrike: source.fStrike.take(),
            fStrikeToSourceScale: source.fStrikeToSourceScale,
        }
    }
    // Source: src/core/SkStrikeRef.cpp:34
    pub fn move_assign(&mut self, source: &mut Self) {
        self.fStrike = source.fStrike.take();
        self.fStrikeToSourceScale = source.fStrikeToSourceScale;
    }
    // Source: src/core/SkStrikeRef.cpp:36-39
    /// Safety: this actual strike and its entire retained backend/effect graph
    /// obey the cache's cross-thread transfer and synchronous operation leases.
    pub(crate) unsafe fn from_strike(strike: Option<Arc<SkStrike>>, scale: f32) -> Self {
        Self {
            fStrike: strike,
            fStrikeToSourceScale: scale,
        }
    }
    // Source: include/core/SkStrikeRef.h:43
    pub fn isValid(&self) -> bool {
        self.fStrike.is_some()
    }
    // Source: src/core/SkStrikeRef.cpp:41-43
    pub fn getWidths(&self, glyphs: &[u16], widths: &mut [f32]) {
        self.getWidthsBounds(glyphs, widths, &mut []);
    }
    // Source: src/core/SkStrikeRef.cpp:45-49
    pub fn getWidth(&self, glyph: u16) -> f32 {
        let mut width = [0.0f32; 1];
        self.getWidths(&[glyph], &mut width);
        width[0]
    }
    // Source: src/core/SkStrikeRef.cpp:51-72
    pub fn getWidthsBounds(&self, glyphs: &[u16], widths: &mut [f32], bounds: &mut [SkRect]) {
        let strike = self
            .fStrike
            .as_ref()
            .expect("native SkStrikeRef requires a valid strike");
        let mut glyphPtrs = core::pin::pin!(AutoSTArray::<NonNull<SkGlyph>, 64>::new());
        // Native scratch pointers are unspecified before metrics fills every slot.
        // Dangling values initialize Rust pointer objects only; they are never read
        // or published. The actual producer overwrites every entry under its lock.
        glyphPtrs.as_mut().reset_with_constructor(
            i32::try_from(glyphs.len()).expect("native glyph count fits int"),
            NonNull::dangling,
        );
        unsafe {
            strike.withMetrics(
                glyphs,
                glyphPtrs.as_mut().as_mut_slice().unwrap(),
                |results| {
                    // Extend the native operation lease through consumption, matching existing
                    // Rust glyph producers and preventing arena/payload mutation aliases.
                    if !bounds.is_empty() {
                        let n = bounds.len().min(results.len());
                        for i in 0..n {
                            bounds[i] =
                                scale_rect(results[i].as_ref().rect(), self.fStrikeToSourceScale);
                        }
                    }
                    if !widths.is_empty() {
                        let n = widths.len().min(results.len());
                        for i in 0..n {
                            widths[i] = results[i].as_ref().advanceX() * self.fStrikeToSourceScale;
                        }
                    }
                },
            );
        }
    }
    // Source: src/core/SkStrikeRef.cpp:74-82
    /// Safety: each addressed glyph is aligned readable u32 and each advance is
    /// aligned writable f32; strides count 32-bit elements. Output writes must not
    /// alias strike state or invalidate future input reads, including zero strides.
    pub unsafe fn getWidthsStrided(
        &self,
        count: u32,
        first_glyph: *const u32,
        glyph_stride_32: u32,
        first_advance: *mut f32,
        advance_stride_32: u32,
    ) {
        let strike = self
            .fStrike
            .as_ref()
            .expect("native SkStrikeRef requires a valid strike");
        strike.getWidthsStrided(
            count,
            first_glyph,
            glyph_stride_32,
            first_advance,
            advance_stride_32,
            self.fStrikeToSourceScale,
        );
    }
}
