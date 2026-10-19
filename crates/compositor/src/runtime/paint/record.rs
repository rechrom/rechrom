//! Actual converted PaintOp ownership. Native records keep their producer Arc
//! and range. Iterators project one resident operation at a time; no wrapper
//! list, property-tree copy or retained compositor owner is created.
use super::{CpuPaintWorkletInputs, CpuWorkletInput, PaintOp, PaintRecord};
use crate::{
    adapter::{
        display_item_list::*,
        geometry::{Rect, RectF, Vector2dF},
        input_types::ElementId,
        layer_tree_host_impl::ContentColorUsage,
        layer_tree_impl::Color4f,
        prioritized_tile::ScrollOffsetMap,
    },
    cc::paint::display_item_list::{DisplayItemList, RasterInducingScrollInfo},
    runtime::{geometry::Transform, types::CpuSkiaBackend},
};
use std::{collections::BTreeMap, marker::PhantomData, sync::Arc};
// Source: cc/paint/paint_flags.h:75-82. This is the full native cc quality
// vocabulary; layout InterpolationQuality is a distinct producer enum.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CpuFilterQuality {
    kNone,
    #[default]
    kLow,
    kMedium,
    kHigh,
}
pub type TargetColorParams = crate::adapter::layer_tree_host_impl::HostTargetColorParams<
    crate::runtime::resources::CpuColorSpace,
