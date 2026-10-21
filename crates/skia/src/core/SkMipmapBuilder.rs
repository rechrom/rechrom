use super::{
    SkImageInfo::SkImageInfo,
    SkMipmap::{Level, SkMipmap},
    SkPixmap::SkPixmap,
};
// Source: src/core/SkMipmapBuilder.h:18-19
// Source: src/core/SkMipmapBuilder.h:33-34
pub struct SkMipmapBuilder {
    fMM: Option<SkMipmap>,
}
// Source: src/core/SkMipmapBuilder.cpp:15-19
impl SkMipmapBuilder {
    pub fn new(info: &SkImageInfo) -> Result<Self, crate::RasterError> {
        Ok(Self {
            fMM: unsafe {
                SkMipmap::Build(&SkPixmap::new(info, std::ptr::null(), 0), None, false)
            }?,
        })
    }
}
// Source: src/core/SkMipmapBuilder.cpp:21-21
impl Drop for SkMipmapBuilder {
    fn drop(&mut self) {}
}
// Source: src/core/SkMipmapBuilder.cpp:23-25
impl SkMipmapBuilder {
    pub fn countLevels(&self) -> i32 {
        self.fMM.as_ref().map_or(0, |m| m.countLevels())
    }
}
// Source: src/core/SkMipmapBuilder.cpp:27-28
// Source: src/core/SkMipmapBuilder.cpp:30-35
impl SkMipmapBuilder {
    pub fn level(&self, index: i32) -> SkPixmap {
        let mut pm = SkPixmap::default();
        let mut level = Level {
            fPixmap: SkPixmap::default(),
            fScale: crate::include::core::SkSize::SkSize::MakeEmpty(),
        };
        if self
            .fMM
            .as_ref()
            .is_some_and(|m| m.getLevel(index, Some(&mut level)))
        {
            pm = level.fPixmap.clone();
        }
        pm
    }
}
impl SkMipmapBuilder {
    // Source: src/core/SkMipmapBuilder.cpp:37-39
    /// Source retains immutable root pixels; this builder retains every mip level.
    pub unsafe fn attachTo(
        &self,
        src: &std::sync::Arc<dyn crate::src::image::SkImage::SkImage>,
    ) -> Result<std::sync::Arc<dyn crate::src::image::SkImage::SkImage>, crate::RasterError> {
        unsafe { src.withMipmaps(self.fMM.clone()) }
    }
}
