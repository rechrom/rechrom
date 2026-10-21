use crate::include::core::SkRefCnt::sk_sp;
use crate::include::core::{
    SkBlender::SkBlender,
    SkClipOp::SkClipOp,
    SkSamplingOptions::{SkFilterMode, SkSamplingOptions},
    SkTileMode::SkTileMode,
};
use crate::include::private::SkTemplates::AutoTArray;
use crate::src::core::SkPicture::SkPicture;
use crate::src::core::{
    SkBlendMode::SkBlendMode,
    SkCanvas::{
        ImageSetEntry, LatticeRectType, PointMode, QuadAAFlags, SaveLayerFlags, SrcRectConstraint,
    },
    SkColor::{SkColor, SkColor4f},
    SkData::SkData,
    SkM44::SkM44,
    SkMatrix::SkMatrix,
    SkPaint::SkPaint,
    SkPath::SkPath,
    SkPoint::SkPoint,
    SkRRect::SkRRect,
    SkRSXform::SkRSXform,
    SkRect::{SkIRect, SkRect},
    SkRegion::SkRegion,
    SkString::SkString,
    SkTextBlob::SkTextBlob,
    SkVertices::SkVertices,
};
use crate::src::{
    core::SkImageFilter::SkImageFilter, image::SkImage::SkImage, shaders::SkShader::SkShader,
};
use core::{mem::ManuallyDrop, ptr};
use std::sync::Arc;
// Rust template metadata protocol; language scaffolding has no separate source credit.
pub unsafe trait RecordCommand {
    const TYPE: Type;
    const TAGS: i32;
}
// Source: src/core/SkRecords.h:57-99
macro_rules! SK_RECORD_TYPES {($m:ident)=>{$m!{NoOp:NoOp_Type,Restore:Restore_Type,Save:Save_Type,SaveLayer:SaveLayer_Type,SaveBehind:SaveBehind_Type,SetMatrix:SetMatrix_Type,SetM44:SetM44_Type,Translate:Translate_Type,Scale:Scale_Type,Concat:Concat_Type,Concat44:Concat44_Type,ClipPath:ClipPath_Type,ClipRRect:ClipRRect_Type,ClipRect:ClipRect_Type,ClipRegion:ClipRegion_Type,ClipShader:ClipShader_Type,ResetClip:ResetClip_Type,DrawArc:DrawArc_Type,DrawDrawable:DrawDrawable_Type,DrawImage:DrawImage_Type,DrawImageLattice:DrawImageLattice_Type,DrawImageRect:DrawImageRect_Type,DrawDRRect:DrawDRRect_Type,DrawOval:DrawOval_Type,DrawBehind:DrawBehind_Type,DrawPaint:DrawPaint_Type,DrawPath:DrawPath_Type,DrawPatch:DrawPatch_Type,DrawPicture:DrawPicture_Type,DrawPoints:DrawPoints_Type,DrawRRect:DrawRRect_Type,DrawRect:DrawRect_Type,DrawRegion:DrawRegion_Type,DrawTextBlob:DrawTextBlob_Type,DrawSlug:DrawSlug_Type,DrawAtlas:DrawAtlas_Type,DrawVertices:DrawVertices_Type,DrawMesh:DrawMesh_Type,DrawShadowRec:DrawShadowRec_Type,DrawAnnotation:DrawAnnotation_Type,DrawEdgeAAQuad:DrawEdgeAAQuad_Type,DrawEdgeAAImageSet:DrawEdgeAAImageSet_Type}};}
pub(crate) use SK_RECORD_TYPES;

