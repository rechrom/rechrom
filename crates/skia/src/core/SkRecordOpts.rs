use crate::src::core::{SkBlendMode::SkBlendMode, SkPaint::SkPaint};
use crate::src::core::{
    SkRecord::SkRecord,
    SkRecordPattern::{
        Greedy, Is, IsDraw, IsSingleDraw, Not, Or, Pattern, PatternEnd, PatternMatch,
    },
    SkRecords::{self, ClipRect, NoOp, Restore, Save, SaveLayer},
};
macro_rules! pattern{()=>{PatternEnd};($first:ty $(,$rest:ty)*)=>{Pattern<$first,pattern!($($rest),*)>};}
macro_rules! either{($first:ty)=>{Or<$first>};($first:ty,$($rest:ty),+)=>{Or<$first,either!($($rest),+)>};}
// Source: src/core/SkRecordOpts.cpp:31-44
trait Pass {
    type Match: PatternMatch;
    unsafe fn onMatch(
        &mut self,
        record: &mut SkRecord,
        m: &mut Self::Match,
        begin: i32,
        end: i32,
    ) -> bool;
}
unsafe fn apply<P: Pass>(pass: &mut P, record: &mut SkRecord) -> bool
where
    P::Match: Search,
{
    let mut m = P::Match::default();
    let mut changed = false;
    let mut begin = 0;
    let mut end = 0;
    while unsafe { m.search(record, &mut begin, &mut end) } {
        changed |= unsafe { pass.onMatch(record, &mut m, begin, end) };
    }
    changed
}
trait Search {
    unsafe fn search(&mut self, record: &mut SkRecord, begin: &mut i32, end: &mut i32) -> bool;
}
impl<F: crate::src::core::SkRecordPattern::Matcher, R: PatternMatch> Search for Pattern<F, R> {
    unsafe fn search(&mut self, record: &mut SkRecord, begin: &mut i32, end: &mut i32) -> bool {
        unsafe { Pattern::search(self, record, begin, end) }
    }
}
// Source: src/core/SkRecordOpts.cpp:49-60
struct SaveOnlyDrawsRestoreNooper;
impl Pass for SaveOnlyDrawsRestoreNooper {
    type Match = pattern!(Is<Save>, Greedy<Or<Is<NoOp>, Or<IsDraw>>>, Is<Restore>);
    unsafe fn onMatch(
        &mut self,
        record: &mut SkRecord,
        _: &mut Self::Match,
        begin: i32,
        end: i32,
    ) -> bool {
        unsafe {
            record.replace::<NoOp>(begin).write(NoOp::new());
            record.replace::<NoOp>(end - 1).write(NoOp::new());
        }
        true
    }
}
// Source: src/core/SkRecordOpts.cpp:62-114
pub fn fold_opacity_layer_color_to_paint(
    layer_paint: Option<&SkPaint>,
    is_save_layer: bool,
    paint: &mut SkPaint,
) -> bool {
    if !paint.isSrcOver() {
        return false;
    }
    if !is_save_layer && paint.getImageFilter().is_some() {
        return false;
    }
    if paint.getColorFilter().is_some() {
        return false;
    }
    if let Some(layer) = layer_paint {
        let color = layer.getColor();
        if color & 0x00FFFFFF != 0 {
            return false;
        }
        if layer.getPathEffect().is_some()
            || layer.getShader().is_some()
            || !layer.isSrcOver()
            || layer.getMaskFilter().is_some()
            || layer.getColorFilter().is_some()
            || layer.getImageFilter().is_some()
        {
            return false;
        }
        paint.setAlpha(crate::include::private::SkMath::SkMulDiv255Round(
            paint.getAlpha() as u32,
            color >> 24,
        ));
    }
    true
}
// Source: src/core/SkRecordOpts.cpp:116-134
struct SaveNoDrawsRestoreNooper;
impl Pass for SaveNoDrawsRestoreNooper {
    type Match = pattern!(
        Is<Save>,
        Greedy<Not<either!(Is<Save>, Is<SaveLayer>, Is<Restore>, IsDraw)>>,
        Is<Restore>
    );
    unsafe fn onMatch(
        &mut self,
        record: &mut SkRecord,
        _: &mut Self::Match,
        begin: i32,
        end: i32,
    ) -> bool {
        for i in begin..end {
            unsafe {
                record.replace::<NoOp>(i).write(NoOp::new());
            }
        }
        true
    }
}
// Source: src/core/SkRecordOpts.cpp:135-141
pub unsafe fn SkRecordNoopSaveRestores(record: &mut SkRecord) {
    let mut onlyDraws = SaveOnlyDrawsRestoreNooper;
    let mut noDraws = SaveNoDrawsRestoreNooper;
    while unsafe { apply(&mut onlyDraws, record) || apply(&mut noDraws, record) } {}
}
// Source: src/core/SkRecordOpts.cpp:145-152
#[cfg(not(sk_build_for_android_framework))]
pub fn effectively_srcover(paint: Option<&SkPaint>) -> bool {
    let Some(paint) = paint else {
        return true;
    };
    if paint.isSrcOver() {
        return true;
    }
    paint.getShader().is_none()
        && paint.getColorFilter().is_none()
        && paint.getImageFilter().is_none()
        && paint.getAlpha() == 0xFF
        && paint.asBlendMode() == Some(SkBlendMode::kSrc)
}
// Source: src/core/SkRecordOpts.cpp:156-209
#[cfg(not(sk_build_for_android_framework))]
struct SaveLayerDrawRestoreNooper;
#[cfg(not(sk_build_for_android_framework))]
impl SaveLayerDrawRestoreNooper {
    unsafe fn KillSaveLayerAndRestore(record: &mut SkRecord, i: i32) -> bool {
        unsafe {
            record.replace::<NoOp>(i).write(NoOp::new());
            record.replace::<NoOp>(i + 2).write(NoOp::new());
        }
        true
    }
}
#[cfg(not(sk_build_for_android_framework))]
impl Pass for SaveLayerDrawRestoreNooper {
    type Match = pattern!(Is<SaveLayer>, IsSingleDraw, Is<Restore>);
    unsafe fn onMatch(
        &mut self,
        record: &mut SkRecord,
        m: &mut Self::Match,
        begin: i32,
        _end: i32,
    ) -> bool {
        let layer = unsafe { &mut *m.first() };
        if layer.backdrop.is_some() {
            return false;
        }
        if layer.filters.size() != 0 {
            return false;
        }
        let layer_paint = layer.paint.as_ptr();
        let draw_paint = m.second();
        if layer_paint.is_null() && effectively_srcover(unsafe { draw_paint.as_ref() }) {
            return unsafe { Self::KillSaveLayerAndRestore(record, begin) };
        }
        let Some(draw_paint) = (unsafe { draw_paint.as_mut() }) else {
            return false;
        };
        if !fold_opacity_layer_color_to_paint(unsafe { layer_paint.as_ref() }, false, draw_paint) {
            return false;
        }
        unsafe { Self::KillSaveLayerAndRestore(record, begin) }
    }
}
// Source: src/core/SkRecordOpts.cpp:210-213
#[cfg(not(sk_build_for_android_framework))]
pub unsafe fn SkRecordNoopSaveLayerDrawRestores(record: &mut SkRecord) {
    unsafe {
        apply(&mut SaveLayerDrawRestoreNooper, record);
    }
}
// Source: src/core/SkRecordOpts.cpp:225-266
struct SvgOpacityAndFilterLayerMergePass;
impl SvgOpacityAndFilterLayerMergePass {
    unsafe fn KillSaveLayerAndRestore(record: &mut SkRecord, i: i32) -> bool {
        unsafe {
            record.replace::<NoOp>(i).write(NoOp::new());
            record.replace::<NoOp>(i + 6).write(NoOp::new());
        }
        true
    }
}
impl Pass for SvgOpacityAndFilterLayerMergePass {
    type Match = pattern!(
        Is<SaveLayer>,
        Is<Save>,
        Is<ClipRect>,
        Is<SaveLayer>,
        Is<Restore>,
        Is<Restore>,
        Is<Restore>
    );
    unsafe fn onMatch(
        &mut self,
        record: &mut SkRecord,
        m: &mut Self::Match,
        begin: i32,
        _end: i32,
    ) -> bool {
        let outer = unsafe { &mut *m.first() };
        let inner = unsafe { &mut *m.fourth() };
        if outer.backdrop.is_some() {
            return false;
        }
        if outer.filters.size() != 0 || inner.filters.size() != 0 {
            return false;
        }
        let opacity = outer.paint.as_ptr();
        if opacity.is_null() {
            return unsafe { Self::KillSaveLayerAndRestore(record, begin) };
        }
        let Some(filter_paint) = (unsafe { inner.paint.as_ptr().as_mut() }) else {
            return false;
        };
        if !fold_opacity_layer_color_to_paint(unsafe { opacity.as_ref() }, true, filter_paint) {
            return false;
        }
        unsafe { Self::KillSaveLayerAndRestore(record, begin) }
    }
}
// Source: src/core/SkRecordOpts.cpp:268-271
pub unsafe fn SkRecordMergeSvgOpacityAndFilterLayers(record: &mut SkRecord) {
    unsafe {
        apply(&mut SvgOpacityAndFilterLayerMergePass, record);
    }
}
// Source: src/core/SkRecordOpts.cpp:275-288
pub unsafe fn SkRecordOptimize(record: &mut SkRecord) {
    #[cfg(not(sk_build_for_android_framework))]
    unsafe {
        SkRecordNoopSaveLayerDrawRestores(record);
    }
    unsafe {
        SkRecordMergeSvgOpacityAndFilterLayers(record);
    }
    record.defrag();
}
