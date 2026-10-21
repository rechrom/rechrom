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
        Self {
            color_interpolation_space_: space,
            hue_interpolation_method_: hue,
            color1_: c1.color_or_unresolved_color_function_.clone(),
            color2_: c2.color_or_unresolved_color_function_.clone(),
            percentage_: percentage,
            alpha_multiplier_: alpha_multiplier,
            color1_type_: UnderlyingColorTypeFor(c1),
            color2_type_: UnderlyingColorTypeFor(c2),
        }
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
        let first = ResolveColorOperand(&self.color1_, self.color1_type_, current_color);
        let second = ResolveColorOperand(&self.color2_, self.color2_type_, current_color);
        debug_assert_eq!(self.color_interpolation_space_, ColorSpace::kOklab);
        MixInOklab(first, second, self.percentage_, self.alpha_multiplier_)
    }
}

fn UnderlyingColorTypeFor(color: &StyleColor) -> UnderlyingColorType {
    if color.IsUnresolvedColorFunction() {
        UnderlyingColorType::kColorFunction
    } else if color.IsCurrentColor() {
        UnderlyingColorType::kCurrentColor
    } else {
        UnderlyingColorType::kColor
    }
}

fn ResolveColorOperand(
    color: &ColorOrUnresolvedColorFunction,
    color_type: UnderlyingColorType,
    current_color: &Color,
) -> Color {
    match color_type {
        UnderlyingColorType::kColor => color.color,
        UnderlyingColorType::kCurrentColor => *current_color,
        UnderlyingColorType::kColorFunction => {
            match unsafe { &*color.unresolved_color_function.Get() } {
                UnresolvedColorFunction::ColorMix(value) => value.Resolve(current_color),
                _ => panic!("unretained unresolved color function"),
            }
        }
    }
}

// cpp: Color::FromColorMix for the default Oklab interpolation family. CSS
// Color mixes premultiplied channels, then applies the normalization alpha.
// Production color operands are retained sRGB colors, so conversion can stay
// local to the native StyleColor owner without exposing a second CSS matcher.
fn MixInOklab(first: Color, second: Color, amount: f64, alpha_multiplier: f64) -> Color {
    fn linear(channel: f64) -> f64 {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    }
    fn gamma(channel: f64) -> f64 {
        if channel <= 0.0031308 {
            12.92 * channel
        } else {
            1.055 * channel.powf(1.0 / 2.4) - 0.055
        }
    }
    fn to_oklab(color: Color) -> [f64; 3] {
        let r = linear(color.Param0() as f64 / 255.0);
        let g = linear(color.Param1() as f64 / 255.0);
        let b = linear(color.Param2() as f64 / 255.0);
        let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
        let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
        let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
        [
            0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s,
            1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s,
            0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s,
        ]
    }
    fn from_oklab(value: [f64; 3], alpha: f64) -> Color {
        let l = value[0] + 0.396_337_777_4 * value[1] + 0.215_803_757_3 * value[2];
        let m = value[0] - 0.105_561_345_8 * value[1] - 0.063_854_172_8 * value[2];
        let s = value[0] - 0.089_484_177_5 * value[1] - 1.291_485_548 * value[2];
        let l = l * l * l;
        let m = m * m * m;
        let s = s * s * s;
        Color::FromRGBAFloat(
            gamma(4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s) as f32,
            gamma(-1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s) as f32,
            gamma(-0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701 * s) as f32,
            alpha as f32,
        )
    }

    let first_weight = 1.0 - amount;
    let second_weight = amount;
    let first_alpha = first.Alpha() as f64 * first_weight;
    let second_alpha = second.Alpha() as f64 * second_weight;
    let mixed_alpha = first_alpha + second_alpha;
    let first_lab = to_oklab(first);
    let second_lab = to_oklab(second);
    let mut mixed = [0.0; 3];
    if mixed_alpha > 0.0 {
        for channel in 0..3 {
            mixed[channel] = (first_lab[channel] * first_alpha
                + second_lab[channel] * second_alpha)
                / mixed_alpha;
        }
    }
    from_oklab(mixed, mixed_alpha * alpha_multiplier)
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
    /// Returns the originating system keyword even after it has acquired a
    /// resolved numeric color. Chromium's `ResolveSystemColor` reads the
    /// private `color_keyword_` member directly for this re-resolution case.
    pub fn GetSystemColorKeywordForResolution(&self) -> CSSValueID {
        debug_assert!(self.IsSystemColorIncludingDeprecated());
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
        if self.IsUnresolvedColorFunction() {
            return match self.GetUnresolvedColorFunction() {
                UnresolvedColorFunction::ColorMix(value) => value.Resolve(&current_color),
                _ => panic!("unretained unresolved color function"),
            };
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
        // cpp: style_color.cc:496-503. Keep the originating keyword beside
        // the resolved color so serialization and later invalidation retain
        // Chromium's system-color identity.
        assert!(self.IsSystemColor());
        Self::from_color_and_keyword(
            Self::ColorFromKeyword(
                self.color_keyword_,
                scheme,
                provider,
                can_expose_accent_color,
            ),
            self.color_keyword_,
        )
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
        // cpp: style_color.cc:567-570. The generated CSSValueID interval is
        // part of the native ABI and includes the deprecated system colors.
        (keyword >= CSSValueID::kActiveborder && keyword <= CSSValueID::kWindowtext)
            || keyword == CSSValueID::kMenu
    }
    pub fn IsSystemColorKeyword(keyword: CSSValueID) -> bool {
        // cpp: style_color.cc:572-605.
        matches!(
            keyword,
            CSSValueID::kAccentcolor
                | CSSValueID::kAccentcolortext
                | CSSValueID::kActivetext
                | CSSValueID::kButtonborder
                | CSSValueID::kButtonface
                | CSSValueID::kButtontext
                | CSSValueID::kCanvas
                | CSSValueID::kCanvastext
                | CSSValueID::kField
                | CSSValueID::kFieldtext
                | CSSValueID::kGraytext
                | CSSValueID::kHighlight
                | CSSValueID::kHighlighttext
                | CSSValueID::kInternalGrammarErrorColor
                | CSSValueID::kInternalSpellingErrorColor
                | CSSValueID::kInternalSearchColor
                | CSSValueID::kInternalSearchTextColor
                | CSSValueID::kInternalCurrentSearchColor
                | CSSValueID::kInternalCurrentSearchTextColor
                | CSSValueID::kLinktext
                | CSSValueID::kMark
                | CSSValueID::kMarktext
                | CSSValueID::kSelecteditem
                | CSSValueID::kSelecteditemtext
                | CSSValueID::kVisitedtext
        )
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
        if self.IsSystemColorIncludingDeprecated() {
            CSSValueID::kInvalid
        } else {
            self.color_keyword_
        }
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
