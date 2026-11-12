//! Text-decoration pass orchestration and retained display-item recording.

#![allow(non_snake_case)]

use std::cell::RefCell;

use foundation::{gfx, ETextDecorationSkipInk, FloatValueForLength, LayoutUnit, PhysicalRect};
use layoutng_assembly::internal::layout_input::PaintStyle;
use layoutng_assembly::internal::layout_input::{
    Offset, PaintShadow, Size, TextDecorationPaint, TextDecorationStyle, WritingMode,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::layout_theme::LayoutTheme;
use layoutng_assembly::internal::paint_input::SvgPaintComponent;
use layoutng_assembly::text_decoration_info::{
    CurrentSpellingGrammarLineStyle, CurrentSpellingGrammarThickness,
};
use prepaint::pre_paint_tree_walk::PaintTreeNode;

use crate::box_fragment_painter::{IsVisible, TextPaintOrigin};
use crate::decoration_line_painter::{
    Bounds as DecorationBounds, DecorationGeometry, DecorationLinePainter, StrokeStyle,
};
use crate::line_relative_rect::LineRelativeRect;
use crate::paint_auto_dark_mode::{ElementRole, InvertColorIfNeeded};
use crate::paint_context::PaintContext;
use crate::paint_engine::{DecorationPaintPass, DisplayItem, DisplayItemType, PaintPhase};
use crate::paint_shader_resolver::{PrivacyPreservingPaintServer, ResolvePaintShader};
use crate::PaintRect;

/// Paint-owned counterpart of Chromium's TextDecorationPainter. The caller
/// still owns text/glyph ordering; this object owns decoration pass selection,
/// geometry and display-item recording.
pub struct TextDecorationPainter<'n, 'f, 'c, 'o> {
    node: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
    paint_phase: PaintPhase,
    phase: Phase,
    step: Step,
    selection_rect: Option<PaintRect>,
    clip_rect: Option<PaintRect>,
    privacy_preserving: bool,
    svg_context_paints: Option<&'c crate::svg_context_paints::SvgContextPaints<'c>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Originating,
    Selection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step {
    Begin,
    Except,
    Only,
}

// Source: text_shadow_painter.cc:36-60. Transparent shadows do not create a
// filter branch; effective shadows are reversed to retain CSS paint order.
fn EffectiveTextShadows(style: &PaintStyle, text_clip: bool) -> Vec<PaintShadow> {
    if text_clip {
        return Vec::new();
    }
    let mut shadows = Vec::with_capacity(style.text_shadow_count());
    for index in (0..style.text_shadow_count()).rev() {
        let shadow = style.text_shadow(index);
        if shadow.color.alpha <= 0.0 {
            continue;
        }
        if shadow.inset
            || shadow.spread != 0.0
            || !shadow.offset.x.is_finite()
            || !shadow.offset.y.is_finite()
            || !shadow.blur_radius.is_finite()
            || shadow.blur_radius < 0.0
        {
            panic!("text decoration shadow geometry is invalid");
        }
        shadows.push(shadow);
    }
    shadows
}

impl<'n, 'f, 'c, 'o> TextDecorationPainter<'n, 'f, 'c, 'o> {
    // Source: third_party/blink/renderer/core/paint/text_decoration_painter.cc:19-37.
    pub fn new(
        node: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        paint_phase: PaintPhase,
        selection_rect: Option<PaintRect>,
        privacy_preserving: bool,
    ) -> Self {
        Self {
            node,
            context,
            paint_phase,
            phase: Phase::Originating,
            step: Step::Begin,
            selection_rect,
            clip_rect: None,
            privacy_preserving,
            svg_context_paints: None,
        }
    }

    pub fn SetSvgContextPaints(
        &mut self,
        paints: Option<&'c crate::svg_context_paints::SvgContextPaints<'c>>,
    ) {
        self.svg_context_paints = paints;
    }
    fn ResolveSvgContext(
        &self,
        style: &PaintStyle,
        stroke: bool,
    ) -> Option<crate::svg_context_paints::ResolvedContextPaint> {
        let info = crate::paint_info::PaintInfo::new(self.context, self.paint_phase);
        if let Some(paints) = self.svg_context_paints {
            crate::svg_context_paints::ResolveFor(style, stroke, &info.WithSvgContextPaints(paints))
        } else {
            None
        }
    }
    // Source: text_decoration_painter.cc:104-110.
    pub fn ExpandRectForSVGDecorations(rect: &LineRelativeRect) -> gfx::RectF {
        let clip_rect = rect.ToRectF();
        gfx::RectF::new(
            gfx::PointF::new(clip_rect.x(), clip_rect.y() - clip_rect.height()),
            gfx::SizeF::new(clip_rect.width(), 3.0 * clip_rect.height()),
        )
    }

    // Source: text_decoration_painter.cc:113-135.
    pub fn Begin(&mut self, phase: Phase) {
        debug_assert_eq!(self.step, Step::Begin);
        self.phase = phase;
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        self.clip_rect = self.selection_rect.map(|rect| {
            if fragment.paint.svg_text.is_some() {
                let relative = LineRelativeRect::EnclosingRect(&gfx::RectF::new(
                    gfx::PointF::new(rect.x as f32, rect.y as f32),
                    gfx::SizeF::new(rect.width as f32, rect.height as f32),
                ));
                let expanded = Self::ExpandRectForSVGDecorations(&relative);
                PaintRect {
                    x: f64::from(expanded.x()),
                    y: f64::from(expanded.y()),
                    width: f64::from(expanded.width()),
                    height: f64::from(expanded.height()),
                }
            } else {
                let ink_overflow = fragment
                    .paint
                    .ink_overflow
                    .expect("text fragment has source ink overflow");
                PaintRect {
                    x: rect.x,
                    y: self.node.paint_offset.y + ink_overflow.offset.y,
                    width: rect.width,
                    height: ink_overflow.size.height,
                }
            }
        });
        self.step = Step::Except;
    }

    // Source: text_decoration_painter.cc:256-270. The caller deliberately
    // paints glyphs between this method and PaintOnlyLineThrough.
    pub fn PaintExceptLineThrough(&mut self) {
        debug_assert_eq!(self.step, Step::Except);
        if self.HasAnyLine(false) {
            let clipped = self.ClipIfNeeded();
            self.Paint(false);
            self.RestoreClip(clipped);
        }
        self.step = Step::Only;
    }

    // Source: text_decoration_painter.cc:273-285.
    pub fn PaintOnlyLineThrough(&mut self) {
        debug_assert_eq!(self.step, Step::Only);
        if self.HasAnyLine(true) {
            let clipped = self.ClipIfNeeded();
            self.Paint(true);
            self.RestoreClip(clipped);
        }
        self.step = Step::Begin;
    }

    fn HasAnyLine(&self, line_through: bool) -> bool {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        if fragment.paint.is_ellipsis {
            return false;
        }
        let style = fragment
            .paint
            .svg_text_decoration_style
            .as_ref()
            .unwrap_or(&fragment.paint.style);
        let has_line = |decoration: &TextDecorationPaint| {
            if line_through {
                decoration.line_through
            } else {
                decoration.underline
                    || decoration.overline
                    || decoration.spelling_error
                    || decoration.grammar_error
            }
        };
        if style.applied_text_decoration_count() == 0 {
            has_line(&style.text_decoration)
        } else {
            (0..style.applied_text_decoration_count())
                .any(|index| has_line(&style.applied_text_decoration(index)))
        }
    }

    // Source: text_decoration_painter.cc:149-285. GraphicsContext state is
    // retained as DisplayItems; glyph ordering remains at the caller exactly
    // where Chromium's TextFragmentPainter invokes these two public passes.
    fn Paint(&self, line_through: bool) {
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let style = &fragment.paint.style;
        let text_clip = self.paint_phase == PaintPhase::kTextClip;
        let glyph_runs = self
            .context
            .borrow_mut()
            .TextGlyphRuns(fragment.fragment_instance_id);
        let metrics = glyph_runs
            .first()
            .map(|run| {
                let text_fit_scale = if fragment.paint.svg_text.is_some() {
                    1.0
                } else {
                    fragment.paint.text_fit_scale
                };
                (
                    run.font_size * text_fit_scale,
                    run.ascent * text_fit_scale,
                    run.baseline,
                    run.underline_thickness
                        .map(|thickness| thickness * text_fit_scale),
                    run.writing_mode,
                )
            })
            .or_else(|| {
                fragment.paint.text_combine.as_ref().map(|combine| {
                    (
                        combine.font_size,
                        combine.text_top_offset,
                        combine.text_top_offset,
                        None,
                        fragment.paint.writing_mode,
                    )
                })
            });
        let Some((font_size, ascent, baseline, from_font_thickness, writing_mode)) = metrics else {
            return;
        };
        let origin = TextPaintOrigin(self.node);
        let paint_size = TextPaintSize(self.node);
        let physical_decoration_rect = PhysicalRect::from_units(
            LayoutUnit::from_f64(origin.x),
            LayoutUnit::from_f64(origin.y),
            LayoutUnit::from_f64(paint_size.width),
            LayoutUnit::from_f64(paint_size.height),
        );
        let horizontal = writing_mode == WritingMode::kHorizontalTb;
        let decoration_rect =
            LineRelativeRect::CreateFromLineBox(&physical_decoration_rect, horizontal);
        let x = decoration_rect.LineLeft().ToDouble();
        let y = decoration_rect.LineOver().ToDouble();
        let inline_size = decoration_rect.InlineSize().ToDouble();
        let block_size = decoration_rect.BlockSize().ToDouble();
        let svg_decoration_style: Option<&PaintStyle> =
            fragment.paint.svg_text_decoration_style.as_ref();
        let paint_decoration =
            |decoration: &TextDecorationPaint, shadow_filters: Option<&[PaintShadow]>| {
                let spelling_or_grammar = decoration.spelling_error || decoration.grammar_error;
                if if line_through {
                    !decoration.line_through
                } else {
                    !decoration.underline && !decoration.overline && !spelling_or_grammar
                } {
                    return;
                }
                let originating_color = if spelling_or_grammar {
                    let marker = if decoration.spelling_error {
                        LayoutTheme::GetTheme().PlatformSpellingMarkerUnderlineColor()
                    } else {
                        LayoutTheme::GetTheme().PlatformGrammarMarkerUnderlineColor()
                    };
                    let (red, green, blue) = marker.ToSRGB();
                    Color {
                        red,
                        green,
                        blue,
                        alpha: marker.Alpha(),
                    }
                } else {
                    decoration.color.unwrap_or_else(|| style.color())
                };
                let phase_color = if self.phase == Phase::Selection {
                    if spelling_or_grammar {
                        originating_color
                    } else {
                        fragment
                            .paint
                            .selection_text_decoration
                            .as_ref()
                            .filter(|selection| SameDecorationLines(decoration, selection))
                            .and_then(|selection| selection.color)
                            .unwrap_or(originating_color)
                    }
                } else {
                    originating_color
                };
                let color = LineColorForPhase(phase_color, shadow_filters.is_some(), text_clip);
                let auto_thickness = font_size / 10.0;
                let used_thickness = if spelling_or_grammar {
                    f64::from(CurrentSpellingGrammarThickness(
                        fragment.paint.effective_zoom,
                    ))
                } else {
                    ResolveExportedDecorationThickness(
                        decoration,
                        font_size,
                        from_font_thickness,
                        fragment.paint.svg_text.is_some(),
                    )
                };
                let (resolved_style, marker_antialias, marker_wave) = if spelling_or_grammar {
                    let (style, antialias, wave) =
                        CurrentSpellingGrammarLineStyle(fragment.paint.effective_zoom);
                    let style = match style {
                        StrokeStyle::kSolidStroke => TextDecorationStyle::kSolid,
                        StrokeStyle::kDottedStroke => TextDecorationStyle::kDotted,
                        StrokeStyle::kWavyStroke => TextDecorationStyle::kWavy,
                        StrokeStyle::kDashedStroke | StrokeStyle::kDoubleStroke => {
                            unreachable!("spelling marker platform style")
                        }
                    };
                    (style, antialias, wave)
                } else {
                    (decoration.style, false, None)
                };
                let paint_line = |line: PaintRect,
                                  skip_ink: ETextDecorationSkipInk,
                                  double_offset: f64,
                                  wavy_offset: f64| {
                    let thickness = used_thickness;
                    if !decoration.underline_offset.is_finite() {
                        panic!("text decoration geometry is invalid");
                    }
                    let make_items = |item_color: Color,
                                      shadow_offset: Offset,
                                      blur_radius: f64,
                                      is_shadow: bool,
                                      paint_pass: DecorationPaintPass,
                                      svg_paint_style: Option<&PaintStyle>,
                                      use_shader: bool| {
                        let attach_ink_skip_bounds =
                            |mut items: Vec<DisplayItem>, geometry: &DecorationGeometry| {
                                if skip_ink != ETextDecorationSkipInk::kNone && horizontal {
                                    let mut bounds = DecorationBounds(geometry);
                                    // Wavy geometry is built at a local 0,0 and
                                    // DecorationLinePainter places its PaintRecord
                                    // at `line`; the other styles already retain
                                    // physical coordinates in their geometry.
                                    if geometry.style == StrokeStyle::kWavyStroke {
                                        bounds.Offset(line.x as f32, line.y as f32);
                                    }
                                    let bounds = PaintRect {
                                        x: f64::from(bounds.x()),
                                        y: f64::from(bounds.y()),
                                        width: f64::from(bounds.width()),
                                        height: f64::from(bounds.height()),
                                    };
                                    for item in &mut items {
                                        item.decoration_ink_bounds = Some(bounds);
                                    }
                                }
                                items
                            };
                        let mut item = DisplayItem {
                            r#type: if resolved_style == TextDecorationStyle::kWavy {
                                DisplayItemType::kStrokeWavyLine
                            } else {
                                DisplayItemType::kStrokeLine
                            },
                            phase: self.paint_phase,
                            node_id: fragment.node_id,
                            rect: line,
                            color: item_color,
                            decoration_style: resolved_style,
                            stroke_width: thickness,
                            decoration_thickness: thickness,
                            shadow_offset,
                            blur_radius,
                            is_shadow,
                            is_text_decoration: true,
                            skip_ink,
                            decoration_ink_baseline: y + baseline,
                            decoration_text_origin: origin,
                            decoration_text_fit_scale: if fragment.paint.svg_text.is_some() {
                                1.0
                            } else {
                                fragment.paint.text_fit_scale
                            },
                            decoration_ink_runs: if skip_ink != ETextDecorationSkipInk::kNone {
                                std::sync::Arc::clone(&glyph_runs)
                            } else {
                                Default::default()
                            },
                            decoration_paint_pass: paint_pass,
                            ..Default::default()
                        };
                        item.antialias |= marker_antialias;
                        if let Some(svg_style) = svg_paint_style {
                            item.antialias = true;
                            if paint_pass == DecorationPaintPass::SvgStroke {
                                item.stroke_width = svg_style.svg_stroke_width;
                                item.dash_intervals = svg_style.svg_stroke_dash_array.clone();
                                item.dash_offset = svg_style.svg_stroke_dash_offset;
                                item.svg_line_cap = svg_style.svg_stroke_line_cap;
                                item.svg_line_join = svg_style.svg_stroke_line_join;
                                item.miter_limit = svg_style.svg_stroke_miter_limit;
                            }
                            let context_paint = self.ResolveSvgContext(
                                svg_style,
                                paint_pass == DecorationPaintPass::SvgStroke,
                            );
                            let server = match paint_pass {
                                DecorationPaintPass::SvgFill => svg_style.svg_fill_server.as_ref(),
                                DecorationPaintPass::SvgStroke => {
                                    svg_style.svg_stroke_server.as_ref()
                                }
                                DecorationPaintPass::Html => None,
                            };
                            let server = context_paint
                                .as_ref()
                                .map_or(server, |paint| paint.server.as_ref());
                            if use_shader {
                                if let Some(server) = server {
                                    let shader = if let Some(context) = &context_paint {
                                        crate::svg_context_paints::ResolveShader(
                                            context,
                                            fragment.paint.resources.as_deref(),
                                            self.privacy_preserving,
                                        )
                                        .expect("admitted SVG decoration context server")
                                    } else {
                                        ResolvePaintShader(
                                            server,
                                            fragment.paint.resources.as_deref(),
                                            Offset {
                                                x: line.x,
                                                y: line.y,
                                            },
                                            Some(Size {
                                                width: line.width,
                                                height: thickness,
                                            }),
                                        )
                                    };
                                    if let Some(mut shader) = PrivacyPreservingPaintServer(
                                        &shader,
                                        self.privacy_preserving,
                                    ) {
                                        if svg_style.auto_dark_mode_paused() {
                                            shader.apply_dark_mode = false;
                                        }
                                        item.paint_shader = Some(shader);
                                    }
                                }
                            }
                        }
                        if item.paint_shader.is_none() {
                            item.color = InvertColorIfNeeded(
                                svg_paint_style.unwrap_or(style),
                                item.color,
                                ElementRole::kForeground,
                            );
                        }
                        if writing_mode == WritingMode::kHorizontalTb
                            && matches!(
                                resolved_style,
                                TextDecorationStyle::kDashed | TextDecorationStyle::kDotted
                            )
                        {
                            let stroke_style = match resolved_style {
                                TextDecorationStyle::kDashed => StrokeStyle::kDashedStroke,
                                TextDecorationStyle::kDotted => StrokeStyle::kDottedStroke,
                                _ => unreachable!(),
                            };
                            let mut geometry = DecorationGeometry::Make(
                                stroke_style,
                                &gfx::RectF::new(
                                    gfx::PointF::new(line.x as f32, line.y as f32),
                                    gfx::SizeF::new(line.width as f32, thickness as f32),
                                ),
                                0.0,
                                0.0,
                                None,
                            );
                            geometry.antialias = true;
                            return attach_ink_skip_bounds(
                                DecorationLinePainter::Paint(&geometry, line, item),
                                &geometry,
                            );
                        }
                        if resolved_style == TextDecorationStyle::kWavy {
                            let logical_width = if writing_mode == WritingMode::kHorizontalTb {
                                line.width
                            } else {
                                line.height
                            };
                            let mut geometry = DecorationGeometry::Make(
                                StrokeStyle::kWavyStroke,
                                &gfx::RectF::new(
                                    gfx::PointF::new(0.0, 0.0),
                                    gfx::SizeF::new(logical_width as f32, thickness as f32),
                                ),
                                double_offset as f32,
                                wavy_offset as f32,
                                marker_wave.as_ref(),
                            );
                            geometry.antialias = marker_antialias || !spelling_or_grammar;
                            return attach_ink_skip_bounds(
                                DecorationLinePainter::Paint(&geometry, line, item),
                                &geometry,
                            );
                        }
                        if writing_mode == WritingMode::kHorizontalTb
                            && matches!(
                                resolved_style,
                                TextDecorationStyle::kSolid | TextDecorationStyle::kDouble
                            )
                        {
                            let stroke_style = if resolved_style == TextDecorationStyle::kDouble {
                                StrokeStyle::kDoubleStroke
                            } else {
                                StrokeStyle::kSolidStroke
                            };
                            let geometry = DecorationGeometry::Make(
                                stroke_style,
                                &gfx::RectF::new(
                                    gfx::PointF::new(line.x as f32, line.y as f32),
                                    gfx::SizeF::new(line.width as f32, thickness as f32),
                                ),
                                double_offset as f32,
                                0.0,
                                None,
                            );
                            return attach_ink_skip_bounds(
                                DecorationLinePainter::Paint(&geometry, line, item),
                                &geometry,
                            );
                        }
                        vec![item]
                    };
                    let mut paint_passes = Vec::new();
                    if text_clip || svg_decoration_style.is_none() {
                        paint_passes.push((DecorationPaintPass::Html, color, None));
                    } else if let Some(svg_style) = svg_decoration_style {
                        let fill_context = self.ResolveSvgContext(svg_style, false);
                        let stroke_context = self.ResolveSvgContext(svg_style, true);
                        let fill = fill_context
                            .as_ref()
                            .map_or(svg_style.svg_fill, |paint| paint.color);
                        let stroke = stroke_context
                            .as_ref()
                            .map_or(svg_style.svg_stroke, |paint| paint.color);
                        let fill_server = fill_context
                            .as_ref()
                            .map_or(svg_style.svg_fill_server.as_ref(), |paint| {
                                paint.server.as_ref()
                            });
                        let stroke_server = stroke_context
                            .as_ref()
                            .map_or(svg_style.svg_stroke_server.as_ref(), |paint| {
                                paint.server.as_ref()
                            });
                        for component in svg_style.svg_paint_order {
                            if component == SvgPaintComponent::kFill
                                && (fill_server.is_some()
                                    || fill.is_some_and(|fill| fill.alpha > 0.0))
                                && !paint_passes
                                    .iter()
                                    .any(|pass| pass.0 == DecorationPaintPass::SvgFill)
                            {
                                paint_passes.push((
                                    DecorationPaintPass::SvgFill,
                                    fill.unwrap_or_default(),
                                    Some(svg_style),
                                ));
                            } else if component == SvgPaintComponent::kStroke
                                && svg_style.svg_stroke_width > 0.0
                                && (stroke_server.is_some()
                                    || stroke.is_some_and(|stroke| stroke.alpha > 0.0))
                                && !paint_passes
                                    .iter()
                                    .any(|pass| pass.0 == DecorationPaintPass::SvgStroke)
                            {
                                paint_passes.push((
                                    DecorationPaintPass::SvgStroke,
                                    stroke.unwrap_or_default(),
                                    Some(svg_style),
                                ));
                            }
                        }
                    }
                    if let Some(shadows) = shadow_filters {
                        let Some((paint_pass, _, svg_style)) = paint_passes.first().copied() else {
                            return;
                        };
                        for item in make_items(
                            LineColorForPhase(color, true, text_clip),
                            Offset::default(),
                            0.0,
                            false,
                            paint_pass,
                            svg_style,
                            false,
                        ) {
                            let visible = shadows.iter().any(|shadow| {
                                let mut bounds = PaintRect {
                                    x: item.rect.x + shadow.offset.x,
                                    y: item.rect.y + shadow.offset.y,
                                    width: item.rect.width,
                                    height: item.rect.height,
                                };
                                let outset = 1.5 * shadow.blur_radius + thickness;
                                bounds.x -= outset;
                                bounds.y -= outset;
                                bounds.width += 2.0 * outset;
                                bounds.height += 2.0 * outset;
                                IsVisible(self.node, &bounds)
                            });
                            if visible {
                                self.context.borrow_mut().Append(item, self.node);
                            }
                        }
                        return;
                    }
                    for (paint_pass, pass_color, svg_style) in paint_passes {
                        for item in make_items(
                            pass_color,
                            Offset::default(),
                            0.0,
                            false,
                            paint_pass,
                            svg_style,
                            true,
                        ) {
                            let mut bounds = item.rect;
                            bounds.x -= thickness;
                            bounds.y -= thickness;
                            bounds.width += 2.0 * thickness;
                            bounds.height += 2.0 * thickness;
                            if IsVisible(self.node, &bounds) {
                                self.context.borrow_mut().Append(item, self.node);
                            }
                        }
                    }
                };
                if line_through {
                    if horizontal {
                        paint_line(
                            PaintRect {
                                x,
                                y: y + 2.0 * ascent / 3.0 - used_thickness / 2.0,
                                width: inline_size,
                                height: 0.0,
                            },
                            ETextDecorationSkipInk::kNone,
                            (used_thickness + 1.0).floor(),
                            0.0,
                        );
                    } else {
                        paint_line(
                            PaintRect {
                                x: x + block_size / 2.0,
                                y,
                                width: 0.0,
                                height: inline_size,
                            },
                            ETextDecorationSkipInk::kNone,
                            (used_thickness + 1.0).floor(),
                            0.0,
                        );
                    }
                    return;
                }
                if spelling_or_grammar {
                    let baseline_offset = (used_thickness / 2.0).ceil().max(1.0);
                    let wavy_offset = used_thickness + 1.0;
                    if horizontal {
                        paint_line(
                            PaintRect {
                                x,
                                y: y + baseline + baseline_offset,
                                width: inline_size,
                                height: 0.0,
                            },
                            ETextDecorationSkipInk::kNone,
                            used_thickness + 1.0,
                            wavy_offset,
                        );
                    } else {
                        let direction = if writing_mode == WritingMode::kVerticalRl {
                            -1.0
                        } else {
                            1.0
                        };
                        paint_line(
                            PaintRect {
                                x: if direction < 0.0 { x } else { x + block_size },
                                y,
                                width: 0.0,
                                height: inline_size,
                            },
                            ETextDecorationSkipInk::kNone,
                            used_thickness + 1.0,
                            wavy_offset,
                        );
                    }
                    return;
                }
                if decoration.overline {
                    if horizontal {
                        paint_line(
                            PaintRect {
                                x,
                                y: y + auto_thickness / 2.0,
                                width: inline_size,
                                height: 0.0,
                            },
                            decoration.skip_ink,
                            -(used_thickness + 1.0),
                            -(used_thickness + 1.0),
                        );
                    } else {
                        let line_x = if writing_mode == WritingMode::kVerticalRl {
                            x + block_size
                        } else {
                            x
                        };
                        paint_line(
                            PaintRect {
                                x: line_x,
                                y,
                                width: 0.0,
                                height: inline_size,
                            },
                            decoration.skip_ink,
                            -(used_thickness + 1.0),
                            -(used_thickness + 1.0),
                        );
                    }
                }
                if decoration.underline {
                    if horizontal {
                        let baseline_offset = if decoration.underline_offset_auto {
                            (used_thickness / 2.0).ceil().max(1.0)
                        } else {
                            decoration.underline_offset.round()
                        };
                        let wavy_offset = used_thickness + 1.0;
                        paint_line(
                            PaintRect {
                                x,
                                y: y + baseline + baseline_offset,
                                width: inline_size,
                                height: 0.0,
                            },
                            decoration.skip_ink,
                            used_thickness + 1.0,
                            wavy_offset,
                        );
                    } else {
                        let direction = if writing_mode == WritingMode::kVerticalRl {
                            -1.0
                        } else {
                            1.0
                        };
                        let line_x = (if direction < 0.0 { x } else { x + block_size })
                            + direction * decoration.underline_offset;
                        paint_line(
                            PaintRect {
                                x: line_x,
                                y,
                                width: 0.0,
                                height: inline_size,
                            },
                            decoration.skip_ink,
                            used_thickness + 1.0,
                            used_thickness + 1.0,
                        );
                    }
                }
            };
        let decoration_style = svg_decoration_style.unwrap_or(style);
        let paint_all_decorations = |shadow| {
            if decoration_style.applied_text_decoration_count() == 0 {
                paint_decoration(&decoration_style.text_decoration, shadow);
            } else {
                for index in 0..decoration_style.applied_text_decoration_count() {
                    paint_decoration(&decoration_style.applied_text_decoration(index), shadow);
                }
            }
        };
        let shadow_style = if self.phase == Phase::Selection {
            &fragment.paint.selection_style
        } else {
            decoration_style
        };
        // text_decoration_painter.cc:162-216,228-253 and
        // text_shadow_painter.cc:36-74. Draw the complete black source batch
        // once into a filtered layer; the parallel shadow-only branches merge
        // its alpha before the ordinary foreground batch is recorded.
        let shadows = EffectiveTextShadows(shadow_style, text_clip);
        if !shadows.is_empty() {
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kSaveLayerFilter,
                    phase: self.paint_phase,
                    node_id: fragment.node_id,
                    text_shadow_filters: shadows.clone(),
                    ..Default::default()
                },
                self.node,
            );
            paint_all_decorations(Some(&shadows));
            self.context.borrow_mut().Append(
                DisplayItem {
                    r#type: DisplayItemType::kRestore,
                    phase: self.paint_phase,
                    node_id: fragment.node_id,
                    ..Default::default()
                },
                self.node,
            );
        }
        paint_all_decorations(None);
    }

    // Source: text_decoration_painter.cc:312-323. GraphicsContext save/clip is
    // represented by the package's retained state DisplayItems.
    fn ClipIfNeeded(&self) -> bool {
        let Some(rect) = self.clip_rect else {
            return false;
        };
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        let mut context = self.context.borrow_mut();
        context.Append(
            DisplayItem {
                r#type: DisplayItemType::kSave,
                phase: self.paint_phase,
                node_id: fragment.node_id,
                ..Default::default()
            },
            self.node,
        );
        context.Append(
            DisplayItem {
                r#type: if self.phase == Phase::Selection {
                    DisplayItemType::kClipRect
                } else {
                    DisplayItemType::kClipOutRect
                },
                phase: self.paint_phase,
                node_id: fragment.node_id,
                rect,
                ..Default::default()
            },
            self.node,
        );
        true
    }

    fn RestoreClip(&self, clipped: bool) {
        if !clipped {
            return;
        }
        let fragment = self
            .node
            .fragment
            .as_deref()
            .expect("paint tree node has a fragment");
        self.context.borrow_mut().Append(
            DisplayItem {
                r#type: DisplayItemType::kRestore,
                phase: self.paint_phase,
                node_id: fragment.node_id,
                ..Default::default()
            },
            self.node,
        );
    }
}

