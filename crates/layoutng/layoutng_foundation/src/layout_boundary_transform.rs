//! Engine-neutral transform carried across Layout and Paint package boundaries.

/// Column-major 4×4 transform matching Chromium's TransformationMatrix payload.
// cpp: layoutng/internal/layout_input.h:679-685
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformMatrix {
    pub values: [f64; 16],
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self {
            values: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_the_identity_transform() {
        let matrix = TransformMatrix::default();
        assert_eq!(matrix.values[0], 1.0);
        assert_eq!(matrix.values[5], 1.0);
        assert_eq!(matrix.values[10], 1.0);
        assert_eq!(matrix.values[15], 1.0);
        assert_eq!(
            matrix.values.iter().filter(|value| **value != 0.0).count(),
            4
        );
    }
}
