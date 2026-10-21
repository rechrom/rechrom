// Exact scalar expressions from upstream; double intermediates retained.

// Source: src/core/SkMatrixInvert.cpp:12-33
pub fn SkInvert2x2Matrix(inMatrix: &[f32; 4], outMatrix: Option<&mut [f32; 4]>) -> f32 {
    let a00: f64 = inMatrix[0] as f64;
    let a01: f64 = inMatrix[1] as f64;
    let a10: f64 = inMatrix[2] as f64;
    let a11: f64 = inMatrix[3] as f64;

    // Calculate the determinant
    let mut determinant: f64 = a00 * a11 - a01 * a10;
    if let Some(outMatrix) = outMatrix {
        let invdet: f64 = 1.0 / determinant;
        outMatrix[0] = (a11 * invdet) as f32;
        outMatrix[1] = (-a01 * invdet) as f32;
        outMatrix[2] = (-a10 * invdet) as f32;
        outMatrix[3] = (a00 * invdet) as f32;
        // If 1/det overflows to infinity (i.e. det is denormalized) or any of the inverted matrix
        // values is non-finite, return zero to indicate a non-invertible matrix.
        if !outMatrix.iter().all(|v| v.is_finite()) {
            determinant = 0.0;
        }
    }
    return determinant as f32;
}

// Source: src/core/SkMatrixInvert.cpp:35-70
pub fn SkInvert3x3Matrix(inMatrix: &[f32; 9], outMatrix: Option<&mut [f32; 9]>) -> f32 {
    let a00: f64 = inMatrix[0] as f64;
    let a01: f64 = inMatrix[1] as f64;
    let a02: f64 = inMatrix[2] as f64;
    let a10: f64 = inMatrix[3] as f64;
    let a11: f64 = inMatrix[4] as f64;
    let a12: f64 = inMatrix[5] as f64;
    let a20: f64 = inMatrix[6] as f64;
    let a21: f64 = inMatrix[7] as f64;
    let a22: f64 = inMatrix[8] as f64;

    let b01: f64 = a22 * a11 - a12 * a21;
    let b11: f64 = -a22 * a10 + a12 * a20;
    let b21: f64 = a21 * a10 - a11 * a20;

    // Calculate the determinant
    let mut determinant: f64 = a00 * b01 + a01 * b11 + a02 * b21;
    if let Some(outMatrix) = outMatrix {
        let invdet: f64 = 1.0 / determinant;
        outMatrix[0] = (b01 * invdet) as f32;
        outMatrix[1] = ((-a22 * a01 + a02 * a21) * invdet) as f32;
        outMatrix[2] = ((a12 * a01 - a02 * a11) * invdet) as f32;
        outMatrix[3] = (b11 * invdet) as f32;
        outMatrix[4] = ((a22 * a00 - a02 * a20) * invdet) as f32;
        outMatrix[5] = ((-a12 * a00 + a02 * a10) * invdet) as f32;
        outMatrix[6] = (b21 * invdet) as f32;
        outMatrix[7] = ((-a21 * a00 + a01 * a20) * invdet) as f32;
        outMatrix[8] = ((a11 * a00 - a01 * a10) * invdet) as f32;
        // If 1/det overflows to infinity (i.e. det is denormalized) or any of the inverted matrix
        // values is non-finite, return zero to indicate a non-invertible matrix.
        if !outMatrix.iter().all(|v| v.is_finite()) {
            determinant = 0.0;
        }
    }
    return determinant as f32;
}

