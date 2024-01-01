use super::{Line, LineData};
use crate::{
    data_structures::neighbour_list::traits::{NeighbourMapTransform, WithNeighboursTransform},
    rendering::buffered_geometry::{
        create_buffered_geometry_layout, vert_type, BufferedGeometry, BufferedVertexData,
        RenderingPrimitive,
        VertexFormat::{Float32, Float32x2},
        VertexType,
    },
    utils::default,
};
use bytemuck::{Pod, Zeroable};
use glam::{bool, Vec2};

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
pub struct VertexData {
    position: Vec2,
    width: f32,
    length: f32,
    uv: Vec2,
    local_uv: Vec2,
}

impl BufferedVertexData for VertexData {
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

#[derive(Clone, Copy)]
pub struct LineGeometryProps {
    pub smouth_depth: u8,
    pub smouth_angle_threshold: f32,
    pub smouth_min_length: f32,
    pub cap_width_length_ratio: f32,
    pub total_length: Option<f32>,
    pub prev_direction: Option<Vec2>,
    pub next_direction: Option<Vec2>,
    pub swap_texture_orientation: bool,
}

impl Default for LineGeometryProps {
    fn default() -> Self {
        Self {
            smouth_depth: 0,
            smouth_min_length: 3.0,
            smouth_angle_threshold: 0.05,
            cap_width_length_ratio: 1.0,
            total_length: None,
            prev_direction: None,
            next_direction: None,
            swap_texture_orientation: false,
        }
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

fn cross_2d(v1: Vec2, v2: Vec2) -> f32 {
    v1.x * v2.y - v1.y * v2.x
}

impl Line {
    pub fn to_buffered_geometry_with(&self, props: LineGeometryProps) -> BufferedGeometry {
        let mut top_line = LineData::<f32>::new(self.default_width);
        let mut bottom_line = LineData::<f32>::new(self.default_width);
        let mut line_length = self.len_offset;

        for (prev, v, next) in self.iter().with_neighbours() {
            let mut new_points =
                line_mitter_positions(&v.pos, &v.dir, v.width, prev.map(|x| &x.dir));

            if prev.is_none() {
                top_line.add_width_data(v.pos, v.width, line_length);
                bottom_line.add_width_data(v.pos, v.width, line_length);
            }

            // adjust first vertex
            if prev.is_none() && props.prev_direction.is_some() {
                let prev_dir = props.prev_direction.unwrap();
                let c = v.width / (prev_dir * -1.0 + v.dir).normalize().dot(v.dir);
                let a = f32::sqrt(c * c - v.width * v.width);
                if a > 0.001 {
                    if cross_2d(v.dir, prev_dir) > 0. {
                        new_points[0] += -a * v.dir;
                        new_points[1] += a * v.dir;
                    } else {
                        new_points[0] += a * v.dir;
                        new_points[1] += -a * v.dir;
                    }
                }
            }

            // adjust last vertex
            if next.is_none() && props.next_direction.is_some() {
                let next_dir = props.next_direction.unwrap();
                let c = v.width / (v.dir * -1.0 + next_dir).normalize().dot(next_dir);
                let a = f32::sqrt(c * c - v.width * v.width);

                if a > 0.001 {
                    if cross_2d(next_dir, v.dir) > 0. {
                        new_points[0] += a * v.dir;
                        new_points[1] += -a * v.dir;
                    } else {
                        new_points[0] += -a * v.dir;
                        new_points[1] += a * v.dir;
                    }
                }
            }

            top_line.add_width_data(new_points[0], v.width, line_length);
            bottom_line.add_width_data(new_points[1], v.width, line_length);

            if next.is_none() {
                top_line.add_width_data(v.pos, v.width, line_length);
                bottom_line.add_width_data(v.pos, v.width, line_length);
            }

            line_length += v.len;
        }

        if props.smouth_depth > 0 {
            for _ in 0..props.smouth_depth {
                top_line = top_line.smouth_edges_threshold(
                    0.25,
                    props.smouth_min_length,
                    props.smouth_angle_threshold,
                );
                bottom_line = bottom_line.smouth_edges_threshold(
                    0.25,
                    props.smouth_min_length,
                    props.smouth_angle_threshold,
                );
            }
        }

        let mut buffer = vec![];
        let mut indices: Vec<u32> = vec![];

        let total_length = props.total_length.unwrap_or(line_length);

        let mut top_idx: u32 = 0;
        let mut bottom_idx: u32 = 0;
        let mut next_idx: u32 = 0;

        let mut top_length: f32 = 0.;
        let mut bottom_length: f32 = 0.;
        let mut balance: f32 = 0.;

        let mut top_i: usize = 0;
        let mut bottom_i: usize = 0;

        while top_i < top_line.vert_count() || bottom_i < bottom_line.vert_count() {
            let top_opt = top_line.get_opt(top_i);
            let bottom_opt = bottom_line.get_opt(bottom_i);

            if top_opt.is_some() && balance <= 0. {
                let top = top_opt.unwrap();
                top_length = top.data;

                let v = if top_i == 0 || top_i == top_line.vert_count() - 1 {
                    0.5
                } else {
                    if props.swap_texture_orientation {
                        1.0
                    } else {
                        0.0
                    }
                };
                let top_uv = Vec2::new(top_length / total_length, v);
                let top_local_uv = Vec2::new((top_length - self.len_offset) / self.len, v);
                let top_vertex = VertexData {
                    position: top.pos,
                    width: top.width,
                    uv: top_uv,
                    local_uv: top_local_uv,
                    length: top_length,
                };

                buffer.push(top_vertex);

                indices.push(next_idx);
                top_idx = next_idx;
                next_idx += 1;
                top_i += 1;
            } else {
                indices.push(top_idx);
            }

            if bottom_opt.is_some() && balance >= 0. {
                let bottom = bottom_opt.unwrap();
                bottom_length = bottom.data;
                let v = if bottom_i == 0 || bottom_i == bottom_line.vert_count() - 1 {
                    0.5
                } else {
                    if props.swap_texture_orientation {
                        0.0
                    } else {
                        1.0
                    }
                };
                let bottom_uv = Vec2::new(bottom_length / total_length, v);
                let bottom_local_uv = Vec2::new((bottom_length - self.len_offset) / self.len, v);
                let bottom_vertex = VertexData {
                    position: bottom.pos,
                    width: bottom.width,
                    uv: bottom_uv,
                    local_uv: bottom_local_uv,
                    length: bottom_length,
                };

                buffer.push(bottom_vertex);

                indices.push(next_idx);
                bottom_idx = next_idx;
                next_idx += 1;
                bottom_i += 1;
            } else {
                indices.push(bottom_idx);
            }

            balance = top_length - bottom_length;
        }

        let indices_len = indices.len();

        let geom_layout = create_buffered_geometry_layout(VertexData::vertex_layout());

        BufferedGeometry {
            buffer: bytemuck::cast_slice(&buffer).to_vec(),
            rendering_primitive: RenderingPrimitive::TriangleStrip,
            indices: Some(bytemuck::cast_slice(&indices).to_vec()),
            vertex_size: geom_layout.vertex_size,
            vertex_count: indices_len as u32,
            vertex_layout: geom_layout.vertex_layout,
        }
    }

    pub fn to_buffered_geometry(&self) -> BufferedGeometry {
        self.to_buffered_geometry_with(default())
    }
}

pub trait LineBufferedGeometryVec {
    fn to_buffered_geometry_with(&self, props: LineGeometryProps) -> Vec<BufferedGeometry>;
    fn to_buffered_geometry(&self) -> Vec<BufferedGeometry>;
}

impl LineBufferedGeometryVec for Vec<Line> {
    fn to_buffered_geometry_with(&self, props: LineGeometryProps) -> Vec<BufferedGeometry> {
        let total_length = self.iter().fold(0.0, |acc, x| acc + x.len);

        self.iter()
            .enumerate()
            .map_with_prev_next(|(i, line), prev, next| {
                line.to_buffered_geometry_with(LineGeometryProps {
                    total_length: Some(total_length),
                    swap_texture_orientation: i % 2 != 0,
                    prev_direction: prev.map(|(_, x)| x.last().dir),
                    next_direction: next.map(|(_, x)| x.first().dir),
                    ..props
                })
            })
            .collect()
    }

    fn to_buffered_geometry(&self) -> Vec<BufferedGeometry> {
        self.to_buffered_geometry_with(default())
    }
}
