//! Actual Paint value projections consumed by the canonical metadata owner.
//! No traversal summary is rebuilt when a getter is called: analysis happens
//! once at insertion, and nested queries borrow their resident buffer metadata.
use super::*;
use crate::{adapter::layer_tree_host_impl::ContentColorUsage, runtime::types::CpuSkiaBackend};
use paint::cc::paint::{
    paint_flags::{Cap, Style},
    paint_op::{MetadataFlags, PaintOpMetadata, PaintOpMetadataFields},
    paint_op_buffer::PaintOpBufferMetadata,
    paint_op_buffer_iterator::{PaintOpType, PaintOpView},
};
use skia::include::core::SkPath::SkPath;

pub(crate) fn visit_shader_images(
    shader: &paint::paint_engine::PaintShader,
    visit: &mut dyn FnMut(u64),
) {
    if shader.resource_id != 0 {
        visit(shader.resource_id);
    }
    if let Some(pattern) = &shader.pattern {
        visit_pattern_images(&pattern.commands, visit);
    }
}
fn visit_pattern_images(
    commands: &[paint::paint_engine::SvgPatternCommand],
    visit: &mut dyn FnMut(u64),
) {
    use paint::paint_engine::SvgPatternCommand::*;
    for command in commands {
        match command {
            DrawImage(image) => visit(image.resource_id),
            DrawShape(shape) | DrawGlyph(shape) => {
                if let Some(shader) = &shape.fill_server {
                    visit_shader_images(shader, visit);
                }
                if let Some(shader) = &shape.stroke_server {
                    visit_shader_images(shader, visit);
                }
            }
            DrawText(text) => {
                if let Some(shader) = &text.paint.fill_server {
                    visit_shader_images(shader, visit);
                }
                if let Some(shader) = &text.paint.stroke_server {
                    visit_shader_images(shader, visit);
                }
            }
            SaveLayerMask(mask) => visit_pattern_images(&mask.commands, visit),
            _ => {}
        }
    }
}
pub(crate) fn shader_image_id<R: CpuSkiaBackend>(
    shader: &CpuPaintShader<R>,
) -> Option<(&PaintRecord, u64)> {
    let (source, mask) = shader.source()?;
    let resource = mask.map_or(source.op().resource_id, |i| {
        source.op().mask_layers[i].resource_id
    });
    match shader {
        CpuPaintShader::ImagePattern { .. } => Some((source.record(), resource)),
        _ => None,
    }
}
pub(crate) fn shader_has_images<R: CpuSkiaBackend>(shader: &CpuPaintShader<R>) -> bool {
    if shader_image_id(shader).is_some() {
        return true;
    }
    if let CpuPaintShader::PaintRecord { record, .. } = shader {
        return record
            .buffer()
            .is_some_and(|b| b.canonical_metadata().has_discardable_images());
    }
    let (source, mask) = shader.source().expect("native shader variant");
    let shader = mask.map_or(source.op().paint_shader.as_ref(), |i| {
        source.op().mask_layers[i].paint_shader.as_ref()
    });
    let mut result = false;
    if let Some(shader) = shader {
        visit_shader_images(shader, &mut |_| result = true);
    }
    result
}
pub(crate) fn flags<R: CpuSkiaBackend>(op: CpuPaintOpRef<'_, R>) -> Option<&CpuPaintFlags<R>> {
    match op {
        CpuPaintOpRef::Converted(
            CpuPaintOp::DrawPath { flags: f, .. }
            | CpuPaintOp::DrawRect(_, f)
            | CpuPaintOp::DrawRRect(_, f)
            | CpuPaintOp::DrawDRRect { flags: f, .. }
            | CpuPaintOp::SaveLayer { flags: f, .. }
            | CpuPaintOp::SaveLayerFilters { flags: f, .. },
        ) => Some(f),
        _ => None,
    }
}
fn image_usage(record: &PaintRecord, resource: u64, usage: &mut ContentColorUsage) -> bool {
    let image = CpuPaintImage::FromPaintResource(record, resource)
        .expect("PaintInput validates the actual resource and its native facts");
    *usage = (*usage).max(image.0.ContentColorUsage());
    // The actual CPU bitmap owner is not texture-backed. Source cc counts an
    // eager bitmap too; lazy generation controls decoding hints, not this bit.
    true
}
pub(crate) fn flag_images<R: CpuSkiaBackend>(
    op: CpuPaintOpRef<'_, R>,
    usage: &mut ContentColorUsage,
) -> bool {
    if let Some(f) = flags(op) {
        return f.shader.as_deref().is_some_and(shader_has_images)
            || f.image_filter
                .as_ref()
                .is_some_and(|f| filter_has_images(f.as_ref()));
    }
    if let CpuPaintOpRef::Native { op, record } = op {
        let mut any = false;
        if let Some(shader) = &op.paint_shader {
            visit_shader_images(shader, &mut |id| {
                any |= image_usage(record, id, usage);
            });
        }
        // Scalar CSS filters contain no image inputs. Reference/reflection
        // graphs remain typed producer blockers until their replay owner exists.
        any
    } else {
        false
    }
}
pub(crate) fn filter_has_images(
    filter: &dyn paint::cc::paint::filter_operation::PaintFilter,
) -> bool {
    use paint::cc::paint::filter_operation::PaintFilterReplay::*;
    match filter.ReplayGraph() {
        Operations(ops) => ops
            .operations()
            .iter()
            .filter_map(|o| o.image_filter().as_deref())
            .any(filter_has_images),
        Reference(_) | Reflection(_) => true,
    }
}
pub(crate) struct MetadataFields<'a, R: CpuSkiaBackend> {
    op: CpuPaintOpRef<'a, R>,
    path: SkPath,
}
impl<'a, R: CpuSkiaBackend> MetadataFields<'a, R> {
    pub(crate) fn new(op: CpuPaintOpRef<'a, R>) -> Self {
        let path = match op {
            CpuPaintOpRef::Native { op, .. } if !op.path.is_empty() => {
                paint::paint_clip_path::PaintClipPath::from_authored(
                    &op.path,
                    &[],
                    op.even_odd,
                    Default::default(),
                )
                .single_path()
                .expect("one native path")
                .as_ref()
                .clone()
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawPath { path, .. }) => path.as_ref().clone(),
            CpuPaintOpRef::Converted(CpuPaintOp::ClipPath { path, .. }) => path
                .single_path()
                .expect("PaintInput validates native union path capability")
                .as_ref()
                .clone(),
            _ => SkPath::default(),
        };
        Self { op, path }
    }
}
pub(crate) struct MetadataFlagView<'a, R: CpuSkiaBackend>(CpuPaintOpRef<'a, R>);
impl<R: CpuSkiaBackend> MetadataFlags<ContentColorUsage> for MetadataFlagView<'_, R> {
    fn IsValid(&self) -> bool {
        flags(self.0).is_none_or(|f| f.IsValid())
    }
    fn isAntiAlias(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => op.antialias,
            _ => flags(self.0).is_some_and(|f| f.isAntiAlias()),
        }
    }
    fn HasPathEffect(&self) -> bool {
        matches!(self.0,CpuPaintOpRef::Native{op,..} if !op.dash_intervals.is_empty())
    }
    fn DashIntervalCount(&self) -> i32 {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => op.dash_intervals.len() as i32,
            _ => 0,
        }
    }
    fn StrokeCapIsRound(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => op.round_cap,
            _ => flags(self.0).is_some_and(|f| f.getStrokeCap() == Cap::kRound_Cap),
        }
    }
    fn IsStroke(&self) -> bool {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => matches!(
                op.r#type,
                paint::paint_engine::DisplayItemType::kStrokePath
                    | paint::paint_engine::DisplayItemType::kStrokeRect
                    | paint::paint_engine::DisplayItemType::kStrokeEllipse
                    | paint::paint_engine::DisplayItemType::kStrokeLine
                    | paint::paint_engine::DisplayItemType::kStrokeWavyLine
            ),
            _ => flags(self.0).is_some_and(|f| f.getStyle() == Style::kStroke_Style),
        }
    }
    fn StrokeWidth(&self) -> f32 {
        match self.0 {
            CpuPaintOpRef::Native { op, .. } => op.stroke_width as f32,
            _ => flags(self.0).map_or(0., |f| f.getStrokeWidth()),
        }
    }
    fn HasDiscardableImages(&self, usage: &mut ContentColorUsage) -> bool {
        flag_images(self.0, usage)
    }
}
impl<R: CpuSkiaBackend> PaintOpMetadataFields<ContentColorUsage> for MetadataFields<'_, R> {
    type Flags<'a>
        = MetadataFlagView<'a, R>
    where
        Self: 'a;
    fn GetType(&self) -> PaintOpType {
        self.op.GetType()
    }
    fn Flags(&self) -> Option<Self::Flags<'_>> {
        self.op
            .IsPaintOpWithFlags()
            .then_some(MetadataFlagView(self.op))
    }
    fn IsValidGeometry(&self) -> bool {
        self.path.isFinite()
    }
    fn ClipAntialias(&self) -> bool {
        match self.op {
            CpuPaintOpRef::Native { op, .. } => op.antialias,
            CpuPaintOpRef::Converted(
                CpuPaintOp::ClipPath { antialias, .. } | CpuPaintOp::ClipRect { antialias, .. },
            ) => *antialias,
            _ => false,
        }
    }
    fn NestedBytesUsed(&self) -> usize {
        match self.op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => r.buffer().map_or(0, |b| {
                b.canonical_metadata()
                    .bytes_used(std::mem::size_of_val(b), b.allocation_capacity())
            }),
            CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => {
                let b = list.paint_op_buffer();
                b.canonical_metadata()
                    .bytes_used(std::mem::size_of_val(b), b.allocation_capacity())
            }
            _ => 0,
        }
    }
    fn Path(&self) -> &SkPath {
        &self.path
    }
    fn NestedMetadata(&self) -> &PaintOpBufferMetadata<ContentColorUsage> {
        match self.op {
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(CpuPaintRecord::Buffer(b))) => {
                b.canonical_metadata()
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(CpuPaintRecord::Empty)) => {
                static EMPTY: std::sync::LazyLock<PaintOpBufferMetadata<ContentColorUsage>> =
                    std::sync::LazyLock::new(|| {
                        PaintOpBufferMetadata::new(ContentColorUsage::kSRGB)
                    });
                &EMPTY
            }
            CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => {
                list.paint_op_buffer().canonical_metadata()
            }
            _ => unreachable!("native nested metadata requires a nonempty record"),
        }
    }
    fn IsDrawOp(&self) -> bool {
        self.op.IsDrawOp()
    }
    fn IsDiscardableImage(&self, usage: &mut ContentColorUsage) -> bool {
        match self.op {
            CpuPaintOpRef::Native { op, record }
                if op.r#type == paint::paint_engine::DisplayItemType::kDrawImageRect =>
            {
                image_usage(record, op.resource_id, usage)
            }
            _ => false,
        }
    }
}
pub(crate) fn analyze<R: CpuSkiaBackend>(
    op: CpuPaintOpRef<'_, R>,
    metadata: &mut PaintOpBufferMetadata<ContentColorUsage>,
) {
    metadata.AnalyzeAddedOp(&PaintOpMetadata(&MetadataFields::new(op)));
}
pub(crate) fn project<R: CpuSkiaBackend>(buffer: &CpuPaintOpBuffer<R>) -> CpuBufferMetadata {
    let m = buffer.canonical_metadata();
    CpuBufferMetadata {
        num_slow_paths_up_to_min_for_MSAA: m.num_slow_paths_up_to_min_for_MSAA(),
        has_non_aa_paint: m.has_non_aa_paint(),
        total_op_count: m.total_op_count(),
        bytes_used: m.bytes_used(std::mem::size_of_val(buffer), buffer.allocation_capacity()),
        paint_ops_size: m.paint_ops_size(),
        has_draw_ops: m.has_draw_ops(),
        has_draw_text_ops: m.has_draw_text_ops(),
        has_save_layer_ops: m.has_save_layer_ops(),
        has_save_layer_alpha_ops: m.has_save_layer_alpha_ops(),
        has_effects_preventing_lcd_text_for_save_layer_alpha: m
            .has_effects_preventing_lcd_text_for_save_layer_alpha(),
        has_discardable_images: m.has_discardable_images(),
        content_color_usage: m.content_color_usage(),
    }
}
