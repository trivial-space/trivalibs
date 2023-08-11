use super::{Line, LineData};
use crate::{
    data_structures::neighbour_list::traits::WithNeighboursTransform,
    rendering::buffered_geometry::{
        create_buffered_geometry_layout, vert_type, BufferedGeometry, BufferedVertexData,
        RenderingPrimitive,
        VertexFormat::{Float32, Float32x2},
        VertexType,
    },
    utils::default,
};
use bytemuck::{Pod, Zeroable};
use glam::Vec2;

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

pub struct LineGeometryOpts {
    pub total_length: Option<f32>,
    pub prev_direction: Option<Vec2>,
    pub next_direction: Option<Vec2>,

    pub smouth_depth: u8,
    pub smouth_angle_threshold: f32,
    pub smouth_min_length: f32,

    pub cap_width_length_ratio: f32,
    pub swap_texture_orientation: bool,
}

impl Default for LineGeometryOpts {
    fn default() -> Self {
        Self {
            smouth_depth: 0,
            smouth_min_length: 3.0,
            smouth_angle_threshold: 0.05,

            cap_width_length_ratio: 1.0,
            swap_texture_orientation: false,

            total_length: None,
            prev_direction: None,
            next_direction: None,
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

impl Line {
    pub fn to_buffered_geometry_with(&self, opts: LineGeometryOpts) -> BufferedGeometry {
        let mut top_line = LineData::<f32>::new(self.default_width);
        let mut bottom_line = LineData::<f32>::new(self.default_width);
        let mut line_length = self.len_offset;

        for (prev, v, next) in self.iter().with_neighbours() {
            if prev.is_none() {
                top_line.add_width_data(v.pos, v.width, line_length);
            }

            // TODO: adjust for first and last vertex, if prev_direction or next_direction are given
            let new_points = line_mitter_positions(&v.pos, &v.dir, v.width, prev.map(|x| &x.dir));

            top_line.add_width_data(new_points[0], v.width, line_length);
            bottom_line.add_width_data(new_points[1], v.width, line_length);

            if next.is_none() {
                bottom_line.add_width_data(v.pos, v.width, line_length);
            }

            line_length += v.len;
        }

        // TODO: smouth top and bottom lines if smouth_depth is > 0

        let mut buffer = vec![];
        let mut indices: Vec<u32> = vec![];

        let total_length = opts.total_length.unwrap_or(line_length);

        let n = usize::max(top_line.vert_count(), bottom_line.vert_count());

        let mut top_idx: u32 = 0;
        let mut bottom_idx: u32 = 0;
        let mut next_idx: u32 = 0;

        for i in 0..n {
            let top_opt = top_line.get_opt(i);
            let bottom_opt = bottom_line.get_opt(i);

            if let Some(top) = top_opt {
                let v = if i == 0 { 0.5 } else { 0.0 };
                let top_uv = Vec2::new(top.data / total_length, v);
                let top_local_uv = Vec2::new(top.data / self.len, v);
                let top_vertex = VertexData {
                    position: top.pos,
                    width: top.width,
                    length: top.data,
                    uv: top_uv,
                    local_uv: top_local_uv,
                };

                buffer.push(top_vertex);

                indices.push(next_idx);
                top_idx = next_idx;
                next_idx += 1;
            } else {
                indices.push(top_idx);
            }

            if let Some(bottom) = bottom_opt {
                let v = if i == bottom_line.vert_count() - 1 {
                    0.5
                } else {
                    1.0
                };
                let bottom_uv = Vec2::new(bottom.data / total_length, v);
                let bottom_local_uv = Vec2::new(bottom.data / self.len, v);
                let bottom_vertex = VertexData {
                    position: bottom.pos,
                    width: bottom.width,
                    length: bottom.data,
                    uv: bottom_uv,
                    local_uv: bottom_local_uv,
                };

                buffer.push(bottom_vertex);

                indices.push(next_idx);
                bottom_idx = next_idx;
                next_idx += 1;
            } else {
                indices.push(bottom_idx);
            }
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
