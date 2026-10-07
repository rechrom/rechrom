#![allow(non_snake_case, non_camel_case_types)]
use cssom::{CSSDeclaration, CSSStyleSheet};
use dom::{dom_mutation::DOMMutation, UserInteractionState};
use layoutng_assembly::internal::layout_input::{FontFace, Offset, PaintImage, Size};
use std::sync::Arc;

// cpp: page_mutation/page_mutation.h:17-22
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResourceKind {
    kStyleSheet,
    kScript,
    #[default]
    kImage,
    kFont,
}
// cpp: page_mutation/page_mutation.h:27-30
#[derive(Clone)]
pub struct ImageResourceReady {
    pub source: String,
    pub image: PaintImage,
}
#[derive(Clone)]
pub struct DocumentImageFrameChanged {
    pub source: String,
    pub frame: Arc<image_resource::DocumentImageFrame>,
}
#[derive(Clone)]
pub struct DocumentImageIntrinsicSizeChanged {
    pub source: String,
    pub resource_id: image_resource::ImageId,
    pub revision: u64,
    pub size: image_resource::IntrinsicSize,
}
// cpp: page_mutation/page_mutation.h:32-34
#[derive(Clone)]
pub struct FontResourceReady {
    pub font: FontFace,
}
// cpp: page_mutation/page_mutation.h:36-40
#[derive(Clone, Default)]
pub struct ResourceLoadFailed {
    pub kind: ResourceKind,
    pub url: String,
    pub error: String,
}
// cpp: page_mutation/page_mutation.h:42-43
#[derive(Clone)]
pub enum ResourceMutation {
    ImageResourceReady(ImageResourceReady),
    DocumentImageFrameChanged(DocumentImageFrameChanged),
    DocumentImageIntrinsicSizeChanged(DocumentImageIntrinsicSizeChanged),
    FontResourceReady(FontResourceReady),
    ResourceLoadFailed(ResourceLoadFailed),
}
// cpp: page_mutation/page_mutation.h:45-47
#[derive(Clone, Default)]
pub struct InteractionStateMutation {
    pub state: UserInteractionState,
}
// cpp: page_mutation/page_mutation.h:49-52
#[derive(Clone, Default)]
pub struct CSSOMMutation {
    pub style_sheet: CSSStyleSheet,
    pub base_url: String,
}
// cpp: page_mutation/page_mutation.h:54-56
#[derive(Clone, Default)]
pub struct ViewportMutation {
    pub size: Size,
}
// cpp: page_mutation/page_mutation.h:58-61
#[derive(Clone, Default)]
pub struct ScrollMutation {
    pub target_node_id: u64,
    pub offset: Offset,
}
// cpp: page_mutation/page_mutation.h:63-68
#[derive(Clone, Default)]
pub struct AnimationStyleSample {
    pub node_id: u64,
    pub effect_id: u64,
    pub declarations: Vec<CSSDeclaration>,
}
// cpp: page_mutation/page_mutation.h:70-73
#[derive(Clone, Default)]
pub struct AnimationTick {
    pub monotonic_time: f64,
    pub samples: Vec<AnimationStyleSample>,
}
// cpp: page_mutation/page_mutation.h:76-82
#[derive(Clone)]
pub enum PageMutation {
    DOMMutation(DOMMutation),
    ResourceMutation(ResourceMutation),
    InteractionStateMutation(InteractionStateMutation),
    CSSOMMutation(CSSOMMutation),
    ViewportMutation(ViewportMutation),
    ScrollMutation(ScrollMutation),
    AnimationTick(AnimationTick),
}
// cpp: page_mutation/page_mutation.h:83
// Rc<Fn> preserves the const-callable, synchronously reentrant C++ callback.
pub type PageMutationEmitter = std::rc::Rc<dyn Fn(PageMutation)>;
