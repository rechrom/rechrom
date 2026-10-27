#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::{
    BorderLineStyle, Offset, PaintPathCommand, PaintPathVerb, TextDirection, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_style::style::appearance::AppearanceValue;

use crate::border_shape_utils::UniformCornerRadii;
use crate::geometry_mapper::MapRectToRoot;
use crate::paint_context::PaintContext;
use crate::paint_engine::{DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/theme_painter.cc:12-18
fn Mix(a: Color, b: Color, amount: f32) -> Color {
    let amount = amount.clamp(0.0, 1.0);
    Color {
        red: a.red + (b.red - a.red) * amount,
        green: a.green + (b.green - a.green) * amount,
        blue: a.blue + (b.blue - a.blue) * amount,
        alpha: a.alpha + (b.alpha - a.alpha) * amount,
    }
}

// cpp: paint/theme_painter.cc:20-26
fn Inset(mut rect: PaintRect, amount: f64) -> PaintRect {
    rect.x += amount;
    rect.y += amount;
    rect.width = (rect.width - 2.0 * amount).max(0.0);
    rect.height = (rect.height - 2.0 * amount).max(0.0);
    rect
}

// cpp: paint/theme_painter.cc:28-37
fn ControlBox(node: &PaintTreeNode<'_>) -> PaintRect {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    let left = (node.paint_offset.x + 0.5).floor();
    let top = (node.paint_offset.y + 0.5).floor();
    let right = (node.paint_offset.x + fragment.size.width + 0.5).floor();
    let bottom = (node.paint_offset.y + fragment.size.height + 0.5).floor();
    PaintRect {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

// cpp: paint/theme_painter.cc:39-161
struct ControlCanvas<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
    phase: PaintPhase,
}

impl<'n, 'f, 'c, 'o> ControlCanvas<'n, 'f, 'c, 'o> {
    // cpp: paint/theme_painter.cc:41-56
    fn new(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        phase: PaintPhase,
    ) -> Self {
        let canvas = Self {
            node,
            context,
            phase,
        };
        let box_rect = ControlBox(canvas.node);
        canvas.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase,
                node_id: canvas.node_id(),
                ..Default::default()
            },
            canvas.node,
        );
        canvas.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kClipRect,
                phase,
                node_id: canvas.node_id(),
                rect: box_rect,
                antialias: false,
                ..Default::default()
            },
            canvas.node,
        );
        canvas
    }

    fn node_id(&self) -> u64 {
        self.node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment")
            .node_id
    }

    // cpp: paint/theme_painter.cc:65-82
    fn Fill(&self, rect: PaintRect, color: Color, radius: f64, antialias: bool) {
        if rect.width <= 0.0 || rect.height <= 0.0 || color.alpha <= 0.0 {
            return;
        }
        let radii = UniformCornerRadii(radius.max(0.0));
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: if radius > 0.0 {
                    DisplayItemType::kDrawRoundedRect
                } else {
                    DisplayItemType::kDrawRect
                },
                phase: self.phase,
                node_id: self.node_id(),
                rect,
                color,
                corner_radius: radius.max(0.0),
                corner_radii: radii,
                antialias,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/theme_painter.cc:84-105
    fn Stroke(&self, rect: PaintRect, color: Color, width: f64, radius: f64, antialias: bool) {
        if rect.width <= 0.0 || rect.height <= 0.0 || width <= 0.0 || color.alpha <= 0.0 {
            return;
        }
        let radii = UniformCornerRadii(radius.max(0.0));
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kStrokeRect,
                phase: self.phase,
                node_id: self.node_id(),
                rect,
                color,
                line_style: BorderLineStyle::kSolid,
                stroke_width: width,
                corner_radius: radius.max(0.0),
                corner_radii: radii,
                antialias,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/theme_painter.cc:107-114
    fn FillEllipse(&self, rect: PaintRect, color: Color) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kDrawEllipse,
                phase: self.phase,
                node_id: self.node_id(),
                rect,
                color,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/theme_painter.cc:116-124
    fn StrokeEllipse(&self, rect: PaintRect, color: Color, width: f64) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kStrokeEllipse,
                phase: self.phase,
                node_id: self.node_id(),
                rect,
                color,
                stroke_width: width,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/theme_painter.cc:126-142
    fn Line(&self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, width: f64, round: bool) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kStrokeLine,
                phase: self.phase,
                node_id: self.node_id(),
                rect: PaintRect {
                    x: x1,
                    y: y1,
                    width: x2,
                    height: y2,
                },
                color,
                line_style: BorderLineStyle::kSolid,
                stroke_width: width,
                round_cap: round,
                ..Default::default()
            },
            self.node,
        );
    }

    // cpp: paint/theme_painter.cc:144-155
    fn StrokePath(&self, path: Vec<PaintPathCommand>, color: Color, width: f64) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kStrokePath,
                phase: self.phase,
                node_id: self.node_id(),
                color,
                line_style: BorderLineStyle::kSolid,
                stroke_width: width,
                path,
                ..Default::default()
            },
            self.node,
        );
    }
}

