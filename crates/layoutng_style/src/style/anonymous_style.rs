use foundation::{EDisplay, Member};

use super::applied_text_decoration::AppliedTextDecorationVector;
use super::computed_style::{ComputedStyle, ComputedStyleBuilder};
use super::computed_style_base::IsAtShadowBoundary;

// cpp: layoutng_style/style/anonymous_style.h:10-17
// cpp: layoutng_style/style/anonymous_style.cc:35-44
#[allow(non_snake_case)]
pub fn CreateAnonymousStyleBuilderWithDisplay(
    parent_style: &ComputedStyle,
    display: EDisplay,
    parent_decorations: *mut AppliedTextDecorationVector,
) -> ComputedStyleBuilder {
    let mut builder = ComputedStyleBuilder::from_initial_and_parent(
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() },
        parent_style,
        IsAtShadowBoundary::kNotAtShadowBoundary,
    );
    builder.SetUnicodeBidi(parent_style.GetUnicodeBidi());
    builder.SetBaseTextDecorationData(Member::from_ptr(parent_decorations));
    builder.SetDisplay(display);
    builder
}
