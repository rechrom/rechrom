//! LayoutObject style selection used by retained fragments and paint policy.
//!
//! The standard variant stays on an inline fast path. Non-standard variants
//! may resolve a containing block, so the slow path retains one fragment-owned
//! sampling frame.

use super::*;

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2492-2497
    pub fn EffectiveStyle(&self, style_variant: StyleVariant) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        if style_variant == StyleVariant::kStandard {
            self.StyleRef()
        } else {
            self.SlowEffectiveStyle(style_variant)
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:380-400
    #[inline(never)]
    pub fn SlowEffectiveStyle(&self, style_variant: StyleVariant) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        match style_variant {
            StyleVariant::kStandard => self.StyleRef(),
            StyleVariant::kFirstLine => {
                if self.IsAtomicInline() {
                    self.StyleRef()
                } else {
                    self.FirstLineStyleRef()
                }
            }
            StyleVariant::kStandardEllipsis => {
                if !self.IsLayoutBlockFlow() {
                    let block = self.ContainingBlock();
                    if !block.is_null() {
                        return unsafe { &*block }.StyleRef();
                    }
                }
                self.StyleRef()
            }
            StyleVariant::kFirstLineEllipsis => {
                let block = self.ContainingBlock();
                if !block.is_null() {
                    return unsafe { &*block }.FirstLineStyleRef();
                }
                self.FirstLineStyleRef()
            }
        }
    }

    // cpp: layoutng/internal/layout_object.h:2499-2502
    pub fn ResolveColorWithStyle(style: &ComputedStyle, color_property: &Longhand) -> Color {
        style.VisitedDependentColor(color_property, None)
    }

    // cpp: layoutng/internal/layout_object.h:2504-2507
    pub fn ResolveColor(&self, color_property: &Longhand) -> Color {
        self.CheckIsNotDestroyed();
        self.StyleRef().VisitedDependentColor(color_property, None)
    }

    // cpp: layoutng/internal/layout_object.h:2687-2691
    pub fn CreatesGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasGroupingProperty(self.HasReflection())
    }

    // cpp: layoutng/internal/layout_object.h:1519-1519
    pub fn HasClipRelatedProperty(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasClipRelatedPropertyProvider(self) }
    }
}
