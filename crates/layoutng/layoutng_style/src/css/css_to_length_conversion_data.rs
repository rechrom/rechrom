// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Borrowed CSS length environment. Font data remains on the canonical
//! ComputedStyle owner; token parsing and document/container selection remain
//! outside this pure conversion model.

use foundation::{Traceable, Visitor};

// cpp: core/css/css_to_length_conversion_data.h:223-263
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ContainerSizes {
    cached_width: Option<f64>,
    cached_height: Option<f64>,
}

#[allow(non_snake_case)]
impl ContainerSizes {
    pub fn PreCached(width: Option<f64>, height: Option<f64>) -> Self {
        Self {
            cached_width: width,
            cached_height: height,
        }
    }

    // cpp: core/css/css_to_length_conversion_data.cc:248-258
    pub fn PreCachedCopy(&self) -> Self {
        *self
    }

    // cpp: core/css/css_to_length_conversion_data.cc:264-267
    pub fn SizesEqual(&self, other: &Self) -> bool {
        self.Width() == other.Width() && self.Height() == other.Height()
    }

    // cpp: core/css/css_to_length_conversion_data.cc:269-278
    pub fn Width(&self) -> Option<f64> {
        self.cached_width
    }

    pub fn Height(&self) -> Option<f64> {
        self.cached_height
    }

    // cpp: core/css/css_to_length_conversion_data.cc:260-262
    pub fn Trace(&self, _: &mut Visitor) {}
}

impl Traceable for ContainerSizes {
    fn Trace(&self, visitor: &mut Visitor) {
        ContainerSizes::Trace(self, visitor);
    }
}

use super::{
    css_math_expression_node::{CSSMathLengthResolver, MathError},
    css_primitive_value::UnitType,
};
use crate::style::computed_style::ComputedStyle;

// cpp: css_to_length_conversion_data.cc:90-190. Borrow the source style/font;
// do not capture resolved metrics before the Layout font-miss retry gate.
#[derive(Clone, Copy)]
pub struct FontSizes<'a> {
    style: &'a ComputedStyle,
    root_style: &'a ComputedStyle,
}
#[allow(non_snake_case)]
impl<'a> FontSizes<'a> {
    pub fn new(style: &'a ComputedStyle, root_style: Option<&'a ComputedStyle>) -> Self {
        Self {
            style,
            root_style: root_style.unwrap_or(style),
        }
    }
    pub fn Em(&self, zoom: f32) -> f32 {
        self.style.SpecifiedFontSize() * zoom
    }
    pub fn Rem(&self, zoom: f32) -> f32 {
        self.root_style.SpecifiedFontSize() * zoom
    }
    fn ExFor(style: &ComputedStyle, zoom: f32) -> f32 {
        let font = unsafe { style.GetFont().as_ref() };
        let data = font.and_then(|font| unsafe { font.PrimaryFont().as_ref() });
        match data {
            Some(data) if data.GetFontMetrics().HasXHeight() => {
                data.GetFontMetrics().XHeight() / style.EffectiveZoom() * zoom
            }
            _ => style.SpecifiedFontSize() / 2.,
        }
    }
    pub fn Ex(&self, zoom: f32) -> f32 {
        Self::ExFor(self.style, zoom)
    }
    pub fn Rex(&self, zoom: f32) -> f32 {
        Self::ExFor(self.root_style, zoom)
    }
    fn ChFor(style: &ComputedStyle, zoom: f32) -> f32 {
        unsafe { style.GetFont().as_ref() }
            .and_then(|font| unsafe { font.PrimaryFontWithDigitZero().as_ref() })
            .map_or(0., |data| {
                data.GetFontMetrics().ZeroWidth() / style.EffectiveZoom() * zoom
            })
    }
    pub fn Ch(&self, zoom: f32) -> f32 {
        Self::ChFor(self.style, zoom)
    }
    pub fn Rch(&self, zoom: f32) -> f32 {
        Self::ChFor(self.root_style, zoom)
    }
    fn IcFor(style: &ComputedStyle, zoom: f32) -> f32 {
        unsafe { style.GetFont().as_ref() }
            .and_then(|font| unsafe { font.PrimaryFontWithCjkWater().as_ref() })
            .and_then(|data| *data.IdeographicInlineSize())
            .map_or_else(
                || style.SpecifiedFontSize() * zoom,
                |size| size / style.EffectiveZoom() * zoom,
            )
    }
    pub fn Ic(&self, zoom: f32) -> f32 {
        Self::IcFor(self.style, zoom)
    }
    pub fn Ric(&self, zoom: f32) -> f32 {
        Self::IcFor(self.root_style, zoom)
    }
    fn CapFor(style: &ComputedStyle, zoom: f32) -> f32 {
        unsafe { style.GetFont().as_ref() }
            .and_then(|font| unsafe { font.PrimaryFont().as_ref() })
            .map_or(0., |data| {
                data.GetFontMetrics().CapHeight() / style.EffectiveZoom() * zoom
            })
    }
    pub fn Cap(&self, zoom: f32) -> f32 {
        Self::CapFor(self.style, zoom)
    }
    pub fn Rcap(&self, zoom: f32) -> f32 {
        Self::CapFor(self.root_style, zoom)
    }
}