>;
pub type ImageDecodingMode = crate::adapter::layer_tree_host_impl::HostImageDecodingMode;
/// The external immutable image allocation supplies actual source metadata.
/// It may own decoded/encoded image resources; it cannot hold compositor owners.
pub trait CpuPaintImageValue: Send + Sync {
    /// The source PaintImage GetSwSkImage allocation, with actual premul pixels.
    fn GetSwSkImage(&self) -> std::io::Result<Arc<Vec<u8>>>;
    fn GetPaintRecord(&self) -> Option<PaintRecord>;
    fn stable_id(&self) -> crate::adapter::image_animation::PaintImageId;
    fn ShouldAnimate(&self) -> bool;
    fn IsLazyGenerated(&self) -> bool;
    fn IsComplete(&self) -> bool;
    fn IsMultipart(&self) -> bool;
    fn size(&self) -> crate::adapter::geometry::Size;
    fn animation_metadata(&self)
        -> Option<&crate::adapter::image_animation::AnimatedImageMetadata>;
    fn ContentColorUsage(&self) -> ContentColorUsage;
    fn WorkletInput(&self) -> Option<&CpuWorkletInput>;
    fn GetReinterpretAsSRGB(&self) -> bool;
}
#[derive(Clone)]
pub struct CpuPaintImage(pub Arc<dyn CpuPaintImageValue>);
#[derive(Clone)]
pub struct CpuDrawImage {
    pub image: CpuPaintImage,
    pub source_rect: Rect,
    pub filter_quality: CpuFilterQuality,
    pub use_dark_mode: bool,
    pub scale: Vector2dF,
    pub matrix_is_decomposable: bool,
    pub frame_index: usize,
    pub target: Option<TargetColorParams>,
}
/// Immutable native image-map allocation supplied by true Paint image analysis.
/// All returned references belong to this allocation, without wrapper copies.
pub trait CpuDiscardableImageMapValue: Send + Sync {
    fn empty(&self) -> bool;
    fn animated_images_metadata(
        &self,
    ) -> &BTreeMap<
        crate::adapter::image_animation::PaintImageId,
        crate::adapter::image_animation::AnimatedImageMetadata,
    >;
    fn GetRectsForImage(&self, id: crate::adapter::image_animation::PaintImageId) -> &[Rect];
    fn GetDiscardableImagesInRect(&self, rect: &Rect) -> Vec<&CpuDrawImage>;
}
pub struct CpuDiscardableImageMap(pub Arc<dyn CpuDiscardableImageMapValue>);
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuBufferMetadata {
    pub num_slow_paths_up_to_min_for_MSAA: i32,
    pub has_non_aa_paint: bool,
    pub total_op_count: usize,
    pub bytes_used: usize,
    pub paint_ops_size: usize,
    pub has_draw_ops: bool,
    pub has_draw_text_ops: bool,
    pub has_save_layer_ops: bool,
    pub has_save_layer_alpha_ops: bool,
    pub has_effects_preventing_lcd_text_for_save_layer_alpha: bool,
    pub has_discardable_images: bool,
    pub content_color_usage: ContentColorUsage,
}
pub type CpuPaintFilter = Arc<dyn paint::cc::paint::filter_operation::PaintFilter>;
/// Canonical Skia image-filter input graph. Operations own their real immutable
/// references; this is not a second authored projection or executable cache.
#[derive(Debug)]
pub struct CpuOperationsFilter(pub paint::cc::paint::filter_operations::FilterOperations);
impl paint::cc::paint::filter_operation::PaintFilter for CpuOperationsFilter {
    fn MapRect(
        &self,
        rect: foundation::gfx::Rect,
        ctm: Option<&skia::include::core::SkRRect::SkMatrix3x3>,
        direction: paint::cc::paint::filter_operation::MapDirection,
    ) -> foundation::gfx::Rect {
        match direction {
            paint::cc::paint::filter_operation::MapDirection::Forward => self.0.MapRect(rect, ctm),
            paint::cc::paint::filter_operation::MapDirection::Reverse => self.0.MapRectReverse(
                rect,
                ctm.unwrap_or(&skia::include::core::SkRRect::SkMatrix3x3 {
                    values: [1., 0., 0., 0., 1., 0., 0., 0., 1.],
                }),
            ),
        }
    }
    fn TypeToString(&self) -> &'static str {
        "CompositorFilterOperations"
    }
    fn ReplayGraph(&self) -> paint::cc::paint::filter_operation::PaintFilterReplay<'_> {
        paint::cc::paint::filter_operation::PaintFilterReplay::Operations(&self.0)
    }
}
/// Runtime Paint flags retain the complete canonical scalar state. Image-filter
/// values remain the actual immutable external Paint graph; this owner never
/// projects a layer/tree or stores a raster execution capability.
pub struct CpuPaintFlags<R: CpuSkiaBackend = crate::runtime::skia::SkRegion> {
    core: paint::cc::paint::paint_flags::CorePaintFlags,
    pub image_filter: Option<CpuPaintFilter>,
    pub shader: Option<Arc<super::CpuPaintShader<R>>>,
    pub color_filter: Option<super::CpuPaintColorFilter>,
    pub looper: Option<Arc<paint::cc::paint::draw_looper::DrawLooper>>,
}
impl<R: CpuSkiaBackend> Default for CpuPaintFlags<R> {
    fn default() -> Self {
        Self {
            core: Default::default(),
            image_filter: None,
            shader: None,
            color_filter: None,
            looper: None,
        }
    }
}
impl<R: CpuSkiaBackend> Clone for CpuPaintFlags<R> {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            image_filter: self.image_filter.clone(),
            shader: self.shader.clone(),
            color_filter: self.color_filter,
            looper: self.looper.clone(),
        }
    }
}
impl<R: CpuSkiaBackend> std::ops::Deref for CpuPaintFlags<R> {
    type Target = paint::cc::paint::paint_flags::CorePaintFlags;
    fn deref(&self) -> &Self::Target {
        &self.core
    }
}
impl<R: CpuSkiaBackend> std::ops::DerefMut for CpuPaintFlags<R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}
/// Pure value/image metadata and true Paint solid analysis only. Each query
/// borrows the actual buffer. No tile/layerization/invalidation policy lives here.
pub trait CpuPaintValueServices<R: CpuSkiaBackend>: Send + Sync {
    fn Metadata(&self, buffer: &CpuPaintOpBuffer<R>) -> CpuBufferMetadata;
    fn AnalyzeAddedOp(
        &self,
        op: CpuPaintOpRef<'_, R>,
        metadata: &mut paint::cc::paint::paint_op_buffer::PaintOpBufferMetadata<ContentColorUsage>,
    );
    fn NeedsAdditionalInvalidationForLCDText(
        &self,
        new: &CpuPaintOpBuffer<R>,
        old: &CpuPaintOpBuffer<R>,
    ) -> bool;
    fn GenerateDiscardableImageMap(
        &self,
        buffer: &CpuPaintOpBuffer<R>,
        bounds: Rect,
        scrolls: &ScrollOffsetMap,
        decoding: Option<
            &mut BTreeMap<crate::adapter::image_animation::PaintImageId, ImageDecodingMode>,
        >,
        worklets: Option<&mut CpuPaintWorkletInputs>,
    ) -> CpuDiscardableImageMap;
    fn DetermineIfSolidColor(
        &self,
        buffer: &CpuPaintOpBuffer<R>,
        rect: Rect,
        max_ops: i32,
        offsets: Option<&[usize]>,
    ) -> Option<Color4f>;
    fn TraceSnapshot(
        &self,
        buffer: &CpuPaintOpBuffer<R>,
        bounds: Option<Rect>,
        all_bounds: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<ElementId, RasterInducingScrollInfo>,
    );
    fn ToTraceString(
        &self,
        buffer: &CpuPaintOpBuffer<R>,
        bounds: Option<Rect>,
        all_bounds: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<ElementId, RasterInducingScrollInfo>,
    ) -> String;
    fn TraceRasterBegin(&self, total_ops: usize) -> bool;
    fn TraceRasterEnd(&self, rastered_ops: usize);
}
/// Raster consumes this stream directly. Native variants borrow the original
/// Paint replay ABI; converted variants retain the canonical cc paint commands.
pub trait CpuPaintCanvas<R: CpuSkiaBackend>: DisplayCanvas {
    fn Playback(
        &mut self,
        buffer: &CpuPaintOpBuffer<R>,
        params: &mut CpuPlaybackParams<'_, R>,
        local_ctm: bool,
        offsets: Option<&[usize]>,
    );
}
/// Source PlaybackParams owns no provider: the raw mutable pointee is an
/// exclusive, call-scoped Rust borrow. Mutable envelope propagation preserves
/// synchronous nested-record access without RefCell or a retained callback.
pub struct CpuPlaybackParams<'a, R: CpuSkiaBackend> {
    pub raster_inducing_scroll_offsets: Option<&'a ScrollOffsetMap>,
    pub image_provider: Option<&'a mut dyn super::CpuImageProvider<R>>,
}
impl<R: CpuSkiaBackend> Default for CpuPlaybackParams<'_, R> {
    fn default() -> Self {
        Self {
            raster_inducing_scroll_offsets: None,
            image_provider: None,
        }
    }
}

