use super::{Line, LineVertex};
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

// export function lineMitterPositions(node: LineNode, thickness?: number) {
// 	if (!node.prev && !node.next) {
// 		throw 'incomplete Line'
// 	}
// 	const point = node.val
// 	thickness = point.width || thickness || 1
// 	if (!node.prev) {
// 		const tangent = getTangent(point.direction)
// 		return linePositions(point.vertex, tangent, thickness)
// 	}
// 	if (!node.next) {
// 		const tangent = getTangent(node.prev.val.direction)
// 		return linePositions(point.vertex, tangent, thickness)
// 	}

// 	const nextTangent = getTangent(node.val.direction)
// 	const prevTangent = getTangent(node.prev.val.direction)
// 	const tangent = normalize(add(nextTangent, prevTangent))
// 	let mitterLenght = thickness / dot(tangent, prevTangent)
// 	mitterLenght = Math.min(mitterLenght, thickness * 5)
// 	return linePositions(node.val.vertex, tangent as Vec2D, mitterLenght)
// }
fn line_mitter_positions(node: LineVertex, thickness: f32) -> [Vec2; 2] {
    // for math see
    // https://mattdesl.svbtle.com/drawing-lines-is-hard
    // https://cesium.com/blog/2013/04/22/robust-polyline-rendering-with-webgl/ "Vertex Shader Details"
    // https://www.npmjs.com/package/polyline-normals
    //
    todo!();
}

impl Line {
    pub fn to_buffered_geometry_with(&self, opts: LineGeometryOpts) -> BufferedGeometry {
        let mut top_line = Line::new(self.default_width);
        let mut bottom_line = Line::new(self.default_width);
        let mut opt_prev = None;

        for (i, v) in self.iter().enumerate() {
            let v = *v;

            if i == 0 {
                top_line.add_width(v.pos, v.width);
                bottom_line.add_width(v.pos, v.width);
            }

            let prev = opt_prev.unwrap();

            let new_points = line_mitter_positions(v, v.width);

            top_line.add_width(new_points[0], v.width);
            bottom_line.add_width(new_points[1], v.width);

            if i == self.vert_count() - 1 {
                top_line.add_width(v.pos, v.width);
                bottom_line.add_width(v.pos, v.width);
            }
            opt_prev = Some(v);
        }
        todo!()
    }
}
