// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Rechrom: adapted from the pinned codecs/dav1d.rs to rav1d 1.1.0 typed Rust API.
// Scheduling and picture/data ownership follow upstream; see RECHROM.md.
// rav1d functions retain their historical dav1d_* names but are Rust definitions.
#![allow(clippy::unnecessary_cast)]

use crate::codecs::Decoder;
use crate::codecs::DecoderConfig;
use crate::decoder::item::Item;
use crate::decoder::GridImageHelper;
use crate::image::Image;
use crate::image::YuvRange;
use crate::internal_utils::u32_from_usize;
use crate::utils::pixels::*;
use crate::*;

use rav1d::include::dav1d::{
    data::Dav1dData,
    dav1d::{Dav1dContext, Dav1dSettings},
    picture::Dav1dPicture,
};
use rav1d::src::lib::*;
use std::ptr::NonNull;
// Same limit as the pinned dav1d API and rav1d's settings validation.
const DAV1D_MAX_THREADS: i32 = 256;

use std::ffi::CStr;
use std::mem::MaybeUninit;

#[derive(Default)]
pub struct Rav1d {
    context: Option<Dav1dContext>,
    picture: Option<Dav1dPictureWrapper>,
    config: Option<DecoderConfig>,
}

// See https://code.videolan.org/videolan/dav1d/-/blob/9849ede1304da1443cfb4a86f197765081034205/include/dav1d/common.h#L55-59
const DAV1D_EAGAIN: i32 = if libc::EPERM > 0 {
    -libc::EAGAIN
} else {
    libc::EAGAIN
};

struct Dav1dPictureWrapper {
    picture: Dav1dPicture,
}

impl Default for Dav1dPictureWrapper {
    fn default() -> Self {
        Self {
            picture: Dav1dPicture::default(),
        }
    }
}

impl Dav1dPictureWrapper {
    fn mut_ptr(&mut self) -> *mut Dav1dPicture {
        (&mut self.picture) as *mut _
    }

    fn get(&self) -> &Dav1dPicture {
        &self.picture
    }

    fn use_layer(&self, spatial_id: u8) -> bool {
        // # Safety: frame_hdr is popualated by dav1d and is guaranteed to be valid.
        spatial_id == 0xFF
            || spatial_id == unsafe { self.get().frame_hdr.unwrap().as_ref().spatial_id as u8 }
    }
}

impl Drop for Dav1dPictureWrapper {
    fn drop(&mut self) {
        // # Safety: Calling the Rust rav1d API with valid, live context/storage.
        unsafe {
            dav1d_picture_unref(NonNull::new(self.mut_ptr()));
        }
    }
}

struct Dav1dDataWrapper {
    data: Dav1dData,
}

impl Default for Dav1dDataWrapper {
    fn default() -> Self {
        Self {
            data: Dav1dData::default(),
        }
    }
}

impl Dav1dDataWrapper {
    fn mut_ptr(&mut self) -> *mut Dav1dData {
        (&mut self.data) as *mut _
    }

    fn has_data(&self) -> bool {
        self.data.sz > 0 && self.data.data.is_some()
    }

    fn wrap(&mut self, payload: &[u8]) -> AvifResult<()> {
        // Rechrom boundary hardening: no owned empty buffer can enter the loop.
        if payload.is_empty() {
            return AvifError::unknown_error("empty AV1 payload");
        }
        let data = unsafe { dav1d_data_create(Some(NonNull::from(&mut self.data)), payload.len()) };
        if data.is_null() {
            return Err(AvifError::UnknownError(
                "dav1d_data_create returned nullptr".into(),
            ));
        }
        // # Safety: data was allocated in the successful call to dav1d_data_create and is
        // guaranteed to be large enough to hold payload.len() bytes.
        unsafe {
            std::ptr::copy_nonoverlapping(payload.as_ptr(), data, payload.len());
        }
        Ok(())
    }
}

impl Drop for Dav1dDataWrapper {
    fn drop(&mut self) {
        // Rechrom: unref even consumed/empty data. The Rust API accepts a
        // Default buffer, and ownership is not implied solely by sz > 0.
        // # Safety: data is initialized live storage, never unref'd elsewhere.
        unsafe {
            dav1d_data_unref(NonNull::new(self.mut_ptr()));
        }
    }
}

