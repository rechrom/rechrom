// Copyright 2012 Google Inc. BSD-style license.
// UNVERIFIED. Native strings retain arbitrary bytes plus a trailing NUL; arrays
// use MaybeUninit so partially read or newly reset storage is never fabricated.
// Native field order is grouped; Drop explicitly preserves reverse C++ ownership.
use super::{
    SkData::SkData,
    SkScalar::{SkFloatInterpFunc, SkScalarRoundToInt},
    SkStream::{SkMemoryStream, SkStream, SkStreamAsset, SkStreamPriv, SkWStream},
};
use crate::include::core::{
    SkFontArguments::{Coordinate, Palette, PaletteOverride, SkFontArguments, VariationPosition},
    SkFontStyle::{SkFontStyle, Slant},
};
use core::{
    ffi::c_char,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
    ptr,
};
use std::sync::Arc;

// Rust allocation adapter for Auto[ST]Malloc; no initialization of native fields.
fn uninit_array<T>(count: usize) -> Vec<MaybeUninit<T>> {
    let mut values = Vec::with_capacity(count);
    values.resize_with(count, MaybeUninit::uninit);
    values
}
fn set_name(dst: &mut Vec<MaybeUninit<u8>>, src: &[u8]) {
    let n = src.iter().position(|b| *b == 0).unwrap_or(src.len());
    dst.clear();
    dst.extend(src[..n].iter().copied().map(MaybeUninit::new));
    dst.push(MaybeUninit::new(0));
}

// Source: src/core/SkFontDescriptor.h:24-24
// Source: src/core/SkFontDescriptor.h:76-82
pub struct SkFontData {
    fStream: ManuallyDrop<Option<Pin<Box<dyn SkStreamAsset>>>>,
    fIndex: i32,
    fPaletteIndex: i32,
    fAxisCount: i32,
    fPaletteOverrideCount: i32,
    fAxis: ManuallyDrop<Vec<MaybeUninit<i32>>>,
    fPaletteOverrides: ManuallyDrop<Vec<MaybeUninit<PaletteOverride>>>,
}
impl SkFontData {
    // Source: src/core/SkFontDescriptor.h:27-44
    /// Safety: input arrays cover their nonnegative counts with initialized values.
    pub unsafe fn new(
        stream: Option<Pin<Box<dyn SkStreamAsset>>>,
        index: i32,
        palette_index: i32,
        axis: *const i32,
        axis_count: i32,
        palette_overrides: *const PaletteOverride,
        palette_override_count: i32,
    ) -> Self {
        assert!(axis_count >= 0 && palette_override_count >= 0);
        let mut result = Self {
            fStream: ManuallyDrop::new(stream),
            fIndex: index,
            fPaletteIndex: palette_index,
            fAxisCount: axis_count,
            fPaletteOverrideCount: palette_override_count,
            fAxis: ManuallyDrop::new(uninit_array(axis_count as usize)),
            fPaletteOverrides: ManuallyDrop::new(uninit_array(palette_override_count as usize)),
        };
        for i in 0..axis_count as usize {
            result.fAxis[i].write(axis.add(i).read());
        }
        for i in 0..palette_override_count as usize {
            result.fPaletteOverrides[i].write(palette_overrides.add(i).read());
        }
        result
    }
    // Source: src/core/SkFontDescriptor.h:46-61
    /// Native copying requires a nonnull stream; its duplicate may itself fail.
    pub fn copy(&self) -> Self {
        let stream = SkStreamAsset::duplicate(
            self.fStream
                .as_ref()
                .expect("native copy requires stream")
                .as_ref()
                .get_ref(),
        );
        let mut result = Self {
            fStream: ManuallyDrop::new(stream),
            fIndex: self.fIndex,
            fPaletteIndex: self.fPaletteIndex,
            fAxisCount: self.fAxisCount,
            fPaletteOverrideCount: self.fPaletteOverrideCount,
            fAxis: ManuallyDrop::new(uninit_array(self.fAxisCount as usize)),
            fPaletteOverrides: ManuallyDrop::new(uninit_array(self.fPaletteOverrideCount as usize)),
        };
        for i in 0..self.fAxisCount as usize {
            result.fAxis[i] = self.fAxis[i];
        }
        for i in 0..self.fPaletteOverrideCount as usize {
            result.fPaletteOverrides[i] = self.fPaletteOverrides[i];
        }
        result
    }
    // Source: src/core/SkFontDescriptor.h:62-73
    pub fn hasStream(&self) -> bool {
        self.fStream.is_some()
    }
    pub fn detachStream(&mut self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        self.fStream.take()
    }
    pub fn getStream_mut(&mut self) -> Option<Pin<&mut (dyn SkStreamAsset + 'static)>> {
        self.fStream.as_mut().map(|stream| stream.as_mut())
    }
    pub fn getStream(&self) -> Option<&dyn SkStreamAsset> {
        self.fStream
            .as_ref()
            .map(|stream| stream.as_ref().get_ref())
    }
    pub fn getIndex(&self) -> i32 {
        self.fIndex
    }
    pub fn getAxisCount(&self) -> i32 {
        self.fAxisCount
    }
    pub fn getAxis(&self) -> *const i32 {
        self.fAxis.as_ptr().cast()
    }
    pub fn getPaletteIndex(&self) -> i32 {
        self.fPaletteIndex
    }
    pub fn getPaletteOverrideCount(&self) -> i32 {
        self.fPaletteOverrideCount
    }
    pub fn getPaletteOverrides(&self) -> *const PaletteOverride {
        self.fPaletteOverrides.as_ptr().cast()
    }
}
impl Drop for SkFontData {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fPaletteOverrides);
            ManuallyDrop::drop(&mut self.fAxis);
            ManuallyDrop::drop(&mut self.fStream);
        }
    }
}

