//! Chromium PaintOpBuffer cursors and playback folding. Storage owns the ops;
//! these cursors borrow resident values. Offsets are native storage offsets and
//! the injected view supplies the equivalent of AlignedSize, never a copied list.
#![allow(non_snake_case, non_camel_case_types)]

// External value vocabulary: cc/paint/paint_op.h:73-111.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaintOpType {
    kAnnotate,
    kClipPath,
    kClipRect,
    kClipRRect,
    kConcat,
    kCustomData,
    kDrawArc,
    kDrawArcLite,
    kDrawColor,
    kDrawDRRect,
    kDrawImage,
    kDrawImageRect,
    kDrawIRect,
    kDrawLine,
    kDrawLineLite,
    kDrawOval,
    kDrawPath,
    kDrawRecord,
    kDrawRect,
    kDrawRRect,
    kDrawScrollingContents,
    kDrawSkottie,
    kDrawSlug,
    kDrawTextBlob,
    kDrawVertices,
    kNoop,
    kRestore,
    kRotate,
    kSave,
    kSaveLayer,
    kSaveLayerAlpha,
    kSaveLayerFilters,
    kScale,
    kSetMatrix,
    kSetNodeId,
    kTranslate,
}
/// Typed borrowed storage boundary. No ownership or folding decision is here.
pub trait PaintOpBufferView {
    type Op<'a>: PaintOpView<'a, Buffer = Self>
    where
        Self: 'a;
    fn Used(&self) -> usize;
    fn GetOp(&self, offset: usize) -> Self::Op<'_>;
    fn NextOffset(&self, offset: usize) -> usize;
}
pub trait PaintOpView<'a>: Copy {
    type Buffer: PaintOpBufferView + ?Sized + 'a;
    fn GetType(self) -> PaintOpType;
    fn IsDrawOp(self) -> bool;
    fn IsPaintOpWithFlags(self) -> bool;
    /// Direct PaintFlags::SupportsFoldingAlpha, cc/paint/paint_flags.cc:130-145.
    fn FlagsSupportFoldingAlpha(self) -> bool;
    fn IsPaintWorklet(self) -> bool;
    fn SaveLayerAlpha(self) -> f32;
    fn DrawColor(self) -> ([f32; 4], bool);
    fn RecordBuffer(self) -> Option<&'a Self::Buffer>;
}
// Source: cc/paint/paint_op_buffer_iterator.h:23-31. Rust Iterator supplies
// value/difference/category operations; the item is a live borrowed op view.
// Source: cc/paint/paint_op_buffer_iterator.h:33-92.
pub struct PaintOpIterator<'a, B: PaintOpBufferView + ?Sized> {
    buffer_: &'a B,
    op_offset_: usize,
}
impl<'a, B: PaintOpBufferView + ?Sized> Copy for PaintOpIterator<'a, B> {}
impl<'a, B: PaintOpBufferView + ?Sized> Clone for PaintOpIterator<'a, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> PaintOpIterator<'a, B> {
    pub fn new(buffer: &'a B) -> Self {
        Self {
            buffer_: buffer,
            op_offset_: 0,
        }
    }
    pub fn get(&self) -> B::Op<'a> {
        self.buffer_.GetOp(self.op_offset_)
    }
    pub fn begin(&self) -> Self {
        Self::new(self.buffer_)
    }
    pub fn end(&self) -> Self {
        Self {
            buffer_: self.buffer_,
            op_offset_: self.buffer_.Used(),
        }
    }
    pub fn is_valid(&self) -> bool {
        self.op_offset_ < self.buffer_.Used()
    }
    pub fn Advance(&mut self) {
        debug_assert!(self.is_valid());
        self.op_offset_ = self.buffer_.NextOffset(self.op_offset_);
        assert!(self.op_offset_ <= self.buffer_.Used());
    }
}
impl<B: PaintOpBufferView + ?Sized> PartialEq for PaintOpIterator<'_, B> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert!(std::ptr::eq(self.buffer_, other.buffer_));
        self.op_offset_ == other.op_offset_
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> Iterator for PaintOpIterator<'a, B> {
    type Item = B::Op<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_valid() {
            return None;
        }
        let op = self.get();
        self.Advance();
        Some(op)
    }
}

