/*
 * Copyright (C) 1999 Lars Knoll (knoll@kde.org)
 * Copyright (C) 2009 Apple Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */
#![allow(non_upper_case_globals)]
// cpp: third_party/blink/renderer/platform/resolution_units.h:27-29,33,37
const kMillimetersPerCentimeter: f64 = 10.0;
const kQuarterMillimetersPerCentimeter: f64 = 40.0;
const kCentimetersPerInch: f64 = 2.54;
const kPicasPerInch: f64 = 6.0;
const kCssPixelsPerInch: f64 = 96.0;

// cpp: third_party/blink/renderer/core/css/css_resolution_units.h:30-36
pub const kCssPixelsPerCentimeter: f64 = kCssPixelsPerInch / kCentimetersPerInch;
pub const kCssPixelsPerMillimeter: f64 = kCssPixelsPerCentimeter / kMillimetersPerCentimeter;
pub const kCssPixelsPerQuarterMillimeter: f64 =
    kCssPixelsPerCentimeter / kQuarterMillimetersPerCentimeter;
pub const kCssPixelsPerPica: f64 = kCssPixelsPerInch / kPicasPerInch;
