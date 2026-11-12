// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// menu_list_inner_element.cc:15-127; layout_theme_default.cc:67-127.
use super::{
    anonymous_style::CreateAnonymousStyleBuilderWithDisplay,
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    computed_style_constants::{ItemPosition, OverflowAlignment},
    style_self_alignment_data::StyleSelfAlignmentData,
};
use foundation::{
    EDisplay, EFieldSizing, EOverflow, EUserModify, IsHorizontalWritingMode, Length,
    LogicalToPhysicalSetter, Persistent,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// Alias of SelectType::option_style_, shared with the native select owner.
/// Style borrows this same property owner; it never reconstructs selection.
pub type MenuListOptionStyleOwner = Rc<RefCell<Persistent<ComputedStyle>>>;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MenuListNativeThemeMetrics {
    pub scrollbar_up_arrow_width_dip: i32,
    pub window_to_viewport_scale: f32,
}
#[derive(Default)]
pub struct MenuListPopupTheme {
    cached_arrow_zoom: Cell<f32>,
    cached_arrow_padding: Cell<f32>,
}
impl MenuListPopupTheme {
    // LayoutTheme base h:168-169; LayoutThemeMac h:50-51.
    pub const fn PopsMenuByArrowKeys() -> bool {
        cfg!(target_os = "macos")
    }
    pub const fn PopsMenuByReturnKey() -> bool {
        !cfg!(target_os = "macos")
    }
    fn InternalPadding(style: &ComputedStyle, padding: i32) -> i32 {
        if !style.HasEffectiveAppearance() {
            return 0;
        }
        (padding as f32 * style.EffectiveZoom()) as i32
    }
    fn ArrowPadding(&self, style: &ComputedStyle, metrics: MenuListNativeThemeMetrics) -> f32 {
        let zoom = style.EffectiveZoom();
        if self.cached_arrow_padding.get() > 0.0 && zoom == self.cached_arrow_zoom.get() {
            return self.cached_arrow_padding.get();
        }
        self.cached_arrow_zoom.set(zoom);
        let original = if metrics.scrollbar_up_arrow_width_dip > 0 {
            metrics.scrollbar_up_arrow_width_dip
        } else {
            15
        };
        let scaled = (original as f32 * metrics.window_to_viewport_scale) as i32;
        let device_scale = scaled as f32 / original as f32;
        let padding = if zoom < device_scale {
            scaled as f32
        } else {
            original as f32 * zoom
        };
        self.cached_arrow_padding.set(padding);
        padding
    }
    pub fn PopupInternalPaddingStart(&self, style: &ComputedStyle) -> i32 {
        Self::InternalPadding(style, 4)
    }
    pub fn PopupInternalPaddingEnd(
        &self,
        style: &ComputedStyle,
        metrics: MenuListNativeThemeMetrics,
    ) -> i32 {
        if !style.HasEffectiveAppearance() {
            return 0;
        }
        (style.EffectiveZoom() + self.ArrowPadding(style, metrics)) as i32
    }
    pub fn PopupInternalPaddingTop(&self, style: &ComputedStyle) -> i32 {
        Self::InternalPadding(style, 1)
    }
    pub fn PopupInternalPaddingBottom(&self, style: &ComputedStyle) -> i32 {
        Self::InternalPadding(style, 1)
    }
}
// computed_style.h:2595-2598 / form_control_sizing_service.cc:27-39.
// Both Layout's native metadata adapter and Style use this source predicate.
pub fn ApplyControlFixedSize(style: &ComputedStyle, is_autofilled: impl FnOnce() -> bool) -> bool {
    style.FieldSizing() == EFieldSizing::kFixed || is_autofilled()
}
/// The appearance:base branch returns the regular HTMLDiv style unchanged.
/// The caller retains that owner and invokes this only for the native class.
pub fn MenuListInnerCustomStyle(
    parent: &ComputedStyle,
    option: Option<&ComputedStyle>,
    is_autofilled: bool,
    primary_font_height: Option<f32>,
    remove_overflow_hidden: bool,
    theme: &MenuListPopupTheme,
    metrics: MenuListNativeThemeMetrics,
) -> *const ComputedStyle {
    let decorations = parent.AppliedTextDecorations() as *const _ as *mut _;
    let mut b = CreateAnonymousStyleBuilderWithDisplay(parent, EDisplay::kBlock, decorations);
    b.SetFlexGrow(1.0);
    b.SetFlexShrink(1.0);
    b.SetMinWidth(&Length::Fixed(0));
    if ApplyControlFixedSize(parent, || is_autofilled) {
        b.SetHasLineIfEmpty(true);
    }
    b.SetTextOverflow(parent.TextOverflow());
    if !remove_overflow_hidden {
        b.SetOverflowX(EOverflow::kHidden);
        b.SetOverflowY(EOverflow::kHidden);
        b.SetShouldIgnoreOverflowPropertyForInlineBlockBaseline();
    } else if IsHorizontalWritingMode(b.GetWritingMode()) {
        b.SetOverflowY(EOverflow::kVisible);
        b.SetOverflowX(EOverflow::kClip);
    } else {
        b.SetOverflowY(EOverflow::kClip);
        b.SetOverflowX(EOverflow::kVisible);
    }
    b.SetUserModify(EUserModify::kReadOnly);
    if b.HasInitialLineHeight() {
        let font_height = primary_font_height.unwrap_or(b.FontSize() as f32);
        b.SetLineHeight(&Length::Fixed(font_height));
    }
    if matches!(
        parent.AlignItems().GetPosition(),
        ItemPosition::kCenter | ItemPosition::kAnchorCenter
    ) {
        b.SetMarginTop(&Length::default());
        b.SetMarginBottom(&Length::default());
        b.SetAlignSelf(&StyleSelfAlignmentData::new_nonlegacy(
            ItemPosition::kStart,
            OverflowAlignment::kDefault,
        ));
    }
    let writing_direction = b.GetWritingDirection();
    {
        let mut margin = LogicalToPhysicalSetter::new(
            writing_direction,
            &mut b,
            |b: &mut ComputedStyleBuilder, value: Length| b.SetMarginTop(&value),
            |b, value| b.SetMarginRight(&value),
            |b, value| b.SetMarginBottom(&value),
            |b, value| b.SetMarginLeft(&value),
        );
        margin.SetInlineEnd(Length::Fixed(
            theme.PopupInternalPaddingEnd(parent, metrics),
        ));
        margin.SetInlineStart(Length::Fixed(theme.PopupInternalPaddingStart(parent)));
    }
    b.SetTextAlign(parent.GetTextAlignForLine(true));
    {
        let mut padding = LogicalToPhysicalSetter::new(
            writing_direction,
            &mut b,
            |b: &mut ComputedStyleBuilder, value: Length| b.SetPaddingTop(&value),
            |b, value| b.SetPaddingRight(&value),
            |b, value| b.SetPaddingBottom(&value),
            |b, value| b.SetPaddingLeft(&value),
        );
        padding.SetBlockStart(Length::Fixed(theme.PopupInternalPaddingTop(parent)));
        padding.SetBlockEnd(Length::Fixed(theme.PopupInternalPaddingBottom(parent)));
    }
    if let Some(option) = option {
        b.SetDirection(option.Direction());
        b.SetUnicodeBidi(option.GetUnicodeBidi());
    }
    b.TakeStyle()
}