// Source: text_decoration_painter.cc:137-147. Chromium returns black while a
// GraphicsContext shadow filter supplies the shadow color. Retained PaintOps
// flatten that filter color into the shadow DisplayItem, so `shadow_color`
// is the output-equivalent representation of the same phase decision.
fn LineColorForPhase(originating_color: Color, shadow_phase: bool, text_clip: bool) -> Color {
    if shadow_phase || text_clip {
        return Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            alpha: 1.0,
        };
    }
    originating_color
}

// Source: text_decoration_painter.cc:39-41. A painter may not be dropped
// halfway through the decoration/glyph/line-through protocol.
impl Drop for TextDecorationPainter<'_, '_, '_, '_> {
    fn drop(&mut self) {
        debug_assert_eq!(self.step, Step::Begin);
    }
}

// Source: text_decoration_info.cc:119-146,658-681. The source UsedFont has
// already resolved its text-fit scaling into `used_font_size` and
// `from_font_thickness`; fixed, percentage, and calculated CSS lengths remain
// intact across the Layout/Paint boundary.
pub(crate) fn ResolveExportedDecorationThickness(
    decoration: &TextDecorationPaint,
    used_font_size: f64,
    from_font_thickness: Option<f64>,
    is_svg_text: bool,
) -> f64 {
    let resolved = if decoration.thickness.IsAuto() {
        used_font_size / 10.0
    } else if decoration.thickness.IsFromFont() {
        from_font_thickness.unwrap_or(used_font_size / 10.0)
    } else {
        f64::from(
            FloatValueForLength(decoration.thickness.Thickness(), used_font_size as f32).round(),
        )
    };
    if !resolved.is_finite() {
        panic!("text decoration geometry is invalid");
    }
    resolved.max(if is_svg_text { 0.0 } else { 1.0 })
}

