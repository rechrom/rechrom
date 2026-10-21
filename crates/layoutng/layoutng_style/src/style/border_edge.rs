use foundation::{Color, EBorderStyle};

// cpp: layoutng_style/style/border_edge.h:16-56
pub struct BorderEdge {
    color_: Color,
    is_present_: bool,
    style_: EBorderStyle,
    width_: i32,
}

// cpp: layoutng_style/style/border_edge.h:39-42
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoubleBorderStripe {
    kDoubleBorderStripeOuter,
    kDoubleBorderStripeInner,
}

#[allow(non_snake_case)]
impl BorderEdge {
    // cpp: layoutng_style/style/border_edge.h:20-24
    // Constructor definitions are absent from the supplied package.
    pub fn new(
        edge_width: i32,
        edge_color: &Color,
        edge_style: EBorderStyle,
        edge_is_present: bool,
    ) -> Self {
        unsafe { BorderEdgeConstruct(edge_width, edge_color, edge_style, edge_is_present) }
    }
    pub fn default_edge() -> Self {
        unsafe { BorderEdgeDefaultConstruct() }
    }

    // cpp: layoutng_style/style/border_edge.h:26
    pub fn EffectiveStyle(style: EBorderStyle, width: i32) -> EBorderStyle {
        unsafe { BorderEdgeEffectiveStyle(style, width) }
    }

    // cpp: layoutng_style/style/border_edge.h:28-35
    pub fn HasVisibleColorAndStyle(&self) -> bool {
        unsafe { BorderEdgeHasVisibleColorAndStyle(self) }
    }
    pub fn ShouldRender(&self) -> bool {
        unsafe { BorderEdgeShouldRender(self) }
    }
    pub fn PresentButInvisible(&self) -> bool {
        unsafe { BorderEdgePresentButInvisible(self) }
    }
    pub fn ObscuresBackgroundEdge(&self) -> bool {
        unsafe { BorderEdgeObscuresBackgroundEdge(self) }
    }
    pub fn ObscuresBackground(&self) -> bool {
        unsafe { BorderEdgeObscuresBackground(self) }
    }
    pub fn UsedWidth(&self) -> i32 {
        unsafe { BorderEdgeUsedWidth(self) }
    }
    pub fn SharesColorWith(&self, other: &Self) -> bool {
        unsafe { BorderEdgeSharesColorWith(self, other) }
    }

    // cpp: layoutng_style/style/border_edge.h:37
    pub fn BorderStyle(&self) -> EBorderStyle {
        self.style_
    }

    // cpp: layoutng_style/style/border_edge.h:44
    pub fn GetDoubleBorderStripeWidth(&self, stripe: DoubleBorderStripe) -> i32 {
        unsafe { BorderEdgeGetDoubleBorderStripeWidth(self, stripe) }
    }

    // cpp: layoutng_style/style/border_edge.h:46-47
    pub fn Width(&self) -> i32 {
        self.width_
    }
    pub fn GetColor(&self) -> &Color {
        &self.color_
    }

    // cpp: layoutng_style/style/border_edge.h:49
    pub fn ClampWidth(&mut self, max_width: i32) {
        unsafe { BorderEdgeClampWidth(self, max_width) }
    }
}

// cpp: layoutng_style/style/border_edge.h:58
pub type BorderEdgeArray = [BorderEdge; 4];

unsafe extern "Rust" {
    fn BorderEdgeConstruct(
        edge_width: i32,
        edge_color: &Color,
        edge_style: EBorderStyle,
        edge_is_present: bool,
    ) -> BorderEdge;
    fn BorderEdgeDefaultConstruct() -> BorderEdge;
    fn BorderEdgeEffectiveStyle(style: EBorderStyle, width: i32) -> EBorderStyle;
    fn BorderEdgeHasVisibleColorAndStyle(value: &BorderEdge) -> bool;
    fn BorderEdgeShouldRender(value: &BorderEdge) -> bool;
    fn BorderEdgePresentButInvisible(value: &BorderEdge) -> bool;
    fn BorderEdgeObscuresBackgroundEdge(value: &BorderEdge) -> bool;
    fn BorderEdgeObscuresBackground(value: &BorderEdge) -> bool;
    fn BorderEdgeUsedWidth(value: &BorderEdge) -> i32;
    fn BorderEdgeSharesColorWith(value: &BorderEdge, other: &BorderEdge) -> bool;
    fn BorderEdgeGetDoubleBorderStripeWidth(value: &BorderEdge, stripe: DoubleBorderStripe) -> i32;
    fn BorderEdgeClampWidth(value: &mut BorderEdge, max_width: i32);
}
