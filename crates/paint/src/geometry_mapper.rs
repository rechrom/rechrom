#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Offset, Size, TransformMatrix};
use layoutng_assembly::internal::paint_input::{
    PaintTransform, PaintTransformOperationKind, PaintTransformOrigin,
};

use crate::PaintRect;

// cpp: paint/geometry_mapper.h:15-29
pub fn MultiplyTransforms(left: &TransformMatrix, right: &TransformMatrix) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    for column in 0..4 {
        for row in 0..4 {
            let mut value = 0.0;
            for index in 0..4 {
                value += left.values[index * 4 + row] * right.values[column * 4 + index];
            }
            output.values[column * 4 + row] = value;
        }
    }
    output
}

// cpp: paint/geometry_mapper.h:31-36
pub fn TranslationTransform(x: f64, y: f64) -> TransformMatrix {
    let mut output = TransformMatrix::default();
    output.values[12] = x;
    output.values[13] = y;
    output
}

pub fn TranslationTransform3D(x: f64, y: f64, z: f64) -> TransformMatrix {
    let mut output = TranslationTransform(x, y);
    output.values[14] = z;
    output
}

// cpp: paint/geometry_mapper.h:38-78
pub fn InvertTransform(transform: &TransformMatrix) -> Option<TransformMatrix> {
    let mut augmented = [[0.0_f64; 8]; 4];
    for row in 0..4 {
        for column in 0..4 {
            augmented[row][column] = transform.values[column * 4 + row];
        }
        augmented[row][row + 4] = 1.0;
    }
    for column in 0..4 {
        let mut pivot = column;
        for row in column + 1..4 {
            if augmented[row][column].abs() > augmented[pivot][column].abs() {
                pivot = row;
            }
        }
        if !augmented[pivot][column].is_finite() || augmented[pivot][column].abs() < 1e-12 {
            return None;
        }
        if pivot != column {
            augmented.swap(pivot, column);
        }
        let divisor = augmented[column][column];
        for value in &mut augmented[column] {
            *value /= divisor;
        }
        let pivot_row = augmented[column];
        for row in 0..4 {
            if row == column {
                continue;
            }
            let factor = augmented[row][column];
            for index in 0..8 {
                augmented[row][index] -= factor * pivot_row[index];
            }
        }
    }
    let mut inverse = TransformMatrix::default();
    for row in 0..4 {
        for column in 0..4 {
            let value = augmented[row][column + 4];
            if !value.is_finite() {
                return None;
            }
            inverse.values[column * 4 + row] = value;
        }
    }
    Some(inverse)
}

