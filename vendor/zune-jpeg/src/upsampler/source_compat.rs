/*
 * jdsample.c
 *
 * This file was part of the Independent JPEG Group's software:
 * Copyright (C) 1991-1996, Thomas G. Lane.
 * libjpeg-turbo Modifications:
 * Copyright 2009 Pierre Ossman <ossman@cendio.se> for Cendio AB
 * Copyright (C) 2010, 2015-2016, 2022, 2024, D. R. Commander.
 * Copyright (C) 2014, MIPS Technologies, Inc., California.
 * Copyright (C) 2015, Google, Inc.
 * Copyright (C) 2019-2020, Arm Limited.
 * For conditions of distribution and use, see the accompanying README.ijg
 * file.
 *
 * This file contains upsampling routines.
 *
 * Upsampling input data is counted in "row groups".  A row group
 * is defined to be (v_samp_factor * DCT_scaled_size / min_DCT_scaled_size)
 * sample rows of each component.  Upsampling will normally produce
 * max_v_samp_factor pixel rows from each row group (but this could vary
 * if the upsampler is applying a scale factor of its own).
 *
 * An excellent reference for image resampling is
 *   Digital Image Warping, George Wolberg, 1990.
 *   Pub. by IEEE Computer Society Press, Los Alamitos, CA. ISBN 0-8186-8944-7.
 */
// Modified Rust implementation for the opt-in source_compat feature.
// libjpeg-turbo terms: source-compat-licenses/README.ijg.
/*
 * Copyright (c) 2023.
 *
 * This software is free software;
 *
 * You can redistribute it or modify it under terms of the MIT, Apache License or Zlib license
 */

pub fn upsample_horizontal(
    input: &[i16], _ref: &[i16], _in_near: &[i16], _scratch: &mut [i16], output: &mut [i16]
) {
    assert_eq!(
        input.len() * 2,
        output.len(),
        "Input length is not half the size of the output length"
    );
    assert!(
        output.len() > 4 && input.len() > 2,
        "Too Short of a vector, cannot upsample"
    );

    output[0] = input[0];
    output[1] = (input[0] * 3 + input[1] + 2) >> 2;

    // This code is written for speed and not readability
    //
    // The readable code is
    //
    //      for i in 1..input.len() - 1{
    //         let sample = 3 * input[i] + 2;
    //         out[i * 2] = (sample + input[i - 1]) >> 2;
    //         out[i * 2 + 1] = (sample + input[i + 1]) >> 2;
    //     }
    //
    // The output of a pixel is determined by it's surrounding neighbours but we attach more weight to it's nearest
    // neighbour (input[i]) than to the next nearest neighbour.

    for (output_window, input_window) in output[2..].chunks_exact_mut(2).zip(input.windows(3)) {
        let sample = 3 * input_window[1] + 2;

        output_window[0] = (sample + input_window[0] - 1) >> 2;
        output_window[1] = (sample + input_window[2]) >> 2;
    }
    // Get lengths
    let out_len = output.len() - 2;
    let input_len = input.len() - 2;

    // slice the output vector
    let f_out = &mut output[out_len..];
    let i_last = &input[input_len..];

    // write out manually..
    f_out[0] = (3 * i_last[1] + i_last[0] + 1) >> 2;
    f_out[1] = i_last[1];
}
pub fn upsample_vertical(
    input: &[i16], in_near: &[i16], in_far: &[i16], _scratch_space: &mut [i16], output: &mut [i16]
) {
    assert_eq!(input.len() * 2, output.len());
    assert_eq!(in_near.len(), input.len());
    assert_eq!(in_far.len(), input.len());

    let middle = output.len() / 2;

    let (out_top, out_bottom) = output.split_at_mut(middle);

    // for the first row, closest row is in_near
    for ((near, far), x) in input.iter().zip(in_near.iter()).zip(out_top) {
        *x = (((3 * near) + 1) + far) >> 2;
    }
    // for the second row, the closest row to input is in_far
    for ((near, far), x) in input.iter().zip(in_far.iter()).zip(out_bottom) {
        *x = (((3 * near) + 2) + far) >> 2;
    }
}

// libjpeg-turbo/src/jdsample.c::h2v2_fancy_upsample. Keep vertical
// sums unrounded until the joint two-dimensional 16-weight filter.
pub fn upsample_hv(input: &[i16], in_near: &[i16], in_far: &[i16],
    _scratch_space: &mut [i16], output: &mut [i16]) {
    assert_eq!(input.len() * 4, output.len());
    for (far, row) in [in_near, in_far].into_iter().zip(output.chunks_exact_mut(input.len() * 2)) {
        let sum = |x: usize| 3 * i32::from(input[x]) + i32::from(far[x]);
        for x in 0..input.len() {
            let c = sum(x);
            row[x * 2] = ((3 * c + sum(x.saturating_sub(1)) + 8) >> 4) as i16;
            row[x * 2 + 1] = ((3 * c + sum((x + 1).min(input.len() - 1)) + 7) >> 4) as i16;
        }
    }
}

pub fn upsample_generic(
    input: &[i16], _in_near: &[i16], _in_far: &[i16], _scratch_space: &mut [i16],
    output: &mut [i16]
) {
    // use nearest sample
    let difference = output.len() / input.len();
    if difference > 0 {
        // nearest neighbour
        for (input, chunk_output) in input.iter().zip(output.chunks_exact_mut(difference)) {
            chunk_output.iter_mut().for_each(|x| *x = *input);
        }
    }
}
