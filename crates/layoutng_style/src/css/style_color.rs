use foundation::{
    CSSValueID, CalculationValue, Color, ColorSpace, HueInterpolationMethod, Member, Visitor,
};

use super::forward::{ui::ColorProvider, CSSLengthResolver};
use crate::style::color_scheme::mojom::blink::ColorScheme;
use crate::style::forward::CSSValue;

// Bind the source-mapped Trace bodies to the layout heap's tracing interface.
macro_rules! impl_traceable_color {
    ($($type:ty),+ $(,)?) => {$ (
        impl foundation::Traceable for $type {
            fn Trace(&self, visitor: &mut Visitor<'_>) {
                <$type>::Trace(self, visitor);
            }
        }
    )+ };
}

impl_traceable_color!(
    ColorOrUnresolvedColorFunction,
    UnresolvedColorFunction,
    UnresolvedColorMix,
    UnresolvedRelativeColor,
    UnresolvedContrastColor,
    UnresolvedAlphaColor,
    StyleColor,
);

// cpp: layoutng_style/css/style_color.h:66-70
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnderlyingColorType {
    kColor,
    kColorFunction,
    kCurrentColor,
}

// cpp: layoutng_style/css/style_color.h:71-101
#[derive(Clone)]
pub struct ColorOrUnresolvedColorFunction {
    color: Color,
    unresolved_color_function: Member<UnresolvedColorFunction>,
}

impl Default for ColorOrUnresolvedColorFunction {
    fn default() -> Self {
        Self {
            color: Color::kTransparent,
            unresolved_color_function: Member::default(),
        }
    }
}

#[allow(non_snake_case)]
impl ColorOrUnresolvedColorFunction {
    // cpp: layoutng_style/css/style_color.h:75-79
    pub fn from_color(color: Color) -> Self {
        Self {
            color: color,
            unresolved_color_function: Member::default(),
        }
    }
    pub fn from_function(color_function: *mut UnresolvedColorFunction) -> Self {
        Self {
            color: Color::default(),
            unresolved_color_function: Member::from_ptr(color_function),
        }
    }

    // cpp: layoutng_style/css/style_color.h:81
    // cpp: layoutng_style/css/style_color_data.cc:62-64
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.unresolved_color_function);
    }

    // cpp: layoutng_style/css/style_color.h:83-97
    pub fn Equals(first: &Self, second: &Self, color_type: UnderlyingColorType) -> bool {
        match color_type {
            UnderlyingColorType::kCurrentColor => true,
            UnderlyingColorType::kColor => first.color == second.color,
            UnderlyingColorType::kColorFunction => unsafe {
                &*first.unresolved_color_function.Get() == &*second.unresolved_color_function.Get()
            },
        }
    }
}

// cpp: layoutng_style/css/style_color.h:116
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnresolvedColorFunctionType {
    kColorMix,
    kRelativeColor,
    kContrastColor,
    kAlphaColor,
}

// cpp: layoutng_style/css/style_color.h:103-124
pub enum UnresolvedColorFunction {
    ColorMix(UnresolvedColorMix),
    RelativeColor(UnresolvedRelativeColor),
    ContrastColor(UnresolvedContrastColor),
    AlphaColor(UnresolvedAlphaColor),
}

#[allow(non_snake_case)]
impl UnresolvedColorFunction {
    // cpp: layoutng_style/css/style_color.h:106
    pub fn Trace(&self, visitor: &mut Visitor) {
        match self {
            Self::ColorMix(value) => value.Trace(visitor),
            Self::RelativeColor(value) => value.Trace(visitor),
            Self::ContrastColor(value) => value.Trace(visitor),
            Self::AlphaColor(value) => value.Trace(visitor),
        }
    }

    // cpp: layoutng_style/css/style_color.h:107
    pub fn ToCSSValue(&self) -> *mut CSSValue {
        match self {
            Self::ColorMix(value) => value.ToCSSValue(),
            Self::RelativeColor(value) => value.ToCSSValue(),
            Self::ContrastColor(value) => value.ToCSSValue(),
            Self::AlphaColor(value) => value.ToCSSValue(),
        }
    }

