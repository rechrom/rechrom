// Copyright 2021 Google LLC. BSD-style license.
use crate::include::core::SkRefCnt::SkRefCntObject;
use crate::src::core::SkStream::SkWStream;
use core::pin::Pin;

// Source: include/sksl/SkSLDebugTrace.h:17-21
// Native SkRefCnt inheritance uses the object's sole intrusive counter and
// dynamic concrete disposal. This trait introduces no owner or second counter.
pub trait DebugTrace: SkRefCntObject {
    fn dump(&self, o: Pin<&mut dyn SkWStream>);
}
