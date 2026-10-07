#![allow(non_snake_case)]

use std::ffi::{c_void, CString};

use layoutng_assembly::internal::layout_input::{
    FontSmoothing, PaintPathCommand, PaintPathVerb, TextDecorationStyle, TransformMatrix,
};
use layoutng_assembly::internal::layout_input_types::Color;
use layoutng_assembly::internal::paint_input::{
    PaintBlendMode, PaintCornerRadii, PaintShaderKind, SvgStrokeLineCap, SvgStrokeLineJoin,
};
use paint::paint_engine::{DisplayItem, DisplayItemType, PaintArtifact, PaintRect};

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeVariation {
    tag: u32,
    value: f32,
}

impl From<PaintRect> for NativeRect {
    fn from(rect: PaintRect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeRadii {
    values: [f64; 8],
}

impl From<PaintCornerRadii> for NativeRadii {
    fn from(radii: PaintCornerRadii) -> Self {
        Self {
            values: [
                radii.top_left.x,
                radii.top_left.y,
                radii.top_right.x,
                radii.top_right.y,
                radii.bottom_right.x,
                radii.bottom_right.y,
                radii.bottom_left.x,
                radii.bottom_left.y,
            ],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeColor {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeGradientStop {
    color: NativeColor,
    offset: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct NativePathCommand {
    verb: u32,
    x: f32,
    y: f32,
    c1x: f32,
    c1y: f32,
    c2x: f32,
    c2y: f32,
    weight: f32,
}

impl From<&PaintPathCommand> for NativePathCommand {
    fn from(c: &PaintPathCommand) -> Self {
        Self {
            verb: match c.verb {
                PaintPathVerb::kMoveTo => 0,
                PaintPathVerb::kLineTo => 1,
                PaintPathVerb::kQuadraticTo => 2,
                PaintPathVerb::kConicTo => 3,
                PaintPathVerb::kCubicTo => 4,
                PaintPathVerb::kClose => 5,
            },
            x: c.point.x as f32,
            y: c.point.y as f32,
            c1x: c.control1.x as f32,
            c1y: c.control1.y as f32,
            c2x: c.control2.x as f32,
            c2y: c.control2.y as f32,
            weight: c.conic_weight as f32,
        }
    }
}

impl From<Color> for NativeColor {
    fn from(color: Color) -> Self {
        Self {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: color.alpha,
        }
    }
}

unsafe extern "C" {
    fn LayoutngCanvasSaveLayerDstIn(canvas: *mut c_void);
    fn LayoutngCanvasSaveLayerBlurFilters(canvas: *mut c_void, sigmas: *const f32, count: usize);
    fn LayoutngCanvasClipPath(
        canvas: *mut c_void,
        commands: *const NativePathCommand,
        count: usize,
        even_odd: bool,
        antialias: bool,
    );
    fn LayoutngCanvasDrawPath(
        canvas: *mut c_void,
        commands: *const NativePathCommand,
        count: usize,
        even_odd: bool,
        inverse: bool,
        color: NativeColor,
        antialias: bool,
        stroke: bool,
        width: f32,
        miter: f32,
        cap: i32,
        join: i32,
        dashes: *const f64,
        dash_count: usize,
        dash_offset: f32,
        blur_radius: f32,
    );
    fn LayoutngCanvasDrawLinearGradient(
        canvas: *mut c_void,
        destination: NativeRect,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: *const NativeGradientStop,
        stop_count: usize,
        spread: i32,
        matrix: *const f32,
        premultiplied: bool,
        antialias: bool,
        dither: bool,
    );
    fn LayoutngCanvasCreate(width: i32, height: i32) -> *mut c_void;
    fn LayoutngCanvasDestroy(canvas: *mut c_void);
    fn LayoutngCanvasSave(canvas: *mut c_void);
    fn LayoutngCanvasTranslate(canvas: *mut c_void, x: f32, y: f32);
    fn LayoutngCanvasConcat(canvas: *mut c_void, matrix: *const f32);
    fn LayoutngCanvasRestore(canvas: *mut c_void);
    fn LayoutngCanvasSaveCount(canvas: *mut c_void) -> i32;
    fn LayoutngCanvasClipRect(canvas: *mut c_void, rect: NativeRect, antialias: bool);
    fn LayoutngCanvasClipRRect(
        canvas: *mut c_void,
        rect: NativeRect,
        radii: NativeRadii,
        antialias: bool,
    );
    fn LayoutngCanvasClipOutRect(canvas: *mut c_void, rect: NativeRect, antialias: bool);
    fn LayoutngCanvasClipOutRRect(
        canvas: *mut c_void,
        rect: NativeRect,
        radii: NativeRadii,
        antialias: bool,
    );
    fn LayoutngCanvasSaveLayer(canvas: *mut c_void, rect: NativeRect);
    fn LayoutngCanvasSaveLayerAlpha(canvas: *mut c_void, rect: NativeRect, opacity: f32);
    fn LayoutngCanvasSaveLayerBlend(canvas: *mut c_void, rect: NativeRect, mode: i32, opacity: f32);
    fn LayoutngCanvasDrawRect(
        canvas: *mut c_void,
        rect: NativeRect,
        color: NativeColor,
        antialias: bool,
    );
    fn LayoutngCanvasStrokeLine(
        canvas: *mut c_void,
        rect: NativeRect,
        color: NativeColor,
        antialias: bool,
        stroke_width: f32,
        round_cap: bool,
        dashes: *const f64,
        dash_count: usize,
        dash_offset: f32,
    );
    fn LayoutngCanvasDrawOval(
        canvas: *mut c_void,
        rect: NativeRect,
        color: NativeColor,
        antialias: bool,
    );
    fn LayoutngCanvasDrawRRect(
        canvas: *mut c_void,
        rect: NativeRect,
        radii: NativeRadii,
        color: NativeColor,
        antialias: bool,
        has_radius: bool,
    );
    fn LayoutngCanvasDrawDRRect(
        canvas: *mut c_void,
        outer: NativeRect,
        outer_radii: NativeRadii,
        inner: NativeRect,
        inner_radii: NativeRadii,
        color: NativeColor,
        antialias: bool,
        has_radius: bool,
        stroke_width: f32,
    );
    fn LayoutngCanvasDrawBoxShadow(
        canvas: *mut c_void,
        rect: NativeRect,
        radii: NativeRadii,
        color: NativeColor,
        offset_x: f64,
        offset_y: f64,
        blur_radius: f64,
        spread: f64,
        inset: bool,
        opaque_background: bool,
        antialias: bool,
    );
    fn LayoutngCanvasRegisterImage(
        canvas: *mut c_void,
        id: u64,
        width: u32,
        height: u32,
        rgba: *const u8,
        byte_count: usize,
    ) -> i32;
    fn LayoutngCanvasImageDeviceSize(
        canvas: *mut c_void,
        rect: NativeRect,
        width: *mut i32,
        height: *mut i32,
    );
    fn LayoutngCanvasImageDimensions(
        canvas: *mut c_void,
        id: u64,
        width: *mut i32,
        height: *mut i32,
    ) -> i32;
    fn LayoutngCanvasBuildMipImage(canvas: *mut c_void, id: u64, level: i32) -> i32;
    fn LayoutngCanvasDrawImageRect(
        canvas: *mut c_void,
        id: u64,
        mip_level: i32,
        source: NativeRect,
        destination: NativeRect,
    );
    fn LayoutngCanvasRegisterFont(
        canvas: *mut c_void,
        bytes: *const u8,
        byte_count: usize,
        face_index: u32,
        native_family: *const i8,
        weight: f64,
        italic: bool,
        variations: *const NativeVariation,
        variation_count: usize,
    ) -> i32;
    fn LayoutngCanvasDrawGlyphRun(
        canvas: *mut c_void,
        face_index: usize,
        font_size: f32,
        ids: *const u16,
        xy: *const f32,
        glyph_count: usize,
        origin_x: f32,
        origin_y: f32,
        color: NativeColor,
        synthetic_bold: bool,
        synthetic_italic: bool,
        smoothing: u8,
        stroke: bool,
        stroke_width: f32,
        variations: *const NativeVariation,
        variation_count: usize,
    );
    fn LayoutngCanvasReadRgba(canvas: *mut c_void, rgba: *mut u8, length: usize) -> i32;
}

struct Canvas(*mut c_void);

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        let width = i32::try_from(width).expect("raster width exceeds i32");
        let height = i32::try_from(height).expect("raster height exceeds i32");
        let canvas = unsafe { LayoutngCanvasCreate(width, height) };
        assert!(!canvas.is_null(), "Skia could not create raster surface");
        Self(canvas)
    }
}

impl Drop for Canvas {
    fn drop(&mut self) {
        unsafe { LayoutngCanvasDestroy(self.0) };
    }
}

// cpp: skia_renderer/skia_renderer.cc:196-230
fn CanStrokeDoubleRoundedRect(item: &DisplayItem) -> Option<f32> {
    let left = item.inner_rect.x - item.rect.x;
    let top = item.inner_rect.y - item.rect.y;
    let right = item.rect.x + item.rect.width - item.inner_rect.x - item.inner_rect.width;
    let bottom = item.rect.y + item.rect.height - item.inner_rect.y - item.inner_rect.height;
    let nearly_equal = |a: f64, b: f64| (a - b).abs() <= 1e-5;
    if left < 0.0
        || !nearly_equal(left, top)
        || !nearly_equal(left, right)
        || !nearly_equal(left, bottom)
    {
        return None;
    }
    let simple_corner =
        |outer: layoutng_assembly::internal::paint_input::PaintCornerRadius,
         inner: layoutng_assembly::internal::paint_input::PaintCornerRadius| {
            if outer.x == 0.0 && outer.y == 0.0 && inner.x == 0.0 && inner.y == 0.0 {
                return true;
            }
            nearly_equal(outer.x, outer.y)
                && nearly_equal(inner.x, inner.y)
                && nearly_equal(outer.x, inner.x + left)
        };
    if !simple_corner(item.corner_radii.top_left, item.inner_corner_radii.top_left)
        || !simple_corner(
            item.corner_radii.top_right,
            item.inner_corner_radii.top_right,
        )
        || !simple_corner(
            item.corner_radii.bottom_right,
            item.inner_corner_radii.bottom_right,
        )
        || !simple_corner(
            item.corner_radii.bottom_left,
            item.inner_corner_radii.bottom_left,
        )
    {
        return None;
    }
    Some(left as f32)
}

// cpp: skia_renderer/skia_renderer.cc:269-298
// The SkPaint object itself lives in native_canvas.cc; this validates the
// source paint fields handled by the current replay branch before FFI.
fn ItemColor(item: &DisplayItem) -> NativeColor {
    assert_eq!(
        item.blend_mode,
        PaintBlendMode::kNormal,
        "blend-mode replay is pending"
    );
    assert!(item.paint_shader.is_none(), "shader replay is pending");
    assert_eq!(item.blur_radius, 0.0, "blurred paint replay is pending");
    item.color.into()
}

// cpp: skia_renderer/skia_renderer.cc:919-981
fn DrawGlyphs(canvas: &Canvas, item: &DisplayItem) {
    if item.glyphs.is_empty() {
        return;
    }
    assert_eq!(
        item.transform,
        TransformMatrix::default(),
        "glyph transform replay is pending"
    );
    assert!(
        item.glyphs.iter().all(|glyph| glyph.canvas_rotation == 0),
        "rotated glyph replay is pending"
    );
    let mut ids = Vec::with_capacity(item.glyphs.len());
    let mut xy = Vec::with_capacity(item.glyphs.len() * 2);
    let variations: Vec<NativeVariation> = item
        .font_variations
        .iter()
        .map(|axis| NativeVariation {
            tag: axis.tag,
            value: axis.value,
        })
        .collect();
    for glyph in &item.glyphs {
        ids.push(glyph.id as u16);
        xy.push(glyph.offset.x as f32);
        xy.push(glyph.offset.y as f32);
    }
    let smoothing = match item.font_smoothing {
        FontSmoothing::kNone => 1,
        FontSmoothing::kAntialiased => 2,
        FontSmoothing::kAuto | FontSmoothing::kSubpixelAntialiased => 0,
    };
    unsafe {
        LayoutngCanvasDrawGlyphRun(
            canvas.0,
            item.font_face_index as usize,
            item.font_size as f32,
            ids.as_ptr(),
            xy.as_ptr(),
            ids.len(),
            item.text_blob_origin.x as f32,
            item.text_blob_origin.y as f32,
            ItemColor(item),
            item.synthetic_bold,
            item.synthetic_italic,
            smoothing,
            item.stroke_glyphs,
            item.stroke_width as f32,
            variations.as_ptr(),
            variations.len(),
        );
    }
}

// cpp: skia_renderer/skia_renderer.cc:342-358
fn MipLevelForSize(
    source_width: i32,
    source_height: i32,
    target_width: i32,
    target_height: i32,
) -> i32 {
    let mut level = 0;
    loop {
        let divisor = 1_i64 << (level + 1);
        let next_width = ((i64::from(source_width) + divisor - 1) / divisor).max(1) as i32;
        let next_height = ((i64::from(source_height) + divisor - 1) / divisor).max(1) as i32;
        if next_width < target_width
            || next_height < target_height
            || (source_width == 1 && source_height == 1)
        {
            return level;
        }
        // The C++ loop has no termination for a 1xN source scaled to 1x1.
        // Stop when another level cannot change the image dimensions.
        if (next_width == 1 && next_height == 1) || level == 29 {
            return level + 1;
        }
        level += 1;
    }
}

// cpp: skia_renderer/skia_renderer.cc:628-684
fn DrawImage(canvas: &Canvas, item: &DisplayItem, destination: PaintRect) {
    assert_eq!(
        item.blend_mode,
        PaintBlendMode::kNormal,
        "image blend-mode replay is pending"
    );
    let (mut width, mut height) = (0_i32, 0_i32);
    if unsafe { LayoutngCanvasImageDimensions(canvas.0, item.resource_id, &mut width, &mut height) }
        != 1
    {
        return;
    }
    let mut source = item.source_rect;
    let uses_full_source = source.x == 0.0
        && source.y == 0.0
        && source.width == f64::from(width)
        && source.height == f64::from(height);
    let (mut target_width, mut target_height) = (1, 1);
    // Diagnostic replay uses the independently mapped C++ SkMatrix size.
    unsafe {
        LayoutngCanvasImageDeviceSize(
            canvas.0,
            destination.into(),
            &mut target_width,
            &mut target_height,
        );
    }
    let mip_level = if uses_full_source {
        MipLevelForSize(width, height, target_width, target_height)
    } else {
        0
    };
    let mut draw_mip_level = 0;
    if mip_level > 0
        && unsafe { LayoutngCanvasBuildMipImage(canvas.0, item.resource_id, mip_level) } == 1
    {
        draw_mip_level = mip_level;
        let divisor = 1_i64 << mip_level;
        source = PaintRect {
            x: 0.0,
            y: 0.0,
            width: ((i64::from(width) + divisor - 1) / divisor).max(1) as f64,
            height: ((i64::from(height) + divisor - 1) / divisor).max(1) as f64,
        };
    }
    unsafe {
        LayoutngCanvasDrawImageRect(
            canvas.0,
            item.resource_id,
            draw_mip_level,
            source.into(),
            destination.into(),
        );
    }
}

// cpp: skia_renderer/skia_renderer.cc:756-778
fn DrawTiledImage(canvas: &Canvas, item: &DisplayItem) {
    let step_x = item.tile_rect.width + item.tile_spacing.width;
    let step_y = item.tile_rect.height + item.tile_spacing.height;
    if step_x <= 0.0 || step_y <= 0.0 {
        return;
    }
    let first_x = if item.repeat_x {
        item.tile_rect.x - ((item.tile_rect.x - item.rect.x) / step_x).ceil() * step_x
    } else {
        item.tile_rect.x
    };
    let first_y = if item.repeat_y {
        item.tile_rect.y - ((item.tile_rect.y - item.rect.y) / step_y).ceil() * step_y
    } else {
        item.tile_rect.y
    };
    let mut y = first_y;
    let mut rows = 0;
    while y < item.rect.y + item.rect.height && rows < 10_000 {
        rows += 1;
        let mut x = first_x;
        let mut columns = 0;
        while x < item.rect.x + item.rect.width && columns < 10_000 {
            columns += 1;
            DrawImage(
                canvas,
                item,
                PaintRect {
                    x,
                    y,
                    width: item.tile_rect.width,
                    height: item.tile_rect.height,
                },
            );
            x += if item.repeat_x {
                step_x
            } else {
                item.rect.width + step_x
            };
        }
        y += if item.repeat_y {
            step_y
        } else {
            item.rect.height + step_y
        };
    }
}

// cpp: skia_renderer/skia_renderer.cc:779-826
fn DrawShaderTile(canvas: &Canvas, item: &DisplayItem, destination: PaintRect, dither: bool) {
    let shader = item
        .paint_shader
        .as_ref()
        .expect("gradient display item has no shader");
    assert_eq!(
        shader.kind,
        PaintShaderKind::kLinearGradient,
        "non-linear gradient source replay is pending"
    );
    assert_eq!(
        item.blend_mode,
        PaintBlendMode::kNormal,
        "gradient blend-mode source replay is pending"
    );
    let source_x = item.tile_rect.x;
    let source_y = item.tile_rect.y;
    let mut matrix = shader.transform.values.map(|value| value as f32);
    matrix[12] = (shader.transform.values[12] + destination.x) as f32;
    matrix[13] = (shader.transform.values[13] + destination.y) as f32;
    let stops: Vec<NativeGradientStop> = shader
        .stops
        .iter()
        .map(|stop| NativeGradientStop {
            color: stop.color.into(),
            offset: stop.offset as f32,
        })
        .collect();
    unsafe {
        LayoutngCanvasDrawLinearGradient(
            canvas.0,
            destination.into(),
            (shader.start.x - source_x) as f32,
            (shader.start.y - source_y) as f32,
            (shader.end.x - source_x) as f32,
            (shader.end.y - source_y) as f32,
            stops.as_ptr(),
            stops.len(),
            shader.spread as i32,
            matrix.as_ptr(),
            shader.interpolate_premultiplied,
            item.antialias,
            dither,
        );
    }
}

// cpp: skia_renderer/skia_renderer.cc:828-907
fn DrawShader(canvas: &Canvas, item: &DisplayItem) {
    let same = |a: f64, b: f64| (a - b).abs() <= 1e-6;
    let single_tile = same(item.rect.x, item.tile_rect.x)
        && same(item.rect.y, item.tile_rect.y)
        && same(item.rect.width, item.tile_rect.width)
        && same(item.rect.height, item.tile_rect.height);
    if item.r#type != DisplayItemType::kDrawTiledGradient || single_tile {
        DrawShaderTile(canvas, item, item.rect, true);
        return;
    }
    let step_x = item.tile_rect.width + item.tile_spacing.width;
    let step_y = item.tile_rect.height + item.tile_spacing.height;
    if step_x <= 0.0 || step_y <= 0.0 {
        return;
    }
    let first_x = if item.repeat_x {
        item.tile_rect.x - ((item.tile_rect.x - item.rect.x) / step_x).ceil() * step_x
    } else {
        item.tile_rect.x
    };
    let first_y = if item.repeat_y {
        item.tile_rect.y - ((item.tile_rect.y - item.rect.y) / step_y).ceil() * step_y
    } else {
        item.tile_rect.y
    };
    unsafe {
        LayoutngCanvasSave(canvas.0);
        LayoutngCanvasClipRect(canvas.0, item.rect.into(), false);
    }
    let mut y = first_y;
    let mut rows = 0;
    while y < item.rect.y + item.rect.height && rows < 10_000 {
        rows += 1;
        let mut x = first_x;
        let mut columns = 0;
        while x < item.rect.x + item.rect.width && columns < 10_000 {
            columns += 1;
            DrawShaderTile(
                canvas,
                item,
                PaintRect {
                    x,
                    y,
                    width: item.tile_rect.width,
                    height: item.tile_rect.height,
                },
                false,
            );
            x += if item.repeat_x {
                step_x
            } else {
                item.rect.width + step_x
            };
        }
        y += if item.repeat_y {
            step_y
        } else {
            item.rect.height + step_y
        };
    }
    unsafe {
        LayoutngCanvasRestore(canvas.0);
    }
}

// cpp: skia_renderer/skia_renderer.cc:983-1091
// cpp: skia_renderer/skia_renderer.cc:1140-1203
// cpp: skia_renderer/skia_renderer.cc:1358-1360
fn ReplayItem(canvas: &Canvas, item: &DisplayItem) {
    let rect = item.rect.into();
    match item.r#type {
        DisplayItemType::kSave => unsafe { LayoutngCanvasSave(canvas.0) },
        DisplayItemType::kRestore => unsafe { LayoutngCanvasRestore(canvas.0) },
        DisplayItemType::kConcat => {
            let matrix = item.transform.values.map(|value| value as f32);
            unsafe { LayoutngCanvasConcat(canvas.0, matrix.as_ptr()) };
        }
        DisplayItemType::kClipRect => unsafe {
            LayoutngCanvasClipRect(canvas.0, rect, item.antialias)
        },
        DisplayItemType::kClipRoundedRect => unsafe {
            LayoutngCanvasClipRRect(canvas.0, rect, item.corner_radii.into(), item.antialias)
        },
        DisplayItemType::kClipOutRect => unsafe {
            LayoutngCanvasClipOutRect(canvas.0, rect, item.antialias)
        },
        DisplayItemType::kClipOutRoundedRect => unsafe {
            LayoutngCanvasClipOutRRect(canvas.0, rect, item.corner_radii.into(), item.antialias)
        },
        DisplayItemType::kSaveLayer => unsafe { LayoutngCanvasSaveLayer(canvas.0, rect) },
        DisplayItemType::kSaveLayerBlend => unsafe {
            LayoutngCanvasSaveLayerBlend(canvas.0, rect, item.blend_mode as i32, item.opacity)
        },
        DisplayItemType::kSaveLayerFilter => {
            let sigmas: Vec<f32> = item
                .filters
                .iter()
                .filter(|f| {
                    f.r#type == layoutng_assembly::internal::paint_input::PaintFilterType::kBlur
                        && f.amount > 0.0
                })
                .map(|f| f.amount as f32)
                .collect();
            assert_eq!(
                sigmas.len(),
                item.filters.len(),
                "native oracle admits only positive blur filters"
            );
            unsafe {
                LayoutngCanvasSaveLayerBlurFilters(canvas.0, sigmas.as_ptr(), sigmas.len());
            }
        }
        DisplayItemType::kSaveLayerAlpha => unsafe {
            LayoutngCanvasSaveLayerAlpha(canvas.0, rect, item.opacity)
        },
        DisplayItemType::kDrawRect => unsafe {
            LayoutngCanvasDrawRect(canvas.0, rect, ItemColor(item), item.antialias)
        },
        // cpp: skia_renderer/skia_renderer.cc:1144-1146
        DisplayItemType::kDrawEllipse => unsafe {
            LayoutngCanvasDrawOval(canvas.0, rect, ItemColor(item), item.antialias)
        },
        DisplayItemType::kDrawRoundedRect => unsafe {
            LayoutngCanvasDrawRRect(
                canvas.0,
                rect,
                item.corner_radii.into(),
                ItemColor(item),
                item.antialias,
                item.corner_radii.HasRadius(),
            )
        },
        DisplayItemType::kDrawDoubleRoundedRect => unsafe {
            LayoutngCanvasDrawDRRect(
                canvas.0,
                rect,
                item.corner_radii.into(),
                item.inner_rect.into(),
                item.inner_corner_radii.into(),
                ItemColor(item),
                item.antialias,
                item.corner_radii.HasRadius() || item.inner_corner_radii.HasRadius(),
                CanStrokeDoubleRoundedRect(item).unwrap_or(-1.0),
            )
        },
        DisplayItemType::kStrokeLine => {
            if item.is_text_decoration
                && item.rect.height == 0.0
                && matches!(
                    item.decoration_style,
                    TextDecorationStyle::kSolid | TextDecorationStyle::kDouble
                )
            {
                let thickness = item.stroke_width.floor().max(1.0);
                let draw = |y: f64| unsafe {
                    LayoutngCanvasDrawRect(
                        canvas.0,
                        PaintRect {
                            x: item.rect.x,
                            y: (y + 0.5).floor(),
                            width: item.rect.width,
                            height: thickness,
                        }
                        .into(),
                        ItemColor(item),
                        item.antialias,
                    )
                };
                draw(item.rect.y);
                if item.decoration_style == TextDecorationStyle::kDouble {
                    draw(item.rect.y + (item.stroke_width + 1.0).floor());
                }
            } else {
                unsafe {
                    LayoutngCanvasStrokeLine(
                        canvas.0,
                        rect,
                        ItemColor(item),
                        item.antialias,
                        item.stroke_width as f32,
                        item.round_cap || item.svg_line_cap == SvgStrokeLineCap::kRound,
                        item.dash_intervals.as_ptr(),
                        item.dash_intervals.len(),
                        item.dash_offset as f32,
                    )
                }
            }
        }
        DisplayItemType::kDrawBoxShadow => unsafe {
            LayoutngCanvasDrawBoxShadow(
                canvas.0,
                rect,
                item.corner_radii.into(),
                item.color.into(),
                item.shadow_offset.x,
                item.shadow_offset.y,
                item.blur_radius,
                item.spread,
                item.inset,
                item.shadow_has_opaque_background,
                item.antialias,
            )
        },
        DisplayItemType::kDrawGlyphRun => DrawGlyphs(canvas, item),
        // cpp: skia_renderer/skia_renderer.cc:1153-1160,232-268,270-311
        DisplayItemType::kClipPath => {
            let commands: Vec<_> = item.path.iter().map(NativePathCommand::from).collect();
            unsafe {
                LayoutngCanvasClipPath(
                    canvas.0,
                    commands.as_ptr(),
                    commands.len(),
                    item.even_odd,
                    item.antialias,
                );
            }
        }
        DisplayItemType::kDrawPath | DisplayItemType::kStrokePath => {
            let commands: Vec<_> = item.path.iter().map(NativePathCommand::from).collect();
            let stroke = item.r#type == DisplayItemType::kStrokePath;
            let cap = if item.round_cap || item.svg_line_cap == SvgStrokeLineCap::kRound {
                1
            } else if item.svg_line_cap == SvgStrokeLineCap::kSquare {
                2
            } else {
                0
            };
            let join = match item.svg_line_join {
                SvgStrokeLineJoin::kRound => 1,
                SvgStrokeLineJoin::kBevel => 2,
                _ => 0,
            };
            unsafe {
                LayoutngCanvasDrawPath(
                    canvas.0,
                    commands.as_ptr(),
                    commands.len(),
                    item.even_odd,
                    !stroke && item.inverse_winding,
                    ItemColor(item),
                    item.antialias,
                    stroke,
                    item.stroke_width as f32,
                    item.miter_limit as f32,
                    cap,
                    join,
                    item.dash_intervals.as_ptr(),
                    item.dash_intervals.len(),
                    item.dash_offset as f32,
                    item.blur_radius as f32,
                )
            };
        }
        DisplayItemType::kDrawImageRect => DrawImage(canvas, item, item.rect),
        DisplayItemType::kDrawTiledImage => DrawTiledImage(canvas, item),
        DisplayItemType::kDrawGradientRect | DisplayItemType::kDrawTiledGradient => {
            DrawShader(canvas, item)
        }
        other => panic!("source Skia replay for {other:?} is pending"),
    }
}

// cpp: skia_renderer/skia_renderer.cc:460-540
fn RegisterResources(canvas: &Canvas, list: &PaintArtifact) {
    let Some(resources) = &list.resources else {
        return;
    };
    for image in &resources.images {
        let result = unsafe {
            LayoutngCanvasRegisterImage(
                canvas.0,
                image.id,
                image.width,
                image.height,
                image.rgba8.as_ptr(),
                image.rgba8.len(),
            )
        };
        assert_eq!(result, 1, "Skia could not register paint image");
    }
    for face in &resources.fonts {
        let variations: Vec<NativeVariation> = face
            .variations
            .iter()
            .map(|axis| NativeVariation {
                tag: axis.tag,
                value: axis.value,
            })
            .collect();
        let family = CString::new(face.native_family.as_str()).expect("font family contains NUL");
        let result = unsafe {
            LayoutngCanvasRegisterFont(
                canvas.0,
                face.bytes.as_ptr(),
                face.bytes.len(),
                face.face_index,
                family.as_ptr(),
                face.weight,
                face.italic,
                variations.as_ptr(),
                variations.len(),
            )
        };
        assert_eq!(result, 1, "Skia could not open paint font");
    }
}

// cpp: skia_renderer/skia_renderer.cc:542-546
fn Replay(canvas: &Canvas, list: &PaintArtifact) {
    RegisterResources(canvas, list);
    let mut masks = Vec::new();
    for item in list.items.iter() {
        match item.r#type {
            DisplayItemType::kBeginMask => {
                masks.push(&item.mask_layers);
                unsafe {
                    LayoutngCanvasSaveLayer(canvas.0, PaintRect::default().into());
                }
            }
            DisplayItemType::kEndMask => {
                let Some(layers) = masks.pop() else {
                    continue;
                };
                unsafe {
                    LayoutngCanvasSaveLayerDstIn(canvas.0);
                }
                for layer in layers {
                    unsafe {
                        LayoutngCanvasSave(canvas.0);
                        if layer.clip_radii.HasRadius() {
                            LayoutngCanvasClipRRect(
                                canvas.0,
                                layer.clip_rect.into(),
                                layer.clip_radii.into(),
                                true,
                            );
                        } else {
                            LayoutngCanvasClipRect(canvas.0, layer.clip_rect.into(), false);
                        }
                    }
                    if let Some(shader) = &layer.paint_shader {
                        assert_eq!(shader.kind, PaintShaderKind::kLinearGradient);
                        let matrix = shader.transform.values.map(|x| x as f32);
                        let stops: Vec<_> = shader
                            .stops
                            .iter()
                            .map(|stop| NativeGradientStop {
                                color: stop.color.into(),
                                offset: stop.offset as f32,
                            })
                            .collect();
                        unsafe {
                            LayoutngCanvasDrawLinearGradient(
                                canvas.0,
                                layer.clip_rect.into(),
                                shader.start.x as f32,
                                shader.start.y as f32,
                                shader.end.x as f32,
                                shader.end.y as f32,
                                stops.as_ptr(),
                                stops.len(),
                                shader.spread as i32,
                                matrix.as_ptr(),
                                shader.interpolate_premultiplied,
                                false,
                                true,
                            );
                        }
                    } else {
                        unsafe {
                            LayoutngCanvasDrawRect(
                                canvas.0,
                                layer.clip_rect.into(),
                                Color {
                                    red: 1.0,
                                    green: 1.0,
                                    blue: 1.0,
                                    alpha: 1.0,
                                }
                                .into(),
                                false,
                            );
                        }
                    }
                    unsafe {
                        LayoutngCanvasRestore(canvas.0);
                    }
                }
                unsafe {
                    LayoutngCanvasRestore(canvas.0);
                    LayoutngCanvasRestore(canvas.0);
                }
            }
            _ => ReplayItem(canvas, item),
        }
    }
    while unsafe { LayoutngCanvasSaveCount(canvas.0) } > 1 {
        unsafe { LayoutngCanvasRestore(canvas.0) };
    }
}

#[derive(Clone, Copy)]
struct RasterTile {
    origin: u32,
    length: u32,
    leading_border: u32,
    trailing_border: u32,
}

fn tiles_for_axis(extent: u32) -> Vec<RasterTile> {
    let mut tiles = Vec::new();
    let mut origin = 0;
    while origin < extent {
        let leading_border = u32::from(origin != 0);
        let length = (256 - leading_border - 1).min(extent - origin);
        let trailing_border = u32::from(origin + length < extent);
        tiles.push(RasterTile {
            origin,
            length,
            leading_border,
            trailing_border,
        });
        origin += length;
    }
    tiles
}

// cpp: skia_renderer/skia_renderer.cc:1474-1551
pub fn RasterizeSourceDisplayItemList(list: &PaintArtifact, width: u32, height: u32) -> Vec<u8> {
    let length = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .expect("raster dimensions overflow");
    let mut output = vec![255; length];
    for y_tile in tiles_for_axis(height) {
        for x_tile in tiles_for_axis(width) {
            let tile_width = x_tile.leading_border + x_tile.length + x_tile.trailing_border;
            let tile_height = y_tile.leading_border + y_tile.length + y_tile.trailing_border;
            let canvas = Canvas::new(tile_width, tile_height);
            unsafe {
                LayoutngCanvasTranslate(
                    canvas.0,
                    (x_tile.leading_border as i32 - x_tile.origin as i32) as f32,
                    (y_tile.leading_border as i32 - y_tile.origin as i32) as f32,
                );
            }
            Replay(&canvas, list);
            let mut tile_rgba = vec![0; tile_width as usize * tile_height as usize * 4];
            let ok = unsafe {
                LayoutngCanvasReadRgba(canvas.0, tile_rgba.as_mut_ptr(), tile_rgba.len())
            };
            assert_eq!(ok, 1, "Skia could not read raster tile pixels");
            for row in 0..y_tile.length as usize {
                let source = ((y_tile.leading_border as usize + row) * tile_width as usize
                    + x_tile.leading_border as usize)
                    * 4;
                let destination =
                    ((y_tile.origin as usize + row) * width as usize + x_tile.origin as usize) * 4;
                let count = x_tile.length as usize * 4;
                output[destination..destination + count]
                    .copy_from_slice(&tile_rgba[source..source + count]);
            }
        }
    }
    output
}

/// Reference-only CPU benchmark. Unlike the tiled entry point, use one surface
/// so its dimensions and scale match the pure Rust profiling entry point.
#[cfg(feature = "profiling")]
pub fn ProfileSourceDisplayItemListWithScale(
    list: &PaintArtifact,
    width: u32,
    height: u32,
    scale: f64,
) -> (Vec<u8>, [std::time::Duration; 3]) {
    use std::time::Instant;
    let start = Instant::now();
    let canvas = Canvas::new(width, height);
    let mut matrix = TransformMatrix::default();
    matrix.values[0] = scale;
    matrix.values[5] = scale;
    let matrix = matrix.values.map(|value| value as f32);
    unsafe { LayoutngCanvasConcat(canvas.0, matrix.as_ptr()) };
    let setup = start.elapsed();
    let start = Instant::now();
    Replay(&canvas, list);
    let replay = start.elapsed();
    let start = Instant::now();
    let mut pixels = vec![0; width as usize * height as usize * 4];
    let ok = unsafe { LayoutngCanvasReadRgba(canvas.0, pixels.as_mut_ptr(), pixels.len()) };
    assert_eq!(ok, 1, "Skia could not read reference CPU pixels");
    (pixels, [setup, replay, start.elapsed()])
}
