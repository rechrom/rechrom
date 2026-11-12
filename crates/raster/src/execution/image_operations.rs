//! ImageOperations::Resize's RESIZE_BOX/RESIZE_LANCZOS3 and scalar BGRAConvolve2D.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
//! skia/ext/image_operations.cc:36-60,126-289,328-432;
//! skia/ext/convolver.cc:30-289,315-360,409-559; convolver.h:53-79.
//! RGBA and BGRA have the same independent convolution coefficients. Alpha is
//! retained in its original representation until the final native packet edge.
use std::io;
#[derive(Clone, Copy)]
pub(super) enum ResizeMethod {
    Box,
    Lanczos3,
}
struct Filter {
    offset: usize,
    data_location: usize,
    length: usize,
}
struct ConvolutionFilter1D {
    filters: Vec<Filter>,
    values: Vec<i16>,
    max_filter: usize,
}
impl ConvolutionFilter1D {
    fn Values(&self, filter: &Filter) -> &[i16] {
        &self.values[filter.data_location..filter.data_location + filter.length]
    }
    fn AddFilter(&mut self, mut offset: usize, values: &[i16]) {
        let data_location = self.values.len();
        if let Some(first) = values.iter().position(|&v| v != 0) {
            let last = values.iter().rposition(|&v| v != 0).unwrap();
            offset += first;
            self.values.extend_from_slice(&values[first..=last]);
        }
        let length = self.values.len() - data_location;
        self.max_filter = self.max_filter.max(length);
        self.filters.push(Filter {
            offset,
            data_location,
            length,
        });
    }
    fn Compute(src_size: usize, dest_size: usize, method: ResizeMethod) -> Self {
        let scale = dest_size as f32 / src_size as f32;
        let clamped = scale.min(1.);
        let support = match method {
            ResizeMethod::Box => 0.5,
            ResizeMethod::Lanczos3 => 3.,
        } / clamped;
        let inverse = 1. / scale;
        let mut output = Self {
            filters: Vec::with_capacity(dest_size),
            values: Vec::new(),
            max_filter: 0,
        };
        let mut values = Vec::new();
        let mut fixed = Vec::new();
        for dest in 0..dest_size {
            values.clear();
            let src_pixel = (dest as f32 + 0.5) * inverse;
            let begin = ((src_pixel - support).floor() as i64).max(0) as usize;
            let end = ((src_pixel + support).ceil() as usize).min(src_size - 1);
            let mut sum = 0f32;
            for source in begin..=end {
                let x = ((source as f32 + 0.5) - src_pixel) * clamped;
                let value = match method {
                    ResizeMethod::Box => {
                        if x >= -0.5 && x < 0.5 {
                            1.
                        } else {
                            0.
                        }
                    }
                    ResizeMethod::Lanczos3 => {
                        if x <= -3. || x >= 3. {
                            0.
                        } else if x > -f32::EPSILON && x < f32::EPSILON {
                            1.
                        } else {
                            let xpi = x * std::f32::consts::PI;
                            (xpi.sin() / xpi) * (xpi / 3.).sin() / (xpi / 3.)
                        }
                    }
                };
                values.push(value);
                sum += value;
            }
            fixed.clear();
            let mut fixed_sum = 0i16;
            for &value in &values {
                let coefficient = (value / sum * 16384.) as i16;
                fixed_sum = fixed_sum.wrapping_add(coefficient);
                fixed.push(coefficient);
            }
            let center = fixed.len() / 2;
            fixed[center] = fixed[center].wrapping_add(16384i16.wrapping_sub(fixed_sum));
            output.AddFilter(begin, &fixed);
        }
        output
    }
}
fn Allocate(len: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(len).map_err(io::Error::other)?;
    bytes.resize(len, 0);
    Ok(bytes)
}
fn Horizontal(source: &[u8], filters: &ConvolutionFilter1D, alpha: bool, output: &mut [u8]) {
    for (x, filter) in filters.filters.iter().enumerate() {
        let mut accum = [0i32; 4];
        for (i, &value) in filters.Values(filter).iter().enumerate() {
            let pixel = (filter.offset + i) * 4;
            for channel in 0..if alpha { 4 } else { 3 } {
                accum[channel] += i32::from(value) * i32::from(source[pixel + channel]);
            }
        }
        for channel in 0..if alpha { 4 } else { 3 } {
            output[x * 4 + channel] = (accum[channel] >> 14).clamp(0, 255) as u8;
        }
    }
}
/// Source scalar circular-row-buffer branch. Work/storage are bounded by the
/// touched filter rows; there is no full horizontally resized image snapshot.
pub(super) fn Resize(
    source: &[u8],
    source_size: (u32, u32),
    stride: usize,
    has_alpha: bool,
    destination: (u32, u32),
    method: ResizeMethod,
) -> io::Result<Option<Vec<u8>>> {
    let (sw, sh) = (source_size.0 as usize, source_size.1 as usize);
    let (dw, dh) = (destination.0 as usize, destination.1 as usize);
    if sw == 0 || sh == 0 || dw == 0 || dh == 0 {
        return Ok(None);
    }
    if stride > i32::MAX as usize || sh.checked_mul(stride).is_none_or(|n| n > i32::MAX as usize) {
        return Ok(None);
    }
    if sw.checked_mul(4).is_none_or(|n| n > stride)
        || sh.checked_mul(stride).is_none_or(|n| n > source.len())
    {
        return Err(io::Error::other(
            "ImageOperations source pixmap is incomplete",
        ));
    }
    let horizontal = ConvolutionFilter1D::Compute(sw, dw, method);
    let vertical = ConvolutionFilter1D::Compute(sh, dh, method);
    let row_bytes = dw
        .checked_mul(4)
        .ok_or_else(|| io::Error::other("ImageOperations output stride overflow"))?;
    let mut output = Allocate(
        row_bytes
            .checked_mul(dh)
            .ok_or_else(|| io::Error::other("ImageOperations output allocation overflow"))?,
    )?;
    let rows = vertical.max_filter;
    if rows == 0 {
        return Ok(Some(output));
    }
    let mut buffer = Allocate(
        row_bytes
            .checked_mul(rows)
            .ok_or_else(|| io::Error::other("ImageOperations row buffer overflow"))?,
    )?;
    let mut next = vertical.filters[0].offset;
    for (y, filter) in vertical.filters.iter().enumerate() {
        while next < filter.offset + filter.length {
            let row = next % rows;
            Horizontal(
                &source[next * stride..next * stride + sw * 4],
                &horizontal,
                has_alpha,
                &mut buffer[row * row_bytes..(row + 1) * row_bytes],
            );
            next += 1;
        }
        let output_row = &mut output[y * row_bytes..(y + 1) * row_bytes];
        for x in 0..dw {
            let mut accum = [0i32; 4];
            for (i, &value) in vertical.Values(filter).iter().enumerate() {
                let offset = ((filter.offset + i) % rows) * row_bytes + x * 4;
                for channel in 0..if has_alpha { 4 } else { 3 } {
                    accum[channel] += i32::from(value) * i32::from(buffer[offset + channel]);
                }
            }
            let pixel = &mut output_row[x * 4..x * 4 + 4];
            for channel in 0..3 {
                pixel[channel] = (accum[channel] >> 14).clamp(0, 255) as u8;
            }
            // Source protects premultiplied RGB <= alpha after the final pass.
            pixel[3] = if has_alpha {
                ((accum[3] >> 14).clamp(0, 255) as u8).max(pixel[0].max(pixel[1]).max(pixel[2]))
            } else {
                255
            };
        }
    }
    Ok(Some(output))
}
