// Copyright 2014 Google Inc. BSD-style license.
use crate::include::core::SkSurfaceProps::SkSurfaceProps;
// Source: src/core/SkSurfacePriv.h:15-17
pub fn SkSurfacePropsCopyOrDefault(props: Option<&SkSurfaceProps>) -> SkSurfaceProps {
    props.copied().unwrap_or_default()
}
// Source: src/core/SkSurfacePriv.h:19-19
pub const kIgnoreRowBytesValue: usize = !0usize;
// Source: src/core/SkSurfacePriv.h:21-21
// The original declaration resolves to the actual SkSurface_Raster.cpp body;
// reexport that canonical implementation instead of duplicating validation.
pub use crate::src::image::SkSurface_Raster::SkSurfaceValidateRasterInfo;
