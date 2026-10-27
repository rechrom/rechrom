//! Source image occurrence/animation index. Every field belongs to the one
//! generated map; op, shader and filter values are borrowed from their owners.
#![allow(non_snake_case)]
use super::paint_op_buffer_iterator::{
    PaintOpBufferView, PaintOpIterator, PaintOpType, PaintOpView,
};
use std::{collections::BTreeMap, sync::OnceLock};

pub trait ImageRect: Copy {
    fn IsEmpty(self) -> bool;
    fn Union(&mut self, other: Self);
}
pub trait ImageFloatRect: Copy {
    type Rect: ImageRect;
    fn MakeWH(width: i32, height: i32) -> Self;
    fn FromRect(rect: Self::Rect) -> Self;
    fn roundOut(self) -> Self::Rect;
}
pub trait ImageMatrix<F>: Clone {
    fn Identity() -> Self;
    fn RectToRect(src: F, dst: F, center_scale_to_fit: bool) -> Self;
    fn Multiply(&self, rhs: &Self) -> Self;
}
pub trait ImageSpatialIndex<R> {
    fn Build(size: usize, rect: impl Fn(usize) -> R) -> Self;
    fn Search(&self, rect: &R, result: &mut Vec<usize>);
}
pub trait ImageMapSequenceChecker {
    fn Check(&self);
}
pub trait DiscardableImageTypes: Sized {
    type Id: Copy + Ord;
    type ScrollOffsets: ?Sized;
    type Rect: ImageRect;
    type FloatRect: ImageFloatRect<Rect = Self::Rect>;
    type Matrix: ImageMatrix<Self::FloatRect>;
    type FilterQuality: Copy;
    type DecodingMode: Copy;
    type WorkletInput: Clone;
    type CompletionState: Copy;
    type FrameMetadata: Clone;
    type AnimationSequenceId: Copy;
    type Image: DiscardablePaintImage<Self>;
    type DrawImage;
    type SpatialIndex: ImageSpatialIndex<Self::Rect>;
    type SequenceChecker: ImageMapSequenceChecker;
    fn NewSequenceChecker() -> Self::SequenceChecker;
    fn NewDrawImage(
        image: Self::Image,
        dark: bool,
        source: Self::Rect,
        quality: Self::FilterQuality,
        matrix: Self::Matrix,
    ) -> Self::DrawImage;
    fn ConservativeDecodingMode(a: Self::DecodingMode, b: Self::DecodingMode)
        -> Self::DecodingMode;
}
pub trait DiscardablePaintImage<T: DiscardableImageTypes>: Clone {
    fn IsTextureBacked(&self) -> bool;
    fn stable_id(&self) -> T::Id;
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn IsPaintWorklet(&self) -> bool;
    fn GetPaintWorkletInput(&self) -> T::WorkletInput;
    fn IsLazyGenerated(&self) -> bool;
    fn decoding_mode(&self) -> T::DecodingMode;
    fn ShouldAnimate(&self) -> bool;
    fn completion_state(&self) -> T::CompletionState;
    fn GetFrameMetadata(&self) -> Vec<T::FrameMetadata>;
    fn repetition_count(&self) -> i32;
    fn reset_animation_sequence_id(&self) -> T::AnimationSequenceId;
    fn sync_animation_target_id(&self) -> T::Id;
    fn sync_animation_sequence_id(&self) -> T::AnimationSequenceId;
}
pub trait ImageGatheringCanvas<T: DiscardableImageTypes> {
    fn getSaveCount(&self) -> i32;
    fn save(&mut self);
    fn restore(&mut self);
    fn restoreToCount(&mut self, count: i32);
    fn translate(&mut self, x: f32, y: f32);
    fn getLocalToDevice(&self) -> T::Matrix;
    fn getDeviceClipBounds(&self) -> T::Rect;
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    Image,
    PaintRecord,
    Other,
}
/// These operations are typed field projections and native Skia/Paint value
/// primitives. All gather/visibility/animation/index decisions remain below.
pub trait DiscardableImageBackend<B: PaintOpBufferView + ?Sized>: DiscardableImageTypes {
    type Canvas: ImageGatheringCanvas<Self>;
    type TraceScope;
    type Flags<'a>: Copy
    where
        Self: 'a,
        B: 'a;
    type Shader<'a>: Copy
    where
        Self: 'a,
        B: 'a;
    type Filter<'a>: Copy
    where
        Self: 'a,
        B: 'a;
    type SkottieImages<'a>: Iterator<Item = (Self::Image, Self::FilterQuality)>
    where
        Self: 'a,
        B: 'a;
    fn MakeNoDrawCanvas(&self, bounds: Self::Rect) -> Self::Canvas;
    fn HasDiscardableImages(&self, buffer: &B) -> bool;
    fn OpHasDiscardableImages(&self, op: B::Op<'_>) -> bool;
    fn RasterState(&self, op: B::Op<'_>, canvas: &mut Self::Canvas, original_ctm: &Self::Matrix);
    fn ComputePaintRect(&self, op: B::Op<'_>, clip: Self::Rect, ctm: &Self::Matrix) -> Self::Rect;
    fn Flags<'a>(&'a self, op: B::Op<'a>) -> Self::Flags<'a>;
    fn FlagShader<'a>(&'a self, flags: Self::Flags<'a>) -> Option<Self::Shader<'a>>;
    fn FlagFilter<'a>(&'a self, flags: Self::Flags<'a>) -> Option<Self::Filter<'a>>;
    fn FlagQuality(&self, flags: Self::Flags<'_>) -> Self::FilterQuality;
    fn FlagUseDarkMode(&self, flags: Self::Flags<'_>) -> bool;
    fn PreserveDiscardableImageMapQuality(&self) -> bool;
    fn Image<'a>(&'a self, op: B::Op<'a>) -> Self::Image;
    fn ImageQuality(&self, op: B::Op<'_>) -> Self::FilterQuality;
    fn ImageSourceRect(&self, op: B::Op<'_>) -> Self::FloatRect;
    fn ImageDestinationRect(&self, op: B::Op<'_>) -> Self::FloatRect;
    fn SkottieImages<'a>(&'a self, op: B::Op<'a>) -> Self::SkottieImages<'a>;
    fn SkottieNativeSize(&self, op: B::Op<'_>) -> Self::FloatRect;
    fn ScrollingBuffer<'a>(&'a self, op: B::Op<'a>) -> &'a B;
    fn ScrollingOffset(&self, op: B::Op<'_>, offsets: &Self::ScrollOffsets) -> (f32, f32);
    fn ShaderHasDiscardableImages(&self, shader: Self::Shader<'_>) -> bool;
    fn ShaderType(&self, shader: Self::Shader<'_>) -> ShaderType;
    fn ShaderImage(&self, shader: Self::Shader<'_>) -> Self::Image;
    fn ShaderLocalMatrix(&self, shader: Self::Shader<'_>) -> Self::Matrix;
    fn ShaderHasNoAnimatedImages(&self, shader: Self::Shader<'_>) -> bool;
    fn ShaderRasterizationTileRect(
        &self,
        shader: Self::Shader<'_>,
        ctm: &Self::Matrix,
    ) -> Option<Self::FloatRect>;
    fn ShaderRecord<'a>(&'a self, shader: Self::Shader<'a>) -> &'a B;
    fn ShaderSetHasAnimatedImages(&self, shader: Self::Shader<'_>, has_images: bool);
    fn FilterHasDiscardableImages(&self, filter: Self::Filter<'_>) -> bool;
    fn FilterHasNoAnimatedImages(&self, filter: Self::Filter<'_>) -> bool;
    /// Native PaintFilter::SnapshotWithImages traversal; callback maps the
    /// provider's GetRasterContent, and returns no decoded raster content.
    fn FilterSnapshotWithImages(
        &self,
        filter: Self::Filter<'_>,
        provider: &mut dyn FnMut(Self::Image, Self::FloatRect, Self::FilterQuality),
    );
    fn FilterSetHasAnimatedImages(&self, filter: Self::Filter<'_>, has_images: bool);
    fn TraceGenerate(&self) -> Self::TraceScope;
}
// Source: cc/paint/discardable_image_map.h:39-69;
// cc/paint/discardable_image_map.cc:406-425.
pub struct AnimatedImageMetadata<T: DiscardableImageTypes> {
    pub paint_image_id: T::Id,
    pub sync_animation_target_id: T::Id,
    pub sync_animation_sequence_id: T::AnimationSequenceId,
    pub completion_state: T::CompletionState,
    pub frames: Vec<T::FrameMetadata>,
    pub repetition_count: i32,
    pub reset_animation_sequence_id: T::AnimationSequenceId,
}
impl<T: DiscardableImageTypes> AnimatedImageMetadata<T> {
    pub fn new(
        id: T::Id,
        completion: T::CompletionState,
        frames: Vec<T::FrameMetadata>,
        repetition: i32,
        reset: T::AnimationSequenceId,
        sync_target: T::Id,
        sync_sequence: T::AnimationSequenceId,
    ) -> Self {
        Self {
            paint_image_id: id,
            sync_animation_target_id: sync_target,
            sync_animation_sequence_id: sync_sequence,
            completion_state: completion,
            frames,
            repetition_count: repetition,
            reset_animation_sequence_id: reset,
        }
    }
}
impl<T: DiscardableImageTypes> Clone for AnimatedImageMetadata<T> {
    fn clone(&self) -> Self {
        Self {
            paint_image_id: self.paint_image_id,
            sync_animation_target_id: self.sync_animation_target_id,
            sync_animation_sequence_id: self.sync_animation_sequence_id,
            completion_state: self.completion_state,
            frames: self.frames.clone(),
            repetition_count: self.repetition_count,
            reset_animation_sequence_id: self.reset_animation_sequence_id,
        }
    }
}
pub type DecodingModeMap<T> =
    BTreeMap<<T as DiscardableImageTypes>::Id, <T as DiscardableImageTypes>::DecodingMode>;
pub type PaintWorkletInputs<T> = Vec<(
    <T as DiscardableImageTypes>::WorkletInput,
    <T as DiscardableImageTypes>::Id,
)>;
// Source: cc/paint/discardable_image_map.h:34-37,88-112.
pub struct DiscardableImageMap<T: DiscardableImageTypes> {
    image_id_to_rects_: BTreeMap<T::Id, Vec<T::Rect>>,
    animated_images_metadata_: BTreeMap<T::Id, AnimatedImageMetadata<T>>,
    images_: Vec<(T::DrawImage, T::Rect)>,
    images_rtree_: OnceLock<T::SpatialIndex>,
    sequence_checker_: T::SequenceChecker,
}
impl<T: DiscardableImageTypes> DiscardableImageMap<T> {
    // Source: cc/paint/discardable_image_map.cc:362-366. Rust owns both
    // containers and their members; map destruction may occur on any thread.
    fn new() -> Self {
        Self {
            image_id_to_rects_: BTreeMap::new(),
            animated_images_metadata_: BTreeMap::new(),
            images_: Vec::new(),
            images_rtree_: OnceLock::new(),
            sequence_checker_: T::NewSequenceChecker(),
        }
    }
    /// Ownership conversion at the typed runtime boundary. Generation is
    /// finished; the caller moves each field into its single metadata owner.
    /// This is not a second metadata map or a cached projection.
    pub fn TakeAnimatedImagesMetadata(&mut self) -> BTreeMap<T::Id, AnimatedImageMetadata<T>> {
        std::mem::take(&mut self.animated_images_metadata_)
    }
    // Source: cc/paint/discardable_image_map.h:81-87.
    pub fn empty(&self) -> bool {
        self.sequence_checker_.Check();
        self.image_id_to_rects_.is_empty()
    }
    pub fn animated_images_metadata(&self) -> &BTreeMap<T::Id, AnimatedImageMetadata<T>> {
        self.sequence_checker_.Check();
        &self.animated_images_metadata_
    }
    // Source: cc/paint/discardable_image_map.cc:383-396. Source stores stable
    // DrawImage pointers. Safe Rust stores stable slots into this immutable
    // images_ owner and returns references resolved against that same owner.
    pub fn GetDiscardableImagesInRect(&self, rect: &T::Rect) -> Vec<&T::DrawImage> {
        self.sequence_checker_.Check();
        let index = self
            .images_rtree_
            .get_or_init(|| T::SpatialIndex::Build(self.images_.len(), |i| self.images_[i].1));
        let mut slots = Vec::new();
        index.Search(rect, &mut slots);
        slots.into_iter().map(|i| &self.images_[i].0).collect()
    }
    // Source: cc/paint/discardable_image_map.cc:398-404.
    pub fn GetRectsForImage(&self, id: T::Id) -> &[T::Rect] {
        self.sequence_checker_.Check();
        self.image_id_to_rects_
            .get(&id)
            .map_or(&[], |r| r.as_slice())
    }
}
// Source: cc/paint/discardable_image_map.cc:35-38,53-74,353-360.
struct Generator<'a, B: PaintOpBufferView + ?Sized, A: DiscardableImageBackend<B>> {
    map_: &'a mut DiscardableImageMap<A>,
    canvas_: A::Canvas,
    backend: &'a A,
    decoding_mode_map_: Option<&'a mut DecodingModeMap<A>>,
    paint_worklet_inputs_: Option<&'a mut PaintWorkletInputs<A>>,
    raster_inducing_scroll_offsets_: &'a A::ScrollOffsets,
    only_gather_animated_images_: bool,
    collect_invisible_images_: bool,
    _buffer: std::marker::PhantomData<&'a B>,
}
impl<'a, B: PaintOpBufferView + ?Sized, A: DiscardableImageBackend<B>> Generator<'a, B, A> {
    // Source: cc/paint/discardable_image_map.cc:39-51.
    fn new(
        map: &'a mut DiscardableImageMap<A>,
        buffer: &B,
        bounds: A::Rect,
        offsets: &'a A::ScrollOffsets,
        decoding: Option<&'a mut DecodingModeMap<A>>,
        worklets: Option<&'a mut PaintWorkletInputs<A>>,
        backend: &'a A,
    ) -> Self {
        let mut result = Self {
            map_: map,
            canvas_: backend.MakeNoDrawCanvas(bounds),
            backend,
            decoding_mode_map_: decoding,
            paint_worklet_inputs_: worklets,
            raster_inducing_scroll_offsets_: offsets,
            only_gather_animated_images_: false,
            collect_invisible_images_: false,
            _buffer: std::marker::PhantomData,
        };
        result.GatherDiscardableImages(buffer, None);
        result
    }
    // Source: cc/paint/discardable_image_map.cc:76-123.
    fn GatherDiscardableImages(&mut self, buffer: &B, top_level_op_rect: Option<&A::Rect>) {
        if !self.backend.HasDiscardableImages(buffer) {
            return;
        }
        let count = self.canvas_.getSaveCount();
        self.canvas_.save();
        let original_ctm = self.canvas_.getLocalToDevice();
        let backend = self.backend;
        for op in PaintOpIterator::new(buffer) {
            if !op.IsDrawOp() {
                backend.RasterState(op, &mut self.canvas_, &original_ctm)
            }
            if !backend.OpHasDiscardableImages(op) {
                continue;
            }
            let op_rect = if let Some(rect) = top_level_op_rect {
                *rect
            } else {
                backend.ComputePaintRect(
                    op,
                    self.canvas_.getDeviceClipBounds(),
                    &self.canvas_.getLocalToDevice(),
                )
            };
            if !self.collect_invisible_images_ && op_rect.IsEmpty() {
                continue;
            }
            let ctm = self.canvas_.getLocalToDevice();
            if op.IsPaintOpWithFlags() {
                self.AddImageFromFlags(op_rect, backend.Flags(op), &ctm)
            }
            // Source: cc/paint/discardable_image_map.cc:124-207.
            match op.GetType() {
                PaintOpType::kDrawImage => {
                    let flags = backend.Flags(op);
                    let image = backend.Image(op);
                    let quality = if backend.PreserveDiscardableImageMapQuality() {
                        backend.ImageQuality(op)
                    } else {
                        backend.FlagQuality(flags)
                    };
                    let src = A::FloatRect::MakeWH(image.width(), image.height());
                    self.AddImage(
                        image,
                        backend.FlagUseDarkMode(flags),
                        src,
                        op_rect,
                        ctm,
                        quality,
                    )
                }
                PaintOpType::kDrawImageRect => {
                    let flags = backend.Flags(op);
                    let matrix = ctm.Multiply(&A::Matrix::RectToRect(
                        backend.ImageSourceRect(op),
                        backend.ImageDestinationRect(op),
                        false,
                    ));
                    let quality = if backend.PreserveDiscardableImageMapQuality() {
                        backend.ImageQuality(op)
                    } else {
                        backend.FlagQuality(flags)
                    };
                    self.AddImage(
                        backend.Image(op),
                        backend.FlagUseDarkMode(flags),
                        backend.ImageSourceRect(op),
                        op_rect,
                        matrix,
                        quality,
                    )
                }
                PaintOpType::kDrawSkottie => {
                    for (image, quality) in backend.SkottieImages(op) {
                        let src = A::FloatRect::MakeWH(image.width(), image.height());
                        let dst = op_rect;
                        let matrix = ctm.Multiply(&A::Matrix::RectToRect(
                            backend.SkottieNativeSize(op),
                            A::FloatRect::FromRect(dst),
                            true,
                        ));
                        self.AddImage(image, false, src, dst, matrix, quality)
                    }
                }
                PaintOpType::kDrawRecord => {
                    if let Some(record) = op.RecordBuffer() {
                        self.GatherDiscardableImages(record, top_level_op_rect)
                    }
                }
                PaintOpType::kDrawScrollingContents => {
                    let list = backend.ScrollingBuffer(op);
                    if backend.HasDiscardableImages(list) {
                        self.canvas_.save();
                        let old = self.collect_invisible_images_;
                        self.collect_invisible_images_ = true;
                        let (x, y) =
                            backend.ScrollingOffset(op, self.raster_inducing_scroll_offsets_);
                        self.canvas_.translate(-x, -y);
                        self.GatherDiscardableImages(list, top_level_op_rect);
                        self.canvas_.restore();
                        self.collect_invisible_images_ = old;
                    }
                }
                _ => {}
            }
        }
        self.canvas_.restoreToCount(count);
    }
    // Source: cc/paint/discardable_image_map.cc:209-216.
    fn AddImageFromFlags<'b>(&mut self, rect: A::Rect, flags: A::Flags<'b>, ctm: &A::Matrix)
    where
        'a: 'b,
    {
        let backend: &'b A = self.backend;
        self.AddImageFromShader(
            rect,
            backend.FlagShader(flags),
            ctm,
            backend.FlagQuality(flags),
        );
        self.AddImageFromFilter(rect, backend.FlagFilter(flags));
    }
    // Source: cc/paint/discardable_image_map.cc:218-263.
    fn AddImageFromShader<'b>(
        &mut self,
        rect: A::Rect,
        shader: Option<A::Shader<'b>>,
        ctm: &A::Matrix,
        quality: A::FilterQuality,
    ) where
        'a: 'b,
    {
        let Some(shader) = shader else { return };
        let backend: &'b A = self.backend;
        if !backend.ShaderHasDiscardableImages(shader) {
            return;
        }
        if backend.ShaderType(shader) == ShaderType::Image {
            let image = backend.ShaderImage(shader);
            let matrix = ctm.Multiply(&backend.ShaderLocalMatrix(shader));
            let src = A::FloatRect::MakeWH(image.width(), image.height());
            self.AddImage(image, false, src, rect, matrix, quality);
            return;
        }
        if backend.ShaderType(shader) == ShaderType::PaintRecord {
            if backend.ShaderHasNoAnimatedImages(shader) {
                return;
            }
            if backend.ShaderRasterizationTileRect(shader, ctm).is_none() {
                return;
            }
            let old = self.only_gather_animated_images_;
            self.only_gather_animated_images_ = true;
            let previous = self.map_.images_.len();
            self.GatherDiscardableImages(backend.ShaderRecord(shader), Some(&rect));
            debug_assert!(self.map_.images_.len() >= previous);
            backend.ShaderSetHasAnimatedImages(shader, self.map_.images_.len() > previous);
            self.only_gather_animated_images_ = old;
        }
    }
    // Source: cc/paint/discardable_image_map.cc:265-282;
    // cc/paint/discardable_image_map.cc:56-74 (ImageGatheringProvider).
    fn AddImageFromFilter(&mut self, rect: A::Rect, filter: Option<A::Filter<'_>>) {
        let Some(filter) = filter else { return };
        let backend = self.backend;
        if !backend.FilterHasDiscardableImages(filter) || backend.FilterHasNoAnimatedImages(filter)
        {
            return;
        }
        let old = self.only_gather_animated_images_;
        self.only_gather_animated_images_ = true;
        let previous = self.map_.images_.len();
        backend.FilterSnapshotWithImages(filter, &mut |image, source, quality| {
            self.AddImage(image, false, source, rect, A::Matrix::Identity(), quality)
        });
        debug_assert!(self.map_.images_.len() >= previous);
        backend.FilterSetHasAnimatedImages(filter, self.map_.images_.len() > previous);
        self.only_gather_animated_images_ = old;
    }
    // Source: cc/paint/discardable_image_map.cc:284-351.
    fn AddImage(
        &mut self,
        image: A::Image,
        dark: bool,
        source: A::FloatRect,
        rect: A::Rect,
        matrix: A::Matrix,
        quality: A::FilterQuality,
    ) {
        if image.IsTextureBacked() {
            return;
        }
        let src = source.roundOut();
        let rects = self
            .map_
            .image_id_to_rects_
            .entry(image.stable_id())
            .or_default();
        if rects.len() >= 256 {
            rects.last_mut().unwrap().Union(rect)
        } else {
            rects.push(rect)
        }
        if let Some(worklets) = &mut self.paint_worklet_inputs_ {
            if image.IsPaintWorklet() {
                worklets.push((image.GetPaintWorkletInput(), image.stable_id()))
            }
        }
        if let Some(modes) = &mut self.decoding_mode_map_ {
            if image.IsLazyGenerated() {
                use std::collections::btree_map::Entry;
                match modes.entry(image.stable_id()) {
                    Entry::Vacant(entry) => {
                        entry.insert(image.decoding_mode());
                    }
                    Entry::Occupied(mut entry) => {
                        let conservative =
                            A::ConservativeDecodingMode(*entry.get(), image.decoding_mode());
                        entry.insert(conservative);
                    }
                }
            }
        }
        if image.ShouldAnimate() {
            self.map_.animated_images_metadata_.insert(
                image.stable_id(),
                AnimatedImageMetadata::new(
                    image.stable_id(),
                    image.completion_state(),
                    image.GetFrameMetadata(),
                    image.repetition_count(),
                    image.reset_animation_sequence_id(),
                    image.sync_animation_target_id(),
                    image.sync_animation_sequence_id(),
                ),
            );
        }
        let mut add_image = true;
        if image.IsPaintWorklet() {
            add_image = false
        } else if self.only_gather_animated_images_ {
            add_image = image.ShouldAnimate()
        }
        if add_image {
            self.map_
                .images_
                .push((A::NewDrawImage(image, dark, src, quality, matrix), rect));
        }
    }
}
impl<A: DiscardableImageTypes> DiscardableImageMap<A> {
    // Source: cc/paint/discardable_image_map.h:71-79;
    // cc/paint/discardable_image_map.cc:368-381.
    pub fn Generate<B: PaintOpBufferView + ?Sized>(
        buffer: &B,
        bounds: A::Rect,
        offsets: &A::ScrollOffsets,
        decoding: Option<&mut DecodingModeMap<A>>,
        worklets: Option<&mut PaintWorkletInputs<A>>,
        backend: &A,
    ) -> Self
    where
        A: DiscardableImageBackend<B>,
    {
        let _trace = backend.TraceGenerate();
        let mut map = Self::new();
        Generator::new(
            &mut map, buffer, bounds, offsets, decoding, worklets, backend,
        );
        assert!(map.images_rtree_.get().is_none());
        map
    }
}
// Source scaffolding: cc/paint/discardable_image_map.h:1-33,114;
// cc/paint/discardable_image_map.cc:1-34,427 (includes/namespaces/guards).
// Translation status: complete