pub enum CpuPaintOp<R: CpuSkiaBackend> {
    NativeAtom(super::NativePaintSource),
    DrawRect(RectF, CpuPaintFlags<R>),
    DrawPath {
        path: Arc<skia::include::core::SkPath::SkPath>,
        flags: CpuPaintFlags<R>,
    },
    DrawDRRect {
        outer: crate::adapter::geometry::RRectF,
        inner: crate::adapter::geometry::RRectF,
        flags: CpuPaintFlags<R>,
    },
    DrawRRect(crate::adapter::geometry::RRectF, CpuPaintFlags<R>),
    Save,
    Restore,
    Translate(f32, f32),
    Concat(Transform<R>),
    ClipRect {
        rect: RectF,
        difference: bool,
        antialias: bool,
    },
    ClipRRect(crate::adapter::geometry::RRectF, bool),
    ClipPath {
        path: Arc<paint::paint_clip_path::PaintClipPath>,
        difference: bool,
        antialias: bool,
    },
    SaveLayer {
        bounds: Option<RectF>,
        flags: CpuPaintFlags<R>,
    },
    SaveLayerAlpha {
        bounds: Option<RectF>,
        alpha: f32,
    },
    SaveLayerFilters {
        bounds: Option<RectF>,
        filters: Vec<CpuPaintFilter>,
        backdrop_filter: CpuPaintFilter,
        flags: CpuPaintFlags<R>,
    },
    DrawTransparentColorSrc,
    DrawColor {
        color: Color4f,
        blend: skia::BlendMode,
    },
    DrawRecord(CpuPaintRecord<R>),
    DrawScrollingContents {
        element: ElementId,
        list: Arc<DisplayItemList<CpuPaintTypes<R>>>,
    },
    Noop,
}
pub enum CpuPaintStorage<R: CpuSkiaBackend> {
    Native(PaintRecord),
    Converted(Vec<CpuPaintOp<R>>),
}
pub struct CpuPaintOpBuffer<R: CpuSkiaBackend> {
    storage: CpuPaintStorage<R>,
    values: Arc<dyn CpuPaintValueServices<R>>,
    metadata: paint::cc::paint::paint_op_buffer::PaintOpBufferMetadata<ContentColorUsage>,
}
impl<R: CpuSkiaBackend> CpuPaintOpBuffer<R> {
    pub fn new(values: Arc<dyn CpuPaintValueServices<R>>) -> Self {
        Self {
            storage: CpuPaintStorage::Converted(Vec::new()),
            metadata: paint::cc::paint::paint_op_buffer::PaintOpBufferMetadata::new(
                ContentColorUsage::kSRGB,
            ),
            values,
        }
    }
    pub fn from_native(record: PaintRecord, values: Arc<dyn CpuPaintValueServices<R>>) -> Self {
        let mut result = Self::new(values);
        for op in super::lowering::LowerPaintRecord(record, result.values.clone()) {
            result.push(op);
        }
        result
    }
    pub fn canonical_metadata(
        &self,
    ) -> &paint::cc::paint::paint_op_buffer::PaintOpBufferMetadata<ContentColorUsage> {
        &self.metadata
    }
    pub fn allocation_capacity(&self) -> usize {
        match &self.storage {
            CpuPaintStorage::Converted(v) => v.capacity() * std::mem::size_of::<CpuPaintOp<R>>(),
            CpuPaintStorage::Native(_) => 0,
        }
    }
    pub fn storage(&self) -> &CpuPaintStorage<R> {
        &self.storage
    }
    pub fn values(&self) -> &Arc<dyn CpuPaintValueServices<R>> {
        &self.values
    }
    fn converted_mut(&mut self) -> &mut Vec<CpuPaintOp<R>> {
        match &mut self.storage {
            CpuPaintStorage::Converted(ops) => ops,
            CpuPaintStorage::Native(_) => panic!("immutable native record cannot be edited"),
        }
    }
}
/// An empty default is source PaintRecord's actual empty value. Other variants
/// retain their real buffer allocation; cloning never copies replay operations.
pub enum CpuPaintRecord<R: CpuSkiaBackend> {
    Empty,
    Buffer(Arc<CpuPaintOpBuffer<R>>),
}
impl<R: CpuSkiaBackend> Clone for CpuPaintRecord<R> {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Buffer(b) => Self::Buffer(b.clone()),
        }
    }
}
impl<R: CpuSkiaBackend> Default for CpuPaintRecord<R> {
    fn default() -> Self {
        Self::Empty
    }
}
impl<R: CpuSkiaBackend> CpuPaintRecord<R> {
    pub fn from_native(record: PaintRecord, values: Arc<dyn CpuPaintValueServices<R>>) -> Self {
        Self::Buffer(Arc::new(CpuPaintOpBuffer::from_native(record, values)))
    }
    pub fn buffer(&self) -> Option<&CpuPaintOpBuffer<R>> {
        match self {
            Self::Empty => None,
            Self::Buffer(b) => Some(b),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.buffer().is_none_or(|b| b.size() == 0)
    }
}
pub struct CpuPaintTypes<R: CpuSkiaBackend>(PhantomData<fn() -> R>);
pub enum CpuPaintOpRef<'a, R: CpuSkiaBackend> {
    Native {
        op: &'a PaintOp,
        record: &'a PaintRecord,
    },
    Converted(&'a CpuPaintOp<R>),
}
pub struct CpuPaintIter<'a, R: CpuSkiaBackend> {
    buffer: &'a CpuPaintOpBuffer<R>,
    offsets: Option<&'a [usize]>,
    index: usize,
}
impl<'a, R: CpuSkiaBackend> Iterator for CpuPaintIter<'a, R> {
    type Item = CpuPaintOpRef<'a, R>;
    fn next(&mut self) -> Option<Self::Item> {
        let index = match self.offsets {
            Some(offsets) => *offsets.get(self.index)?,
            None => self.index,
        };
        let result = match &self.buffer.storage {
            CpuPaintStorage::Native(record) => CpuPaintOpRef::Native {
                op: record.ops().get(index)?,
                record,
            },
            CpuPaintStorage::Converted(ops) => ops.get(index)?.AsRef(),
        };
        self.index += 1;
        Some(result)
    }
}