// Source: cc/paint/paint_op_buffer_iterator.h:94-161.
pub struct OffsetIterator<'a, B: PaintOpBufferView + ?Sized> {
    buffer_: &'a B,
    offsets_: &'a [usize],
    op_offset_: usize,
    offsets_index_: usize,
}
impl<'a, B: PaintOpBufferView + ?Sized> Copy for OffsetIterator<'a, B> {}
impl<'a, B: PaintOpBufferView + ?Sized> Clone for OffsetIterator<'a, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> OffsetIterator<'a, B> {
    pub fn new(buffer: &'a B, offsets: &'a [usize]) -> Self {
        Self {
            buffer_: buffer,
            offsets_: offsets,
            op_offset_: offsets.first().copied().unwrap_or(buffer.Used()),
            offsets_index_: 0,
        }
    }
    pub fn get(&self) -> B::Op<'a> {
        self.buffer_.GetOp(self.op_offset_)
    }
    pub fn begin(&self) -> Self {
        Self::new(self.buffer_, self.offsets_)
    }
    pub fn end(&self) -> Self {
        Self {
            buffer_: self.buffer_,
            offsets_: self.offsets_,
            op_offset_: self.buffer_.Used(),
            offsets_index_: 0,
        }
    }
    pub fn is_valid(&self) -> bool {
        self.op_offset_ < self.buffer_.Used()
    }
    pub fn Advance(&mut self) {
        self.offsets_index_ += 1;
        if self.offsets_index_ >= self.offsets_.len() {
            *self = self.end();
            return;
        }
        let target_offset = self.offsets_[self.offsets_index_];
        assert!(target_offset >= self.op_offset_);
        assert!(target_offset < self.buffer_.Used());
        self.op_offset_ = target_offset;
    }
}
impl<B: PaintOpBufferView + ?Sized> PartialEq for OffsetIterator<'_, B> {
    fn eq(&self, other: &Self) -> bool {
        debug_assert!(std::ptr::eq(self.buffer_, other.buffer_));
        self.op_offset_ == other.op_offset_
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> Iterator for OffsetIterator<'a, B> {
    type Item = B::Op<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_valid() {
            return None;
        }
        let op = self.get();
        self.Advance();
        Some(op)
    }
}

// Source: cc/paint/paint_op_buffer_iterator.h:163-212;
// cc/paint/paint_op_buffer_iterator.cc:69-84.
pub enum CompositeIterator<'a, B: PaintOpBufferView + ?Sized> {
    All(PaintOpIterator<'a, B>),
    Offsets(OffsetIterator<'a, B>),
}
impl<'a, B: PaintOpBufferView + ?Sized> Copy for CompositeIterator<'a, B> {}
impl<'a, B: PaintOpBufferView + ?Sized> Clone for CompositeIterator<'a, B> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> CompositeIterator<'a, B> {
    pub fn new(buffer: &'a B, offsets: Option<&'a [usize]>) -> Self {
        match offsets {
            None => Self::All(PaintOpIterator::new(buffer)),
            Some(offsets) => Self::Offsets(OffsetIterator::new(buffer, offsets)),
        }
    }
    pub fn get(&self) -> B::Op<'a> {
        match self {
            Self::All(i) => i.get(),
            Self::Offsets(i) => i.get(),
        }
    }
    pub fn begin(&self) -> Self {
        match self {
            Self::All(i) => Self::All(i.begin()),
            Self::Offsets(i) => Self::Offsets(i.begin()),
        }
    }
    pub fn end(&self) -> Self {
        match self {
            Self::All(i) => Self::All(i.end()),
            Self::Offsets(i) => Self::Offsets(i.end()),
        }
    }
    pub fn is_valid(&self) -> bool {
        match self {
            Self::All(i) => i.is_valid(),
            Self::Offsets(i) => i.is_valid(),
        }
    }
    pub fn Advance(&mut self) {
        match self {
            Self::All(i) => i.Advance(),
            Self::Offsets(i) => i.Advance(),
        }
    }
}
impl<B: PaintOpBufferView + ?Sized> PartialEq for CompositeIterator<'_, B> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::All(a), Self::All(b)) => a == b,
            (Self::Offsets(a), Self::Offsets(b)) => a == b,
            _ => false,
        }
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> Iterator for CompositeIterator<'a, B> {
    type Item = B::Op<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.is_valid() {
            return None;
        }
        let op = self.get();
        self.Advance();
        Some(op)
    }
}

