/*
 * Copyright (C) 2012 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/simulated_click_options.h:1-34

// Source: simulated_click_options.h:26-30
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulatedClickCreationScope {
    kFromScript,
    kFromUserAgent,
    kFromAccessibility,
}