impl Rav1d {
    fn initialize_impl(&mut self, low_latency: bool) -> AvifResult<()> {
        if self.context.is_some() {
            return Ok(());
        }
        let config = self.config.unwrap_ref();
        let mut settings_uninit: MaybeUninit<Dav1dSettings> = MaybeUninit::uninit();
        // # Safety: Calling the Rust rav1d API with valid, live context/storage.
        unsafe { dav1d_default_settings(NonNull::new(settings_uninit.as_mut_ptr()).unwrap()) };
        // # Safety: settings_uninit was initialized by the Rust API above.
        let mut settings = unsafe { settings_uninit.assume_init() };
        if low_latency {
            settings.max_frame_delay = 1;
        }
        settings.n_threads = i32::try_from(config.max_threads)
            .unwrap_or(1)
            .clamp(1, DAV1D_MAX_THREADS as _);
        settings.operating_point = config.operating_point as i32;
        settings.all_layers = if config.all_layers { 1 } else { 0 };
        let frame_size_limit = match config.image_size_limit {
            Some(value) => value.get(),
            None => 0,
        };
        // Set a maximum frame size limit to avoid OOM'ing fuzzers. In 32-bit builds, if
        // frame_size_limit > 8192 * 8192, dav1d reduces frame_size_limit to 8192 * 8192 and logs
        // a message, so we set frame_size_limit to at most 8192 * 8192 to avoid the dav1d_log
        // message.
        settings.frame_size_limit = if cfg!(target_pointer_width = "32") {
            frame_size_limit.min(8192 * 8192)
        } else {
            frame_size_limit
        };

        let mut dec = None;
        // # Safety: Calling the Rust rav1d API with valid, live context/storage.
        let ret = unsafe {
            dav1d_open(
                Some(NonNull::from(&mut dec)),
                Some(NonNull::from(&mut settings)),
            )
            .0
        };
        if ret != 0 {
            return AvifError::unknown_error(format!("dav1d_open returned {ret}"));
        }
        self.context = dec;
        Ok(())
    }

    fn drop_impl(&mut self) {
        self.picture = None;
        if self.context.is_some() {
            // # Safety: Calling the Rust rav1d API with valid, live context/storage.
            unsafe { dav1d_close(Some(NonNull::from(&mut self.context))) };
        }
        self.context = None;
    }

    fn picture_to_image(
        &self,
        dav1d_picture: &Dav1dPicture,
        image: &mut Image,
        category: Category,
        overwrite_alpha_plane_yuv_range: bool,
    ) -> AvifResult<()> {
        // Rechrom boundary validation before Crabby creates raw plane views.
        // rav1d's default allocator supplies positive strides, but do not turn
        // an invalid signed field into a large unsigned slice length.
        if dav1d_picture.p.w <= 0
            || dav1d_picture.p.h <= 0
            || !matches!(dav1d_picture.p.bpc, 8 | 10 | 12)
            || dav1d_picture.seq_hdr.is_none()
            || dav1d_picture.frame_hdr.is_none()
            || !(0..=3).contains(&dav1d_picture.p.layout)
        {
            return AvifError::unknown_error("invalid rav1d picture metadata");
        }
        let plane_count = if category == Category::Alpha || dav1d_picture.p.layout == 0 {
            1
        } else {
            3
        };
        let bytes_per_sample = if dav1d_picture.p.bpc == 8 { 1 } else { 2 };
        for plane in 0..plane_count {
            let stride = dav1d_picture.stride[usize::from(plane != 0)];
            let width = if plane != 0 && dav1d_picture.p.layout != 3 {
                (dav1d_picture.p.w as u64 + 1) / 2
            } else {
                dav1d_picture.p.w as u64
            };
            if dav1d_picture.data[plane].is_none()
                || stride <= 0
                || u32::try_from(stride).is_err()
                || (stride as u64) < width * bytes_per_sample
                || stride as u64 % bytes_per_sample != 0
            {
                return AvifError::unknown_error("invalid rav1d plane pointer/stride");
            }
        }
        match category {
            Category::Alpha => {
                if image.width > 0
                    && image.height > 0
                    && (image.width != (dav1d_picture.p.w as u32)
                        || image.height != (dav1d_picture.p.h as u32)
                        || image.depth != (dav1d_picture.p.bpc as u8))
                {
                    // Alpha plane does not match the previous alpha plane.
                    return AvifError::unknown_error("");
                }
                image.width = dav1d_picture.p.w as u32;
                image.height = dav1d_picture.p.h as u32;
                image.depth = dav1d_picture.p.bpc as u8;
                image.row_bytes[3] = dav1d_picture.stride[0] as u32;
                image.planes[3] = Some(Pixels::from_raw_pointer(
                    dav1d_picture.data[0].unwrap().as_ptr().cast::<u8>(),
                    image.depth as u32,
                    image.height,
                    image.row_bytes[3],
                )?);
                if overwrite_alpha_plane_yuv_range {
                    // # Safety: seq_hdr is populated by dav1d and is guaranteed to be valid.
                    let seq_hdr = unsafe { dav1d_picture.seq_hdr.unwrap().as_ref() };
                    image.yuv_range = if seq_hdr.color_range == 0 {
                        YuvRange::Limited
                    } else {
                        YuvRange::Full
                    };
                }
            }
            _ => {
                image.width = dav1d_picture.p.w as u32;
                image.height = dav1d_picture.p.h as u32;
                image.depth = dav1d_picture.p.bpc as u8;

                image.yuv_format = match dav1d_picture.p.layout {
                    0 => PixelFormat::Yuv400,
                    1 => PixelFormat::Yuv420,
                    2 => PixelFormat::Yuv422,
                    3 => PixelFormat::Yuv444,
                    _ => return AvifError::unknown_error(""), // not reached.
                };
                // # Safety: seq_hdr is popualated by dav1d and is guaranteed to be valid.
                let seq_hdr = unsafe { dav1d_picture.seq_hdr.unwrap().as_ref() };
                image.yuv_range = if seq_hdr.color_range == 0 {
                    YuvRange::Limited
                } else {
                    YuvRange::Full
                };
                image.chroma_sample_position = (seq_hdr.chr as u32).into();

                image.color_primaries = (seq_hdr.pri as u16).into();
                image.transfer_characteristics = (seq_hdr.trc as u16).into();
                image.matrix_coefficients = (seq_hdr.mtrx as u16).into();

                for plane in 0usize..image.yuv_format.plane_count() {
                    let stride_index = if plane == 0 { 0 } else { 1 };
                    image.row_bytes[plane] = dav1d_picture.stride[stride_index] as u32;
                    image.planes[plane] = Some(Pixels::from_raw_pointer(
                        dav1d_picture.data[plane].unwrap().as_ptr().cast::<u8>(),
                        image.depth as u32,
                        u32_from_usize(image.height(plane.into()))?,
                        image.row_bytes[plane],
                    )?);
                }
                if image.yuv_format == PixelFormat::Yuv400 {
                    // Clear left over chroma planes from previous frames.
                    image.clear_chroma_planes();
                }
            }
        }
        Ok(())
    }