impl<R: CpuSkiaBackend> DisplayItemListTypes for CpuPaintTypes<R> {
    type PaintOp = CpuPaintOp<R>;
    type PaintOpBuffer = CpuPaintOpBuffer<R>;
    type Canvas = dyn CpuPaintCanvas<R>;
    type PlaybackParams<'a> = CpuPlaybackParams<'a, R>;
    type PaintRecord = CpuPaintRecord<R>;
    type SaveLayerBounds = RectF;
    type NodeInfo = crate::runtime::display_item_list::NodeInfo;
    type DiscardableImageMap = CpuDiscardableImageMap;
    type DecodingModeMap =
        BTreeMap<crate::adapter::image_animation::PaintImageId, ImageDecodingMode>;
    type PaintWorkletInputs = CpuPaintWorkletInputs;
    fn PlaybackParams<'a>(scrolls: Option<&'a ScrollOffsetMap>) -> CpuPlaybackParams<'a, R> {
        CpuPlaybackParams {
            raster_inducing_scroll_offsets: scrolls,
            image_provider: None,
        }
    }
    fn MakeDrawScrollingContentsOp(
        element: ElementId,
        list: Arc<DisplayItemList<Self>>,
    ) -> CpuPaintOp<R> {
        CpuPaintOp::DrawScrollingContents { element, list }
    }
}
impl<R: CpuSkiaBackend> DisplayPaintOp<CpuPaintTypes<R>> for CpuPaintOpRef<'_, R> {
    fn GetType(&self) -> DisplayPaintOpType {
        use paint::paint_engine::DisplayItemType as T;
        use DisplayPaintOpType as D;
        match self {
            Self::Native { op, .. } => match op.r#type {
                T::kSave => D::Save,
                T::kRestore => D::Restore,
                T::kConcat => D::Concat,
                T::kDrawImageRect => D::DrawImageRect,
                T::kDrawGlyphRun => D::DrawTextBlob,
                _ => D::Other,
            },
            Self::Converted(op) => match op {
                CpuPaintOp::Save => D::Save,
                CpuPaintOp::Restore => D::Restore,
                CpuPaintOp::Translate(..) => D::Translate,
                CpuPaintOp::Concat(_) => D::Concat,
                CpuPaintOp::DrawRecord(_) => D::DrawRecord,
                _ => D::Other,
            },
        }
    }
    fn AdditionalOpCount(&self) -> usize {
        match self {
            Self::Converted(CpuPaintOp::DrawRecord(record)) => {
                record.buffer().map_or(0, |b| b.total_op_count())
            }
            Self::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => list.TotalOpCount(),
            _ => 0,
        }
    }
    fn HasDrawTextOps(&self) -> bool {
        match self {
            Self::Native { op, .. } => {
                op.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun
            }
            Self::Converted(CpuPaintOp::DrawRecord(record)) => {
                record.buffer().is_some_and(|b| b.has_draw_text_ops())
            }
            Self::Converted(CpuPaintOp::DrawScrollingContents { list, .. }) => {
                list.has_draw_text_ops()
            }
            _ => false,
        }
    }
    fn TextNodeId(&self) -> Option<u64> {
        match self {
            Self::Native { op, .. }
                if op.r#type == paint::paint_engine::DisplayItemType::kDrawGlyphRun =>
            {
                Some(op.node_id)
            }
            _ => None,
        }
    }
    fn RecordBuffer(&self) -> Option<&CpuPaintOpBuffer<R>> {
        match self {
            Self::Converted(CpuPaintOp::DrawRecord(record)) => record.buffer(),
            _ => None,
        }
    }
    fn ConcatPreserves2DAxisAlignment(&self) -> bool {
        match self {
            Self::Native { op, .. } => {
                let m = Transform::<R>::FromColMajor(&op.transform.values);
                crate::cc::base::math_util::MathUtil::SkM44Preserves2DAxisAlignment(&Matrix44Ref(
                    &m,
                ))
            }
            Self::Converted(CpuPaintOp::Concat(m)) => {
                crate::cc::base::math_util::MathUtil::SkM44Preserves2DAxisAlignment(&Matrix44Ref(m))
            }
            _ => panic!("source ConcatOp required"),
        }
    }
    fn ConcatTransposesAxes(&self) -> bool {
        match self {
            Self::Native { op, .. } => op.transform.values[0] as f32 == 0.,
            Self::Converted(CpuPaintOp::Concat(m)) => m.native().rc(0, 0) as f32 == 0.,
            _ => panic!("source ConcatOp required"),
        }
    }
    fn DrawImageRectGeometry(&self) -> Option<(f32, f32, f32, f32)> {
        match self {
            Self::Native { op, .. }
                if op.r#type == paint::paint_engine::DisplayItemType::kDrawImageRect =>
            {
                Some((
                    op.source_rect.width as f32,
                    op.source_rect.height as f32,
                    op.rect.width as f32,
                    op.rect.height as f32,
                ))
            }
            _ => None,
        }
    }
    fn DrawImageUsesNearestNeighbor(&self) -> bool {
        match self {
            Self::Native { op, .. } => {
                op.image_interpolation_quality
                    == foundation::graphics_enums::InterpolationQuality::kInterpolationNone
            }
            _ => panic!("source DrawImageRectOp required"),
        }
    }
}
impl<R: CpuSkiaBackend> DisplayPaintOp<CpuPaintTypes<R>> for CpuPaintOp<R> {
    fn GetType(&self) -> DisplayPaintOpType {
        CpuPaintOpRef::Converted(self).GetType()
    }
    fn AdditionalOpCount(&self) -> usize {
        CpuPaintOpRef::Converted(self).AdditionalOpCount()
    }
    fn HasDrawTextOps(&self) -> bool {
        CpuPaintOpRef::Converted(self).HasDrawTextOps()
    }
    fn TextNodeId(&self) -> Option<u64> {
        CpuPaintOpRef::Converted(self).TextNodeId()
    }
    fn RecordBuffer(&self) -> Option<&CpuPaintOpBuffer<R>> {
        match self {
            Self::DrawRecord(r) => r.buffer(),
            _ => None,
        }
    }
    fn ConcatPreserves2DAxisAlignment(&self) -> bool {
        CpuPaintOpRef::Converted(self).ConcatPreserves2DAxisAlignment()
    }
    fn ConcatTransposesAxes(&self) -> bool {
        CpuPaintOpRef::Converted(self).ConcatTransposesAxes()
    }
    fn DrawImageRectGeometry(&self) -> Option<(f32, f32, f32, f32)> {
        CpuPaintOpRef::Converted(self).DrawImageRectGeometry()
    }
    fn DrawImageUsesNearestNeighbor(&self) -> bool {
        CpuPaintOpRef::Converted(self).DrawImageUsesNearestNeighbor()
    }
}
impl<R: CpuSkiaBackend> DisplayPaintOpBuffer<CpuPaintTypes<R>> for CpuPaintOpBuffer<R> {
    type OpRef<'a> = CpuPaintOpRef<'a, R>;
    type Iter<'a> = CpuPaintIter<'a, R>;
    type OffsetIter<'a> = CpuPaintIter<'a, R>;
    type PlaybackItem<'a> =
        paint::cc::paint::paint_op_buffer_iterator::PlaybackItem<CpuPaintOpRef<'a, R>>;
    type PlaybackFoldingIter<'a> =
        paint::cc::paint::paint_op_buffer_iterator::PlaybackFoldingIterator<'a, Self>;
    fn iter(&self) -> Self::Iter<'_> {
        CpuPaintIter {
            buffer: self,
            offsets: None,
            index: 0,
        }
    }
    fn iter_offsets<'a>(&'a self, offsets: &'a [usize]) -> Self::OffsetIter<'a> {
        CpuPaintIter {
            buffer: self,
            offsets: Some(offsets),
            index: 0,
        }
    }
    fn playback_folding_iter<'a>(&'a self, offsets: &'a [usize]) -> Self::PlaybackFoldingIter<'a> {
        paint::cc::paint::paint_op_buffer_iterator::PlaybackFoldingIterator::new(
            self,
            Some(offsets),
        )
    }
    fn size(&self) -> usize {
        match &self.storage {
            CpuPaintStorage::Native(record) => record.len(),
            CpuPaintStorage::Converted(ops) => ops.len(),
        }
    }
    fn next_op_offset(&self) -> usize {
        self.size()
    }
    fn push(&mut self, op: CpuPaintOp<R>) -> usize {
        // Source paint_op_buffer.h:285-310,370-371: account once at insertion.
        self.metadata
            .DidAllocatePaintOp(std::mem::size_of::<CpuPaintOp<R>>());
        self.values.AnalyzeAddedOp(op.AsRef(), &mut self.metadata);
        let ops = self.converted_mut();
        let offset = ops.len();
        ops.push(op);
        offset
    }
    fn UpdateSaveLayerBounds(&mut self, id: usize, rect: RectF) {
        match &mut self.converted_mut()[id] {
            CpuPaintOp::SaveLayer { bounds, .. }
            | CpuPaintOp::SaveLayerAlpha { bounds, .. }
            | CpuPaintOp::SaveLayerFilters { bounds, .. } => *bounds = Some(rect),
            _ => panic!("source save-layer operation expected"),
        }
    }
    fn Playback(
        &self,
        canvas: &mut (dyn CpuPaintCanvas<R> + 'static),
        params: &mut CpuPlaybackParams<'_, R>,
        local_ctm: bool,
        offsets: Option<&[usize]>,
    ) {
        canvas.Playback(self, params, local_ctm, offsets)
    }
    fn ReleaseAsRecord(&mut self) -> CpuPaintRecord<R> {
        let ops = std::mem::take(self.converted_mut());
        let metadata = std::mem::replace(
            &mut self.metadata,
            paint::cc::paint::paint_op_buffer::PaintOpBufferMetadata::new(ContentColorUsage::kSRGB),
        );
        CpuPaintRecord::Buffer(Arc::new(Self {
            storage: CpuPaintStorage::Converted(ops),
            metadata,
            values: self.values.clone(),
        }))
    }
    fn ShrinkToFit(&mut self) {
        self.converted_mut().shrink_to_fit()
    }
    fn num_slow_paths_up_to_min_for_MSAA(&self) -> i32 {
        self.metadata.num_slow_paths_up_to_min_for_MSAA()
    }
    fn has_non_aa_paint(&self) -> bool {
        self.metadata.has_non_aa_paint()
    }
    fn total_op_count(&self) -> usize {
        self.metadata.total_op_count()
    }
    fn bytes_used(&self) -> usize {
        self.metadata
            .bytes_used(std::mem::size_of_val(self), self.allocation_capacity())
    }
    fn paint_ops_size(&self) -> usize {
        self.metadata.paint_ops_size()
    }
    fn has_draw_ops(&self) -> bool {
        self.metadata.has_draw_ops()
    }
    fn has_draw_text_ops(&self) -> bool {
        self.metadata.has_draw_text_ops()
    }
    fn has_save_layer_ops(&self) -> bool {
        self.metadata.has_save_layer_ops()
    }
    fn has_save_layer_alpha_ops(&self) -> bool {
        self.metadata.has_save_layer_alpha_ops()
    }
    fn has_effects_preventing_lcd_text_for_save_layer_alpha(&self) -> bool {
        self.metadata
            .has_effects_preventing_lcd_text_for_save_layer_alpha()
    }
    fn has_discardable_images(&self) -> bool {
        self.metadata.has_discardable_images()
    }
    fn content_color_usage(&self) -> ContentColorUsage {
        self.metadata.content_color_usage()
    }
    fn NeedsAdditionalInvalidationForLCDText(&self, old: &Self) -> bool {
        self.values.NeedsAdditionalInvalidationForLCDText(self, old)
    }
    fn GenerateDiscardableImageMap(
        &self,
        bounds: Rect,
        scrolls: &ScrollOffsetMap,
        decoding: Option<
            &mut BTreeMap<crate::adapter::image_animation::PaintImageId, ImageDecodingMode>,
        >,
        worklets: Option<&mut CpuPaintWorkletInputs>,
    ) -> CpuDiscardableImageMap {
        self.values
            .GenerateDiscardableImageMap(self, bounds, scrolls, decoding, worklets)
    }
    fn DetermineIfSolidColor(
        &self,
        rect: Rect,
        max_ops: i32,
        offsets: Option<&[usize]>,
    ) -> Option<Color4f> {
        self.values
            .DetermineIfSolidColor(self, rect, max_ops, offsets)
    }
    fn EmitTraceSnapshot(
        &self,
        bounds: Option<Rect>,
        all: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<ElementId, RasterInducingScrollInfo>,
    ) {
        self.values.TraceSnapshot(self, bounds, all, scrolls)
    }
    fn ToTraceString(
        &self,
        bounds: Option<Rect>,
        all: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<ElementId, RasterInducingScrollInfo>,
    ) -> String {
        self.values.ToTraceString(self, bounds, all, scrolls)
    }
    fn TraceRasterBegin(&self, total: usize) -> bool {
        self.values.TraceRasterBegin(total)
    }
    fn TraceRasterEnd(&self, rastered: usize) {
        self.values.TraceRasterEnd(rastered)
    }
}

