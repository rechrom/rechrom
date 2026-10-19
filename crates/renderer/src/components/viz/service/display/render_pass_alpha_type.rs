// Source: components/viz/service/display/render_pass_alpha_type.h:1-28
// Chromium revision: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
#![allow(non_camel_case_types)]

// Source: components/viz/service/display/render_pass_alpha_type.h:15-24
// C++ SkAlphaType values are retained. Rust's explicit representation replaces
// the underlying_type static_assert; this enum admits only these two variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum RenderPassAlphaType {
    #[default]
    kPremul = 2,
    kOpaque = 1,
}
