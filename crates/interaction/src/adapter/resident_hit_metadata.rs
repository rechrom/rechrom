//! HitTestResult metadata aliases over live DOM/resource owners.
//! Source hit_test_result.cc:437-536,585-606; resource ownership remains canonical.
use interaction_hit_test::core::layout::hit_test_result as hit;
use std::rc::Rc;
struct ResidentURL<T> {
    _owner: Rc<T>,
    text: String,
}
impl<T> hit::HitTestUrl for ResidentURL<T> {
    fn GetString(&self) -> &str {
        &self.text
    }
}
pub(super) fn URL<T: 'static>(owner: Rc<T>, text: String) -> hit::Url {
    Some(Rc::new(ResidentURL {
        _owner: owner,
        text,
    }))
}
pub(super) fn DocumentURL(owner: Rc<dom::document_base_url::DocumentURLValue>) -> hit::Url {
    let text = owner.AsStr().to_owned();
    URL(owner, text)
}
pub(super) struct ResidentImageContent(pub Rc<dyn image_resource::ImageResourceContentLease>);
struct ResidentImage {
    _owner: Rc<dyn image_resource::ImageResourceContentLease>,
}
impl hit::HitTestImage for ResidentImage {}
impl hit::HitTestImageContent for ResidentImageContent {
    fn ErrorOccurred(&self) -> bool {
        self.0.ErrorOccurred()
    }
    fn HasImage(&self) -> bool {
        self.0.HasImage()
    }
    fn Image(&self) -> Option<hit::ImageRef> {
        self.0.HasImage().then(|| {
            Rc::new(ResidentImage {
                _owner: self.0.clone(),
            }) as hit::ImageRef
        })
    }
    fn IsAutomaticUpgrade(&self) -> bool {
        self.0.IsAutomaticUpgrade()
    }
    fn Url(&self) -> hit::Url {
        // The lease retains ResourceRequest's URL; this is its display value.
        Some(Rc::new(LeaseURL(self.0.clone(), self.0.Url())))
    }
}
struct LeaseURL(Rc<dyn image_resource::ImageResourceContentLease>, String);
impl hit::HitTestUrl for LeaseURL {
    fn GetString(&self) -> &str {
        let _ = &self.0;
        &self.1
    }
}
pub(super) struct ResidentMedia(pub Rc<dom::persistent_document::HTMLMediaElementState>);
struct ResidentStream(Rc<dyn dom::persistent_document::DOMMediaStreamDescriptor>);
impl hit::HitTestMediaStreamDescriptor for ResidentStream {}
struct ResidentSource(Rc<dyn dom::persistent_document::DOMMediaSourceHandle>);
impl hit::HitTestMediaSourceHandle for ResidentSource {}
impl hit::HitTestMedia for ResidentMedia {
    fn CurrentSrc(&self) -> hit::Url {
        let owner = self.0.current_src.borrow().clone()?;
        let text = owner.as_str().to_owned();
        URL(owner, text)
    }
    fn SrcObject(&self) -> hit::HitTestMediaSrcObject {
        match self.0.src_object.borrow().clone() {
            dom::persistent_document::DOMMediaSrcObject::Stream(value) => {
                hit::HitTestMediaSrcObject::Stream(value.map(|owner| {
                    Rc::new(ResidentStream(owner)) as Rc<dyn hit::HitTestMediaStreamDescriptor>
                }))
            }
            dom::persistent_document::DOMMediaSrcObject::Source(value) => {
                hit::HitTestMediaSrcObject::Source(value.map(|owner| {
                    Rc::new(ResidentSource(owner)) as Rc<dyn hit::HitTestMediaSourceHandle>
                }))
            }
        }
    }
}
