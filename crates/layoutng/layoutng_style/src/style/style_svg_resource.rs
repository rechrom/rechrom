use super::forward::{SVGResource, SVGResourceClient};
use foundation::{AtomicString, Member, Visitor};

// cpp: layoutng_style/style/style_svg_resource.h:20-43
pub struct StyleSVGResource {
    resource_: Member<SVGResource>,
    url_: AtomicString,
}

#[allow(non_snake_case)]
impl StyleSVGResource {
    // cpp: layoutng_style/style/style_svg_resource.h:22
    // No constructor definition exists in the supplied C++ tree.
    pub fn new(resource: *mut SVGResource, url: &AtomicString) -> Self {
        unsafe { StyleSVGResourceConstruct(resource, url) }
    }

    // cpp: layoutng_style/style/style_svg_resource.h:25
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.resource_);
    }

    // cpp: layoutng_style/style/style_svg_resource.h:31
    // No definition exists in the supplied C++ tree.
    pub fn AddClient(&mut self, client: &mut SVGResourceClient) {
        unsafe { StyleSVGResourceAddClient(self, client) }
    }

    // cpp: layoutng_style/style/style_svg_resource.h:32
    // No definition exists in the supplied C++ tree.
    pub fn RemoveClient(&mut self, client: &mut SVGResourceClient) {
        unsafe { StyleSVGResourceRemoveClient(self, client) }
    }

    // cpp: layoutng_style/style/style_svg_resource.h:34
    pub fn Resource(&self) -> *mut SVGResource {
        self.resource_.Get()
    }

    // cpp: layoutng_style/style/style_svg_resource.h:35
    pub fn Url(&self) -> &AtomicString {
        &self.url_
    }
}

// cpp: layoutng_style/style/style_svg_resource.h:27-29
impl PartialEq for StyleSVGResource {
    fn eq(&self, other: &Self) -> bool {
        self.resource_.Get() == other.resource_.Get()
    }
}

// cpp: layoutng_style/style/style_svg_resource.h:23
// No destructor definition exists in the supplied C++ tree.
impl Drop for StyleSVGResource {
    fn drop(&mut self) {
        unsafe { StyleSVGResourceDestroy(self) }
    }
}

unsafe extern "Rust" {
    fn StyleSVGResourceConstruct(
        resource: *mut SVGResource,
        url: &AtomicString,
    ) -> StyleSVGResource;
    fn StyleSVGResourceDestroy(value: &mut StyleSVGResource);
    fn StyleSVGResourceAddClient(value: &mut StyleSVGResource, client: &mut SVGResourceClient);
    fn StyleSVGResourceRemoveClient(value: &mut StyleSVGResource, client: &mut SVGResourceClient);
}
