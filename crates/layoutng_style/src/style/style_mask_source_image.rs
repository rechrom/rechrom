//! Chromium style_mask_source_image.{h,cc}. Local SVG resources retain scoped
//! identity, while external sources retain their genuine fetched image owner.
#![allow(non_snake_case)]
use super::{
    forward::CSSValue, natural_sizing_info::NaturalSizingInfo,
    reference_clip_path_operation::SvgClipResourceScope, style_image::*,
};
use crate::css::{css_image_value::CSSImageValue, css_url_data::CSSUrlData};
use foundation::{Member, ScopedRefPtr, Traceable, Visitor};
#[repr(C)]
pub struct StyleMaskSourceImage {
    base: StyleImage,
    url_data: CSSUrlData,
    scope: Option<SvgClipResourceScope>,
    image: Member<StyleImage>,
}
impl StyleMaskSourceImage {
    pub fn Local(url_data: CSSUrlData, scope: SvgClipResourceScope) -> Self {
        Self::new(url_data, Some(scope), std::ptr::null_mut())
    }
    pub fn Fetched(url_data: CSSUrlData, image: *mut StyleImage) -> Self {
        assert!(unsafe { image.as_ref() }.is_some_and(StyleImage::IsImageResource));
        Self::new(url_data, None, image)
    }
    fn new(
        url_data: CSSUrlData,
        scope: Option<SvgClipResourceScope>,
        image: *mut StyleImage,
    ) -> Self {
        let mut base = StyleImage::new_for_derived(&MASK_VTABLE);
        base.set_mask_source_for_derived(true);
        Self {
            base,
            url_data,
            scope,
            image: Member::from_ptr(image),
        }
    }
    pub fn ResourceScope(&self) -> Option<SvgClipResourceScope> {
        self.scope
    }
    pub fn IsLocal(&self) -> bool {
        self.scope.is_some()
    }
    pub fn SourceURL(&self) -> &str {
        self.url_data.resolved_url.as_ref()
    }
    pub fn FetchedImage(&self) -> Option<&StyleImage> {
        unsafe { self.image.Get().as_ref() }
    }
    pub fn NormalizedFragment(&self) -> String {
        // CSSImageValue::NormalizedFragmentIdentifier uses UTF-8-or-isomorphic
        // decoding. Fragment-only values retain their originating document.
        let source = if self.url_data.unresolved_url.starts_with('#') {
            self.url_data.unresolved_url.as_ref()
        } else {
            self.url_data.resolved_url.as_ref()
        };
        let raw = source.split_once('#').map_or("", |(_, fragment)| fragment);
        let bytes = raw.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let hex = |b: u8| (b as char).to_digit(16).map(|n| n as u8);
                if let (Some(a), Some(b)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    out.push(a * 16 + b);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8(out.clone()).unwrap_or_else(|_| out.into_iter().map(char::from).collect())
    }
}
impl Traceable for StyleMaskSourceImage {
    fn Trace(&self, v: &mut Visitor) {
        v.Trace(&self.image);
    }
}
fn mask(base: &StyleImage) -> &StyleMaskSourceImage {
    unsafe { &*(base as *const StyleImage).cast() }
}
pub fn MaskSourceImage(base: &StyleImage) -> Option<&StyleMaskSourceImage> {
    base.IsMaskSource().then(|| mask(base))
}
fn css(base: &StyleImage) -> *mut CSSValue {
    foundation::MakeGarbageCollected(CSSImageValue::new(
        mask(base).url_data.MakeComputed(),
        base as *const _ as *mut _,
    ))
    .cast()
}
static MASK_VTABLE: StyleImageVTable = StyleImageVTable {
    CssValue: css,
    ComputedCSSValue: |b, _, _, _| css(b),
    CanRender: |b| mask(b).FetchedImage().is_none_or(StyleImage::CanRender),
    IsLoaded: |b| mask(b).FetchedImage().is_none_or(StyleImage::IsLoaded),
    IsLoading: |b| mask(b).FetchedImage().is_some_and(StyleImage::IsLoading),
    ErrorOccurred: |b| {
        mask(b)
            .FetchedImage()
            .is_some_and(StyleImage::ErrorOccurred)
    },
    IsCorsSameOrigin: |b| {
        mask(b)
            .FetchedImage()
            .is_none_or(StyleImage::IsCorsSameOrigin)
    },
    GetNaturalSizingInfo: |b, m, o| {
        mask(b)
            .FetchedImage()
            .map_or_else(NaturalSizingInfo::None, |i| i.GetNaturalSizingInfo(m, o))
    },
    ImageSize: |b, m, s, o| {
        mask(b)
            .FetchedImage()
            .map_or_else(Default::default, |i| i.ImageSize(m, s, o))
    },
    HasIntrinsicSize: |b| {
        mask(b)
            .FetchedImage()
            .is_some_and(StyleImage::HasIntrinsicSize)
    },
    AddClient: |_, _| {},
    RemoveClient: |_, _| {},
    GetImage: |b, o, n, s, z| {
        mask(b)
            .FetchedImage()
            .map_or_else(ScopedRefPtr::default, |i| i.GetImage(o, n, s, z))
    },
    Data: |b| {
        mask(b)
            .FetchedImage()
            .map_or(mask(b) as *const _ as WrappedImagePtr, StyleImage::Data)
    },
    ImageScaleFactor: |b| {
        mask(b)
            .FetchedImage()
            .map_or(1., StyleImage::ImageScaleFactor)
    },
    CachedImage: |b| {
        mask(b)
            .FetchedImage()
            .map_or(std::ptr::null_mut(), StyleImage::CachedImage)
    },
    DependsOnCurrentColor: StyleImageDefaultDependsOnCurrentColor,
    IsLoadedAfterMouseover: StyleImageDefaultIsLoadedAfterMouseover,
    Trace: |b, v| mask(b).Trace(v),
    IsEqual: |b, o| {
        MaskSourceImage(o).is_some_and(|o| {
            let b = mask(b);
            b.url_data == o.url_data && b.scope == o.scope && b.FetchedImage() == o.FetchedImage()
        })
    },
};

#[cfg(test)]
mod tests {
    use super::*;
    fn local(url: &str, document_id: u64) -> StyleMaskSourceImage {
        StyleMaskSourceImage::Local(
            CSSUrlData::unresolved(foundation::String::from(url)),
            SvgClipResourceScope {
                document_id,
                tree_scope_id: 1,
            },
        )
    }
    #[test]
    fn local_mask_identity_is_scoped_terminal_and_not_a_fetch() {
        let _heap = foundation::LayoutHeapScope::new();
        let a = local("#mask", 1);
        let b = local("#mask", 2);
        assert!(a.base.IsMaskSource());
        assert!(!a.base.IsImageResource());
        assert!(a.base.IsLoaded());
        assert!(a.base.CanRender());
        assert!(!a.base.IsLoading());
        assert!(!a.base.ErrorOccurred());
        assert!(a.FetchedImage().is_none());
        assert!(a.base != b.base);
        assert!(a.base == local("#mask", 1).base);
    }
    #[test]
    fn normalized_fragment_uses_utf8_or_isomorphic_decoding() {
        let _heap = foundation::LayoutHeapScope::new();
        for (url, expected) in [
            ("#%6dask", "mask"),
            ("#%E4%B8%AD", "中"),
            ("#%FF", "ÿ"),
            ("#a%23b", "a#b"),
            ("#a%2", "a%2"),
        ] {
            assert_eq!(local(url, 1).NormalizedFragment(), expected);
        }
    }
}