// Source: src/core/SkFontDescriptor.h:85-85
// Source: src/core/SkFontDescriptor.h:156-172
pub struct SkFontDescriptor {
    fFamilyName: ManuallyDrop<Vec<MaybeUninit<u8>>>,
    fFullName: ManuallyDrop<Vec<MaybeUninit<u8>>>,
    fPostscriptName: ManuallyDrop<Vec<MaybeUninit<u8>>>,
    fStyle: SkFontStyle,
    fStream: ManuallyDrop<Option<Pin<Box<dyn SkStreamAsset>>>>,
    fCollectionIndex: i32,
    fCoordinateCount: i32,
    fVariation: ManuallyDrop<Vec<MaybeUninit<Coordinate>>>,
    fPaletteIndex: i32,
    fPaletteEntryOverrideCount: i32,
    fPaletteEntryOverrides: ManuallyDrop<Vec<MaybeUninit<PaletteOverride>>>,
    fSyntheticBold: bool,
    fSyntheticOblique: bool,
    fFactoryId: u32,
}
impl SkFontDescriptor {
    // Source: src/core/SkFontDescriptor.h:95-104
    pub fn getStyle(&self) -> SkFontStyle {
        self.fStyle
    }
    pub fn setStyle(&mut self, style: SkFontStyle) {
        self.fStyle = style;
    }
    pub fn getFamilyName(&self) -> *const c_char {
        self.fFamilyName.as_ptr().cast()
    }
    pub fn getFullName(&self) -> *const c_char {
        self.fFullName.as_ptr().cast()
    }
    pub fn getPostscriptName(&self) -> *const c_char {
        self.fPostscriptName.as_ptr().cast()
    }
    pub fn setFamilyName(&mut self, name: impl AsRef<[u8]>) {
        set_name(&mut self.fFamilyName, name.as_ref());
    }
    pub fn setFullName(&mut self, name: impl AsRef<[u8]>) {
        set_name(&mut self.fFullName, name.as_ref());
    }
    pub fn setPostscriptName(&mut self, name: impl AsRef<[u8]>) {
        set_name(&mut self.fPostscriptName, name.as_ref());
    }
    // Source: src/core/SkFontDescriptor.h:106-122
    pub fn hasStream(&self) -> bool {
        self.fStream.is_some()
    }
    pub fn dupStream(&self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        SkStreamAsset::duplicate(
            self.fStream
                .as_ref()
                .expect("native dupStream requires stream")
                .as_ref()
                .get_ref(),
        )
    }
    pub fn getCollectionIndex(&self) -> i32 {
        self.fCollectionIndex
    }
    pub fn getPaletteIndex(&self) -> i32 {
        self.fPaletteIndex
    }
    pub fn getVariationCoordinateCount(&self) -> i32 {
        self.fCoordinateCount
    }
    pub fn getVariation(&self) -> *const Coordinate {
        self.fVariation.as_ptr().cast()
    }
    pub fn getPaletteEntryOverrideCount(&self) -> i32 {
        self.fPaletteEntryOverrideCount
    }
    pub fn getPaletteEntryOverrides(&self) -> *const PaletteOverride {
        self.fPaletteEntryOverrides.as_ptr().cast()
    }
    pub fn getSyntheticBold(&self) -> bool {
        self.fSyntheticBold
    }
    pub fn getSyntheticOblique(&self) -> bool {
        self.fSyntheticOblique
    }
    pub fn getFactoryId(&self) -> u32 {
        self.fFactoryId
    }
    // Source: src/core/SkFontDescriptor.h:124-140
    pub fn detachStream(&mut self) -> Option<Pin<Box<dyn SkStreamAsset>>> {
        self.fStream.take()
    }
    pub fn setStream(&mut self, stream: Option<Pin<Box<dyn SkStreamAsset>>>) {
        *self.fStream = stream;
    }
    pub fn setCollectionIndex(&mut self, index: i32) {
        self.fCollectionIndex = index;
    }
    pub fn setPaletteIndex(&mut self, index: i32) {
        self.fPaletteIndex = index;
    }
    pub fn setVariationCoordinates(&mut self, count: i32) -> *mut Coordinate {
        assert!(count >= 0);
        self.fCoordinateCount = count;
        *self.fVariation = uninit_array(count as usize);
        self.fVariation.as_mut_ptr().cast()
    }
    pub fn setPaletteEntryOverrides(&mut self, count: i32) -> *mut PaletteOverride {
        assert!(count >= 0);
        self.fPaletteEntryOverrideCount = count;
        *self.fPaletteEntryOverrides = uninit_array(count as usize);
        self.fPaletteEntryOverrides.as_mut_ptr().cast()
    }
    pub fn setSyntheticBold(&mut self, bold: bool) {
        self.fSyntheticBold = bold;
    }
    pub fn setSyntheticOblique(&mut self, oblique: bool) {
        self.fSyntheticOblique = oblique;
    }
    pub fn setFactoryId(&mut self, factory_id: u32) {
        self.fFactoryId = factory_id;
    }
    // Source: src/core/SkFontDescriptor.h:142-151
    /// Returned raw arrays borrow this descriptor; mutating/dropping it invalidates them.
    pub fn getFontArguments(&self) -> SkFontArguments {
        let mut args = SkFontArguments::new();
        args.setCollectionIndex(self.getCollectionIndex())
            .setVariationDesignPosition(VariationPosition {
                coordinates: self.getVariation(),
                coordinateCount: self.getVariationCoordinateCount(),
            })
            .setPalette(Palette {
                index: self.getPaletteIndex(),
                overrides: self.getPaletteEntryOverrides(),
                overrideCount: self.getPaletteEntryOverrideCount(),
            })
            .setSyntheticBold(Some(self.fSyntheticBold))
            .setSyntheticOblique(Some(self.fSyntheticOblique));
        args
    }
}
impl Drop for SkFontDescriptor {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fPaletteEntryOverrides);
            ManuallyDrop::drop(&mut self.fVariation);
            ManuallyDrop::drop(&mut self.fStream);
            ManuallyDrop::drop(&mut self.fPostscriptName);
            ManuallyDrop::drop(&mut self.fFullName);
            ManuallyDrop::drop(&mut self.fFamilyName);
        }
    }
}
// Source: src/core/SkFontDescriptor.cpp:21-44
const kInvalid: usize = 0x00;
const kFontFamilyName: usize = 0x01;
const kFullName: usize = 0x04;
const kPostscriptName: usize = 0x06;
const kWeight: usize = 0x10;
const kWidth: usize = 0x11;
const kSlant: usize = 0x12;
const kItalic: usize = 0x13;
const kSyntheticBold: usize = 0xF6;
const kSyntheticOblique: usize = 0xF7;
const kPaletteIndex: usize = 0xF8;
const kPaletteEntryOverrides: usize = 0xF9;
const kFontVariation: usize = 0xFA;
const kFactoryId: usize = 0xFC;
const kFontIndex: usize = 0xFD;
const kSentinel: usize = 0xFF;
// Source: src/core/SkFontDescriptor.cpp:46-46
impl SkFontDescriptor {
    pub fn new() -> Self {
        Self {
            fFamilyName: ManuallyDrop::new(vec![MaybeUninit::new(0)]),
            fFullName: ManuallyDrop::new(vec![MaybeUninit::new(0)]),
            fPostscriptName: ManuallyDrop::new(vec![MaybeUninit::new(0)]),
            fStyle: SkFontStyle::default(),
            fStream: ManuallyDrop::new(None),
            fCollectionIndex: 0,
            fCoordinateCount: 0,
            fVariation: ManuallyDrop::new(Vec::new()),
            fPaletteIndex: 0,
            fPaletteEntryOverrideCount: 0,
            fPaletteEntryOverrides: ManuallyDrop::new(Vec::new()),
            fSyntheticBold: false,
            fSyntheticOblique: false,
            fFactoryId: 0,
        }
    }
}
// Source: src/core/SkFontDescriptor.cpp:48-50
fn write_id(mut stream: Pin<&mut dyn SkWStream>, id: usize) -> bool {
    stream.as_mut().writePackedUInt(id)
}
// Source: src/core/SkFontDescriptor.cpp:51-56
unsafe fn write_string(
    mut stream: Pin<&mut dyn SkWStream>,
    string: &[MaybeUninit<u8>],
    id: usize,
) -> bool {
    let length = string.len() - 1;
    if length == 0 {
        return true;
    }
    write_id(stream.as_mut(), id)
        && stream.as_mut().writePackedUInt(length)
        && stream.as_mut().write(string.as_ptr().cast(), length)
}
// Source: src/core/SkFontDescriptor.cpp:58-61
fn write_uint(mut stream: Pin<&mut dyn SkWStream>, n: usize, id: usize) -> bool {
    write_id(stream.as_mut(), id) && stream.as_mut().writePackedUInt(n)
}
// Source: src/core/SkFontDescriptor.cpp:63-66
fn write_scalar(mut stream: Pin<&mut dyn SkWStream>, n: f32, id: usize) -> bool {
    write_id(stream.as_mut(), id) && stream.as_mut().writeScalar(n)
}
// Source: src/core/SkFontDescriptor.cpp:68-72
fn read_id(mut stream: Pin<&mut dyn SkStream>) -> usize {
    let mut id = 0;
    if !stream.as_mut().readPackedUInt(&mut id) {
        return kInvalid;
    }
    id
}
// Source: src/core/SkFontDescriptor.cpp:74-85
fn read_string(mut stream: Pin<&mut dyn SkStream>, string: &mut Vec<MaybeUninit<u8>>) -> bool {
    let mut length = 0;
    if !stream.as_mut().readPackedUInt(&mut length) {
        return false;
    }
    if length > 0 {
        if SkStreamPriv::RemainingLengthIsBelow(stream.as_ref().get_ref(), length) {
            return false;
        }
        string.resize_with(
            length
                .checked_add(1)
                .expect("native string allocation overflow"),
            MaybeUninit::uninit,
        );
        string[length].write(0);
        if unsafe { stream.as_mut().read(string.as_mut_ptr().cast(), length) } != length {
            return false;
        }
    }
    true
}
// Source: src/core/SkFontDescriptor.cpp:87-89
const usWidths: [f32; 9] = [1., 2., 3., 4., 5., 6., 7., 8., 9.];
// Source: src/core/SkFontDescriptor.cpp:90-94
const width_for_usWidth: [f32; 16] = [
    50., 50., 62.5, 75., 87.5, 100., 112.5, 125., 150., 200., 200., 200., 200., 200., 200., 200.,
];
// Source: src/core/SkFontDescriptor.cpp:96-243
impl SkFontDescriptor {
    /// A false return leaves the native partial state. Raw array/name consumers
    /// must not read uninitialized entries after failure or before filling reset arrays.
    pub fn Deserialize(
        mut stream: Pin<&mut dyn SkStream>,
        result: &mut Self,
        sanitizer: Option<super::SkTypeface::SkTypefaceStreamSanitizerProc>,
    ) -> bool {
        let mut style_bits = 0;
        if !stream.as_mut().readPackedUInt(&mut style_bits) {
            return false;
        }
        let mut weight = ((style_bits >> 16) & 0xFFFF) as f32;
        let mut width = width_for_usWidth[(style_bits >> 8) & 0xF];
        let mut slant = if (style_bits & 0xF) != Slant::kUpright_Slant.0 as usize {
            -20.
        } else {
            0.
        };
        let mut italic = if (style_bits & 0xF) == Slant::kItalic_Slant.0 as usize {
            1.
        } else {
            0.
        };
        loop {
            let id = read_id(stream.as_mut());
            if id == kSentinel {
                break;
            }
            match id {
                kFontFamilyName => {
                    if !read_string(stream.as_mut(), &mut result.fFamilyName) {
                        return false;
                    }
                }
                kFullName => {
                    if !read_string(stream.as_mut(), &mut result.fFullName) {
                        return false;
                    }
                }
                kPostscriptName => {
                    if !read_string(stream.as_mut(), &mut result.fPostscriptName) {
                        return false;
                    }
                }
                kWeight => {
                    if !stream.as_mut().readScalar(&mut weight) {
                        return false;
                    }
                }
                kWidth => {
                    if !stream.as_mut().readScalar(&mut width) {
                        return false;
                    }
                }
                kSlant => {
                    if !stream.as_mut().readScalar(&mut slant) {
                        return false;
                    }
                }
                kItalic => {
                    if !stream.as_mut().readScalar(&mut italic) {
                        return false;
                    }
                }
                kFontVariation => {
                    let mut count = 0;
                    if !stream.as_mut().readPackedUInt(&mut count) {
                        return false;
                    }
                    if count > i32::MAX as usize {
                        return false;
                    }
                    if SkStreamPriv::RemainingLengthIsBelow(stream.as_ref().get_ref(), count) {
                        return false;
                    }
                    let coordinates = result.setVariationCoordinates(count as i32);
                    for i in 0..count {
                        unsafe {
                            let coordinate = coordinates.add(i);
                            if stream
                                .as_mut()
                                .read(ptr::addr_of_mut!((*coordinate).axis).cast(), 4)
                                != 4
                            {
                                return false;
                            }
                            if stream
                                .as_mut()
                                .read(ptr::addr_of_mut!((*coordinate).value).cast(), 4)
                                != 4
                            {
                                return false;
                            }
                        }
                    }
                }
                kFontIndex => {
                    let mut index = 0;
                    if !stream.as_mut().readPackedUInt(&mut index) {
                        return false;
                    }
                    if index > i32::MAX as usize {
                        return false;
                    }
                    result.fCollectionIndex = index as i32;
                }
                kPaletteIndex => {
                    let mut index = 0;
                    if !stream.as_mut().readPackedUInt(&mut index) {
                        return false;
                    }
                    if index > i32::MAX as usize {
                        return false;
                    }
                    result.fPaletteIndex = index as i32;
                }
                kPaletteEntryOverrides => {
                    let mut count = 0;
                    if !stream.as_mut().readPackedUInt(&mut count) {
                        return false;
                    }
                    if count > i32::MAX as usize {
                        return false;
                    }
                    if SkStreamPriv::RemainingLengthIsBelow(stream.as_ref().get_ref(), count) {
                        return false;
                    }
                    let overrides = result.setPaletteEntryOverrides(count as i32);
                    for i in 0..count {
                        let mut index = 0;
                        if !stream.as_mut().readPackedUInt(&mut index) {
                            return false;
                        }
                        if index > u16::MAX as usize {
                            return false;
                        }
                        unsafe {
                            let entry = overrides.add(i);
                            ptr::addr_of_mut!((*entry).index).write(index as u16);
                            if stream
                                .as_mut()
                                .read(ptr::addr_of_mut!((*entry).color).cast(), 4)
                                != 4
                            {
                                return false;
                            }
                        }
                    }
                }
                kSyntheticBold => {
                    result.fSyntheticBold = true;
                }
                kSyntheticOblique => {
                    result.fSyntheticOblique = true;
                }
                kFactoryId => {
                    let mut id = 0;
                    if !stream.as_mut().readPackedUInt(&mut id) {
                        return false;
                    }
                    if id > u32::MAX as usize {
                        return false;
                    }
                    result.fFactoryId = id as u32;
                }
                _ => {
                    crate::SkDEBUGFAIL!(@source b"src/core/SkFontDescriptor.cpp\0".as_ptr(),212; b"Unknown id used by a font descriptor\0".as_ptr());
                    return false;
                }
            }
        }
        let mut slant_enum = Slant::kUpright_Slant;
        if slant != 0. {
            slant_enum = Slant::kOblique_Slant;
        }
        if 0. < italic {
            slant_enum = Slant::kItalic_Slant;
        }
        let width_enum = Self::SkFontStyleWidthForWidthAxisValue(width);
        result.fStyle = SkFontStyle::new(SkScalarRoundToInt(weight), width_enum, slant_enum);
        let mut length = 0;
        if !stream.as_mut().readPackedUInt(&mut length) {
            return false;
        }
        if length > 0 {
            if SkStreamPriv::RemainingLengthIsBelow(stream.as_ref().get_ref(), length) {
                return false;
            }
            let mut data = unsafe { SkData::MakeUninitialized(length) };
            if unsafe { stream.as_mut().read(data.writable_data(), length) } != length {
                crate::SkDEBUGFAIL!(@source b"src/core/SkFontDescriptor.cpp\0".as_ptr(),231; b"Could not read font data\0".as_ptr());
                return false;
            }
            if let Some(sanitize) = sanitizer {
                let Some(sanitized) = sanitize(data) else {
                    return false;
                };
                data = sanitized;
            }
            *result.fStream = Some(SkMemoryStream::Make(Some(data)));
        }
        true
    }
}
// Source: src/core/SkFontDescriptor.cpp:245-301
impl SkFontDescriptor {
    /// Safety: all descriptor arrays and name bytes must be initialized. A
    /// duplicate of the retained font stream must supply its declared length,
    /// as required by native SkWStream::writeStream. Output may not alias them.
    pub unsafe fn serialize(&self, mut stream: Pin<&mut dyn SkWStream>) -> bool {
        let style_bits = ((self.fStyle.weight() as u32) << 16)
            | ((self.fStyle.width() as u32) << 8)
            | self.fStyle.slant().0 as u32;
        if !stream.as_mut().writePackedUInt(style_bits as usize) {
            return false;
        }
        if !write_string(stream.as_mut(), &self.fFamilyName, kFontFamilyName) {
            return false;
        }
        if !write_string(stream.as_mut(), &self.fFullName, kFullName) {
            return false;
        }
        if !write_string(stream.as_mut(), &self.fPostscriptName, kPostscriptName) {
            return false;
        }
        if !write_scalar(stream.as_mut(), self.fStyle.weight() as f32, kWeight) {
            return false;
        }
        if !write_scalar(
            stream.as_mut(),
            width_for_usWidth[self.fStyle.width() as usize],
            kWidth,
        ) {
            return false;
        }
        let slant = if self.fStyle.slant() == Slant::kUpright_Slant {
            0.
        } else {
            -20.
        };
        if !write_scalar(stream.as_mut(), slant, kSlant) {
            return false;
        }
        let italic = if self.fStyle.slant() == Slant::kItalic_Slant {
            1.
        } else {
            0.
        };
        if !write_scalar(stream.as_mut(), italic, kItalic) {
            return false;
        }
        if self.fCollectionIndex > 0 {
            if !write_uint(stream.as_mut(), self.fCollectionIndex as usize, kFontIndex) {
                return false;
            }
        }
        if self.fPaletteIndex > 0 {
            if !write_uint(stream.as_mut(), self.fPaletteIndex as usize, kPaletteIndex) {
                return false;
            }
        }
        if self.fCoordinateCount > 0 {
            if !write_uint(
                stream.as_mut(),
                self.fCoordinateCount as usize,
                kFontVariation,
            ) {
                return false;
            }
            for i in 0..self.fCoordinateCount as usize {
                let coordinate = self.fVariation[i].assume_init_ref();
                if !stream.as_mut().write32(coordinate.axis) {
                    return false;
                }
                if !stream.as_mut().writeScalar(coordinate.value) {
                    return false;
                }
            }
        }
        if self.fPaletteEntryOverrideCount > 0 {
            if !write_uint(
                stream.as_mut(),
                self.fPaletteEntryOverrideCount as usize,
                kPaletteEntryOverrides,
            ) {
                return false;
            }
            for i in 0..self.fPaletteEntryOverrideCount as usize {
                let entry = self.fPaletteEntryOverrides[i].assume_init_ref();
                if !stream.as_mut().writePackedUInt(entry.index as usize) {
                    return false;
                }
                if !stream.as_mut().write32(entry.color) {
                    return false;
                }
            }
        }
        if self.fSyntheticBold {
            if !write_id(stream.as_mut(), kSyntheticBold) {
                return false;
            }
        }
        if self.fSyntheticOblique {
            if !write_id(stream.as_mut(), kSyntheticOblique) {
                return false;
            }
        }
        if !write_uint(stream.as_mut(), self.fFactoryId as usize, kFactoryId) {
            return false;
        }
        if !stream.as_mut().writePackedUInt(kSentinel) {
            return false;
        }
        if let Some(input) = self.fStream.as_ref() {
            let mut font_stream = SkStreamAsset::duplicate(input.as_ref().get_ref())
                .expect("native serialize requires duplicate");
            let length = SkStreamAsset::getLength(&*font_stream);
            if !stream.as_mut().writePackedUInt(length) {
                return false;
            }
            if !stream.as_mut().writeStream(font_stream.as_mut(), length) {
                return false;
            }
        } else {
            if !stream.as_mut().writePackedUInt(0) {
                return false;
            }
        }
        true
    }
}
// Source: src/core/SkFontDescriptor.cpp:303-306
impl SkFontDescriptor {
    pub fn SkFontStyleWidthForWidthAxisValue(width: f32) -> i32 {
        SkScalarRoundToInt(SkFloatInterpFunc(
            width,
            &width_for_usWidth[1..],
            &usWidths,
            9,
        ))
    }
    // Source: src/core/SkFontDescriptor.cpp:308-310
    pub fn SkFontWidthAxisValueForStyleWidth(width: i32) -> f32 {
        width_for_usWidth[(width & 0xF) as usize]
    }
}
