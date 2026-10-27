// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// Source: cc/paint/render_surface_filters.cc:24-157.
#![allow(non_snake_case)]

/// The CSS filter operations represented by Chromium's private
/// `Get*Matrix` helpers.  Keeping this value in Paint lets raster backends
/// consume the same source-defined coefficients without depending on another
/// backend's compatibility module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorMatrixKind {
    Grayscale,
    Sepia,
    Saturate,
    HueRotate,
    Invert,
    Opacity,
    Brightness,
    Contrast,
}

fn GetBrightnessMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix.fill(0.0);
    matrix[0] = amount;
    matrix[6] = amount;
    matrix[12] = amount;
    matrix[18] = 1.0;
}

fn GetContrastMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix.fill(0.0);
    matrix[0] = amount;
    matrix[6] = amount;
    matrix[12] = amount;
    matrix[4] = -0.5 * amount + 0.5;
    matrix[9] = -0.5 * amount + 0.5;
    matrix[14] = -0.5 * amount + 0.5;
    matrix[18] = 1.0;
}

fn GetSaturateMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix[0] = 0.213 + 0.787 * amount;
    matrix[1] = 0.715 - 0.715 * amount;
    matrix[2] = 1.0 - (matrix[0] + matrix[1]);
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = 0.213 - 0.213 * amount;
    matrix[6] = 0.715 + 0.285 * amount;
    matrix[7] = 1.0 - (matrix[5] + matrix[6]);
    matrix[8] = 0.0;
    matrix[9] = 0.0;
    matrix[10] = 0.213 - 0.213 * amount;
    matrix[11] = 0.715 - 0.715 * amount;
    matrix[12] = 1.0 - (matrix[10] + matrix[11]);
    matrix[13] = 0.0;
    matrix[14] = 0.0;
    matrix[15] = 0.0;
    matrix[16] = 0.0;
    matrix[17] = 0.0;
    matrix[18] = 1.0;
    matrix[19] = 0.0;
}

fn GetHueRotateMatrix(hue: f32, matrix: &mut [f32; 20]) {
    let radians = hue.to_radians();
    let cos_hue = radians.cos();
    let sin_hue = radians.sin();
    matrix[0] = 0.213 + cos_hue * 0.787 - sin_hue * 0.213;
    matrix[1] = 0.715 - cos_hue * 0.715 - sin_hue * 0.715;
    matrix[2] = 0.072 - cos_hue * 0.072 + sin_hue * 0.928;
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = 0.213 - cos_hue * 0.213 + sin_hue * 0.143;
    matrix[6] = 0.715 + cos_hue * 0.285 + sin_hue * 0.140;
    matrix[7] = 0.072 - cos_hue * 0.072 - sin_hue * 0.283;
    matrix[8] = 0.0;
    matrix[9] = 0.0;
    matrix[10] = 0.213 - cos_hue * 0.213 - sin_hue * 0.787;
    matrix[11] = 0.715 - cos_hue * 0.715 + sin_hue * 0.715;
    matrix[12] = 0.072 + cos_hue * 0.928 + sin_hue * 0.072;
    matrix[13] = 0.0;
    matrix[14] = 0.0;
    matrix[15] = 0.0;
    matrix[16] = 0.0;
    matrix[17] = 0.0;
    matrix[18] = 1.0;
    matrix[19] = 0.0;
}

fn GetInvertMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix.fill(0.0);
    matrix[0] = 1.0 - 2.0 * amount;
    matrix[6] = 1.0 - 2.0 * amount;
    matrix[12] = 1.0 - 2.0 * amount;
    matrix[4] = amount;
    matrix[9] = amount;
    matrix[14] = amount;
    matrix[18] = 1.0;
}

fn GetOpacityMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix.fill(0.0);
    matrix[0] = 1.0;
    matrix[6] = 1.0;
    matrix[12] = 1.0;
    matrix[18] = amount;
}

fn GetGrayscaleMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix[0] = 0.2126 + 0.7874 * amount;
    matrix[1] = 0.7152 - 0.7152 * amount;
    matrix[2] = 1.0 - (matrix[0] + matrix[1]);
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = 0.2126 - 0.2126 * amount;
    matrix[6] = 0.7152 + 0.2848 * amount;
    matrix[7] = 1.0 - (matrix[5] + matrix[6]);
    matrix[8] = 0.0;
    matrix[9] = 0.0;
    matrix[10] = 0.2126 - 0.2126 * amount;
    matrix[11] = 0.7152 - 0.7152 * amount;
    matrix[12] = 1.0 - (matrix[10] + matrix[11]);
    matrix[13] = 0.0;
    matrix[14] = 0.0;
    matrix[15] = 0.0;
    matrix[16] = 0.0;
    matrix[17] = 0.0;
    matrix[18] = 1.0;
    matrix[19] = 0.0;
}

fn GetSepiaMatrix(amount: f32, matrix: &mut [f32; 20]) {
    matrix[0] = 0.393 + 0.607 * amount;
    matrix[1] = 0.769 - 0.769 * amount;
    matrix[2] = 0.189 - 0.189 * amount;
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = 0.349 - 0.349 * amount;
    matrix[6] = 0.686 + 0.314 * amount;
    matrix[7] = 0.168 - 0.168 * amount;
    matrix[8] = 0.0;
    matrix[9] = 0.0;
    matrix[10] = 0.272 - 0.272 * amount;
    matrix[11] = 0.534 - 0.534 * amount;
    matrix[12] = 0.131 + 0.869 * amount;
    matrix[13] = 0.0;
    matrix[14] = 0.0;
    matrix[15] = 0.0;
    matrix[16] = 0.0;
    matrix[17] = 0.0;
    matrix[18] = 1.0;
    matrix[19] = 0.0;
}

/// Builds the exact matrix selected by
/// `RenderSurfaceFilters::BuildImageFilter` for a CSS filter amount.
pub fn CssColorMatrix(kind: ColorMatrixKind, amount: f32) -> [f32; 20] {
    let mut matrix = [0.0; 20];
    match kind {
        ColorMatrixKind::Grayscale => GetGrayscaleMatrix(1.0 - amount, &mut matrix),
        ColorMatrixKind::Sepia => GetSepiaMatrix(1.0 - amount, &mut matrix),
        ColorMatrixKind::Saturate => GetSaturateMatrix(amount, &mut matrix),
        ColorMatrixKind::HueRotate => GetHueRotateMatrix(amount, &mut matrix),
        ColorMatrixKind::Invert => GetInvertMatrix(amount, &mut matrix),
        ColorMatrixKind::Opacity => GetOpacityMatrix(amount, &mut matrix),
        ColorMatrixKind::Brightness => GetBrightnessMatrix(amount, &mut matrix),
        ColorMatrixKind::Contrast => GetContrastMatrix(amount, &mut matrix),
    }
    matrix
}
