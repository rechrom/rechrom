#![allow(non_snake_case)]

use layoutng_assembly::internal::layout_input::{Edges, Offset, Size, TransformMatrix};
use layoutng_assembly::internal::paint_input::{
    PaintTransform, PaintTransformOperationKind, PaintTransformOrigin, PaintTransformReferenceBox,
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

// Map a point through one paint-property transform. Hit testing uses the
// inverse direction while painting and DOM geometry use the forward direction.
// Keeping the homogeneous-coordinate policy here prevents the consumers from
// drifting on perspective and singular transforms.
pub fn MapPointWithTransform(
    point: Offset,
    transform: &TransformMatrix,
    inverse: bool,
) -> Option<Offset> {
    let transform = if inverse {
        InvertTransform(transform)?
    } else {
        *transform
    };
    let m = transform.values;
    let w = m[3] * point.x + m[7] * point.y + m[15];
    if !w.is_finite() || w <= 0.0 || w.abs() < 1e-12 {
        return None;
    }
    let mapped = Offset {
        x: (m[0] * point.x + m[4] * point.y + m[12]) / w,
        y: (m[1] * point.x + m[5] * point.y + m[13]) / w,
    };
    (mapped.x.is_finite() && mapped.y.is_finite()).then_some(mapped)
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

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransformReferenceRect {
    pub offset: Offset,
    pub size: Size,
}

// cpp: layoutng/internal/transform_utils.cc:17-34
// ComputeReferenceBoxInternal. The immutable fragment transport owns border
// and padding, so Paint reconstructs the same per-fragment reference rect
// without reaching back through the mutable Layout object graph.
pub fn ResolveTransformReferenceRect(
    reference_box: PaintTransformReferenceBox,
    fragment_size: Size,
    border: Edges,
    padding: Edges,
) -> TransformReferenceRect {
    if !fragment_size.width.is_finite()
        || !fragment_size.height.is_finite()
        || !border.top.is_finite()
        || !border.right.is_finite()
        || !border.bottom.is_finite()
        || !border.left.is_finite()
        || !padding.top.is_finite()
        || !padding.right.is_finite()
        || !padding.bottom.is_finite()
        || !padding.left.is_finite()
    {
        panic!("transform reference box geometry must be finite");
    }
    match reference_box {
        PaintTransformReferenceBox::kBorderBox => TransformReferenceRect {
            offset: Offset::default(),
            size: fragment_size,
        },
        PaintTransformReferenceBox::kContentBox => {
            let left = border.left + padding.left;
            let right = border.right + padding.right;
            let top = border.top + padding.top;
            let bottom = border.bottom + padding.bottom;
            TransformReferenceRect {
                offset: Offset { x: left, y: top },
                size: Size {
                    width: (fragment_size.width - left - right).max(0.0),
                    height: (fragment_size.height - top - bottom).max(0.0),
                },
            }
        }
    }
}

fn ResolveReferenceBoxOrigin(
    reference_rect: TransformReferenceRect,
    transform_origin: Option<&PaintTransformOrigin>,
) -> Offset {
    if let Some(origin) = transform_origin {
        Offset {
            x: reference_rect.offset.x
                + origin.pixels.x
                + reference_rect.size.width * origin.percentages.x / 100.0,
            y: reference_rect.offset.y
                + origin.pixels.y
                + reference_rect.size.height * origin.percentages.y / 100.0,
        }
    } else {
        Offset {
            x: reference_rect.offset.x + reference_rect.size.width / 2.0,
            y: reference_rect.offset.y + reference_rect.size.height / 2.0,
        }
    }
}

// cpp: paint/geometry_mapper.h:168-193
pub fn ResolveTransformAroundOrigin(
    transform: &PaintTransform,
    absolute_offset: Offset,
    fragment_size: Size,
    transform_origin: Option<&PaintTransformOrigin>,
) -> TransformMatrix {
    ResolveTransformAroundReferenceBox(
        transform,
        absolute_offset,
        TransformReferenceRect {
            offset: Offset::default(),
            size: fragment_size,
        },
        transform_origin,
    )
}

pub fn ResolveTransformAroundReferenceBox(
    transform: &PaintTransform,
    absolute_offset: Offset,
    reference_rect: TransformReferenceRect,
    transform_origin: Option<&PaintTransformOrigin>,
) -> TransformMatrix {
    let resolved_transform = ResolvePaintTransform(transform, reference_rect.size);
    let local_origin = ResolveReferenceBoxOrigin(reference_rect, transform_origin);
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
    ResolveLocalTransformAroundReferenceBox(
        transform,
        parent_space_offset,
        TransformReferenceRect {
            offset: Offset::default(),
            size: fragment_size,
        },
        transform_origin,
    )
}

pub fn ResolveLocalTransformAroundReferenceBox(
    transform: &PaintTransform,
    parent_space_offset: Offset,
    reference_rect: TransformReferenceRect,
    transform_origin: Option<&PaintTransformOrigin>,
) -> TransformMatrix {
    let resolved_transform = ResolvePaintTransform(transform, reference_rect.size);
    let local_origin = ResolveReferenceBoxOrigin(reference_rect, transform_origin);
    if !parent_space_offset.x.is_finite()
        || !parent_space_offset.y.is_finite()
        || !local_origin.x.is_finite()
        || !local_origin.y.is_finite()
    {
        panic!("transform origin must be finite");
    }
    // cpp: paint/paint_property_tree_builder.cc
    // FragmentPaintPropertyTreeBuilder::UpdateForPaintOffsetTranslation
    //
    // Blink puts the rounded portion in PaintOffsetTranslation and retains
    // the residual as the Transform node's local paint offset. This flat
    // replay matrix has no separate parent node, so use their exact sum here.
    // Rounding again would discard subpixel accumulation (for example moving
    // a transformed box at x=623.5 to x=624).
    MultiplyTransforms(
        &TranslationTransform(
            parent_space_offset.x + local_origin.x,
            parent_space_offset.y + local_origin.y,
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
