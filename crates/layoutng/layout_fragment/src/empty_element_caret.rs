//! Source: core/layout/layout_box_model_object.cc:870-1005.
//! Empty-element caret geometry reads the live native style/font/box owners.
use super::*;
use foundation::{ETextAlign, TextDirection, WritingDirectionMode};
use layoutng_geometry::geometry::logical_size::LogicalSize;

#[allow(non_snake_case)]
pub(super) fn LayoutBoxModelObjectLocalCaretRectForEmptyElement(
    owner: &LayoutBoxModelObject,
    inline_size: LayoutUnit,
    text_indent_offset: LayoutUnit,
    mut shape: CaretShape,
) -> LogicalRect {
    owner.CheckIsNotDestroyed();
    let font = unsafe { owner.StyleRef().GetFont().as_ref() };
    let font_data = font.and_then(|font| unsafe { font.PrimaryFont().as_ref() });
    let node = unsafe { owner.GetNode().as_ref() };
    if node.is_none_or(|node| !node.InputEditingFacts().is_editable) || font_data.is_none() {
        shape = CaretShape::kBar;
    }
    let style = owner.FirstLineStyleRef();
    let mut align = style.GetTextAlign();
    let mut reference_style = style;
    if align == ETextAlign::kMatchParent {
        if let Some(parent) = unsafe { owner.Parent().as_ref() } {
            reference_style = parent.StyleRef();
            align = reference_style.GetTextAlign();
        } else {
            align = ETextAlign::kStart;
        }
    }
    let right = match align {
        ETextAlign::kRight | ETextAlign::kWebkitRight => true,
        ETextAlign::kJustify | ETextAlign::kStart => !reference_style.IsLeftToRightDirection(),
        ETextAlign::kEnd => reference_style.IsLeftToRightDirection(),
        _ => false,
    };
    let line_height = style.ComputedLineHeightAsFixed();
    let bar_width = owner.BarCaretWidth();
    let bar_height = font_data.map_or(line_height, |data| {
        LayoutUnit::from_f64(f64::from(data.GetFontMetrics().Height()))
    });
    let size = match shape {
        CaretShape::kBar => LogicalSize::new(bar_width, bar_height),
        CaretShape::kBlock | CaretShape::kUnderscore => LogicalSize::new(
            LayoutUnit::from_f64(f64::from(font_data.unwrap().GetFontMetrics().ZeroWidth())),
            if shape == CaretShape::kBlock {
                bar_height
            } else {
                bar_width
            },
        ),
    };
    let border_padding = (owner.BorderOutsets() + owner.PaddingOutsets()).ConvertToLogical(
        WritingDirectionMode::new(style.GetWritingMode(), TextDirection::kLtr),
    );
    let max_x = inline_size - border_padding.inline_end;
    let mut x = border_padding.inline_start;
    if matches!(align, ETextAlign::kCenter | ETextAlign::kWebkitCenter) {
        x = (x + max_x) / 2;
        if style.IsLeftToRightDirection() {
            x += text_indent_offset / 2;
        } else {
            x -= text_indent_offset / 2;
        }
    } else if right {
        x = max_x - size.inline_size;
        if !style.IsLeftToRightDirection() {
            x -= text_indent_offset;
        }
    } else if style.IsLeftToRightDirection() {
        x += text_indent_offset;
    }
    x = x.min((max_x - size.inline_size).ClampNegativeToZero());
    let mut y = border_padding.block_start + (line_height - bar_height) / 2;
    if shape == CaretShape::kUnderscore {
        y += bar_height;
    }
    LogicalRect::from_units(x, y, size.inline_size, size.block_size)
}

impl LayoutObject {
    /// LocalFrameView::BarCaretWidth (cc:5286-5288), with the original host
    /// device scale carried through the document's existing global input owner.
    pub fn BarCaretWidth(&self) -> LayoutUnit {
        let mut current = Some(self);
        while let Some(object) = current {
            if let Some(node) = unsafe { object.NonPseudoNode().as_ref() } {
                return LayoutUnit::from_f64(node.InputDevicePixelRatio().max(1.0));
            }
            current = unsafe { object.Parent().as_ref() };
        }
        panic!("caret geometry requires its document viewport input owner")
    }
}
