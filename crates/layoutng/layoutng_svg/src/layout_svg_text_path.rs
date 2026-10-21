#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{gfx, DowncastFrom, DynamicTo, Traceable, Visitor};
use layoutng_assembly::internal::layout_inline::LayoutInline;
use layoutng_assembly::internal::layout_input::SvgTextPathData;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/layout_svg_text_path.h:17-20
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointAndTangent {
    pub point: gfx::PointF,
    pub tangent_in_degrees: f32,
}

// cpp: layoutng_svg/layout_svg_text_path.h:24-40
pub struct PathPositionMapper {
    points: Vec<gfx::PointF>,
    cumulative_lengths: Vec<f32>,
    path_length: f32,
    path_start_offset: f32,
    reverse_direction: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionType {
    kOnPath,
    kBeforePath,
    kAfterPath,
}

impl PathPositionMapper {
    // cpp: layoutng_svg/layout_svg_text_path.cc:15-35
    pub fn new(path: &SvgTextPathData) -> Self {
        let mut mapper = Self {
            points: Vec::new(),
            cumulative_lengths: Vec::new(),
            path_length: 0.0,
            path_start_offset: path.start_offset as f32,
            reverse_direction: path.reverse_direction,
        };
        for point in &path.points {
            let native = gfx::PointF::new(point.x as f32, point.y as f32);
            if mapper.points.last() == Some(&native) {
                continue;
            }
            mapper.points.push(native);
        }
        if mapper.points.is_empty() {
            return mapper;
        }
        mapper.cumulative_lengths.reserve(mapper.points.len());
        mapper.cumulative_lengths.push(0.0);
        for index in 1..mapper.points.len() {
            let dx = mapper.points[index].x() - mapper.points[index - 1].x();
            let dy = mapper.points[index].y() - mapper.points[index - 1].y();
            mapper.path_length += dx.hypot(dy);
            mapper.cumulative_lengths.push(mapper.path_length);
        }
        mapper
    }

    // cpp: layoutng_svg/layout_svg_text_path.h:31-32
    pub fn length(&self) -> f32 {
        self.path_length
    }
    pub fn StartOffset(&self) -> f32 {
        self.path_start_offset
    }

    // cpp: layoutng_svg/layout_svg_text_path.cc:37-71
    pub fn PointAndNormalAtLength(
        &mut self,
        length: f32,
        out: &mut PointAndTangent,
    ) -> PositionType {
        if length < 0.0 {
            return PositionType::kBeforePath;
        }
        if length > self.path_length {
            return PositionType::kAfterPath;
        }
        assert!(self.points.len() >= 2);
        let position = if self.reverse_direction {
            self.path_length - length
        } else {
            length
        };
        let upper = self
            .cumulative_lengths
            .partition_point(|value| !(position < *value));
        let end_index = if upper == self.cumulative_lengths.len() {
            upper - 1
        } else {
            upper
        }
        .max(1);
        let start_index = end_index - 1;
        let segment_start = self.cumulative_lengths[start_index];
        let segment_length = self.cumulative_lengths[end_index] - segment_start;
        assert!(segment_length > 0.0);
        let ratio = (position - segment_start) / segment_length;
        let dx = self.points[end_index].x() - self.points[start_index].x();
        let dy = self.points[end_index].y() - self.points[start_index].y();
        out.point = gfx::PointF::new(
            self.points[start_index].x() + dx * ratio,
            self.points[start_index].y() + dy * ratio,
        );
        out.tangent_in_degrees = dy.atan2(dx) * 180.0 / std::f32::consts::PI;
        if self.reverse_direction {
            out.tangent_in_degrees += 180.0;
        }
        PositionType::kOnPath
    }
}

// cpp: layoutng_svg/layout_svg_text_path.h:42-53
#[repr(C)]
pub struct LayoutSVGTextPath {
    inline: LayoutInline,
}

impl LayoutSVGTextPath {
    // cpp: layoutng_svg/layout_svg_text_path.cc:73-76
    pub fn new(element: *mut Element) -> Self {
        let mut inline = LayoutInline::new(element);
        inline.SetRuntimeClass(LayoutObjectClass::SvgTextPath);
        inline.SetAlwaysCreateLineBoxes(true);
        Self { inline }
    }

    // cpp: layoutng_svg/layout_svg_text_path.cc:78-89
    pub fn LayoutPath(&self) -> Option<PathPositionMapper> {
        let element = DynamicTo::<Element>(self.inline.ModelObjectForInline().GetNode());
        assert!(!element.is_null());
        let data = unsafe { &*element }.InputElementData().as_ref()?;
        let path = data.svg_text_path.as_ref()?;
        let mapper = PathPositionMapper::new(path);
        (mapper.length() > 0.0).then_some(mapper)
    }

    // cpp: layoutng_svg/layout_svg_text_path.cc:91-96
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        assert!(!child.is_null());
        let child = unsafe { &*child };
        if child.IsText() {
            return child.IsSVGInlineText();
        }
        child.IsSVGInline() && !child.IsSVGTextPath()
    }

    // cpp: layoutng_svg/layout_svg_text_path.h:49-52
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGInline(&self) -> bool {
        true
    }
    pub fn IsSVGTextPath(&self) -> bool {
        true
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGTextPath"
    }
}

impl Deref for LayoutSVGTextPath {
    type Target = LayoutInline;
    fn deref(&self) -> &Self::Target {
        &self.inline
    }
}
impl DerefMut for LayoutSVGTextPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inline
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGTextPath, inline) == 0);
impl Traceable for LayoutSVGTextPath {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.inline.Trace(visitor);
    }
}
impl DowncastFrom<LayoutObject> for LayoutSVGTextPath {
    // cpp: layoutng_svg/layout_svg_text_path.h:55-60
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGTextPath()
    }
}

#[cfg(test)]
mod tests {
    use super::{PathPositionMapper, PointAndTangent, PositionType};
    use layoutng_assembly::internal::layout_input::{Offset, SvgTextPathData};

    #[test]
    fn maps_segment_boundary_and_reversed_tangent_after_deduplication() {
        let path = SvgTextPathData {
            points: vec![
                Offset { x: 0.0, y: 0.0 },
                Offset { x: 0.0, y: 0.0 },
                Offset { x: 10.0, y: 0.0 },
                Offset { x: 10.0, y: 10.0 },
            ],
            start_offset: 3.5,
            reverse_direction: false,
        };
        let mut mapper = PathPositionMapper::new(&path);
        assert_eq!(mapper.length(), 20.0);
        assert_eq!(mapper.StartOffset(), 3.5);
        let mut value = PointAndTangent::default();
        assert_eq!(
            mapper.PointAndNormalAtLength(-1.0, &mut value),
            PositionType::kBeforePath
        );
        assert_eq!(
            mapper.PointAndNormalAtLength(21.0, &mut value),
            PositionType::kAfterPath
        );
        assert_eq!(
            mapper.PointAndNormalAtLength(10.0, &mut value),
            PositionType::kOnPath
        );
        assert_eq!((value.point.x(), value.point.y()), (10.0, 0.0));
        assert_eq!(value.tangent_in_degrees, 90.0);

        let mut reverse = PathPositionMapper::new(&SvgTextPathData {
            reverse_direction: true,
            ..path
        });
        assert_eq!(
            reverse.PointAndNormalAtLength(5.0, &mut value),
            PositionType::kOnPath
        );
        assert_eq!((value.point.x(), value.point.y()), (10.0, 5.0));
        assert_eq!(value.tangent_in_degrees, 270.0);
    }
}
