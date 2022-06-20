use super::Line;
use crate::{
    rendering::buffered_geometry::{BufferedGeometry, ToBufferedGeometry},
    utils::default,
};
use std::f32::consts::PI;

pub struct LineGeometryOpts {
    pub split_angle_threshold: f32,
}

impl Default for LineGeometryOpts {
    fn default() -> Self {
        Self {
            split_angle_threshold: PI * 0.666,
        }
    }
}

impl ToBufferedGeometry for Line {
    fn to_buffered_geometry(&self) -> BufferedGeometry {
        self.to_buffered_geometry_with(default())
    }
}

impl Line {
    pub fn to_buffered_geometry_with(&self, opts: LineGeometryOpts) -> BufferedGeometry {
        let line_parts = self.split_at_angle(opts.split_angle_threshold);
        todo!()
    }
}