    // cpp: layoutng_style/css/style_color.h:114
    pub fn Resolve(&self, current_color: &Color) -> Color {
        match self {
            Self::ColorMix(value) => value.Resolve(current_color),
            Self::RelativeColor(value) => value.Resolve(current_color),
            Self::ContrastColor(value) => value.Resolve(current_color),
            Self::AlphaColor(value) => value.Resolve(current_color),
        }
    }

    // cpp: layoutng_style/css/style_color.h:117
    pub fn GetType(&self) -> UnresolvedColorFunctionType {
        match self {
            Self::ColorMix(_) => UnresolvedColorFunctionType::kColorMix,
            Self::RelativeColor(_) => UnresolvedColorFunctionType::kRelativeColor,
            Self::ContrastColor(_) => UnresolvedColorFunctionType::kContrastColor,
            Self::AlphaColor(_) => UnresolvedColorFunctionType::kAlphaColor,
        }
    }
}

// cpp: layoutng_style/css/style_color.h:393-397
// The source declares debug ostream insertion without supplying definitions.
impl std::fmt::Display for StyleColor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { StyleColorFormat(self, formatter) }
    }
}

impl std::fmt::Display for UnresolvedColorFunction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { UnresolvedColorFunctionFormat(self, formatter) }
    }
}

// cpp: layoutng_style/css/style_color_data.cc:9-29
impl PartialEq for UnresolvedColorFunction {
    fn eq(&self, other: &Self) -> bool {
        if self.GetType() != other.GetType() {
            return false;
        }
        match (self, other) {
            (Self::ColorMix(a), Self::ColorMix(b)) => a == b,
            (Self::RelativeColor(a), Self::RelativeColor(b)) => a == b,
            (Self::ContrastColor(a), Self::ContrastColor(b)) => a == b,
            (Self::AlphaColor(a), Self::AlphaColor(b)) => a == b,
            _ => unreachable!(),
        }
    }
}

// cpp: layoutng_style/css/style_color.h:126-170
pub struct UnresolvedColorMix {
    color_interpolation_space_: ColorSpace,
    hue_interpolation_method_: HueInterpolationMethod,
    color1_: ColorOrUnresolvedColorFunction,
    color2_: ColorOrUnresolvedColorFunction,
    percentage_: f64,
    alpha_multiplier_: f64,
    color1_type_: UnderlyingColorType,
    color2_type_: UnderlyingColorType,
}

#[allow(non_snake_case)]
impl UnresolvedColorMix {
    // cpp: layoutng_style/css/style_color.h:128-133
    // No constructor definition is supplied in this package.
    pub fn new(
        space: ColorSpace,
        hue: HueInterpolationMethod,
        c1: &StyleColor,
        c2: &StyleColor,
        percentage: f64,
        alpha_multiplier: f64,
    ) -> Self {
        unsafe { UnresolvedColorMixConstruct(space, hue, c1, c2, percentage, alpha_multiplier) }
    }
    // cpp: layoutng_style/css/style_color.h:135-139
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.color1_.Trace(visitor);
        self.color2_.Trace(visitor);
    }
    // cpp: layoutng_style/css/style_color.h:141
    pub fn ToCSSValue(&self) -> *mut CSSValue {
        unsafe { UnresolvedColorMixToCSSValue(self) }
    }
    // cpp: layoutng_style/css/style_color.h:143
    pub fn Resolve(&self, current_color: &Color) -> Color {
        unsafe { UnresolvedColorMixResolve(self, current_color) }
    }
}

// cpp: layoutng_style/css/style_color.h:145-158
impl PartialEq for UnresolvedColorMix {
    fn eq(&self, other: &Self) -> bool {
        if self.color_interpolation_space_ != other.color_interpolation_space_
            || self.hue_interpolation_method_ != other.hue_interpolation_method_
            || self.percentage_ != other.percentage_
            || self.alpha_multiplier_ != other.alpha_multiplier_
            || self.color1_type_ != other.color1_type_
            || self.color2_type_ != other.color2_type_
        {
            return false;
        }
        ColorOrUnresolvedColorFunction::Equals(&self.color1_, &other.color1_, self.color1_type_)
            && ColorOrUnresolvedColorFunction::Equals(
                &self.color2_,
                &other.color2_,
                self.color2_type_,
            )
    }
}