// cpp: paint/geometry_mapper.h:80-127
pub fn MapRectWithTransforms(
    rect: PaintRect,
    transforms: &[TransformMatrix],
    inverse: bool,
) -> Option<PaintRect> {
    if transforms.is_empty() {
        return Some(rect);
    }
    let mut corners = [
        Offset {
            x: rect.x,
            y: rect.y,
        },
        Offset {
            x: rect.x + rect.width,
            y: rect.y,
        },
        Offset {
            x: rect.x,
            y: rect.y + rect.height,
        },
        Offset {
            x: rect.x + rect.width,
            y: rect.y + rect.height,
        },
    ];
    let mut apply = |transform: &TransformMatrix| {
        let m = &transform.values;
        for point in &mut corners {
            let x = point.x;
            let y = point.y;
            let w = m[3] * x + m[7] * y + m[15];
            if !w.is_finite() || w <= 0.0 || w.abs() < 1e-12 {
                return false;
            }
            *point = Offset {
                x: (m[0] * x + m[4] * y + m[12]) / w,
                y: (m[1] * x + m[5] * y + m[13]) / w,
            };
            if !point.x.is_finite() || !point.y.is_finite() {
                return false;
            }
        }
        true
    };
    if inverse {
        for transform in transforms {
            let inverted = InvertTransform(transform)?;
            if !apply(&inverted) {
                return None;
            }
        }
    } else {
        for transform in transforms.iter().rev() {
            if !apply(transform) {
                return None;
            }
        }
    }
    let mut left = f64::INFINITY;
    let mut top = f64::INFINITY;
    let mut right = f64::NEG_INFINITY;
    let mut bottom = f64::NEG_INFINITY;
    for point in &corners {
        left = left.min(point.x);
        top = top.min(point.y);
        right = right.max(point.x);
        bottom = bottom.max(point.y);
    }
    Some(PaintRect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

// cpp: paint/geometry_mapper.h:129-165
pub fn ResolvePaintTransform(transform: &PaintTransform, fragment_size: Size) -> TransformMatrix {
    if !fragment_size.width.is_finite() || !fragment_size.height.is_finite() {
        panic!("transform box size must be finite");
    }
    let mut output = TransformMatrix::default();
    for operation in &transform.operations {
        let resolved = match operation.kind {
            PaintTransformOperationKind::kMatrix => operation.matrix,
            PaintTransformOperationKind::kTranslate | PaintTransformOperationKind::kTranslate3D => {
                if !operation.pixels.x.is_finite()
                    || !operation.pixels.y.is_finite()
                    || !operation.percentages.x.is_finite()
                    || !operation.percentages.y.is_finite()
                    || !operation.z.is_finite()
                {
                    panic!("transform translation must be finite");
                }
                TranslationTransform3D(
                    operation.pixels.x + fragment_size.width * operation.percentages.x / 100.0,
                    operation.pixels.y + fragment_size.height * operation.percentages.y / 100.0,
                    operation.z,
                )
            }
        };
        for value in resolved.values {
            if !value.is_finite() {
                panic!("transform matrix must be finite");
            }
        }
        output = MultiplyTransforms(&output, &resolved);
    }
    output
}

// cpp: paint/geometry_mapper.h:168-193
pub fn ResolveTransformAroundOrigin(
    transform: &PaintTransform,
    absolute_offset: Offset,
    fragment_size: Size,
    transform_origin: Option<&PaintTransformOrigin>,
) -> TransformMatrix {
    let resolved_transform = ResolvePaintTransform(transform, fragment_size);
    let local_origin = if let Some(origin) = transform_origin {
        Offset {
            x: origin.pixels.x + fragment_size.width * origin.percentages.x / 100.0,
            y: origin.pixels.y + fragment_size.height * origin.percentages.y / 100.0,
        }
    } else {
        Offset {
            x: fragment_size.width / 2.0,
            y: fragment_size.height / 2.0,
        }
    };
    if !absolute_offset.x.is_finite()
        || !absolute_offset.y.is_finite()
        || !local_origin.x.is_finite()
        || !local_origin.y.is_finite()
    {
        panic!("transform origin must be finite");
    }
    let pivot = Offset {
        x: absolute_offset.x + local_origin.x,
        y: absolute_offset.y + local_origin.y,
    };
    MultiplyTransforms(
        &TranslationTransform(pivot.x, pivot.y),
        &MultiplyTransforms(
            &resolved_transform,
            &TranslationTransform(-pivot.x, -pivot.y),
        ),
    )
}

// cpp: paint/geometry_mapper.h:197-224
pub fn ResolveLocalTransformAroundOrigin(
    transform: &PaintTransform,
    parent_space_offset: Offset,
    fragment_size: Size,
    transform_origin: Option<&PaintTransformOrigin>,
) -> TransformMatrix {
    let resolved_transform = ResolvePaintTransform(transform, fragment_size);
    let local_origin = if let Some(origin) = transform_origin {
        Offset {
            x: origin.pixels.x + fragment_size.width * origin.percentages.x / 100.0,
            y: origin.pixels.y + fragment_size.height * origin.percentages.y / 100.0,
        }
    } else {
        Offset {
            x: fragment_size.width / 2.0,
            y: fragment_size.height / 2.0,
        }
    };
    if !parent_space_offset.x.is_finite()
        || !parent_space_offset.y.is_finite()
        || !local_origin.x.is_finite()
        || !local_origin.y.is_finite()
    {
        panic!("transform origin must be finite");
    }
    let snapped_offset = Offset {
        x: (parent_space_offset.x + 0.5).floor(),
        y: (parent_space_offset.y + 0.5).floor(),
    };
    MultiplyTransforms(
        &TranslationTransform(
            snapped_offset.x + local_origin.x,
            snapped_offset.y + local_origin.y,
        ),
        &MultiplyTransforms(
            &resolved_transform,
            &TranslationTransform(-local_origin.x, -local_origin.y),
        ),
    )
}

// cpp: paint/geometry_mapper.h:228-232
pub fn MapRectToRoot(rect: PaintRect, transforms: &[TransformMatrix]) -> Option<PaintRect> {
    MapRectWithTransforms(rect, transforms, false)
}

// cpp: paint/geometry_mapper.h:234-238
pub fn MapRectFromRoot(rect: PaintRect, transforms: &[TransformMatrix]) -> Option<PaintRect> {
    MapRectWithTransforms(rect, transforms, true)
}
