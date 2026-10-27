// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/vision_deficiency.h:11-23
// cpp: third_party/blink/renderer/core/css/vision_deficiency.cc:14-84

use foundation::AtomicString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisionDeficiency {
    kNoVisionDeficiency,
    kBlurredVision,
    kReducedContrast,
    kAchromatopsia,
    kDeuteranopia,
    kProtanopia,
    kTritanopia,
}

fn CreateFilterDataUrl(piece: &str) -> AtomicString {
    AtomicString::from_str(&format!(
        "data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\"><filter id=\"f\">{piece}</filter></svg>#f"
    ))
}

pub fn CreateVisionDeficiencyFilterUrl(vision_deficiency: VisionDeficiency) -> AtomicString {
    // The filter color matrices are based on the following research paper:
    // Gustavo M. Machado, Manuel M. Oliveira, and Leandro A. F. Fernandes,
    // "A Physiologically-based Model for Simulation of Color Vision Deficiency".
    // IEEE Transactions on Visualization and Computer Graphics. Volume 15 (2009),
    // Number 6, November/December 2009. pp. 1291-1298.
    // https://www.inf.ufrgs.br/~oliveira/pubs_files/CVD_Simulation/CVD_Simulation.html
    //
    // The filter grayscale matrix is based on the following research paper:
    // Rang Man Ho Nguyen and Michael S. Brown,
    // "Why You Should Forget Luminance Conversion and Do Something Better".
    // IEEE Conference on Computer Vision and Pattern Recognition (CVPR),
    // Honolulu, HI, 2017. pp. 6750-6758.
    // https://openaccess.thecv.com/content_cvpr_2017/papers/Nguyen_Why_You_Should_CVPR_2017_paper.pdf
    match vision_deficiency {
        VisionDeficiency::kBlurredVision => {
            CreateFilterDataUrl("<feGaussianBlur stdDeviation=\"2\"/>")
        }
        VisionDeficiency::kReducedContrast => CreateFilterDataUrl(
            "<feComponentTransfer>  <feFuncR type=\"gamma\" offset=\"0.5\"/>  <feFuncG type=\"gamma\" offset=\"0.5\"/>  <feFuncB type=\"gamma\" offset=\"0.5\"/></feComponentTransfer>",
        ),
        VisionDeficiency::kAchromatopsia => CreateFilterDataUrl(
            "<feColorMatrix values=\"0.213  0.715  0.072  0.000  0.000 0.213  0.715  0.072  0.000  0.000 0.213  0.715  0.072  0.000  0.000 0.000  0.000  0.000  1.000  0.000 \"/>",
        ),
        VisionDeficiency::kDeuteranopia => CreateFilterDataUrl(
            "<feColorMatrix values=\" 0.367  0.861 -0.228  0.000  0.000  0.280  0.673  0.047  0.000  0.000 -0.012  0.043  0.969  0.000  0.000  0.000  0.000  0.000  1.000  0.000 \"/>",
        ),
        VisionDeficiency::kProtanopia => CreateFilterDataUrl(
            "<feColorMatrix values=\" 0.152  1.053 -0.205  0.000  0.000  0.115  0.786  0.099  0.000  0.000 -0.004 -0.048  1.052  0.000  0.000  0.000  0.000  0.000  1.000  0.000 \"/>",
        ),
        VisionDeficiency::kTritanopia => CreateFilterDataUrl(
            "<feColorMatrix values=\" 1.256 -0.077 -0.179  0.000  0.000 -0.078  0.931  0.148  0.000  0.000  0.005  0.691  0.304  0.000  0.000  0.000  0.000  0.000  1.000  0.000 \"/>",
        ),
        VisionDeficiency::kNoVisionDeficiency => {
            panic!("CreateVisionDeficiencyFilterUrl called without a deficiency")
        }
    }
}