// cpp: paint/theme_painter.cc:58-63
impl Drop for ControlCanvas<'_, '_, '_, '_> {
    fn drop(&mut self) {
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: self.phase,
                node_id: self.node_id(),
                ..Default::default()
            },
            self.node,
        );
    }
}

// cpp: paint/theme_painter.cc:163-167
fn IsVertical(appearance: AppearanceValue, writing_mode: WritingMode) -> bool {
    appearance == AppearanceValue::kSliderVertical
        || appearance == AppearanceValue::kSliderThumbVertical
        || writing_mode != WritingMode::kHorizontalTb
}

// cpp: paint/theme_painter.cc:169-172
fn Intersects(a: &PaintRect, b: &PaintRect) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}

// cpp: paint/theme_painter.cc:174-179
fn IsVisible(node: &PaintTreeNode<'_>, rect: &PaintRect) -> bool {
    let Some(cull_rect) = node.cull_rect else {
        return true;
    };
    let mapped = MapRectToRoot(*rect, &node.transforms);
    mapped.is_none_or(|mapped| Intersects(&mapped, &cull_rect))
}

// cpp: paint/theme_painter.h:9-21
pub struct ThemePainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
}

impl<'n, 'f, 'c, 'o> ThemePainter<'n, 'f, 'c, 'o> {
    // cpp: paint/theme_painter.h:11-12
    pub fn new(node: &'n PaintTreeNode<'f>, context: &'c RefCell<PaintContext<'o>>) -> Self {
        Self { node, context }
    }