struct Matrix44Ref<'a, R: CpuSkiaBackend>(&'a Transform<R>);
impl<R: CpuSkiaBackend> crate::adapter::geometry::Matrix44 for Matrix44Ref<'_, R> {
    fn rc(&self, row: usize, col: usize) -> f32 {
        self.0.native().rc(row, col) as f32
    }
}

impl crate::adapter::layer_tree_host_impl::HostPaintImage for CpuPaintImage {
    fn stable_id(&self) -> crate::adapter::image_animation::PaintImageId {
        self.0.stable_id()
    }
    fn ShouldAnimate(&self) -> bool {
        self.0.ShouldAnimate()
    }
}
impl crate::adapter::layer_tree_host_impl::HostDecodePaintImage for CpuPaintImage {
    fn GetContentColorUsage(&self) -> ContentColorUsage {
        self.0.ContentColorUsage()
    }
}
impl crate::adapter::layer_tree_host_impl::HostDrawImage for CpuDrawImage {
    type PaintImage = CpuPaintImage;
    fn paint_image(&self) -> &CpuPaintImage {
        &self.image
    }
}
impl crate::adapter::tile_manager::TileManagerPaintImage for CpuPaintImage {
    type Id = crate::adapter::image_animation::PaintImageId;
    type WorkletInput = CpuWorkletInput;
    fn stable_id(&self) -> Self::Id {
        self.0.stable_id()
    }
    fn ShouldAnimate(&self) -> bool {
        self.0.ShouldAnimate()
    }
    fn IsLazyGenerated(&self) -> bool {
        self.0.IsLazyGenerated()
    }
    fn IsComplete(&self) -> bool {
        self.0.IsComplete()
    }
    fn IsMultipart(&self) -> bool {
        self.0.IsMultipart()
    }
    fn IsPaintWorklet(&self) -> bool {
        self.0.WorkletInput().is_some()
    }
    fn GetPaintWorkletInput(&self) -> &Self::WorkletInput {
        self.0
            .WorkletInput()
            .expect("source DCHECK(IsPaintWorklet())")
    }
}
impl
    crate::adapter::tile_manager::TileManagerTargetColorParams<
        crate::runtime::resources::CpuColorSpace,
    > for TargetColorParams
{
    fn color_space(&self) -> &crate::runtime::resources::CpuColorSpace {
        &self.color_space
    }
    fn GetHdrHeadroom(&self) -> f32 {
        Self::GetHdrHeadroom(self)
    }
}
impl<
        B: crate::adapter::tile_manager::TileManagerTypes<
            PaintImage = CpuPaintImage,
            TargetColorParams = TargetColorParams,
        >,
    > crate::adapter::tile_manager::TileManagerDrawImage<B> for CpuDrawImage
{
    fn paint_image(&self) -> &CpuPaintImage {
        &self.image
    }
    // Source: cc/tiles/checker_image_tracker.cc:80-92. Overflow vetoes
    // checkering as the largest possible decode; negative dimensions cannot
    // convert to size_t and take the same checked-numeric failure branch.
    fn EstimatedDecodedBytes(&self) -> usize {
        usize::try_from(self.source_rect.width)
            .ok()
            .and_then(|w| {
                usize::try_from(self.source_rect.height)
                    .ok()
                    .and_then(|h| 4usize.checked_mul(w)?.checked_mul(h))
            })
            .unwrap_or(usize::MAX)
    }
    fn WithRasterParameters(
        &self,
        scale: f32,
        frame_index: usize,
        target: &TargetColorParams,
    ) -> Self {
        // Source: cc/paint/draw_image.cc:74-87. The external image owner has
        // already decomposed its original matrix; this constructor scales that value.
        let mut target = target.clone();
        // Source: cc/paint/draw_image.cc:105-113.
        if self.image.0.GetReinterpretAsSRGB() {
            target.color_space = Default::default();
        }
        Self {
            image: self.image.clone(),
            source_rect: self.source_rect,
            filter_quality: self.filter_quality,
            use_dark_mode: self.use_dark_mode,
            scale: Vector2dF::new(self.scale.x() * scale, self.scale.y() * scale),
            matrix_is_decomposable: self.matrix_is_decomposable,
            frame_index,
            target: Some(target),
        }
    }
}