// Source: cc/paint/paint_op_buffer_iterator.cc:13-37.
fn SupportsFoldingAlpha<'a, O: PaintOpView<'a>>(op: O) -> bool {
    if !op.IsPaintOpWithFlags() {
        return false;
    }
    if !op.FlagsSupportFoldingAlpha() {
        return false;
    }
    if op.GetType() == PaintOpType::kDrawTextBlob {
        return false;
    }
    if op.GetType() == PaintOpType::kDrawImage && op.IsPaintWorklet() {
        return false;
    }
    if op.GetType() == PaintOpType::kDrawImageRect && op.IsPaintWorklet() {
        return false;
    }
    true
}
// Source: cc/paint/paint_op_buffer_iterator.cc:39-65.
fn GetNestedSingleDrawingOp<'a, B: PaintOpBufferView + ?Sized>(
    mut op: B::Op<'a>,
) -> Option<B::Op<'a>> {
    if !op.IsDrawOp() {
        return None;
    }
    while op.GetType() == PaintOpType::kDrawRecord {
        let record = op.RecordBuffer()?;
        if record.Used() == 0 {
            return None;
        }
        if record.NextOffset(0) < record.Used() {
            return None;
        }
        op = record.GetOp(0);
        if !op.IsDrawOp() {
            return None;
        }
    }
    Some(op)
}
/// `OwnedDrawColor` maps the iterator's native folded_draw_color_ storage.
/// It owns only this small generated op, never an existing record or snapshot.
#[derive(Clone, Copy)]
pub enum FoldedOp<O> {
    Borrowed(O),
    OwnedDrawColor { color: [f32; 4] },
}
#[derive(Clone, Copy)]
pub struct PlaybackItem<O> {
    pub op: FoldedOp<O>,
    pub alpha: f32,
}
// Source: cc/paint/paint_op_buffer_iterator.h:214-250;
// cc/paint/paint_op_buffer_iterator.cc:86-94.
pub struct PlaybackFoldingIterator<'a, B: PaintOpBufferView + ?Sized> {
    iter_: CompositeIterator<'a, B>,
    stack_: [Option<B::Op<'a>>; 3],
    stack_len_: usize,
    folded_draw_color_: [f32; 4],
    current_op_: Option<FoldedOp<B::Op<'a>>>,
    current_alpha_: f32,
}
impl<'a, B: PaintOpBufferView + ?Sized> PlaybackFoldingIterator<'a, B> {
    pub fn new(buffer: &'a B, offsets: Option<&'a [usize]>) -> Self {
        let mut result = Self {
            iter_: CompositeIterator::new(buffer, offsets),
            stack_: [None; 3],
            stack_len_: 0,
            folded_draw_color_: [0.; 4],
            current_op_: None,
            current_alpha_: 1.,
        };
        result.FindNextOp();
        result
    }
    pub fn get(&self) -> Option<FoldedOp<B::Op<'a>>> {
        self.current_op_
    }
    pub fn alpha(&self) -> f32 {
        self.current_alpha_
    }
    pub fn is_valid(&self) -> bool {
        self.current_op_.is_some()
    }
    pub fn Advance(&mut self) {
        self.FindNextOp()
    }
    // Source: cc/paint/paint_op_buffer_iterator.cc:96-146.
    fn FindNextOp(&mut self) {
        self.current_alpha_ = 1.;
        loop {
            let Some(current) = self.NextUnfoldedOp() else {
                self.current_op_ = None;
                break;
            };
            self.current_op_ = Some(FoldedOp::Borrowed(current));
            if current.GetType() != PaintOpType::kSaveLayerAlpha {
                break;
            }
            let Some(second) = self.NextUnfoldedOp() else {
                break;
            };
            if second.GetType() == PaintOpType::kRestore {
                continue;
            }
            let draw_op = GetNestedSingleDrawingOp::<B>(second);
            let mut third = None;
            if let Some(draw_op) = draw_op {
                third = self.NextUnfoldedOp();
                if third.is_some_and(|op| op.GetType() == PaintOpType::kRestore) {
                    if SupportsFoldingAlpha(draw_op) {
                        self.current_alpha_ = current.SaveLayerAlpha();
                        self.current_op_ = Some(FoldedOp::Borrowed(draw_op));
                        break;
                    } else if draw_op.GetType() == PaintOpType::kDrawColor && draw_op.DrawColor().1
                    {
                        let mut color = draw_op.DrawColor().0;
                        color[3] *= current.SaveLayerAlpha();
                        self.folded_draw_color_ = color;
                        self.current_op_ = Some(FoldedOp::OwnedDrawColor {
                            color: self.folded_draw_color_,
                        });
                        break;
                    }
                }
            }
            self.PushBack(second);
            if let Some(third) = third {
                self.PushBack(third)
            }
            break;
        }
    }
    fn PushBack(&mut self, op: B::Op<'a>) {
        assert!(self.stack_len_ < self.stack_.len());
        self.stack_[self.stack_len_] = Some(op);
        self.stack_len_ += 1;
    }
    // Source: cc/paint/paint_op_buffer_iterator.cc:148-160.
    fn NextUnfoldedOp(&mut self) -> Option<B::Op<'a>> {
        if self.stack_len_ != 0 {
            let op = self.stack_[0].take();
            for index in 1..self.stack_len_ {
                self.stack_[index - 1] = self.stack_[index].take()
            }
            self.stack_len_ -= 1;
            return op;
        }
        if !self.iter_.is_valid() {
            return None;
        }
        let op = self.iter_.get();
        self.iter_.Advance();
        Some(op)
    }
}
impl<'a, B: PaintOpBufferView + ?Sized> Iterator for PlaybackFoldingIterator<'a, B> {
    type Item = PlaybackItem<B::Op<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let op = self.current_op_?;
        let alpha = self.current_alpha_;
        self.FindNextOp();
        Some(PlaybackItem { op, alpha })
    }
}
// Source scaffolding: cc/paint/paint_op_buffer_iterator.h:1-22,252-254;
// cc/paint/paint_op_buffer_iterator.cc:1-12,67,162: includes/namespaces/guards.
// Translation status: complete