// cpp: css_to_length_conversion_data.cc:192-230. During style recalc `style`
// can be the parent's FontSizeStyle; SVG used values supply their own style.
#[derive(Clone, Copy)]
pub struct LineHeightSize<'a> {
    style: &'a ComputedStyle,
    root_style: &'a ComputedStyle,
}
#[allow(non_snake_case)]
impl<'a> LineHeightSize<'a> {
    pub fn new(style: &'a ComputedStyle, root_style: Option<&'a ComputedStyle>) -> Self {
        Self {
            style,
            root_style: root_style.unwrap_or(style),
        }
    }
    fn For(style: &ComputedStyle, zoom: f32) -> f32 {
        unsafe { style.GetFont().as_ref() }.map_or(0., |font| {
            ComputedStyle::ComputedLineHeightFromLength(style.LineHeight(), font)
                / style.EffectiveZoom()
                * zoom
        })
    }
    pub fn Lh(&self, zoom: f32) -> f32 {
        Self::For(self.style, zoom)
    }
    pub fn Rlh(&self, zoom: f32) -> f32 {
        Self::For(self.root_style, zoom)
    }
}

// cpp: css_to_length_conversion_data.h:163-220. Physical CSS viewport sizes;
// the embedding supplies LayoutView's small/large/dynamic values unchanged.
#[derive(Clone, Copy, Debug, Default)]
pub struct ViewportSize {
    pub small: [f64; 2],
    pub large: [f64; 2],
    pub dynamic: [f64; 2],
}

// cpp: css_to_length_conversion_data.h:296-341. Preserve source flag values
// so consumers can transfer precisely the dependencies observed by conversion.
#[repr(u32)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Flag {
    kEm = 1 << 0,
    kRootFontRelative = 1 << 1,
    kGlyphRelative = 1 << 2,
    kViewport = 1 << 3,
    kSmallLargeViewport = 1 << 4,
    kDynamicViewport = 1 << 5,
    kContainerRelative = 1 << 6,
    kTreeScopedReference = 1 << 7,
    kLogicalDirectionRelative = 1 << 8,
    kAnchorRelative = 1 << 9,
    kCapRelative = 1 << 10,
    kRcapRelative = 1 << 11,
    kIcRelative = 1 << 12,
    kRicRelative = 1 << 13,
    kLhRelative = 1 << 14,
    kRlhRelative = 1 << 15,
    kChRelative = 1 << 16,
    kRchRelative = 1 << 17,
    kRexRelative = 1 << 18,
    kSiblingRelative = 1 << 19,
    kElementDependentRandom = 1 << 20,
}
pub type Flags = u32;

