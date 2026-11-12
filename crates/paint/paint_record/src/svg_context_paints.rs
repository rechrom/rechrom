// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Source paint_info.h:45-74; svg_object_painter.cc:104-138,141-202.
//! Context paints borrow the original canonical style and resource payload.
use crate::{paint_info::PaintInfo, PaintRect};
use layoutng_assembly::internal::layout_input::{PaintStyle, TransformMatrix};
use layoutng_style::style::{
    computed_style::ComputedStyle,
    svg_paint::{SVGPaint, SVGPaintType},
};

#[derive(Clone, Copy)]
pub struct ContextPaint<'a> {
    pub style: &'a PaintStyle,
    pub stroke: bool,
    pub reference_box: PaintRect,
}
impl<'a> ContextPaint<'a> {
    pub fn paint(self) -> Option<&'a SVGPaint> {
        let style = self.style.Source()?;
        Some(if self.stroke {
            style.StrokePaint()
        } else {
            style.FillPaint()
        })
    }
    pub fn canonical(self) -> &'a ComputedStyle {
        self.style.Source().expect("native context paint style")
    }
}
#[derive(Clone, Copy)]
pub struct SvgContextPaints<'a> {
    pub fill: ContextPaint<'a>,
    pub stroke: ContextPaint<'a>,
    pub transform: TransformMatrix,
}
impl<'a> SvgContextPaints<'a> {
    pub fn Resolve(
        style: &'a PaintStyle,
        stroke: bool,
        reference_box: PaintRect,
        inherited: Option<&Self>,
    ) -> ContextPaint<'a> {
        let paint = style.Source().map(|s| {
            if stroke {
                s.StrokePaint()
            } else {
                s.FillPaint()
            }
        });
        match (paint.map(|p| p.paint_type), inherited) {
            (Some(SVGPaintType::kContextFill), Some(parent)) => parent.fill,
            (Some(SVGPaintType::kContextStroke), Some(parent)) => parent.stroke,
            _ => ContextPaint {
                style,
                stroke,
                reference_box,
            },
        }
    }
    pub fn New(
        style: &'a PaintStyle,
        reference_box: PaintRect,
        inherited: Option<&Self>,
        transform: TransformMatrix,
    ) -> Self {
        Self {
            fill: Self::Resolve(style, false, reference_box, inherited),
            stroke: Self::Resolve(style, true, reference_box, inherited),
            transform,
        }
    }
}

pub(crate) fn ContextFor<'a>(
    style: &PaintStyle,
    stroke: bool,
    info: &PaintInfo<'a, '_>,
) -> Option<ContextPaint<'a>> {
    let source = style.Source()?;
    match (if stroke {
        source.StrokePaint()
    } else {
        source.FillPaint()
    })
    .paint_type
    {
        SVGPaintType::kContextFill => info.GetSvgContextPaints().map(|paints| paints.fill),
        SVGPaintType::kContextStroke => info.GetSvgContextPaints().map(|paints| paints.stroke),
        _ => None,
    }
}

#[derive(Default)]
pub(crate) struct ResolvedContextPaint {
    pub color: Option<layoutng_assembly::internal::layout_input_types::Color>,
    pub server: Option<layoutng_assembly::internal::paint_input::SvgPaintServer>,
    pub reference_box: PaintRect,
    pub transform: TransformMatrix,
}
// PreparePaint retains the child's opacity/currentColor/visited behavior while
// applying the original context object's paint and paint-server coordinates.
pub(crate) fn ResolveFor(
    style: &PaintStyle,
    stroke: bool,
    info: &PaintInfo<'_, '_>,
) -> Option<ResolvedContextPaint> {
    let child = style.Source()?;
    let initial = if stroke {
        child.StrokePaint()
    } else {
        child.FillPaint()
    };
    if !matches!(
        initial.paint_type,
        SVGPaintType::kContextFill | SVGPaintType::kContextStroke
    ) {
        return None;
    }
    let Some(context) = ContextFor(style, stroke, info) else {
        return Some(Default::default());
    };
    let Some(paint) = context.paint() else {
        return Some(Default::default());
    };
    let alpha = if stroke {
        child.StrokeOpacity()
    } else {
        child.FillOpacity()
    };
    let color = paint.HasColor().then(|| {
        let resolved = if initial.paint_type == SVGPaintType::kContextFill {
            child.VisitedDependentContextFill(paint, context.canonical())
        } else {
            child.VisitedDependentContextStroke(paint, context.canonical())
        };
        let (red, green, blue) = resolved.ToSRGB();
        layoutng_assembly::internal::layout_input_types::Color {
            red,
            green,
            blue,
            alpha: resolved.Alpha() * alpha,
        }
    });
    let mut server = if context.stroke {
        context.style.svg_stroke_server.clone()
    } else {
        context.style.svg_fill_server.clone()
    };
    if let Some(server) = &mut server {
        let shader_alpha = (alpha * 255.).round() / 255.;
        if let Some(stops) = &server.svg_context_stops {
            server.stops = stops.as_ref().clone();
            for stop in &mut server.stops {
                stop.color.alpha *= shader_alpha;
                stop.source_components[3] *= shader_alpha;
            }
        } else if server.pattern.is_none() {
            std::panic::panic_any(foundation::UnsupportedLayout::new(
                "SVG context gradient requires its original raw stop owner",
            ));
        }
        if let Some(pattern) = &mut server.pattern {
            std::sync::Arc::make_mut(pattern).opacity = shader_alpha;
        }
    }
    Some(ResolvedContextPaint {
        color,
        server,
        reference_box: context.reference_box,
        transform: info.GetSvgContextPaints().expect("context owner").transform,
    })
}

pub(crate) fn ResolveShader(
    paint: &ResolvedContextPaint,
    resources: Option<&layoutng_assembly::fragment_tree::PaintResources>,
    privacy_preserving: bool,
) -> Option<layoutng_assembly::internal::paint_input::SvgPaintServer> {
    let server = paint.server.as_ref()?;
    let rect = paint.reference_box;
    let mut shader = crate::paint_shader_resolver::ResolvePaintShaderForBounds(
        server,
        resources,
        Default::default(),
        layoutng_assembly::internal::layout_input::Offset {
            x: rect.x,
            y: rect.y,
        },
        Some(layoutng_assembly::internal::layout_input::Size {
            width: rect.width,
            height: rect.height,
        }),
    );
    let inverse = crate::geometry_mapper::InvertTransform(&paint.transform).unwrap_or_default();
    shader.transform = crate::geometry_mapper::MultiplyTransforms(&inverse, &shader.transform);
    crate::paint_shader_resolver::PrivacyPreservingPaintServer(&shader, privacy_preserving)
}