    // cpp: paint/theme_painter.h:16
    // cpp: paint/theme_painter.cc:183-455
    pub fn Paint(&self, paint_info: &PaintInfo<'c, 'o>) -> bool {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let Some(control) = fragment.paint.form_control.as_ref() else {
            return false;
        };
        if matches!(
            control.appearance,
            AppearanceValue::kNone | AppearanceValue::kAuto
        ) {
            return false;
        }
        let box_rect = ControlBox(self.node);
        if box_rect.width <= 0.0 || box_rect.height <= 0.0 || !IsVisible(self.node, &box_rect) {
            return true;
        }
        let transparent = Color::default();
        let white = Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        };
        let black = Color {
            red: 0.12,
            green: 0.13,
            blue: 0.15,
            alpha: 1.0,
        };
        let border = Color {
            red: 0.36,
            green: 0.39,
            blue: 0.43,
            alpha: 1.0,
        };
        let disabled_border = Color {
            red: 0.67,
            green: 0.68,
            blue: 0.7,
            alpha: 1.0,
        };
        let disabled_fill = Color {
            red: 0.93,
            green: 0.93,
            blue: 0.94,
            alpha: 1.0,
        };
        let autofill = Color {
            red: 1.0,
            green: 0.95,
            blue: 0.72,
            alpha: 1.0,
        };
        let mut accent = control.accent_color;
        let mut face = Color {
            red: 0.95,
            green: 0.95,
            blue: 0.96,
            alpha: 1.0,
        };
        if control.hovered {
            face = Mix(face, white, 0.65);
        }
        if control.active {
            face = Mix(face, black, 0.12);
        }
        if control.disabled {
            face = disabled_fill;
            accent = Mix(accent, disabled_fill, 0.62);
        }
        let edge = if control.disabled {
            disabled_border
        } else {
            border
        };
        let minimum = box_rect.width.min(box_rect.height);
        let one = (minimum / 10.0).min(2.0).max(1.0);
        let canvas = ControlCanvas::new(self.node, self.context, paint_info.phase);
        let paint_frame = |fill: Color, radius: f64| {
            canvas.Fill(box_rect, edge, radius, true);
            canvas.Fill(Inset(box_rect, one), fill, (radius - one).max(0.0), true);
        };
        let paint_focus = || {
            if !control.focused {
                return;
            }
            let focus = PaintRect {
                x: box_rect.x - 2.0,
                y: box_rect.y - 2.0,
                width: box_rect.width + 4.0,
                height: box_rect.height + 4.0,
            };
            canvas.Fill(
                focus,
                Mix(accent, transparent, 0.22),
                focus.width.min(focus.height) * 0.22,
                true,
            );
        };
        match control.appearance {
            AppearanceValue::kCheckbox => {
                paint_focus();
                const RADIUS: f64 = 2.0;
                const BORDER_WIDTH: f64 = 1.0;
                let checkbox_background = if control.disabled {
                    Color {
                        red: 1.0,
                        green: 1.0,
                        blue: 1.0,
                        alpha: 153.0 / 255.0,
                    }
                } else {
                    white
                };
                if control.disabled {
                    canvas.Fill(
                        Inset(box_rect, BORDER_WIDTH * 0.2),
                        Color {
                            red: 169.0 / 255.0,
                            green: 169.0 / 255.0,
                            blue: 169.0 / 255.0,
                            alpha: 51.0 / 255.0,
                        },
                        RADIUS,
                        true,
                    );
                }
                canvas.Fill(
                    Inset(box_rect, BORDER_WIDTH * 0.2),
                    checkbox_background,
                    RADIUS,
                    true,
                );
                if control.checked || control.indeterminate {
                    canvas.Fill(box_rect, accent, RADIUS, true);
                } else {
                    let checkbox_border = if control.disabled {
                        Color {
                            red: 118.0 / 255.0,
                            green: 118.0 / 255.0,
                            blue: 118.0 / 255.0,
                            alpha: 77.0 / 255.0,
                        }
                    } else if control.hovered {
                        Color {
                            red: 79.0 / 255.0,
                            green: 79.0 / 255.0,
                            blue: 79.0 / 255.0,
                            alpha: 1.0,
                        }
                    } else {
                        Color {
                            red: 118.0 / 255.0,
                            green: 118.0 / 255.0,
                            blue: 118.0 / 255.0,
                            alpha: 1.0,
                        }
                    };
                    canvas.Stroke(
                        Inset(box_rect, BORDER_WIDTH * 0.5),
                        checkbox_border,
                        BORDER_WIDTH,
                        RADIUS,
                        true,
                    );
                }
                if control.indeterminate {
                    let y = box_rect.y + box_rect.height * 0.5;
                    canvas.Line(
                        box_rect.x + box_rect.width * 0.24,
                        y,
                        box_rect.x + box_rect.width * 0.76,
                        y,
                        white,
                        (minimum * 0.14).max(1.5),
                        true,
                    );
                } else if control.checked {
                    canvas.StrokePath(
                        vec![
                            PaintPathCommand {
                                verb: PaintPathVerb::kMoveTo,
                                point: Offset {
                                    x: box_rect.x + box_rect.width * 0.2,
                                    y: box_rect.y + box_rect.height * 0.5,
                                },
                                ..Default::default()
                            },
                            PaintPathCommand {
                                verb: PaintPathVerb::kLineTo,
                                point: Offset {
                                    x: box_rect.x + box_rect.width * 0.4,
                                    y: box_rect.y + box_rect.height * 0.7,
                                },
                                ..Default::default()
                            },
                            PaintPathCommand {
                                verb: PaintPathVerb::kLineTo,
                                point: Offset {
                                    x: box_rect.x + box_rect.width * 0.8,
                                    y: box_rect.y + box_rect.height * 0.2,
                                },
                                ..Default::default()
                            },
                        ],
                        white,
                        box_rect.height * 0.16,
                    );
                }
                true
            }
            AppearanceValue::kRadio => {
                paint_focus();
                canvas.FillEllipse(
                    Inset(box_rect, 0.2),
                    if control.disabled {
                        Color {
                            red: 248.0 / 255.0,
                            green: 248.0 / 255.0,
                            blue: 248.0 / 255.0,
                            alpha: 1.0,
                        }
                    } else {
                        white
                    },
                );
                let radio_border = if control.checked {
                    accent
                } else if control.disabled {
                    Color {
                        red: 209.0 / 255.0,
                        green: 209.0 / 255.0,
                        blue: 209.0 / 255.0,
                        alpha: 1.0,
                    }
                } else {
                    Color {
                        red: 118.0 / 255.0,
                        green: 118.0 / 255.0,
                        blue: 118.0 / 255.0,
                        alpha: 1.0,
                    }
                };
                canvas.StrokeEllipse(Inset(box_rect, 0.5), radio_border, 1.0);
                if control.checked {
                    let dot = Inset(box_rect, minimum * 0.2);
                    canvas.FillEllipse(dot, accent);
                }
                true
            }
            AppearanceValue::kButton
            | AppearanceValue::kPushButton
            | AppearanceValue::kSquareButton => {
                paint_focus();
                paint_frame(
                    face,
                    if control.appearance == AppearanceValue::kSquareButton {
                        0.0
                    } else {
                        (minimum * 0.25).min(5.0)
                    },
                );
                true
            }
            AppearanceValue::kTextField
            | AppearanceValue::kTextArea
            | AppearanceValue::kSearchField
            | AppearanceValue::kListbox => {
                paint_focus();
                let bounds = Inset(box_rect, 0.5);
                canvas.Fill(
                    bounds,
                    if control.autofilled { autofill } else { white },
                    2.0,
                    false,
                );
                canvas.Stroke(
                    bounds,
                    if control.disabled {
                        disabled_border
                    } else {
                        Color {
                            red: 118.0 / 255.0,
                            green: 118.0 / 255.0,
                            blue: 118.0 / 255.0,
                            alpha: 1.0,
                        }
                    },
                    1.0,
                    2.0,
                    false,
                );
                true
            }
            AppearanceValue::kMenulist
            | AppearanceValue::kMenulistButton
            | AppearanceValue::kBaseSelect => {
                paint_focus();
                paint_frame(face, (minimum * 0.18).min(4.0));
                let vertical = fragment.paint.writing_mode != WritingMode::kHorizontalTb;
                let reverse = fragment.paint.direction == TextDirection::kRtl;
                let length = (minimum * 0.18).max(3.0);
                let width = (minimum * 0.07).max(1.0);
                if !vertical {
                    let cx = if reverse {
                        box_rect.x + minimum * 0.55
                    } else {
                        box_rect.x + box_rect.width - minimum * 0.55
                    };
                    let cy = box_rect.y + box_rect.height * 0.48;
                    canvas.Line(
                        cx - length,
                        cy - length * 0.45,
                        cx,
                        cy + length * 0.45,
                        black,
                        width,
                        true,
                    );
                    canvas.Line(
                        cx,
                        cy + length * 0.45,
                        cx + length,
                        cy - length * 0.45,
                        black,
                        width,
                        true,
                    );
                } else {
                    let cx = box_rect.x + box_rect.width * 0.5;
                    let cy = if reverse {
                        box_rect.y + minimum * 0.55
                    } else {
                        box_rect.y + box_rect.height - minimum * 0.55
                    };
                    canvas.Line(
                        cx - length * 0.45,
                        cy - length,
                        cx + length * 0.45,
                        cy,
                        black,
                        width,
                        true,
                    );
                    canvas.Line(
                        cx + length * 0.45,
                        cy,
                        cx - length * 0.45,
                        cy + length,
                        black,
                        width,
                        true,
                    );
                }
                true
            }
            AppearanceValue::kSliderHorizontal | AppearanceValue::kSliderVertical => {
                let vertical = IsVertical(control.appearance, fragment.paint.writing_mode);
                let thickness = (minimum * 0.24).max(3.0);
                let track = if vertical {
                    PaintRect {
                        x: box_rect.x + (box_rect.width - thickness) * 0.5,
                        y: box_rect.y,
                        width: thickness,
                        height: box_rect.height,
                    }
                } else {
                    PaintRect {
                        x: box_rect.x,
                        y: box_rect.y + (box_rect.height - thickness) * 0.5,
                        width: box_rect.width,
                        height: thickness,
                    }
                };
                canvas.Fill(track, disabled_fill, thickness * 0.5, true);
                true
            }
            AppearanceValue::kSliderThumbHorizontal
            | AppearanceValue::kSliderThumbVertical
            | AppearanceValue::kMediaSliderThumb
            | AppearanceValue::kMediaVolumeSliderThumb => {
                paint_focus();
                canvas.Fill(box_rect, edge, minimum * 0.5, true);
                canvas.Fill(
                    Inset(box_rect, one),
                    if control.active { accent } else { face },
                    (minimum * 0.5 - one).max(0.0),
                    true,
                );
                true
            }
            AppearanceValue::kInnerSpinButton => {
                paint_frame(face, 2.0);
                let vertical = fragment.paint.writing_mode != WritingMode::kHorizontalTb;
                if !vertical {
                    let mid = box_rect.y + box_rect.height * 0.5;
                    canvas.Line(
                        box_rect.x + one,
                        mid,
                        box_rect.x + box_rect.width - one,
                        mid,
                        edge,
                        one,
                        false,
                    );
                    let cx = box_rect.x + box_rect.width * 0.5;
                    canvas.Line(cx - 2.0, mid - 2.0, cx, mid - 4.0, black, one, true);
                    canvas.Line(cx, mid - 4.0, cx + 2.0, mid - 2.0, black, one, true);
                    canvas.Line(cx - 2.0, mid + 2.0, cx, mid + 4.0, black, one, true);
                    canvas.Line(cx, mid + 4.0, cx + 2.0, mid + 2.0, black, one, true);
                } else {
                    let mid = box_rect.x + box_rect.width * 0.5;
                    canvas.Line(
                        mid,
                        box_rect.y + one,
                        mid,
                        box_rect.y + box_rect.height - one,
                        edge,
                        one,
                        false,
                    );
                }
                true
            }
            AppearanceValue::kSearchFieldCancelButton => {
                let radius = minimum * 0.5;
                canvas.Fill(
                    box_rect,
                    if control.active {
                        edge
                    } else {
                        disabled_border
                    },
                    radius,
                    true,
                );
                canvas.Line(
                    box_rect.x + box_rect.width * 0.3,
                    box_rect.y + box_rect.height * 0.3,
                    box_rect.x + box_rect.width * 0.7,
                    box_rect.y + box_rect.height * 0.7,
                    white,
                    (minimum * 0.1).max(1.0),
                    true,
                );
                canvas.Line(
                    box_rect.x + box_rect.width * 0.7,
                    box_rect.y + box_rect.height * 0.3,
                    box_rect.x + box_rect.width * 0.3,
                    box_rect.y + box_rect.height * 0.7,
                    white,
                    (minimum * 0.1).max(1.0),
                    true,
                );
                true
            }
            AppearanceValue::kMediaSlider | AppearanceValue::kMediaVolumeSlider => {
                canvas.Fill(
                    PaintRect {
                        x: box_rect.x,
                        y: box_rect.y + box_rect.height * 0.4,
                        width: box_rect.width,
                        height: (box_rect.height * 0.2).max(2.0),
                    },
                    edge,
                    (box_rect.height * 0.1).max(1.0),
                    true,
                );
                true
            }
            AppearanceValue::kMeter | AppearanceValue::kProgressBar => {
                let radius = (minimum * 0.25).min(4.0);
                paint_frame(disabled_fill, radius);
                let ratio = control.value_ratio.unwrap_or(0.0).clamp(0.0, 1.0);
                let mut value = Inset(box_rect, one * 2.0);
                if fragment.paint.writing_mode == WritingMode::kHorizontalTb {
                    value.width *= ratio;
                    if fragment.paint.direction == TextDirection::kRtl {
                        value.x = box_rect.x + box_rect.width - one * 2.0 - value.width;
                    }
                } else {
                    value.height *= ratio;
                    value.y = box_rect.y + box_rect.height - one * 2.0 - value.height;
                }
                canvas.Fill(value, accent, (radius - one * 2.0).max(0.0), true);
                true
            }
            AppearanceValue::kNone
            | AppearanceValue::kAuto
            | AppearanceValue::kMediaControl
            | AppearanceValue::kBase => false,
        }
    }
}