// Source: src/core/SkMatrixInvert.cpp:72-144
pub fn SkInvert4x4Matrix(inMatrix: &[f32; 16], outMatrix: Option<&mut [f32; 16]>) -> f32 {
    let a00: f64 = inMatrix[0] as f64;
    let a01: f64 = inMatrix[1] as f64;
    let a02: f64 = inMatrix[2] as f64;
    let a03: f64 = inMatrix[3] as f64;
    let a10: f64 = inMatrix[4] as f64;
    let a11: f64 = inMatrix[5] as f64;
    let a12: f64 = inMatrix[6] as f64;
    let a13: f64 = inMatrix[7] as f64;
    let a20: f64 = inMatrix[8] as f64;
    let a21: f64 = inMatrix[9] as f64;
    let a22: f64 = inMatrix[10] as f64;
    let a23: f64 = inMatrix[11] as f64;
    let a30: f64 = inMatrix[12] as f64;
    let a31: f64 = inMatrix[13] as f64;
    let a32: f64 = inMatrix[14] as f64;
    let a33: f64 = inMatrix[15] as f64;

    let mut b00: f64 = a00 * a11 - a01 * a10;
    let mut b01: f64 = a00 * a12 - a02 * a10;
    let mut b02: f64 = a00 * a13 - a03 * a10;
    let mut b03: f64 = a01 * a12 - a02 * a11;
    let mut b04: f64 = a01 * a13 - a03 * a11;
    let mut b05: f64 = a02 * a13 - a03 * a12;
    let mut b06: f64 = a20 * a31 - a21 * a30;
    let mut b07: f64 = a20 * a32 - a22 * a30;
    let mut b08: f64 = a20 * a33 - a23 * a30;
    let mut b09: f64 = a21 * a32 - a22 * a31;
    let mut b10: f64 = a21 * a33 - a23 * a31;
    let mut b11: f64 = a22 * a33 - a23 * a32;

    // Calculate the determinant
    let mut determinant: f64 =
        b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    if let Some(outMatrix) = outMatrix {
        let invdet: f64 = 1.0 / determinant;
        b00 *= invdet;
        b01 *= invdet;
        b02 *= invdet;
        b03 *= invdet;
        b04 *= invdet;
        b05 *= invdet;
        b06 *= invdet;
        b07 *= invdet;
        b08 *= invdet;
        b09 *= invdet;
        b10 *= invdet;
        b11 *= invdet;

        outMatrix[0] = (a11 * b11 - a12 * b10 + a13 * b09) as f32;
        outMatrix[1] = (a02 * b10 - a01 * b11 - a03 * b09) as f32;
        outMatrix[2] = (a31 * b05 - a32 * b04 + a33 * b03) as f32;
        outMatrix[3] = (a22 * b04 - a21 * b05 - a23 * b03) as f32;
        outMatrix[4] = (a12 * b08 - a10 * b11 - a13 * b07) as f32;
        outMatrix[5] = (a00 * b11 - a02 * b08 + a03 * b07) as f32;
        outMatrix[6] = (a32 * b02 - a30 * b05 - a33 * b01) as f32;
        outMatrix[7] = (a20 * b05 - a22 * b02 + a23 * b01) as f32;
        outMatrix[8] = (a10 * b10 - a11 * b08 + a13 * b06) as f32;
        outMatrix[9] = (a01 * b08 - a00 * b10 - a03 * b06) as f32;
        outMatrix[10] = (a30 * b04 - a31 * b02 + a33 * b00) as f32;
        outMatrix[11] = (a21 * b02 - a20 * b04 - a23 * b00) as f32;
        outMatrix[12] = (a11 * b07 - a10 * b09 - a12 * b06) as f32;
        outMatrix[13] = (a00 * b09 - a01 * b07 + a02 * b06) as f32;
        outMatrix[14] = (a31 * b01 - a30 * b03 - a32 * b00) as f32;
        outMatrix[15] = (a20 * b03 - a21 * b01 + a22 * b00) as f32;

        // If 1/det overflows to infinity (i.e. det is denormalized) or any of the inverted matrix
        // values is non-finite, return zero to indicate a non-invertible matrix.
        if !outMatrix.iter().all(|v| v.is_finite()) {
            determinant = 0.0;
        }
    }
    return determinant as f32;
}
