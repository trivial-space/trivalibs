use super::{Line, LineVertex};
use crate::{
    data_structures::neighbour_list::traits::WithNeighboursTransform,
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
    pub swap_texture_orientation: bool,
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
            swap_texture_orientation: false,
        }
    }
}

impl ToBufferedGeometry for Line {
    fn to_buffered_geometry(&self) -> BufferedGeometry {
        self.to_buffered_geometry_with(default())
    }
}

fn get_normal(direction: &Vec2) -> Vec2 {
    Vec2::new(direction.y, -direction.x)
}

fn line_positions(vertex: Vec2, normal: Vec2, width: f32) -> [Vec2; 2] {
    let p1 = normal * width + vertex;
    let p2 = normal * -width + vertex;
    [p1, p2]
}

fn line_mitter_positions(pos: &Vec2, dir: &Vec2, width: f32, prev_dir: Option<&Vec2>) -> [Vec2; 2] {
    // for math see
    // https://mattdesl.svbtle.com/drawing-lines-is-hard
    // https://cesium.com/blog/2013/04/22/robust-polyline-rendering-with-webgl/ "Vertex Shader Details"
    // https://www.npmjs.com/package/polyline-normals
    //
    let next_normal = get_normal(dir);

    if prev_dir.is_none() || dir == prev_dir.unwrap() {
        return line_positions(*pos, next_normal, width);
    }

    let prev_normal = get_normal(prev_dir.unwrap());
    let normal = (next_normal + prev_normal).normalize();
    let mitter_length = width / normal.dot(prev_normal);
    let mitter_length = mitter_length.min(width * 5.0);
    line_positions(*pos, normal, mitter_length)
}

impl Line {
    pub fn to_buffered_geometry_with(&self, opts: LineGeometryOpts) -> BufferedGeometry {
        let mut top_line = Line::new(self.default_width);
        let mut bottom_line = Line::new(self.default_width);

        for (prev, v, next) in self.iter().with_neighbours() {
            if prev.is_none() {
                top_line.add_width(v.pos, v.width);
                bottom_line.add_width(v.pos, v.width);
            }

            let new_points = line_mitter_positions(&v.pos, &v.dir, v.width, prev.map(|x| &x.dir));

            top_line.add_width(new_points[0], v.width);
            bottom_line.add_width(new_points[1], v.width);

            if next.is_none() {
                top_line.add_width(v.pos, v.width);
                bottom_line.add_width(v.pos, v.width);
            }
        }
        todo!()
    }
}