// cpp: layoutng_style/css/style_color.h:172-207
pub struct UnresolvedRelativeColor {
    origin_color_: ColorOrUnresolvedColorFunction,
    origin_color_type_: UnderlyingColorType,
    color_interpolation_space_: ColorSpace,
    alpha_was_specified_: bool,
    channel0_: Member<CalculationValue>,
    channel1_: Member<CalculationValue>,
    channel2_: Member<CalculationValue>,
    alpha_: Member<CalculationValue>,
}

#[allow(non_snake_case)]
impl UnresolvedRelativeColor {
    // cpp: layoutng_style/css/style_color.h:174-180
    // No constructor definition is supplied in this package.
    pub fn new(
        origin: &StyleColor,
        space: ColorSpace,
        channel0: &CSSValue,
        channel1: &CSSValue,
        channel2: &CSSValue,
        alpha: *const CSSValue,
        resolver: &CSSLengthResolver,
    ) -> Self {
        unsafe {
            UnresolvedRelativeColorConstruct(
                origin, space, channel0, channel1, channel2, alpha, resolver,
            )
        }
    }
    // cpp: layoutng_style/css/style_color.h:182-189
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.origin_color_.Trace(visitor);
        visitor.Trace(&self.channel0_);
        visitor.Trace(&self.channel1_);
        visitor.Trace(&self.channel2_);
        visitor.Trace(&self.alpha_);
    }
    // cpp: layoutng_style/css/style_color.h:190-191
    pub fn ToCSSValue(&self) -> *mut CSSValue {
        unsafe { UnresolvedRelativeColorToCSSValue(self) }
    }
    pub fn Resolve(&self, current_color: &Color) -> Color {
        unsafe { UnresolvedRelativeColorResolve(self, current_color) }
    }
}

// cpp: layoutng_style/css/style_color_data.cc:47-61
impl PartialEq for UnresolvedRelativeColor {
    fn eq(&self, other: &Self) -> bool {
        if self.origin_color_type_ != other.origin_color_type_
            || self.color_interpolation_space_ != other.color_interpolation_space_
            || self.alpha_was_specified_ != other.alpha_was_specified_
            || !foundation::ValuesEquivalent(&self.channel0_, &other.channel0_)
            || !foundation::ValuesEquivalent(&self.channel1_, &other.channel1_)
            || !foundation::ValuesEquivalent(&self.channel2_, &other.channel2_)
            || !foundation::ValuesEquivalent(&self.alpha_, &other.alpha_)
        {
            return false;
        }
        ColorOrUnresolvedColorFunction::Equals(
            &self.origin_color_,
            &other.origin_color_,
            self.origin_color_type_,
        )
    }
}

// cpp: layoutng_style/css/style_color.h:209-224
pub struct UnresolvedContrastColor {
    param_color_: ColorOrUnresolvedColorFunction,
    param_color_type_: UnderlyingColorType,
}

#[allow(non_snake_case)]
impl UnresolvedContrastColor {
    // cpp: layoutng_style/css/style_color.h:211
    // No constructor definition is supplied in this package.
    pub fn new(param_color: &StyleColor) -> Self {
        unsafe { UnresolvedContrastColorConstruct(param_color) }
    }
    // cpp: layoutng_style/css/style_color.h:213-214
    pub fn ToCSSValue(&self) -> *mut CSSValue {
        unsafe { UnresolvedContrastColorToCSSValue(self) }
    }
    pub fn Resolve(&self, current_color: &Color) -> Color {
        unsafe { UnresolvedContrastColorResolve(self, current_color) }
    }
    // cpp: layoutng_style/css/style_color.h:216-219
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.param_color_.Trace(visitor);
    }
}

// cpp: layoutng_style/css/style_color_data.cc:30-35
impl PartialEq for UnresolvedContrastColor {
    fn eq(&self, other: &Self) -> bool {
        self.param_color_type_ == other.param_color_type_
            && ColorOrUnresolvedColorFunction::Equals(
                &self.param_color_,
                &other.param_color_,
                self.param_color_type_,
            )
    }
}

