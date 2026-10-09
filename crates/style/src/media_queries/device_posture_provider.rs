// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
#![allow(non_camel_case_types, non_upper_case_globals)]
// cpp: third_party/blink/public/mojom/device_posture/device_posture_provider.mojom:9-12
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevicePostureType {
    kContinuous,
    kFolded,
}
