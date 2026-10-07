// Copyright 2011 Google Inc.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Positive uniform-scale subset of SkScalerContextRec::computeMatrices.
//! Rust Transform stores the post-2x2 matrix; glyph device translation is not
//! part of the scaler record and remains owned by the caller. No general QR,
//! skew, reflection, synthetic italic, or nonuniform scale is implemented here.
use crate::raster::Transform;

#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug)]
pub struct SkScalerContextRec {
    pub fTextSize: f32,
    pub fPost2x2: Transform,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreMatrixScale {
    Vertical,
}

/// Restricted Rust return-value adapter for upstream scale and remaining matrix.
#[derive(Clone, Copy, Debug)]
pub struct ScalerMatrices {
    pub scale: [f32; 2],
    pub remaining: Transform,
}

impl SkScalerContextRec {
    /// Uniform positive scale plus translation can be represented by CTFont size.
    /// Synthetic italic introduces skew and deliberately remains unsupported.
    pub fn supports_positive_uniform_scale(post: Transform) -> bool {
        post.sx > 0.0
            && post.sx.is_finite()
            && post.sx == post.sy
            && post.kx == 0.0
            && post.ky == 0.0
            && post.tx.is_finite()
            && post.ty.is_finite()
    }

    /// Translates the non-skewed, equal-scale kVertical branch. Upstream builds
    /// A from text size and post2x2, extracts abs(A.sy), and resets sA to identity.
    /// Unsupported/singular matrices are rejected rather than pretending to
    /// implement the other upstream branches.
    #[allow(non_snake_case)]
    pub fn computeMatrices(&self, pre_scale: PreMatrixScale) -> Option<ScalerMatrices> {
        let PreMatrixScale::Vertical = pre_scale;
        if !Self::supports_positive_uniform_scale(self.fPost2x2) {
            return None;
        }
        let a_scale_y = self.fTextSize * self.fPost2x2.sy;
        if !a_scale_y.is_finite() || a_scale_y <= (1.0 / 4096.0) {
            return None;
        }
        Some(ScalerMatrices {
            scale: [a_scale_y, a_scale_y],
            remaining: Transform::identity(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_uniform_scale_absorbs_size_but_not_device_translation() {
        for size in [0.25f32, 12.0, 14.5, 96.0] {
            for scale in [0.25f32, 0.5, 1.0, 1.5, 2.0, 3.0] {
                let post = Transform::from_row(scale, 0.0, 0.0, scale, 17.25, -33.75);
                let result = SkScalerContextRec {
                    fTextSize: size,
                    fPost2x2: post,
                }
                .computeMatrices(PreMatrixScale::Vertical)
                .unwrap();
                assert_eq!(result.scale, [size * scale; 2]);
                assert_eq!(result.remaining, Transform::identity());
                for value in [0.0f32, 0.25, 1.0, 8.0] {
                    // Both paths map EM coordinates to the same device point;
                    // baseline translation was never included in the record.
                    assert_eq!(
                        value * result.scale[0] + post.tx,
                        value * (size * post.sx) + post.tx
                    );
                    assert_eq!(
                        value * result.scale[1] + post.ty,
                        value * (size * post.sy) + post.ty
                    );
                }
            }
        }
    }

    #[test]
    fn unsupported_or_nonfinite_matrices_and_singular_text_sizes_are_rejected() {
        for post in [
            Transform::from_scale(2.0, 3.0),
            Transform::from_scale(-2.0, -2.0),
            Transform::from_scale(0.0, 0.0),
            Transform::from_row(2.0, 0.0, -0.5, 2.0, 0.0, 0.0),
            Transform::from_row(0.0, 1.0, -1.0, 0.0, 0.0, 0.0),
            Transform::from_scale(f32::INFINITY, f32::INFINITY),
            Transform::from_translate(f32::NAN, 0.0),
        ] {
            assert!(SkScalerContextRec {
                fTextSize: 16.0,
                fPost2x2: post
            }
            .computeMatrices(PreMatrixScale::Vertical)
            .is_none());
        }
        for size in [0.0, -1.0, 1.0 / 8192.0, f32::NAN, f32::INFINITY, f32::MAX] {
            assert!(
                SkScalerContextRec {
                    fTextSize: size,
                    fPost2x2: Transform::from_scale(2.0, 2.0)
                }
                .computeMatrices(PreMatrixScale::Vertical)
                .is_none(),
                "size={size}"
            );
        }
    }
}
