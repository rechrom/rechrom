/*
 * Copyright (C) 2012 Intel Corporation. All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 * 1. Redistributions of source code must retain the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer.
 * 2. Redistributions in binary form must reproduce the above
 *    copyright notice, this list of conditions and the following
 *    disclaimer in the documentation and/or other materials
 *    provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDER "AS IS" AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
 * PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY,
 * OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
 * PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
 * PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR
 * TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF
 * THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF
 * SUCH DAMAGE.
 */
// cpp: third_party/blink/renderer/core/css/resolver/viewport_style_resolver.h
// cpp: third_party/blink/renderer/core/css/resolver/viewport_style_resolver.cc
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Source ledger (physical / effective / mapped / omitted / pending):
//   viewport_style_resolver.h: 66 / 27 / 11 / 16 / 0.
//   viewport_style_resolver.cc: 119 / 70 / 53 / 17 / 0.
// Effective excludes comments and blanks, with braces retained. Every
// production declaration and statement is mapped.
// h mapped: 46,48-50,55-58,60-62.
// h omitted: 30-31,33-37,39,41,43-45,52,54,64,66.
// cc mapped: 45-46,48,50-52,54-58,60-62,64-68,70-71,77-79,83-92,94-104,106-113.
// cc omitted: 30,32-41,43,47,115-117,119.
// Omissions are include/preprocessor/namespace/forward/access/GC scaffolding,
// constructor DCHECK and Trace. No production behavior is omitted.
// This source version resolves settings-based UA viewport descriptions only;
// it contains no CSS viewport rules, descriptors or style-builder path.

use std::cell::Cell;
use std::rc::Rc;

/// Typed representation of mojom::blink::ViewportStyle at the settings boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewportStyle {
    kDefault,
    kMobile,
    kTelevision,
}

/// Foreign page/frame data ownership and ViewportDescription/ViewportLength
/// operations. Resolution and update scheduling decisions remain below.
pub trait ViewportStyleResolverBackend {
    type Document;
    type Page;
    type ChromeClient;
    type Settings;
    type ViewportData;
    type ViewportDescription;
    type ViewportLength;

    fn DocumentPage(document: &Self::Document) -> &Self::Page;
    fn PageChromeClient(page: &Self::Page) -> &Self::ChromeClient;
    fn ZoomFactorForViewportLayout(client: &Self::ChromeClient) -> f32;
    fn IsMobileDocument(document: &Self::Document) -> bool;
    fn DocumentSettings(document: &Self::Document) -> Option<&Self::Settings>;
    fn SettingsViewportStyle(settings: &Self::Settings) -> ViewportStyle;
    fn DocumentViewportData(document: &Self::Document) -> &Self::ViewportData;
    fn NewUserAgentViewportDescription() -> Self::ViewportDescription;
    fn SetMinZoom(description: &mut Self::ViewportDescription, zoom: f32);
    fn SetMaxZoom(description: &mut Self::ViewportDescription, zoom: f32);
    fn DeviceWidth() -> Self::ViewportLength;
    fn FixedLength(value: f32) -> Self::ViewportLength;
    fn SetMinWidth(description: &mut Self::ViewportDescription, width: Self::ViewportLength);
    fn SetViewportDescription(data: &Self::ViewportData, description: Self::ViewportDescription);
    fn ScheduleLayoutTreeUpdateIfNeeded(document: &Self::Document);
}

// cpp: viewport_style_resolver.h:60-61
pub struct ViewportStyleResolver<B: ViewportStyleResolverBackend> {
    document_: Rc<B::Document>,
    needs_update_: Cell<bool>,
}

impl<B: ViewportStyleResolverBackend> ViewportStyleResolver<B> {
    // cpp: viewport_style_resolver.cc:45-48
    pub fn new(document: Rc<B::Document>) -> Self {
        Self {
            document_: document,
            needs_update_: Cell::new(true),
        }
    }
    // cpp: viewport_style_resolver.h:50
    pub fn NeedsUpdate(&self) -> bool {
        self.needs_update_.get()
    }
    // cpp: viewport_style_resolver.cc:50-52
    fn Reset(&self) {
        self.needs_update_.set(false);
    }
    // cpp: viewport_style_resolver.cc:54-58
    fn DeviceScaleZoom(&self) -> f32 {
        let zoom =
            B::ZoomFactorForViewportLayout(B::PageChromeClient(B::DocumentPage(&self.document_)));
        if zoom != 0.0 {
            zoom
        } else {
            1.0
        }
    }
    // cpp: viewport_style_resolver.cc:60-92
    fn ResolveViewportDescription(&self, viewport_style: ViewportStyle) -> B::ViewportDescription {
        let mut description = B::NewUserAgentViewportDescription();
        if B::IsMobileDocument(&self.document_) {
            B::SetMinZoom(&mut description, 0.25);
            B::SetMaxZoom(&mut description, 5.0);
            return description;
        }
        match viewport_style {
            ViewportStyle::kDefault => B::SetMinWidth(&mut description, B::DeviceWidth()),
            ViewportStyle::kMobile => B::SetMinWidth(
                &mut description,
                B::FixedLength(980.0 * self.DeviceScaleZoom()),
            ),
            ViewportStyle::kTelevision => B::SetMinWidth(
                &mut description,
                B::FixedLength(1280.0 * self.DeviceScaleZoom()),
            ),
        }
        description
    }
    // cpp: viewport_style_resolver.cc:94-100
    fn Resolve(&self) {
        let viewport_style = B::DocumentSettings(&self.document_)
            .map(B::SettingsViewportStyle)
            .unwrap_or(ViewportStyle::kDefault);
        let data = B::DocumentViewportData(&self.document_);
        let description = self.ResolveViewportDescription(viewport_style);
        B::SetViewportDescription(data, description);
    }
    // cpp: viewport_style_resolver.cc:101-104
    pub fn SetNeedsUpdate(&self) {
        self.needs_update_.set(true);
        B::ScheduleLayoutTreeUpdateIfNeeded(&self.document_);
    }
    // cpp: viewport_style_resolver.cc:106-113
    pub fn UpdateViewport(&self) {
        if !self.needs_update_.get() {
            return;
        }
        self.Reset();
        self.Resolve();
        self.needs_update_.set(false);
    }
}
