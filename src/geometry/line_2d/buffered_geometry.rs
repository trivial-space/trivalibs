use super::Line;
use crate::{
    rendering::buffered_geometry::{
        vert_type, BufferedGeometry, BufferedVertexData, ToBufferedGeometry,
        VertexFormat::{Float32, Float32x2},
        VertexType,
    },
    utils::default,
};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;
use std::f32::consts::PI;

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct LineVertexData {
    position: Vec2,
    width: f32,
    length: f32,
    uv: Vec2,
    local_uv: Vec2,
}

impl BufferedVertexData for LineVertexData {
    fn vertex_layout() -> Vec<VertexType> {
        vec![
            vert_type("position", Float32x2),
            vert_type("width", Float32),
            vert_type("length", Float32),
            vert_type("uv", Float32x2),
            vert_type("localUv", Float32x2),
        ]
    }
}

pub struct LineGeometryOpts {
    pub min_length: f32,
    pub min_len_wid_ratio: f32,
    pub split_angle_threshold: f32,

    pub smouth_edge_depth: u8,
    pub smouth_edge_threshold: f32,

    pub smouth_width_depth: u8,
    pub smouth_width_threshold: f32,

    pub cleanup_vertex_angle_threshold: f32,
    pub cleanup_vertex_len_wid_ratio: f32,

    pub cap_width_length_ratio: f32,
}

impl Default for LineGeometryOpts {
    fn default() -> Self {
        Self {
            min_length: 1.0,
            min_len_wid_ratio: 0.25,
            split_angle_threshold: PI * 0.666,
            smouth_width_depth: 2,
            smouth_width_threshold: 1.0,
            smouth_edge_depth: 2,
            smouth_edge_threshold: 0.05,
            cleanup_vertex_angle_threshold: 0.01,
            cleanup_vertex_len_wid_ratio: 0.5,
            cap_width_length_ratio: 1.0,
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