impl<R: CpuSkiaBackend> Clone for CpuPaintOp<R> {
    fn clone(&self) -> Self {
        match self {
            Self::NativeAtom(source) => Self::NativeAtom(source.clone()),
            Self::DrawRect(r, f) => Self::DrawRect(*r, f.clone()),
            Self::DrawPath { path, flags } => Self::DrawPath {
                path: path.clone(),
                flags: flags.clone(),
            },
            Self::DrawDRRect {
                outer,
                inner,
                flags,
            } => Self::DrawDRRect {
                outer: *outer,
                inner: *inner,
                flags: flags.clone(),
            },
            Self::DrawRRect(r, f) => Self::DrawRRect(*r, f.clone()),
            Self::Save => Self::Save,
            Self::Restore => Self::Restore,
            Self::Translate(x, y) => Self::Translate(*x, *y),
            Self::Concat(m) => Self::Concat(m.clone()),
            Self::ClipRect {
                rect,
                difference,
                antialias,
            } => Self::ClipRect {
                rect: *rect,
                difference: *difference,
                antialias: *antialias,
            },
            Self::ClipRRect(r, a) => Self::ClipRRect(r.clone(), *a),
            Self::ClipPath {
                path,
                difference,
                antialias,
            } => Self::ClipPath {
                path: path.clone(),
                difference: *difference,
                antialias: *antialias,
            },
            Self::SaveLayer { bounds, flags } => Self::SaveLayer {
                bounds: *bounds,
                flags: flags.clone(),
            },
            Self::SaveLayerAlpha { bounds, alpha } => Self::SaveLayerAlpha {
                bounds: *bounds,
                alpha: *alpha,
            },
            Self::SaveLayerFilters {
                bounds,
                filters,
                backdrop_filter,
                flags,
            } => Self::SaveLayerFilters {
                bounds: *bounds,
                filters: filters.clone(),
                backdrop_filter: backdrop_filter.clone(),
                flags: flags.clone(),
            },
            Self::DrawTransparentColorSrc => Self::DrawTransparentColorSrc,
            Self::DrawColor { color, blend } => Self::DrawColor {
                color: *color,
                blend: *blend,
            },
            Self::DrawRecord(r) => Self::DrawRecord(r.clone()),
            Self::DrawScrollingContents { element, list } => Self::DrawScrollingContents {
                element: *element,
                list: list.clone(),
            },
            Self::Noop => Self::Noop,
        }
    }
}
impl<R: CpuSkiaBackend> CpuPaintOpBuffer<R> {
    /// Source PaintOpBuffer::DeepCopyAsRecord explicitly creates a new converted
    /// operation owner. Native record leases keep their original producer Arc;
    /// this does not clone a PaintArtifact, source item or property tree.
    pub fn DeepCopyAsRecord(&self) -> CpuPaintRecord<R> {
        let storage = match &self.storage {
            CpuPaintStorage::Native(r) => CpuPaintStorage::Native(r.clone()),
            CpuPaintStorage::Converted(ops) => CpuPaintStorage::Converted(ops.clone()),
        };
        CpuPaintRecord::Buffer(Arc::new(Self {
            storage,
            metadata: self.metadata.clone(),
            values: self.values.clone(),
        }))
    }
    pub fn RetainOpAt(&self, index: usize) -> CpuPaintOp<R> {
        match &self.storage {
            CpuPaintStorage::Converted(ops) => ops[index].clone(),
            CpuPaintStorage::Native(record) => CpuPaintOp::NativeAtom(
                super::lowering::NativePaintSource::At(record.clone(), index),
            ),
        }
    }
    pub fn AppendBuffer(&mut self, source: &Self) {
        match &source.storage {
            CpuPaintStorage::Native(_) => {
                self.push(CpuPaintOp::DrawRecord(source.DeepCopyAsRecord()));
            }
            CpuPaintStorage::Converted(ops) => {
                for op in ops {
                    self.push(op.clone());
                }
            }
        }
    }
}