pub struct CSSToLengthConversionData<'a> {
    font_sizes: FontSizes<'a>,
    line_height_size: LineHeightSize<'a>,
    viewport: ViewportSize,
    container: ContainerSizes,
    horizontal: bool,
    zoom: f32,
    flags: &'a mut Flags,
}
#[allow(non_snake_case)]
impl<'a> CSSToLengthConversionData<'a> {
    // cpp: css_to_length_conversion_data.cc:315-355. Container selection is
    // performed by its owning engine; the converter receives only its sizes.
    pub fn new(
        style: &'a ComputedStyle,
        parent_style: Option<&'a ComputedStyle>,
        root_style: Option<&'a ComputedStyle>,
        viewport: ViewportSize,
        container: ContainerSizes,
        zoom: f32,
        flags: &'a mut Flags,
    ) -> Self {
        Self {
            font_sizes: FontSizes::new(style, root_style),
            line_height_size: LineHeightSize::new(parent_style.unwrap_or(style), root_style),
            viewport,
            container,
            horizontal: style.IsHorizontalWritingMode(),
            zoom: zoom.max(f32::from_bits(1)),
            flags,
        }
    }
    pub fn SetFlag(&mut self, flag: Flag) {
        *self.flags |= flag as u32;
    }
    pub fn Zoom(&self) -> f32 {
        self.zoom
    }
    pub fn ReferenceTreeScope(&mut self) {
        self.SetFlag(Flag::kTreeScopedReference);
    }
    pub fn ReferenceAnchor(&mut self) {
        self.SetFlag(Flag::kAnchorRelative);
    }
    pub fn ReferenceSibling(&mut self) {
        self.SetFlag(Flag::kSiblingRelative);
    }
    pub fn ReferenceElementDependentRandom(&mut self) {
        self.SetFlag(Flag::kElementDependentRandom);
    }
    pub fn PreCachedContainerSizesCopy(&mut self) -> ContainerSizes {
        self.SetFlag(Flag::kContainerRelative);
        self.container.PreCachedCopy()
    }
    fn Logical(&mut self, size: [f64; 2], inline: bool) -> f64 {
        self.SetFlag(Flag::kLogicalDirectionRelative);
        size[usize::from(self.horizontal != inline)]
    }
    fn ContainerDimension(&mut self, axis: usize) -> f64 {
        self.SetFlag(Flag::kContainerRelative);
        let value = if axis == 0 {
            self.container.Width()
        } else {
            self.container.Height()
        };
        value.unwrap_or_else(|| {
            self.SetFlag(Flag::kSmallLargeViewport);
            self.viewport.small[axis]
        })
    }
    // cpp: css_length_resolver.cc:15-342; css_to_length_conversion_data.cc:390-565.
    pub fn ZoomedComputedPixels(&mut self, value: f64, unit: UnitType) -> Result<f64, MathError> {
        use UnitType::*;
        let zoom = self.zoom;
        let font = match unit {
            kEms | kQuirkyEms => {
                self.SetFlag(Flag::kEm);
                Some(self.font_sizes.Em(zoom))
            }
            kRems => {
                self.SetFlag(Flag::kRootFontRelative);
                Some(self.font_sizes.Rem(zoom))
            }
            kExs => {
                self.SetFlag(Flag::kGlyphRelative);
                Some(self.font_sizes.Ex(zoom))
            }
            kRexs => {
                self.SetFlag(Flag::kRexRelative);
                self.SetFlag(Flag::kGlyphRelative);
                self.SetFlag(Flag::kRootFontRelative);
                Some(self.font_sizes.Rex(zoom))
            }
            kChs | kRchs => {
                self.SetFlag(Flag::kGlyphRelative);
                if unit == kChs {
                    self.SetFlag(Flag::kChRelative);
                    Some(self.font_sizes.Ch(zoom))
                } else {
                    self.SetFlag(Flag::kRchRelative);
                    self.SetFlag(Flag::kRootFontRelative);
                    Some(self.font_sizes.Rch(zoom))
                }
            }
            kIcs | kRics => {
                self.SetFlag(Flag::kGlyphRelative);
                if unit == kIcs {
                    self.SetFlag(Flag::kIcRelative);
                    Some(self.font_sizes.Ic(zoom))
                } else {
                    self.SetFlag(Flag::kRicRelative);
                    self.SetFlag(Flag::kRootFontRelative);
                    Some(self.font_sizes.Ric(zoom))
                }
            }
            kCaps | kRcaps => {
                self.SetFlag(Flag::kGlyphRelative);
                if unit == kCaps {
                    self.SetFlag(Flag::kCapRelative);
                    Some(self.font_sizes.Cap(zoom))
                } else {
                    self.SetFlag(Flag::kRcapRelative);
                    self.SetFlag(Flag::kRootFontRelative);
                    Some(self.font_sizes.Rcap(zoom))
                }
            }
            kLhs | kRlhs => {
                self.SetFlag(Flag::kGlyphRelative);
                if unit == kLhs {
                    self.SetFlag(Flag::kLhRelative);
                    Some(self.line_height_size.Lh(zoom))
                } else {
                    self.SetFlag(Flag::kRlhRelative);
                    self.SetFlag(Flag::kRootFontRelative);
                    Some(self.line_height_size.Rlh(zoom))
                }
            }
            _ => None,
        };
        if let Some(size) = font {
            return Ok(value * size as f64);
        }
        let absolute = match unit {
            kPixels | kUserUnits => Some(1.),
            kCentimeters => Some(96. / 2.54),
            kMillimeters => Some(96. / 25.4),
            kQuarterMillimeters => Some(96. / 101.6),
            kInches => Some(96.),
            kPoints => Some(96. / 72.),
            kPicas => Some(16.),
            _ => None,
        };
        if let Some(scale) = absolute {
            return Ok(value * scale * zoom as f64);
        }
        let size = match unit {
            kViewportWidth | kViewportHeight | kViewportInlineSize | kViewportBlockSize
            | kViewportMin | kViewportMax => {
                self.SetFlag(Flag::kViewport);
                self.viewport.large
            }
            kSmallViewportWidth
            | kSmallViewportHeight
            | kSmallViewportInlineSize
            | kSmallViewportBlockSize
            | kSmallViewportMin
            | kSmallViewportMax => {
                self.SetFlag(Flag::kSmallLargeViewport);
                self.viewport.small
            }
            kLargeViewportWidth
            | kLargeViewportHeight
            | kLargeViewportInlineSize
            | kLargeViewportBlockSize
            | kLargeViewportMin
            | kLargeViewportMax => {
                self.SetFlag(Flag::kSmallLargeViewport);
                self.viewport.large
            }
            kDynamicViewportWidth
            | kDynamicViewportHeight
            | kDynamicViewportInlineSize
            | kDynamicViewportBlockSize
            | kDynamicViewportMin
            | kDynamicViewportMax => {
                self.SetFlag(Flag::kDynamicViewport);
                self.viewport.dynamic
            }
            kContainerWidth => [self.ContainerDimension(0), 0.],
            kContainerHeight => [0., self.ContainerDimension(1)],
            kContainerInlineSize => {
                let axis = usize::from(!self.horizontal);
                let mut size = [0.; 2];
                size[axis] = self.ContainerDimension(axis);
                size
            }
            kContainerBlockSize => {
                let axis = usize::from(self.horizontal);
                let mut size = [0.; 2];
                size[axis] = self.ContainerDimension(axis);
                size
            }
            kContainerMin | kContainerMax => {
                [self.ContainerDimension(0), self.ContainerDimension(1)]
            }
            _ => return Err(MathError::Invalid),
        };
        let dimension = match unit {
            kViewportWidth
            | kSmallViewportWidth
            | kLargeViewportWidth
            | kDynamicViewportWidth
            | kContainerWidth => size[0],
            kViewportHeight
            | kSmallViewportHeight
            | kLargeViewportHeight
            | kDynamicViewportHeight
            | kContainerHeight => size[1],
            kViewportInlineSize
            | kSmallViewportInlineSize
            | kLargeViewportInlineSize
            | kDynamicViewportInlineSize
            | kContainerInlineSize => self.Logical(size, true),
            kViewportBlockSize
            | kSmallViewportBlockSize
            | kLargeViewportBlockSize
            | kDynamicViewportBlockSize
            | kContainerBlockSize => self.Logical(size, false),
            kViewportMin | kSmallViewportMin | kLargeViewportMin | kDynamicViewportMin
            | kContainerMin => size[0].min(size[1]),
            kViewportMax | kSmallViewportMax | kLargeViewportMax | kDynamicViewportMax
            | kContainerMax => size[0].max(size[1]),
            _ => unreachable!(),
        };
        Ok(value * (dimension / 100.) * zoom as f64)
    }
}
impl CSSMathLengthResolver for CSSToLengthConversionData<'_> {
    fn ComputeLength(&mut self, value: f64, unit: UnitType) -> Result<f64, MathError> {
        self.ZoomedComputedPixels(value, unit)
    }
}