// cpp: layoutng_style/css/style_color.h:226-246
pub struct UnresolvedAlphaColor {
    origin_color_: ColorOrUnresolvedColorFunction,
    origin_color_type_: UnderlyingColorType,
    alpha_was_specified_: bool,
    alpha_: Member<CalculationValue>,
}

#[allow(non_snake_case)]
impl UnresolvedAlphaColor {
    // cpp: layoutng_style/css/style_color.h:228-230
    // No constructor definition is supplied in this package.
    pub fn new(origin: &StyleColor, alpha: *const CSSValue, resolver: &CSSLengthResolver) -> Self {
        unsafe { UnresolvedAlphaColorConstruct(origin, alpha, resolver) }
    }
    // cpp: layoutng_style/css/style_color.h:232-233
    pub fn ToCSSValue(&self) -> *mut CSSValue {
        unsafe { UnresolvedAlphaColorToCSSValue(self) }
    }
    pub fn Resolve(&self, current_color: &Color) -> Color {
        unsafe { UnresolvedAlphaColorResolve(self, current_color) }
    }
    // cpp: layoutng_style/css/style_color.h:235-239
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.origin_color_.Trace(visitor);
        visitor.Trace(&self.alpha_);
    }
}

// cpp: layoutng_style/css/style_color_data.cc:37-44
impl PartialEq for UnresolvedAlphaColor {
    fn eq(&self, other: &Self) -> bool {
        self.origin_color_type_ == other.origin_color_type_
            && self.alpha_was_specified_ == other.alpha_was_specified_
            && ColorOrUnresolvedColorFunction::Equals(
                &self.origin_color_,
                &other.origin_color_,
                self.origin_color_type_,
            )
            && foundation::ValuesEquivalent(&self.alpha_, &other.alpha_)
    }
}

// cpp: layoutng_style/css/style_color.h:58-358
#[derive(Clone)]
pub struct StyleColor {
    pub(crate) color_keyword_: CSSValueID,
    color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction,
}

// cpp: layoutng_style/css/style_color.h:248
// cpp: layoutng_style/css/style_color.h:338-339
impl Default for StyleColor {
    fn default() -> Self {
        Self {
            color_keyword_: CSSValueID::kCurrentcolor,
            color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction::default(),
        }
    }
}

#[allow(non_snake_case)]
impl StyleColor {
    // cpp: layoutng_style/css/style_color.h:249-260
    pub fn from_color(color: Color) -> Self {
        Self {
            color_keyword_: CSSValueID::kInvalid,
            color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction::from_color(color),
        }
    }
    pub fn from_keyword(keyword: CSSValueID) -> Self {
        Self {
            color_keyword_: keyword,
            color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction::default(),
        }
    }
    pub fn from_function(color_function: *mut UnresolvedColorFunction) -> Self {
        Self {
            color_keyword_: CSSValueID::kInvalid,
            color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction::from_function(
                color_function,
            ),
        }
    }
    pub fn from_color_and_keyword(color: Color, keyword: CSSValueID) -> Self {
        Self {
            color_keyword_: keyword,
            color_or_unresolved_color_function_: ColorOrUnresolvedColorFunction::from_color(color),
        }
    }

    // cpp: layoutng_style/css/style_color.h:262-264
    pub fn Trace(&self, visitor: &mut Visitor) {
        self.color_or_unresolved_color_function_.Trace(visitor);
    }

    // cpp: layoutng_style/css/style_color.h:266
    pub fn CurrentColor() -> Self {
        Self::default()
    }

    // cpp: layoutng_style/css/style_color.h:268-284
    pub fn IsCurrentColor(&self) -> bool {
        self.color_keyword_ == CSSValueID::kCurrentcolor
    }
    pub fn IsUnresolvedColorFunction(&self) -> bool {
        !self
            .color_or_unresolved_color_function_
            .unresolved_color_function
            .Get()
            .is_null()
    }
    pub fn DependsOnCurrentColor(&self) -> bool {
        self.IsCurrentColor() || self.IsUnresolvedColorFunction()
    }
    pub fn IsSystemColorIncludingDeprecated(&self) -> bool {
        Self::IsSystemColorIncludingDeprecatedKeyword(self.color_keyword_)
    }
    pub fn IsSystemColor(&self) -> bool {
        Self::IsSystemColorKeyword(self.color_keyword_)
    }
    pub fn IsAbsoluteColor(&self) -> bool {
        !self.IsCurrentColor() && !self.IsUnresolvedColorFunction()
    }

