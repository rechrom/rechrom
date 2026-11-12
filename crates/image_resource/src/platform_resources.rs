//! Immutable Blink platform-resource identities, independent from URL fetching.
//! Source blink_image_resources.grd:13; Image::LoadPlatformResource image.cc:101-111.
use crate::{ImageResourceContentLease, PaintImage};
use std::{rc::Rc, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformImageResource {
    BrokenImage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformResourceScale {
    Scale100,
    Scale200,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlatformImageVariant {
    pub resource: PlatformImageResource,
    pub scale: PlatformResourceScale,
}
impl PlatformImageVariant {
    pub fn EncodedBytes(self) -> &'static [u8] {
        match (self.resource, self.scale) {
            (PlatformImageResource::BrokenImage, PlatformResourceScale::Scale100) => {
                include_bytes!("../resources/blink/100_percent/broken_image.png")
            }
            (PlatformImageResource::BrokenImage, PlatformResourceScale::Scale200) => {
                include_bytes!("../resources/blink/200_percent/broken_image.png")
            }
        }
    }
}
/// The normal bitmap decoder's immutable owner, cached once per resource engine.
pub struct PlatformDecodedImage {
    pub variant: PlatformImageVariant,
    pub image: PaintImage,
}
/// Both native resource-scale variants are published before UA fallback layout.
/// LayoutImageResource owns selecting the DPR bucket (>=2 selects Scale200).
#[derive(Clone)]
pub struct BrokenImageResourceVariants {
    pub scale100: Rc<dyn ImageResourceContentLease>,
    pub scale200: Rc<dyn ImageResourceContentLease>,
}
pub type PlatformDecodedImageHandle = Arc<PlatformDecodedImage>;
