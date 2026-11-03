//! LayoutObject style selection used by retained fragments and paint policy.
//!
//! The standard variant stays on an inline fast path. Non-standard variants
//! may resolve a containing block, so the slow path retains one fragment-owned
//! sampling frame.

use super::*;

// cpp: third_party/blink/renderer/core/layout/layout_object.cc:888-914
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutObjectHasClipRelatedPropertyProvider(
    object: *const LayoutObject,
) -> bool {
    let object = unsafe { &*object };
    object.CheckIsNotDestroyed();
    if object.HasCSSClip() || object.ShouldClipOverflowAlongEitherAxis() {
        return true;
    }
    object.ShouldApplyPaintContainment()
        || (object.ShouldApplyStyleContainment() && object.ShouldApplyLayoutContainment())
}

impl LayoutObject {
    pub fn EffectiveStyle(&self, style_variant: StyleVariant) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        if style_variant == StyleVariant::kStandard {
            self.StyleRef()
        } else {
            self.SlowEffectiveStyle(style_variant)
        }
    }

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

    pub fn ResolveColorWithStyle(style: &ComputedStyle, color_property: &Longhand) -> Color {
        style.VisitedDependentColor(color_property, None)
    }

    pub fn ResolveColor(&self, color_property: &Longhand) -> Color {
        self.CheckIsNotDestroyed();
        self.StyleRef().VisitedDependentColor(color_property, None)
    }

    pub fn CreatesGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasGroupingProperty(self.HasReflection())
    }

    pub fn HasClipRelatedProperty(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasClipRelatedPropertyProvider(self) }
    }
}
