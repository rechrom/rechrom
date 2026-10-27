//! Invalidation-owned comparison of projected layout style inputs.

use layoutng_style::style::computed_style_base::{ComputedStyleBase, FieldDifference};

use super::ComputedStyle;

impl PartialEq for ComputedStyle {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        computed_style_eq(self, other)
    }
}

// Trait-method symbols name the implementing type, not the module containing
// the impl. Keep the sampled comparison under the invalidation package name.
#[inline(never)]
fn computed_style_eq(style: &ComputedStyle, other: &ComputedStyle) -> bool {
    style.native_style == other.native_style
        && style.paint == other.paint
        && geometry_inputs_equal(style, other)
}

impl ComputedStyle {
    /// Compare the inputs consumed by layout. Paint data may require paint or
    /// paint-property invalidation, but changing its value does not ordinarily
    /// invalidate geometry. Keep the few transitions which change the native
    /// box/containing-block model in this implementation.
    #[inline(always)]
    pub fn LayoutEquivalent(&self, other: &Self) -> bool {
        layout_equivalent(self, other)
    }
}

// Keep the sampled symbol package-qualified. Rust omits an inherent impl's
// source module from the method's demangled symbol.
#[inline(never)]
fn layout_equivalent(style: &ComputedStyle, other: &ComputedStyle) -> bool {
    // Chromium ComputedStyle::VisualInvalidationDiff/ DiffNeedsFullLayout
    // consume the generated FieldInvalidationDiff kLayout/kReshape bits. Native
    // styles are canonical at this boundary; the minimal projection does
    // not repeat every geometry property (for example text-align). Keep
    // the translated invalidation owner in this comparison so retained
    // LayoutObjects cannot mistake such changes for paint-only updates.
    if style.native_style != other.native_style {
        // Native styles are immutable and retained by the input owner for
        // the layout tree update, just as in PrepareNativeStyle.
        let (Some(old), Some(new)) = (unsafe { style.native_style.as_ref() }, unsafe {
            other.native_style.as_ref()
        }) else {
            return false;
        };
        if ComputedStyleBase::FieldInvalidationDiff(old, new)
            & (FieldDifference::kLayout.bits() | FieldDifference::kReshape.bits())
            != 0
        {
            return false;
        }
    }
    // Adding/removing a background can change whether an inline needs a
    // physical box fragment, even though its CSS dimensions are unchanged.
    if (style.paint.background_color.alpha > 0.0) != (other.paint.background_color.alpha > 0.0) {
        return false;
    }
    // `image-orientation` swaps the intrinsic axes for EXIF orientations
    // 5..8. DOM metadata and LayoutReplaced recompute natural sizing on
    // this transition, so it cannot be normalized away as paint-only.
    if style.paint.respect_image_orientation != other.paint.respect_image_orientation {
        return false;
    }
    // Blink treats ordinary transform changes, including adding/removing
    // the transform property node, as paint-property updates. LayoutBox
    // only escalates them to layout for anchor-positioning fragments whose
    // propagated anchor geometry depends on the transform. Rechrom does
    // not yet expose that anchor dependency, so making every transform
    // animation establish/remove a layout boundary is both more
    // conservative than Chromium and turns staggered animation completion
    // into repeated full-document layouts.
    //
    // Filter presence still changes the native layer/containing-block
    // model represented by this translated layout tree.
    if style.paint.filters.is_empty() != other.paint.filters.is_empty()
        || style.paint.backdrop_filters.is_empty() != other.paint.backdrop_filters.is_empty()
        || style.paint.backdrop_filter_has_reference != other.paint.backdrop_filter_has_reference
    {
        return false;
    }
    geometry_inputs_equal(style, other)
}

// Published groups are immutable. Match base::ValuesEquivalent: identical
// shared storage proves equality, while distinct storage compares values.
#[inline(always)]
fn geometry_inputs_equal(style: &ComputedStyle, other: &ComputedStyle) -> bool {
    let extended_equal = match (&style.extended, &other.extended) {
        (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b) || a == b,
        (None, None) => true,
        _ => false,
    };
    // Exhaustive destructuring forces new fields to enter this comparison.
    macro_rules! same_inputs {
            ($($field:ident),* $(,)?) => {{
                let ComputedStyle { native_style: _, paint: _, extended: _, $($field),* } = style;
                $($field == &other.$field)&&*
            }};
        }
    extended_equal
        && same_inputs!(
            svg_clip_resources,
            display,
            width,
            height,
            min_height,
            max_height,
            flex_basis,
            flex_grow,
            flex_shrink,
            min_width,
            max_width,
            margin,
            border,
            border_styles,
            padding,
            gap,
            flex_direction,
            flex_wrap,
            align_items,
            direction,
            writing_mode,
            position,
            left,
            top,
            right,
            bottom,
            css_clip,
            floating,
            column_count,
            grid_template_columns,
        )
}