    // cpp: layoutng_style/css/style_color.h:285
    // cpp: layoutng_style/css/style_color_data.cc:66-69
    pub fn GetColor(&self) -> Color {
        debug_assert!(!self.IsUnresolvedColorFunction());
        self.color_or_unresolved_color_function_.color.clone()
    }

    // cpp: layoutng_style/css/style_color.h:287-293
    pub fn GetColorKeyword(&self) -> CSSValueID {
        debug_assert!(!self.IsNumeric());
        self.color_keyword_
    }
    pub fn HasColorKeyword(&self) -> bool {
        self.color_keyword_ != CSSValueID::kInvalid
    }

    // cpp: layoutng_style/css/style_color.h:296-298
    // cpp: layoutng_style/css/style_color_data.cc:71-84
    pub fn Resolve(
        &self,
        current_color: Color,
        _color_scheme: ColorScheme,
        is_current_color: Option<&mut bool>,
    ) -> Color {
        if let Some(output) = is_current_color {
            *output = self.IsCurrentColor();
        }
        if self.IsCurrentColor() {
            return current_color;
        }
        assert!(!self.IsUnresolvedColorFunction());
        assert!(!self.HasColorKeyword());
        self.GetColor()
    }

    // cpp: layoutng_style/css/style_color.h:304-306
    // No definition is supplied in this package.
    pub fn ResolveSystemColor(
        &self,
        scheme: ColorScheme,
        provider: *const ColorProvider,
        can_expose_accent_color: bool,
    ) -> Self {
        unsafe { StyleColorResolveSystemColor(self, scheme, provider, can_expose_accent_color) }
    }

    // cpp: layoutng_style/css/style_color.h:308
    // No definition is supplied in this package.
    pub fn ToCSSValue(&self) -> *const CSSValue {
        unsafe { StyleColorToCSSValue(self) }
    }

    // cpp: layoutng_style/css/style_color.h:310-312
    pub fn IsNumeric(&self) -> bool {
        self.EffectiveColorKeyword() == CSSValueID::kInvalid
    }

    // cpp: layoutng_style/css/style_color.h:314-320
    // No definitions are supplied in this package.
    pub fn ColorFromKeyword(
        keyword: CSSValueID,
        scheme: ColorScheme,
        provider: *const ColorProvider,
        can_expose_accent_color: bool,
    ) -> Color {
        unsafe { StyleColorColorFromKeyword(keyword, scheme, provider, can_expose_accent_color) }
    }
    pub fn IsColorKeyword(keyword: CSSValueID) -> bool {
        unsafe { StyleColorIsColorKeyword(keyword) }
    }
    pub fn IsSystemColorIncludingDeprecatedKeyword(keyword: CSSValueID) -> bool {
        unsafe { StyleColorIsSystemColorIncludingDeprecated(keyword) }
    }
    pub fn IsSystemColorKeyword(keyword: CSSValueID) -> bool {
        unsafe { StyleColorIsSystemColor(keyword) }
    }

    // cpp: layoutng_style/css/style_color.h:352-355
    pub fn GetUnresolvedColorFunction(&self) -> &UnresolvedColorFunction {
        debug_assert!(self.IsUnresolvedColorFunction());
        unsafe {
            &*self
                .color_or_unresolved_color_function_
                .unresolved_color_function
                .Get()
        }
    }

    // cpp: layoutng_style/css/style_color.h:357
    // No definition is supplied in this package.
    pub fn EffectiveColorKeyword(&self) -> CSSValueID {
        unsafe { StyleColorEffectiveColorKeyword(self) }
    }
}

