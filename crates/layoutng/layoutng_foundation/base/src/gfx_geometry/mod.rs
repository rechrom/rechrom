pub mod point;
pub mod point_conversions;
mod point_conversions_impl;
pub mod point_f;
mod point_f_impl;
mod point_impl;
pub mod size;
pub mod size_conversions;
mod size_conversions_impl;
pub mod size_f;
mod size_f_impl;
mod size_impl;

pub use point::Point;
pub use point_f::PointF;
pub use point_f_impl::{ScalePoint, ScalePointUniform};
pub use point_impl::{
    ScaleToCeiledPoint, ScaleToCeiledPointUniform, ScaleToFlooredPoint, ScaleToFlooredPointUniform,
    ScaleToRoundedPoint, ScaleToRoundedPointUniform,
};
pub use size::Size;
pub use size_f::SizeF;
pub use size_f_impl::{ScaleSize, ScaleSizeUniform};
pub mod vector2d;
pub mod vector2d_f;
mod vector2d_f_impl;
mod vector2d_impl;
pub use size_impl::{
    ScaleToCeiledSize, ScaleToCeiledSizeUniform, ScaleToFlooredSize, ScaleToFlooredSizeUniform,
    ScaleToRoundedSize, ScaleToRoundedSizeUniform,
};
pub use vector2d::Vector2d;
pub use vector2d_f::Vector2dF;