    fn get_next_image_grid_impl(
        &mut self,
        payloads: &[Vec<u8>],
        spatial_id: u8,
        grid_image_helper: &mut GridImageHelper,
    ) -> AvifResult<()> {
        if self.context.is_none() {
            self.initialize_impl(false)?;
        }
        let mut res;
        let context = self.context.unwrap();
        let mut payloads_iter = payloads.iter().peekable();
        let mut data = Dav1dDataWrapper::default();
        let max_retries = 500;
        let mut retries = 0;
        while !grid_image_helper.is_grid_complete()? {
            if !data.has_data() && payloads_iter.peek().is_some() {
                data.wrap(payloads_iter.next().unwrap())?;
            }
            if data.has_data() {
                // # Safety: Calling the Rust rav1d API with valid, live context/storage.
                res = unsafe { dav1d_send_data(Some(context), NonNull::new(data.mut_ptr())).0 };
                if res != 0 && res != DAV1D_EAGAIN {
                    return AvifError::unknown_error(format!("dav1d_send_data returned {res}"));
                }
            }
            let mut picture = Dav1dPictureWrapper::default();
            // # Safety: Calling the Rust rav1d API with valid, live context/storage.
            res = unsafe { dav1d_get_picture(Some(context), NonNull::new(picture.mut_ptr())).0 };
            if res != 0 && res != DAV1D_EAGAIN {
                return AvifError::unknown_error(format!("dav1d_get_picture returned {res}"));
            } else if res == 0 && picture.use_layer(spatial_id) {
                let mut cell_image = Image::default();
                self.picture_to_image(
                    picture.get(),
                    &mut cell_image,
                    grid_image_helper.category,
                    true,
                )?;
                grid_image_helper.copy_from_cell_image(&mut cell_image)?;
                retries = 0;
            } else {
                retries += 1;
                if retries > max_retries {
                    return AvifError::unknown_error(format!(
                        "dav1d_get_picture never returned a frame after {max_retries} calls"
                    ));
                }
            }
        }
        self.flush()?;
        Ok(())
    }

    fn flush(&mut self) -> AvifResult<()> {
        loop {
            let mut picture = Dav1dPictureWrapper::default();
            // # Safety: Calling the Rust rav1d API with valid, live context/storage.
            let res = unsafe { dav1d_get_picture(self.context, NonNull::new(picture.mut_ptr())).0 };
            if res < 0 && res != DAV1D_EAGAIN {
                return AvifError::unknown_error(format!("error draining buffered frames {res}"));
            }
            if res != 0 {
                break;
            }
        }
        Ok(())
    }

    pub(crate) fn version() -> String {
        let version = match unsafe { CStr::from_ptr(dav1d_version()) }.to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => String::new(),
        };
        format!("rav1d 1.1.0 (embedded version {version}, Rust without assembly)")
    }
}

impl Decoder for Rav1d {
    fn codec(&self) -> CodecChoice {
        CodecChoice::Rav1d
    }