fn SameDecorationLines(a: &TextDecorationPaint, b: &TextDecorationPaint) -> bool {
    a.underline == b.underline
        && a.overline == b.overline
        && a.line_through == b.line_through
        && a.spelling_error == b.spelling_error
        && a.grammar_error == b.grammar_error
}

fn TextPaintSize(node: &PaintTreeNode<'_>) -> Size {
    let fragment = node
        .fragment
        .as_deref()
        .expect("paint tree node has a fragment");
    if let Some(svg) = fragment.paint.svg_text.as_ref() {
        return svg.untransformed_size;
    }
    fragment.size
}

#[cfg(test)]
mod tests {
    use foundation::{CSSValueID, LayoutUnit, Length, PhysicalRect, TextDecorationThickness};
    use layoutng_assembly::internal::layout_input::{
        Offset, PaintShadow, PaintStyle, TextDecorationPaint,
    };
    use layoutng_assembly::internal::layout_input_types::Color;

    use super::{
        CurrentSpellingGrammarLineStyle, CurrentSpellingGrammarThickness, EffectiveTextShadows,
        LineColorForPhase, LineRelativeRect, ResolveExportedDecorationThickness,
        SameDecorationLines, StrokeStyle, TextDecorationPainter,
    };

    #[test]
    fn effective_text_shadows_skip_transparent_and_reverse_paint_order() {
        let mut style = PaintStyle::default();
        style.text_shadows = vec![
            PaintShadow {
                offset: Offset { x: 1.0, y: 2.0 },
                color: Color {
                    red: 1.0,
                    alpha: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            PaintShadow {
                offset: Offset { x: 3.0, y: 4.0 },
                color: Color::default(),
                ..Default::default()
            },
            PaintShadow {
                offset: Offset { x: 5.0, y: 6.0 },
                color: Color {
                    blue: 1.0,
                    alpha: 0.5,
                    ..Default::default()
                },
                ..Default::default()
            },
        ];
        let shadows = EffectiveTextShadows(&style, false);
        assert_eq!(shadows.len(), 2);
        assert_eq!(shadows[0].offset, Offset { x: 5.0, y: 6.0 });
        assert_eq!(shadows[1].offset, Offset { x: 1.0, y: 2.0 });
        assert!(EffectiveTextShadows(&style, true).is_empty());
    }

    #[test]
    fn svg_selection_clip_expands_one_height_above_and_below() {
        let rect = LineRelativeRect::CreateFromLineBox(
            &PhysicalRect::from_units(
                LayoutUnit::from_signed(10),
                LayoutUnit::from_signed(20),
                LayoutUnit::from_signed(30),
                LayoutUnit::from_signed(8),
            ),
            true,
        );
        let expanded = TextDecorationPainter::ExpandRectForSVGDecorations(&rect);
        assert_eq!(expanded.x(), 10.0);
        assert_eq!(expanded.y(), 12.0);
        assert_eq!(expanded.width(), 30.0);
        assert_eq!(expanded.height(), 24.0);
    }

    #[test]
    fn line_color_phase_flattens_shadow_and_text_clip_colors() {
        let line = Color {
            red: 0.2,
            green: 0.3,
            blue: 0.4,
            alpha: 1.0,
        };
        assert_eq!(LineColorForPhase(line, false, false), line);
        assert_eq!(
            LineColorForPhase(line, true, false),
            Color {
                alpha: 1.0,
                ..Default::default()
            }
        );
        assert_eq!(
            LineColorForPhase(line, false, true),
            Color {
                red: 0.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            }
        );
    }

    #[test]
    fn exported_decoration_thickness_keeps_auto_from_font_and_fixed_distinct() {
        let mut decoration = TextDecorationPaint::default();
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), false),
            6.0
        );
        decoration.thickness = TextDecorationThickness::from_keyword(CSSValueID::kFromFont);
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), false),
            3.5
        );
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, None, false),
            6.0
        );
        decoration.thickness = TextDecorationThickness::new(&Length::Fixed(2.25));
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), false),
            2.0
        );
        decoration.thickness = TextDecorationThickness::new(&Length::Fixed(0.25));
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), false),
            1.0
        );
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), true),
            0.0
        );
        decoration.thickness = TextDecorationThickness::new(&Length::Percent(10.0));
        assert_eq!(
            ResolveExportedDecorationThickness(&decoration, 60.0, Some(3.5), false),
            6.0
        );
    }

    #[test]
    fn spelling_grammar_bits_participate_in_line_identity() {
        let underline = TextDecorationPaint {
            underline: true,
            ..Default::default()
        };
        let spelling = TextDecorationPaint {
            spelling_error: true,
            ..Default::default()
        };
        let grammar = TextDecorationPaint {
            grammar_error: true,
            ..Default::default()
        };
        assert!(!SameDecorationLines(&underline, &spelling));
        assert!(!SameDecorationLines(&spelling, &grammar));
        assert!(SameDecorationLines(&spelling, &spelling));
    }

    #[test]
    fn current_platform_spelling_marker_policy_is_reachable_from_recording() {
        let (style, antialias, wave) = CurrentSpellingGrammarLineStyle(2.0);
        #[cfg(target_vendor = "apple")]
        {
            assert_eq!(style, StrokeStyle::kDottedStroke);
            assert!(antialias);
            assert!(wave.is_none());
            assert_eq!(CurrentSpellingGrammarThickness(2.0), 4.0);
        }
        #[cfg(all(not(target_vendor = "apple"), not(target_os = "android")))]
        {
            assert_eq!(style, StrokeStyle::kWavyStroke);
            assert!(!antialias);
            assert!(wave.is_some());
            assert_eq!(CurrentSpellingGrammarThickness(2.0), 2.0);
        }
        #[cfg(target_os = "android")]
        {
            foundation::blink_features::SetAndroidSpellcheckNativeUiEnabled(false);
            assert_eq!(style, StrokeStyle::kWavyStroke);
            assert!(!antialias);
            assert!(wave.is_some());
            assert_eq!(CurrentSpellingGrammarThickness(2.0), 2.0);
            foundation::blink_features::SetAndroidSpellcheckNativeUiEnabled(true);
            let (style, antialias, wave) = CurrentSpellingGrammarLineStyle(2.0);
            assert_eq!(style, StrokeStyle::kSolidStroke);
            assert!(antialias);
            assert!(wave.is_none());
            assert_eq!(CurrentSpellingGrammarThickness(2.0), 5.0);
            foundation::blink_features::SetAndroidSpellcheckNativeUiEnabled(false);
        }
    }
}
