use foundation::Member;

use super::content_data::{CloneContentData, ContentData};
use super::paint_images::PaintImages;

// cpp: layoutng_style/style/member_copy.cc:7-9
#[allow(non_snake_case)]
pub fn MemberCopyContentData(
    value: &Option<Member<dyn ContentData>>,
) -> Option<Member<dyn ContentData>> {
    value.as_ref().map(|member| {
        // A Rust trait-object pointer carries a vtable, so the nullable GC
        // edge uses Option<NonNull<dyn ContentData>> at this boundary.
        let ptr = member
            .GetNonNull()
            .expect("ContentData member is null")
            .as_ptr();
        Member::from_ptr(CloneContentData(unsafe { &*ptr }))
    })
}

// cpp: layoutng_style/style/member_copy.cc:10-12
#[allow(non_snake_case)]
pub fn MemberCopyPaintImages(value: &Member<PaintImages>) -> Member<PaintImages> {
    let ptr = value.Get();
    if ptr.is_null() {
        Member::default()
    } else {
        Member::from_ptr(unsafe { (&*ptr).Clone() })
    }
}