// cpp: layoutng_style/css/style_color.h:322-335
impl PartialEq for StyleColor {
    fn eq(&self, other: &Self) -> bool {
        if self.color_keyword_ != other.color_keyword_ {
            return false;
        }
        if self.IsUnresolvedColorFunction() || other.IsUnresolvedColorFunction() {
            return foundation::ValuesEquivalent(
                &self
                    .color_or_unresolved_color_function_
                    .unresolved_color_function,
                &other
                    .color_or_unresolved_color_function_
                    .unresolved_color_function,
            );
        }
        self.color_or_unresolved_color_function_.color
            == other.color_or_unresolved_color_function_.color
    }
}

// cpp: layoutng_style/css/style_color.h:360-390
#[allow(non_snake_case)]
impl UnresolvedColorFunction {
    pub fn IsColorMix(&self) -> bool {
        self.GetType() == UnresolvedColorFunctionType::kColorMix
    }
    pub fn IsRelativeColor(&self) -> bool {
        self.GetType() == UnresolvedColorFunctionType::kRelativeColor
    }
    pub fn IsContrastColor(&self) -> bool {
        self.GetType() == UnresolvedColorFunctionType::kContrastColor
    }
    pub fn IsAlphaColor(&self) -> bool {
        self.GetType() == UnresolvedColorFunctionType::kAlphaColor
    }
}

unsafe extern "Rust" {
    fn StyleColorFormat(
        value: &StyleColor,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;
    fn UnresolvedColorFunctionFormat(
        value: &UnresolvedColorFunction,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result;
    fn UnresolvedColorMixConstruct(
        space: ColorSpace,
        hue: HueInterpolationMethod,
        c1: &StyleColor,
        c2: &StyleColor,
        percentage: f64,
        alpha_multiplier: f64,
    ) -> UnresolvedColorMix;
    fn UnresolvedColorMixToCSSValue(value: &UnresolvedColorMix) -> *mut CSSValue;
    fn UnresolvedColorMixResolve(value: &UnresolvedColorMix, current_color: &Color) -> Color;
    fn UnresolvedRelativeColorConstruct(
        origin: &StyleColor,
        space: ColorSpace,
        channel0: &CSSValue,
        channel1: &CSSValue,
        channel2: &CSSValue,
        alpha: *const CSSValue,
        resolver: &CSSLengthResolver,
    ) -> UnresolvedRelativeColor;
    fn UnresolvedRelativeColorToCSSValue(value: &UnresolvedRelativeColor) -> *mut CSSValue;
    fn UnresolvedRelativeColorResolve(
        value: &UnresolvedRelativeColor,
        current_color: &Color,
    ) -> Color;
    fn UnresolvedContrastColorConstruct(color: &StyleColor) -> UnresolvedContrastColor;
    fn UnresolvedContrastColorToCSSValue(value: &UnresolvedContrastColor) -> *mut CSSValue;
    fn UnresolvedContrastColorResolve(
        value: &UnresolvedContrastColor,
        current_color: &Color,
    ) -> Color;
    fn UnresolvedAlphaColorConstruct(
        origin: &StyleColor,
        alpha: *const CSSValue,
        resolver: &CSSLengthResolver,
    ) -> UnresolvedAlphaColor;
    fn UnresolvedAlphaColorToCSSValue(value: &UnresolvedAlphaColor) -> *mut CSSValue;
    fn UnresolvedAlphaColorResolve(value: &UnresolvedAlphaColor, current_color: &Color) -> Color;
    fn StyleColorResolveSystemColor(
        value: &StyleColor,
        scheme: ColorScheme,
        provider: *const ColorProvider,
        can_expose_accent_color: bool,
    ) -> StyleColor;
    fn StyleColorToCSSValue(value: &StyleColor) -> *const CSSValue;
    fn StyleColorColorFromKeyword(
        keyword: CSSValueID,
        scheme: ColorScheme,
        provider: *const ColorProvider,
        can_expose_accent_color: bool,
    ) -> Color;
    fn StyleColorIsColorKeyword(keyword: CSSValueID) -> bool;
    fn StyleColorIsSystemColorIncludingDeprecated(keyword: CSSValueID) -> bool;
    fn StyleColorIsSystemColor(keyword: CSSValueID) -> bool;
    fn StyleColorEffectiveColorKeyword(value: &StyleColor) -> CSSValueID;
}
