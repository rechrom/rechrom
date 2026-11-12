// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/platform/network/mime/mime_type_registry.cc:82-88.
pub(crate) fn GetWellKnownMIMETypeForExtension(extension: &str) -> &'static str {
    crate::net::base::mime_util::GetWellKnownMimeTypeFromExtension(extension)
}