impl<R: CpuSkiaBackend> Copy for CpuPaintOpRef<'_, R> {}
impl<R: CpuSkiaBackend> Clone for CpuPaintOpRef<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: CpuSkiaBackend> paint::cc::paint::paint_op_buffer_iterator::PaintOpBufferView
    for CpuPaintOpBuffer<R>
{
    type Op<'a> = CpuPaintOpRef<'a, R>;
    fn Used(&self) -> usize {
        self.size()
    }
    fn GetOp(&self, offset: usize) -> Self::Op<'_> {
        match &self.storage {
            CpuPaintStorage::Native(record) => CpuPaintOpRef::Native {
                op: &record.ops()[offset],
                record,
            },
            CpuPaintStorage::Converted(ops) => ops[offset].AsRef(),
        }
    }
    fn NextOffset(&self, offset: usize) -> usize {
        assert!(offset < self.size());
        offset + 1
    }
}
impl<'a, R: CpuSkiaBackend> paint::cc::paint::paint_op_buffer_iterator::PaintOpView<'a>
    for CpuPaintOpRef<'a, R>
{
    type Buffer = CpuPaintOpBuffer<R>;
    fn GetType(self) -> paint::cc::paint::paint_op_buffer_iterator::PaintOpType {
        use paint::cc::paint::paint_op_buffer_iterator::PaintOpType as T;
        use paint::paint_engine::DisplayItemType as N;
        match self {
            Self::Native { op, .. } => match op.r#type {
                N::kSave => T::kSave,
                N::kRestore => T::kRestore,
                N::kConcat => T::kConcat,
                N::kClipRect | N::kClipOutRect => T::kClipRect,
                N::kClipRoundedRect | N::kClipOutRoundedRect => T::kClipRRect,
                N::kClipPath => T::kClipPath,
                N::kSaveLayer | N::kSaveLayerBlend | N::kSaveLayerDstIn | N::kBeginMask => {
                    T::kSaveLayer
                }
                N::kSaveLayerAlpha => T::kSaveLayerAlpha,
                N::kSaveLayerFilter => T::kSaveLayerFilters,
                N::kEndMask => T::kRestore,
                N::kDrawRect
                | N::kStrokeRect
                | N::kDrawScrollbarTrack
                | N::kDrawScrollbarThumb
                | N::kDrawScrollbarButton
                | N::kDrawScrollbarCorner => T::kDrawRect,
                N::kDrawRoundedRect => T::kDrawRRect,
                N::kDrawDoubleRoundedRect => T::kDrawDRRect,
                N::kDrawEllipse | N::kStrokeEllipse => T::kDrawOval,
                N::kStrokeLine => T::kDrawLine,
                N::kDrawGlyphRun => T::kDrawTextBlob,
                N::kDrawImageRect => T::kDrawImageRect,
                N::kDrawPath | N::kStrokePath => T::kDrawPath,
                N::kStrokeWavyLine
                | N::kDrawGradientRect
                | N::kDrawTiledGradient
                | N::kDrawTiledImage
                | N::kDrawBoxShadow
                | N::kDrawMask => unreachable!(
                    "compound Paint input must pass LowerPaintRecord before cc traversal"
                ),
            },
            Self::Converted(op) => match op {
                CpuPaintOp::NativeAtom(source) => Self::Native {
                    op: source.op(),
                    record: source.record(),
                }
                .GetType(),
                CpuPaintOp::DrawRect(..) => T::kDrawRect,
                CpuPaintOp::DrawPath { .. } => T::kDrawPath,
                CpuPaintOp::DrawDRRect { .. } => T::kDrawDRRect,
                CpuPaintOp::DrawRRect(..) => T::kDrawRRect,
                CpuPaintOp::Save => T::kSave,
                CpuPaintOp::Restore => T::kRestore,
                CpuPaintOp::Translate(..) => T::kTranslate,
                CpuPaintOp::Concat(_) => T::kConcat,
                CpuPaintOp::ClipRect { .. } => T::kClipRect,
                CpuPaintOp::ClipRRect(..) => T::kClipRRect,
                CpuPaintOp::ClipPath { .. } => T::kClipPath,
                CpuPaintOp::SaveLayer { .. } => T::kSaveLayer,
                CpuPaintOp::SaveLayerAlpha { .. } => T::kSaveLayerAlpha,
                CpuPaintOp::SaveLayerFilters { .. } => T::kSaveLayerFilters,
                CpuPaintOp::DrawTransparentColorSrc | CpuPaintOp::DrawColor { .. } => T::kDrawColor,
                CpuPaintOp::DrawRecord(_) => T::kDrawRecord,
                CpuPaintOp::DrawScrollingContents { .. } => T::kDrawScrollingContents,
                CpuPaintOp::Noop => T::kNoop,
            },
        }
    }
    fn IsDrawOp(self) -> bool {
        use paint::cc::paint::paint_op_buffer_iterator::PaintOpType::*;
        matches!(
            paint::cc::paint::paint_op_buffer_iterator::PaintOpView::GetType(self),
            kDrawArc
                | kDrawArcLite
                | kDrawColor
                | kDrawDRRect
                | kDrawImage
                | kDrawImageRect
                | kDrawIRect
                | kDrawLine
                | kDrawLineLite
                | kDrawOval
                | kDrawPath
                | kDrawRecord
                | kDrawRect
                | kDrawRRect
                | kDrawScrollingContents
                | kDrawSkottie
                | kDrawSlug
                | kDrawTextBlob
                | kDrawVertices
        )
    }
    fn IsPaintOpWithFlags(self) -> bool {
        match self {
            Self::Native { .. } => self.IsDrawOp() || matches!(paint::cc::paint::paint_op_buffer_iterator::PaintOpView::GetType(self), paint::cc::paint::paint_op_buffer_iterator::PaintOpType::kSaveLayer | paint::cc::paint::paint_op_buffer_iterator::PaintOpType::kSaveLayerFilters),
            Self::Converted(
                CpuPaintOp::DrawRRect(..)
                | CpuPaintOp::DrawRect(..)
                | CpuPaintOp::DrawPath { .. }
                | CpuPaintOp::DrawDRRect { .. } | CpuPaintOp::SaveLayer { .. } | CpuPaintOp::SaveLayerFilters { .. },
            ) => true,
            _ => false,
        }
    }
    fn FlagsSupportFoldingAlpha(self) -> bool {
        match self {
            Self::Native { op, .. } => {
                op.blend_mode == paint::paint_property_tree::PaintBlendMode::kNormal
                    && op.filters.is_empty()
                    && !op.is_shadow
                    && op.r#type != paint::paint_engine::DisplayItemType::kDrawBoxShadow
            }
            Self::Converted(
                CpuPaintOp::DrawRRect(_, flags)
                | CpuPaintOp::DrawRect(_, flags)
                | CpuPaintOp::DrawPath { flags, .. }
                | CpuPaintOp::DrawDRRect { flags, .. },
            ) => {
                flags.getBlendMode() == skia::BlendMode::kSrcOver
                    && flags.image_filter.is_none()
                    && flags.looper.is_none()
                    && flags.color_filter.is_none()
            }
            _ => false,
        }
    }
    fn IsPaintWorklet(self) -> bool {
        false
    } // Neither producer stream has a DrawImage paint-worklet variant.
    fn SaveLayerAlpha(self) -> f32 {
        match self {
            Self::Native { op, .. }
                if op.r#type == paint::paint_engine::DisplayItemType::kSaveLayerAlpha =>
            {
                op.opacity
            }
            Self::Converted(CpuPaintOp::SaveLayerAlpha { alpha, .. }) => *alpha,
            _ => panic!("source SaveLayerAlphaOp required"),
        }
    }
    fn DrawColor(self) -> ([f32; 4], bool) {
        match self {
            Self::Converted(CpuPaintOp::DrawTransparentColorSrc) => ([0.; 4], false),
            Self::Converted(CpuPaintOp::DrawColor { color, blend }) => (
                [color.red, color.green, color.blue, color.alpha],
                *blend == skia::BlendMode::kSrcOver,
            ),
            _ => panic!("source DrawColorOp required"),
        }
    }
    fn RecordBuffer(self) -> Option<&'a Self::Buffer> {
        match self {
            Self::Converted(CpuPaintOp::DrawRecord(record)) => record.buffer(),
            _ => None,
        }
    }
}
impl<R: CpuSkiaBackend> DisplayPlaybackItem
    for paint::cc::paint::paint_op_buffer_iterator::PlaybackItem<CpuPaintOpRef<'_, R>>
{
    fn AdditionalOpCount(&self) -> usize {
        match self.op {
            paint::cc::paint::paint_op_buffer_iterator::FoldedOp::Borrowed(op) => {
                op.AdditionalOpCount()
            }
            paint::cc::paint::paint_op_buffer_iterator::FoldedOp::OwnedDrawColor { .. } => 0,
        }
    }
}

impl<R: CpuSkiaBackend> CpuPaintOp<R> {
    pub fn AsRef(&self) -> CpuPaintOpRef<'_, R> {
        match self {
            Self::NativeAtom(source) => CpuPaintOpRef::Native {
                op: source.op(),
                record: source.record(),
            },
            _ => CpuPaintOpRef::Converted(self),
        }
    }
}
