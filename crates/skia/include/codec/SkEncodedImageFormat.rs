// Copyright 2015 Google Inc. BSD-style license in LICENSE.
// Source: include/codec/SkEncodedImageFormat.h:16-31.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkEncodedImageFormat {
    kBMP,
    kGIF,
    kICO,
    kJPEG,
    kPNG,
    kWBMP,
    kWEBP,
    kPKM,
    kKTX,
    kASTC,
    kDNG,
    kHEIF,
    kAVIF,
    kJPEGXL,
}