    fn initialize(&mut self, config: &DecoderConfig) -> AvifResult<()> {
        self.config = Some(config.clone());
        Ok(())
    }

    fn get_next_image(
        &mut self,
        av1_payload: &[u8],
        spatial_id: u8,
        image: &mut Image,
        category: Category,
        _item: Option<&Item>,
        _signal_eos: bool,
    ) -> AvifResult<()> {
        if self.context.is_none() {
            self.initialize_impl(true)?;
        }
        let mut data = Dav1dDataWrapper::default();
        data.wrap(av1_payload)?;
        let next_picture: Option<Dav1dPictureWrapper>;
        loop {
            if data.has_data() {
                // # Safety: Calling the Rust rav1d API with valid, live context/storage.
                let res = unsafe { dav1d_send_data(self.context, NonNull::new(data.mut_ptr())).0 };
                if res < 0 && res != DAV1D_EAGAIN {
                    return AvifError::unknown_error(format!("dav1d_send_data returned {res}"));
                }
            }

            let mut picture = Dav1dPictureWrapper::default();
            // # Safety: Calling the Rust rav1d API with valid, live context/storage.
            let res = unsafe { dav1d_get_picture(self.context, NonNull::new(picture.mut_ptr())).0 };
            if res == DAV1D_EAGAIN {
                if data.has_data() {
                    continue;
                }
                return AvifError::unknown_error("");
            } else if res < 0 {
                return AvifError::unknown_error(format!("dav1d_send_picture returned {res}"));
            } else if picture.use_layer(spatial_id) {
                // Got a picture.
                next_picture = Some(picture);
                break;
            }
        }
        self.flush()?;
        if next_picture.is_some() {
            self.picture = next_picture;
        } else if category == Category::Alpha && self.picture.is_some() {
            // Special case for alpha, re-use last frame.
        } else {
            return AvifError::unknown_error("");
        }
        self.picture_to_image(self.picture.unwrap_ref().get(), image, category, true)?;
        Ok(())
    }

    fn get_next_image_grid(
        &mut self,
        payloads: &[Vec<u8>],
        spatial_id: u8,
        grid_image_helper: &mut GridImageHelper,
    ) -> AvifResult<()> {
        let res = self.get_next_image_grid_impl(payloads, spatial_id, grid_image_helper);
        if res.is_err() {
            self.drop_impl();
        }
        res
    }

    fn get_last_image(
        &mut self,
        payloads: &[Vec<u8>],
        spatial_id: u8,
        image: &mut Image,
        category: Category,
    ) -> AvifResult<()> {
        if self.context.is_none() {
            self.initialize_impl(false)?;
        }
        let mut res;
        let context = self.context.unwrap();
        let mut payloads_iter = payloads.iter().peekable();
        let mut data = Dav1dDataWrapper::default();
        let max_retries = 500;
        let mut retries = 0;
        let mut got_picture_count = 0;
        let next_picture: Option<Dav1dPictureWrapper>;
        loop {
            if !data.has_data() && payloads_iter.peek().is_some() {
                data.wrap(payloads_iter.next().unwrap())?;
            }
            if data.has_data() {
                // # Safety: Calling the Rust rav1d API with valid, live context/storage.
                res = unsafe { dav1d_send_data(Some(context), NonNull::new(data.mut_ptr())).0 };
                if res != 0 && res != DAV1D_EAGAIN {
                    return AvifError::unknown_error(format!("dav1d_send_data returned {res}"));
                }
            }
            let mut picture = Dav1dPictureWrapper::default();
            // # Safety: Calling the Rust rav1d API with valid, live context/storage.
            res = unsafe { dav1d_get_picture(Some(context), NonNull::new(picture.mut_ptr())).0 };
            if res != 0 && res != DAV1D_EAGAIN {
                return AvifError::unknown_error(format!("dav1d_get_picture returned {res}"));
            } else if res == 0 && picture.use_layer(spatial_id) {
                got_picture_count += 1;
                retries = 0;
                if got_picture_count == payloads.len() {
                    next_picture = Some(picture);
                    break;
                }
            } else {
                retries += 1;
                if retries > max_retries {
                    return AvifError::unknown_error(format!(
                        "dav1d_get_picture never returned a frame after {max_retries} calls"
                    ));
                }
            }
        }
        self.flush()?;
        if next_picture.is_some() {
            self.picture = next_picture;
        } else if category == Category::Alpha && self.picture.is_some() {
            // Special case for alpha, re-use last frame.
        } else {
            return AvifError::unknown_error("");
        }
        // In this case, the source image from the caller is used for both color and alpha planes,
        // so do not overwrite the yuv_range field when processing the alpha plane.
        self.picture_to_image(self.picture.unwrap_ref().get(), image, category, false)?;
        Ok(())
    }
}

impl Drop for Rav1d {
    fn drop(&mut self) {
        self.drop_impl();
    }
}
