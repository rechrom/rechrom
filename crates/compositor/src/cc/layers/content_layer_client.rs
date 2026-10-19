// Translation status: complete
// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/layers/content_layer_client.h:1-36

use std::sync::Arc;

// Source: cc/layers/content_layer_client.h:14-32. The generic parameter is the
// source `cc::DisplayItemList` boundary supplied by the Paint adapter.
pub trait ContentLayerClient<DisplayItemList, Context: ?Sized = ()> {
    // Source: cc/layers/content_layer_client.h:17-20. Rust trait objects are not
    // Clone or Copy unless a separate wrapper explicitly adds that behavior.

    // Source: cc/layers/content_layer_client.h:22-23
    fn PaintContentsToDisplayList(&mut self, context: &mut Context)
        -> Option<Arc<DisplayItemList>>;

    // Source: cc/layers/content_layer_client.h:25-27
    fn FillsBoundsCompletely(&self) -> bool;

    // Source: cc/layers/content_layer_client.h:29-31. Construction and
    // destruction are controlled by each implementer and Rust ownership.
}

// Source: cc/layers/content_layer_client.h:5-12,34-36. Includes, namespaces,
// export decoration, and include guards are represented by Rust modules.