// Source: src/core/SkRecords.h:103-105
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Type {
    NoOp_Type,
    Restore_Type,
    Save_Type,
    SaveLayer_Type,
    SaveBehind_Type,
    SetMatrix_Type,
    SetM44_Type,
    Translate_Type,
    Scale_Type,
    Concat_Type,
    Concat44_Type,
    ClipPath_Type,
    ClipRRect_Type,
    ClipRect_Type,
    ClipRegion_Type,
    ClipShader_Type,
    ResetClip_Type,
    DrawArc_Type,
    DrawDrawable_Type,
    DrawImage_Type,
    DrawImageLattice_Type,
    DrawImageRect_Type,
    DrawDRRect_Type,
    DrawOval_Type,
    DrawBehind_Type,
    DrawPaint_Type,
    DrawPath_Type,
    DrawPatch_Type,
    DrawPicture_Type,
    DrawPoints_Type,
    DrawRRect_Type,
    DrawRect_Type,
    DrawRegion_Type,
    DrawTextBlob_Type,
    DrawSlug_Type,
    DrawAtlas_Type,
    DrawVertices_Type,
    DrawMesh_Type,
    DrawShadowRec_Type,
    DrawAnnotation_Type,
    DrawEdgeAAQuad_Type,
    DrawEdgeAAImageSet_Type,
}
// Source: src/core/SkRecords.h:107-110
// Native pointer projection; these arena pointers do not own their allocation.
// Source: src/core/SkRecords.h:113-128
pub struct Optional<T> {
    fPtr: *mut T,
}
impl<T> Optional<T> {
    pub fn new() -> Self {
        Self {
            fPtr: ptr::null_mut(),
        }
    }
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { fPtr: ptr }
    }
    pub fn move_from(o: &mut Self) -> Self {
        Self {
            fPtr: core::mem::replace(&mut o.fPtr, ptr::null_mut()),
        }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.fPtr
    }
    pub fn arrow(&self) -> *mut T {
        self.fPtr
    }
    pub fn data(&self) -> *mut T {
        self.fPtr
    }
    pub unsafe fn as_ref(&self) -> Option<&T> {
        self.fPtr.as_ref()
    }
    pub unsafe fn as_mut(&mut self) -> Option<&mut T> {
        self.fPtr.as_mut()
    }
}
impl<T> Drop for Optional<T> {
    fn drop(&mut self) {
        if !self.fPtr.is_null() {
            unsafe { ptr::drop_in_place(self.fPtr) }
        }
    }
}
// Source: src/core/SkRecords.h:131-143
pub struct PODArray<T> {
    fPtr: *mut T,
}
impl<T> PODArray<T> {
    pub fn new_uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self { fPtr: ptr }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.fPtr
    }
    pub fn arrow(&self) -> *mut T {
        self.fPtr
    }
    pub fn data(&self) -> *mut T {
        self.fPtr
    }
}
impl<T> Copy for PODArray<T> {}
impl<T> Clone for PODArray<T> {
    fn clone(&self) -> Self {
        *self
    }
}
// Source: src/core/SkRecords.h:147-150
#[repr(transparent)]
#[derive(Clone)]
pub struct TypedMatrix {
    pub matrix: SkMatrix,
}
impl Default for TypedMatrix {
    fn default() -> Self {
        Self {
            matrix: SkMatrix::default(),
        }
    }
}
impl core::ops::Deref for TypedMatrix {
    type Target = SkMatrix;
    fn deref(&self) -> &SkMatrix {
        &self.matrix
    }
}
impl core::ops::DerefMut for TypedMatrix {
    fn deref_mut(&mut self) -> &mut SkMatrix {
        &mut self.matrix
    }
}
// Source: src/core/SkRecords.h:152-161
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tags {
    kDraw_Tag = 1,
    kHasImage_Tag = 2,
    kHasText_Tag = 4,
    kHasPaint_Tag = 8,
    kMultiDraw_Tag = 16,
    kDrawWithPaint_Tag = 9,
}
pub const kDraw_Tag: i32 = 1;
pub const kHasImage_Tag: i32 = 2;
pub const kHasText_Tag: i32 = 4;
pub const kHasPaint_Tag: i32 = 8;
pub const kMultiDraw_Tag: i32 = 16;
pub const kDrawWithPaint_Tag: i32 = kDraw_Tag | kHasPaint_Tag;
// Source: src/core/SkRecords.h:164-175
// ManuallyDrop is layout-transparent field storage. The generated destructor
// drops native members in reverse declaration order, unlike Rust's default order.
macro_rules! reverse_drop {($self:ident;) => {};($self:ident;$f:ident $(,$rest:ident)*)=>{reverse_drop!($self;$($rest),*);unsafe{ManuallyDrop::drop(&mut $self.$f);}};}
macro_rules! record {($name:ident,$variant:ident,$tags:expr,{$($field:ident:$ty:ty),*})=>{
 pub struct $name {$(pub $field:ManuallyDrop<$ty>),*}
 impl $name {pub const kType:Type=Type::$variant;pub const kTags:i32=$tags;
  pub fn new($($field:$ty),*)->Self {Self{$($field:ManuallyDrop::new($field)),*}}
 }
 unsafe impl RecordCommand for $name {const TYPE:Type=Type::$variant;const TAGS:i32=$tags;}
 impl Drop for $name {fn drop(&mut self){reverse_drop!(self;$($field),*);}}
};}
// Source: src/core/SkRecords.h:177-177
record!(NoOp, NoOp_Type, 0, {});
// Source: src/core/SkRecords.h:178-179
record!(Restore,Restore_Type,0,{matrix:TypedMatrix});
// Source: src/core/SkRecords.h:180-180
record!(Save, Save_Type, 0, {});
// Source: src/core/SkRecords.h:182-189
record!(SaveLayer,SaveLayer_Type,kHasPaint_Tag,{bounds:Optional<SkRect>,paint:Optional<SkPaint>,backdrop:Option<Arc<dyn SkImageFilter>>,saveLayerFlags:SaveLayerFlags,backdropScale:f32,backdropTileMode:SkTileMode,filters:AutoTArray<Option<Arc<dyn SkImageFilter>>>});
// Source: src/core/SkRecords.h:191-192
record!(SaveBehind,SaveBehind_Type,0,{subset:Optional<SkRect>});
// Source: src/core/SkRecords.h:194-195
record!(SetMatrix,SetMatrix_Type,0,{matrix:TypedMatrix});
// Source: src/core/SkRecords.h:196-197
record!(SetM44,SetM44_Type,0,{matrix:SkM44});
// Source: src/core/SkRecords.h:198-199
record!(Concat,Concat_Type,0,{matrix:TypedMatrix});
// Source: src/core/SkRecords.h:200-201
record!(Concat44,Concat44_Type,0,{matrix:SkM44});
// Source: src/core/SkRecords.h:203-205
record!(Translate,Translate_Type,0,{dx:f32,dy:f32});
// Source: src/core/SkRecords.h:207-209
record!(Scale,Scale_Type,0,{sx:f32,sy:f32});
// Source: src/core/SkRecords.h:211-222
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ClipOpAndAA {
    bits: u32,
}
impl ClipOpAndAA {
    pub fn new_uninit() -> core::mem::MaybeUninit<Self> {
        core::mem::MaybeUninit::uninit()
    }
    pub fn new(op: crate::include::core::SkClipOp::SkClipOp, aa: bool) -> Self {
        Self {
            bits: op as u32 | ((aa as u32) << 31),
        }
    }
    pub fn op(&self) -> crate::include::core::SkClipOp::SkClipOp {
        match self.bits & 0x7fffffff {
            0 => crate::include::core::SkClipOp::SkClipOp::kDifference,
            1 => crate::include::core::SkClipOp::SkClipOp::kIntersect,
            _ => unreachable!("native clip operation representation"),
        }
    }
    pub fn aa(&self) -> bool {
        self.bits >> 31 != 0
    }
}
const _: () = assert!(core::mem::size_of::<ClipOpAndAA>() == 4);
// Source: src/core/SkRecords.h:224-226
record!(ClipPath,ClipPath_Type,0,{path:SkPath,opAA:ClipOpAndAA});
// Source: src/core/SkRecords.h:227-229
record!(ClipRRect,ClipRRect_Type,0,{rrect:SkRRect,opAA:ClipOpAndAA});
// Source: src/core/SkRecords.h:230-232
record!(ClipRect,ClipRect_Type,0,{rect:SkRect,opAA:ClipOpAndAA});
// Source: src/core/SkRecords.h:233-235
record!(ClipRegion,ClipRegion_Type,0,{region:SkRegion,op:SkClipOp});
// Source: src/core/SkRecords.h:236-238
record!(ClipShader,ClipShader_Type,0,{shader:Option<Arc<dyn SkShader>>,op:SkClipOp});
// Source: src/core/SkRecords.h:239-239
record!(ResetClip, ResetClip_Type, 0, {});
// Source: src/core/SkRecords.h:242-247
record!(DrawArc,DrawArc_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,oval:SkRect,startAngle:f32,sweepAngle:f32,useCenter:u32});
// Source: src/core/SkRecords.h:248-251
record!(DrawDRRect,DrawDRRect_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,outer:SkRRect,inner:SkRRect});
// Source: src/core/SkRecords.h:252-255
record!(DrawDrawable,DrawDrawable_Type,kDraw_Tag,{matrix:Optional<SkMatrix>,worstCaseBounds:SkRect,index:i32});
// Source: src/core/SkRecords.h:256-261
record!(DrawImage,DrawImage_Type,kDraw_Tag|kHasImage_Tag|kHasPaint_Tag,{paint:Optional<SkPaint>,image:Option<Arc<dyn SkImage>>,left:f32,top:f32,sampling:SkSamplingOptions});
// Source: src/core/SkRecords.h:262-274
record!(DrawImageLattice,DrawImageLattice_Type,kDraw_Tag|kHasImage_Tag|kHasPaint_Tag,{paint:Optional<SkPaint>,image:Option<Arc<dyn SkImage>>,xCount:i32,xDivs:PODArray<i32>,yCount:i32,yDivs:PODArray<i32>,flagCount:i32,flags:PODArray<LatticeRectType>,colors:PODArray<SkColor>,src:SkIRect,dst:SkRect,filter:SkFilterMode});
// Source: src/core/SkRecords.h:275-281
record!(DrawImageRect,DrawImageRect_Type,kDraw_Tag|kHasImage_Tag|kHasPaint_Tag,{paint:Optional<SkPaint>,image:Option<Arc<dyn SkImage>>,src:SkRect,dst:SkRect,sampling:SkSamplingOptions,constraint:SrcRectConstraint});
// Source: src/core/SkRecords.h:282-284
record!(DrawOval,DrawOval_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,oval:SkRect});
// Source: src/core/SkRecords.h:285-286
record!(DrawPaint,DrawPaint_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint});
// Source: src/core/SkRecords.h:287-288
record!(DrawBehind,DrawBehind_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint});
// Source: src/core/SkRecords.h:289-291
record!(DrawPath,DrawPath_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,path:SkPath});
// Source: src/core/SkRecords.h:292-295
record!(DrawPicture,DrawPicture_Type,kDraw_Tag|kHasPaint_Tag,{paint:Optional<SkPaint>,picture:sk_sp<dyn SkPicture>,matrix:TypedMatrix});
// Source: src/core/SkRecords.h:296-300
record!(DrawPoints,DrawPoints_Type,kDraw_Tag|kHasPaint_Tag|kMultiDraw_Tag,{paint:SkPaint,mode:PointMode,count:u32,pts:PODArray<SkPoint>});
// Source: src/core/SkRecords.h:301-303
record!(DrawRRect,DrawRRect_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,rrect:SkRRect});
// Source: src/core/SkRecords.h:304-306
record!(DrawRect,DrawRect_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,rect:SkRect});
// Source: src/core/SkRecords.h:307-309
record!(DrawRegion,DrawRegion_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,region:SkRegion});
// Source: src/core/SkRecords.h:310-314
record!(DrawTextBlob,DrawTextBlob_Type,kDraw_Tag|kHasText_Tag|kHasPaint_Tag,{paint:SkPaint,blob:sk_sp<SkTextBlob>,x:f32,y:f32});
// Source: src/core/SkRecords.h:315-317
record!(DrawSlug,DrawSlug_Type,kDraw_Tag|kHasText_Tag|kHasPaint_Tag,{paint:SkPaint,slug:sk_sp<dyn crate::include::private::chromium::Slug::Slug>});
// Source: src/core/SkRecords.h:318-323
record!(DrawPatch,DrawPatch_Type,kDraw_Tag|kHasPaint_Tag,{paint:SkPaint,cubics:PODArray<SkPoint>,colors:PODArray<SkColor>,texCoords:PODArray<SkPoint>,bmode:SkBlendMode});
// Source: src/core/SkRecords.h:324-333
record!(DrawAtlas,DrawAtlas_Type,kDraw_Tag|kHasImage_Tag|kHasPaint_Tag|kMultiDraw_Tag,{paint:Optional<SkPaint>,atlas:Option<Arc<dyn SkImage>>,xforms:PODArray<SkRSXform>,texs:PODArray<SkRect>,colors:PODArray<SkColor>,count:u32,mode:SkBlendMode,sampling:SkSamplingOptions,cull:Optional<SkRect>});
// Source: src/core/SkRecords.h:334-337
record!(DrawVertices,DrawVertices_Type,kDraw_Tag|kHasPaint_Tag|kMultiDraw_Tag,{paint:SkPaint,vertices:sk_sp<SkVertices>,bmode:SkBlendMode});
// Source: src/core/SkRecords.h:338-341
record!(DrawMesh,DrawMesh_Type,kDraw_Tag|kHasPaint_Tag|kMultiDraw_Tag,{paint:SkPaint,mesh:crate::src::core::SkMesh::SkMesh,blender:sk_sp<dyn SkBlender>});
// Source: src/core/SkRecords.h:342-344
record!(DrawShadowRec,DrawShadowRec_Type,kDraw_Tag,{path:SkPath,rec:crate::src::core::SkDrawShadowInfo::SkDrawShadowRec});
// Source: src/core/SkRecords.h:345-348
record!(DrawAnnotation,DrawAnnotation_Type,0,{rect:SkRect,key:SkString,value:Option<Arc<SkData>>});
// Source: src/core/SkRecords.h:349-354
record!(DrawEdgeAAQuad,DrawEdgeAAQuad_Type,kDraw_Tag,{rect:SkRect,clip:PODArray<SkPoint>,aa:QuadAAFlags,color:SkColor4f,mode:SkBlendMode});
// Source: src/core/SkRecords.h:355-362
record!(DrawEdgeAAImageSet,DrawEdgeAAImageSet_Type,kDraw_Tag|kHasImage_Tag|kHasPaint_Tag|kMultiDraw_Tag,{paint:Optional<SkPaint>,set:AutoTArray<ImageSetEntry>,count:i32,dstClips:PODArray<SkPoint>,preViewMatrices:PODArray<SkMatrix>,sampling:SkSamplingOptions,constraint:SrcRectConstraint});
// Source: src/core/SkRecords.cpp:10-13
impl TypedMatrix {
    pub fn new(matrix: &SkMatrix) -> Self {
        let result = Self {
            matrix: matrix.clone(),
        };
        let _ = result.matrix.getType();
        result
    }
}
