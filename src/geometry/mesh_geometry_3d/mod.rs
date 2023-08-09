use std::collections::HashMap;

use glam::{vec3, Vec3};

use crate::rendering::buffered_geometry::{
    create_buffered_geometry_layout, BufferedGeometry, BufferedVertexData, OverrideAttributesWith,
    RenderingPrimitive, ToBufferedGeometry, VertexFormat, VertexType,
};

use super::vertex_index::VertexIndex;

#[derive(Debug)]
pub struct Face<BV>
where
    BV: BufferedVertexData + OverrideAttributesWith,
{
    pub vertices: Vec<usize>,
    pub face_normal: Option<Vec3>,
    pub data: Option<BV>,
}

pub trait Position3D: BufferedVertexData {
    fn position(&self) -> Vec3;
}

#[derive(Debug, Copy, Clone)]
pub struct MeshVertex<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    pub vertex_index: Idx,
    pub data: BV,
}

impl<Idx, BV> PartialEq for MeshVertex<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    fn eq(&self, other: &Self) -> bool {
        self.vertex_index == other.vertex_index
    }
}

pub enum MeshBufferedGeometryType {
    NoNormals,
    VertexNormals,
    VertexNormalFaceData,
    FaceNormals,
}

pub struct MeshVertexData<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    pub vertex: MeshVertex<Idx, BV>,
    pub faces: Vec<usize>,
    pub vertex_normal: Option<Vec3>,
}

pub struct MeshGeometry<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    next_index: usize,
    vertex_indices: HashMap<Idx, usize>,
    pub vertices: Vec<MeshVertexData<Idx, BV>>,
    pub faces: Vec<Face<BV>>,
}

impl<Idx, BV> MeshGeometry<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    pub fn new() -> Self {
        Self {
            next_index: 0,
            faces: vec![],
            vertices: vec![],
            vertex_indices: HashMap::new(),
        }
    }

    pub fn add_face3_data(
        &mut self,
        v1: MeshVertex<Idx, BV>,
        v2: MeshVertex<Idx, BV>,
        v3: MeshVertex<Idx, BV>,
        normal: Option<Vec3>,
        data: Option<BV>,
    ) {
        let v1_idx = self.get_vertex_index(v1.vertex_index);
        let v2_idx = self.get_vertex_index(v2.vertex_index);
        let v3_idx = self.get_vertex_index(v3.vertex_index);

        let face_idx = self.faces.len();

        self.create_face3(v1_idx, v2_idx, v3_idx, normal, data);

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
    }

    pub fn add_face4_data(
        &mut self,
        v1: MeshVertex<Idx, BV>,
        v2: MeshVertex<Idx, BV>,
        v3: MeshVertex<Idx, BV>,
        v4: MeshVertex<Idx, BV>,
        normal: Option<Vec3>,
        data: Option<BV>,
    ) {
        let v1_idx = self.get_vertex_index(v1.vertex_index);
        let v2_idx = self.get_vertex_index(v2.vertex_index);
        let v3_idx = self.get_vertex_index(v3.vertex_index);
        let v4_idx = self.get_vertex_index(v4.vertex_index);

        let face_idx = self.faces.len();
        self.create_face4(v1_idx, v2_idx, v3_idx, v4_idx, normal, data);

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
        self.add_vertex(v4_idx, face_idx, v4);
    }

    pub fn add_face3(
        &mut self,
        v1: MeshVertex<Idx, BV>,
        v2: MeshVertex<Idx, BV>,
        v3: MeshVertex<Idx, BV>,
    ) {
        self.add_face3_data(v1, v2, v3, None, None)
    }

    pub fn add_face4(
        &mut self,
        v1: MeshVertex<Idx, BV>,
        v2: MeshVertex<Idx, BV>,
        v3: MeshVertex<Idx, BV>,
        v4: MeshVertex<Idx, BV>,
    ) {
        self.add_face4_data(v1, v2, v3, v4, None, None)
    }

    pub fn vertex(&self, i: usize) -> &MeshVertexData<Idx, BV> {
        &self.vertices[i]
    }

    pub fn face(&self, i: usize) -> &Face<BV> {
        &self.faces[i]
    }

    pub fn triangulate(&mut self) {
        let quads = self
            .faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.vertices.len() == 4)
            .map(|(i, f)| (i, f.vertices.clone(), f.face_normal, f.data))
            .rev()
            .collect::<Vec<_>>();

        for (i, verts, normal, data) in quads {
            self.remove_face(i);

            let face_idx1 = self.faces.len();
            self.create_face3(verts[0], verts[1], verts[2], normal, data);
            self.add_vertex_face(verts[0], face_idx1);
            self.add_vertex_face(verts[1], face_idx1);
            self.add_vertex_face(verts[2], face_idx1);

            let face_idx2 = self.faces.len();
            self.create_face3(verts[0], verts[2], verts[3], normal, data);
            self.add_vertex_face(verts[0], face_idx2);
            self.add_vertex_face(verts[2], face_idx2);
            self.add_vertex_face(verts[3], face_idx2);
        }
    }

    pub fn get_vertex_index(&mut self, vert_idx: Idx) -> usize {
        if let Some(idx) = self.vertex_indices.get(&vert_idx) {
            *idx
        } else {
            let idx = self.next_index;
            self.vertex_indices.insert(vert_idx, idx);
            self.next_index = idx + 1;
            idx
        }
    }

    pub fn remove_face(&mut self, face_idx: usize) {
        let face = self.faces.swap_remove(face_idx);
        face.vertices.into_iter().for_each(|i| {
            let vert = &mut self.vertices[i];
            let new_faces = vert
                .faces
                .iter()
                .map(|j| *j)
                .filter(|j| *j != face_idx)
                .collect();
            vert.faces = new_faces;
        });

        let len = self.faces.len();
        if face_idx != len {
            let new_face = &self.faces[face_idx];
            for v in &new_face.vertices {
                let vert = &mut self.vertices[*v];
                let v_f_idx = vert.faces.iter().position(|i| *i == len).unwrap();
                vert.faces[v_f_idx] = face_idx;
            }
        }
    }

    pub fn set_vertex(&mut self, vertex_idx: usize, vertex: MeshVertex<Idx, BV>) {
        if let Some(data) = self.vertices.get_mut(vertex_idx) {
            data.vertex = vertex
        }
    }

    pub fn generate_face_normals(&mut self) {
        for face in self.faces.iter_mut() {
            let verts = &face.vertices;
            let pos0 = self.vertices[verts[0]].vertex.data.position();
            let pos1 = self.vertices[verts[1]].vertex.data.position();
            let pos2 = self.vertices[verts[2]].vertex.data.position();

            let mut v1 = pos2 - pos0;
            let mut v2 = pos1 - pos0;

            let v1_len = v1.length();
            let v2_len = v2.length();
            let v1_len_0 = v1_len < 0.0001;
            let v2_len_0 = v2_len < 0.0001;
            let v3_len_0 = (v2 / v2_len).dot(v1 / v1_len).abs() > 0.9999;

            if (v1_len_0 || v2_len_0 || v3_len_0) && face.vertices.len() > 3 {
                if v2_len_0 {
                    v2 = pos1 - self.vertices[verts[3]].vertex.data.position();
                } else {
                    v1 = self.vertices[verts[3]].vertex.data.position() - pos0;
                }
            }

            let normal = v2.cross(v1);
            face.face_normal = Some(normal.normalize());
        }
    }

    pub fn generate_vertex_normals(&mut self) {
        for vert in self.vertices.iter_mut() {
            let mut new_normal = vec3(0.0, 0.0, 0.0);
            for face_idx in &vert.faces {
                match self.faces[*face_idx].face_normal {
                    Some(normal) => new_normal += normal,
                    None => panic!("generate face normals before generating vertex normals"),
                }
            }
            vert.vertex_normal = Some(new_normal.normalize());
        }
    }

    pub fn to_buffered_geometry_by_type(
        &self,
        geom_type: MeshBufferedGeometryType,
    ) -> BufferedGeometry {
        let mut buffer = vec![];
        let mut indices = vec![];
        let mut layout: Vec<VertexType> = BV::vertex_layout();

        match geom_type {
            MeshBufferedGeometryType::NoNormals => {
                for vert_data in self.vertices.iter() {
                    buffer.extend(bytemuck::bytes_of(&vert_data.vertex.data));
                }
                self.fill_buffered_geometry_indices(&mut indices);
            }

            MeshBufferedGeometryType::VertexNormals => {
                for vert_data in self.vertices.iter() {
                    let normal = match vert_data.vertex_normal {
                        Some(normal) => normal,
                        None => panic!("generate vertex normals before generating buffers"),
                    };
                    buffer.extend(bytemuck::bytes_of(&vert_data.vertex.data));
                    buffer.extend(bytemuck::bytes_of(&normal));
                }
                self.fill_buffered_geometry_indices(&mut indices);
                layout.push(VertexType {
                    name: "normal",
                    format: VertexFormat::Float32x3,
                })
            }

            MeshBufferedGeometryType::VertexNormalFaceData => {
                for face in self.faces.iter() {
                    if face.vertices.len() != 3 {
                        panic!("triangulate the geometry before generating buffers");
                    }
                    for v in &face.vertices {
                        let normal = match self.vertices[*v].vertex_normal {
                            Some(normal) => normal,
                            None => panic!("generate vertex normals before generating buffers"),
                        };
                        let mut data = self.vertices[*v].vertex.data;
                        if face.data.is_some() {
                            data = data.override_with(&face.data.unwrap());
                        }
                        buffer.extend(bytemuck::bytes_of(&data));
                        buffer.extend(bytemuck::bytes_of(&normal));
                    }
                }
                layout.push(VertexType {
                    name: "normal",
                    format: VertexFormat::Float32x3,
                })
            }

            MeshBufferedGeometryType::FaceNormals => {
                for face in self.faces.iter() {
                    if face.vertices.len() != 3 {
                        panic!("triangulate the geometry before generating buffers");
                    }
                    let normal = match face.face_normal {
                        Some(normal) => normal,
                        None => panic!("generate face normals before generating buffers"),
                    };
                    for v in &face.vertices {
                        let mut data = self.vertices[*v].vertex.data;
                        if face.data.is_some() {
                            data = data.override_with(&face.data.unwrap());
                        }
                        buffer.extend(bytemuck::bytes_of(&data));
                        buffer.extend(bytemuck::bytes_of(&normal));
                    }
                }
                layout.push(VertexType {
                    name: "normal",
                    format: VertexFormat::Float32x3,
                })
            }
        };

        let geom_layout = create_buffered_geometry_layout(layout);

        let buffer_len = buffer.len() as u32;
        let indices_len = indices.len() as u32;

        BufferedGeometry {
            buffer,
            rendering_primitive: RenderingPrimitive::Triangles,
            indices: if indices_len == 0 {
                None
            } else {
                Some(indices)
            },
            vertex_size: geom_layout.vertex_size,
            vertex_count: if indices_len > 0 {
                indices_len / 4 // 4 bytes per u32 index
            } else {
                buffer_len / geom_layout.vertex_size
            },
            vertex_layout: geom_layout.vertex_layout,
        }
    }

    fn fill_buffered_geometry_indices(&self, indices: &mut Vec<u8>) {
        for face in self.faces.iter() {
            if face.vertices.len() != 3 {
                panic!("triangulate the geometry before generating buffers");
            }
            for v in &face.vertices {
                let i = *v as u32;
                indices.extend(bytemuck::bytes_of(&i))
            }
        }
    }

    fn add_vertex(&mut self, vertex_idx: usize, face_idx: usize, vertex: MeshVertex<Idx, BV>) {
        if let Some(data) = self.vertices.get_mut(vertex_idx) {
            data.vertex = vertex;
            self.add_vertex_face(vertex_idx, face_idx);
        } else {
            self.vertices.push(MeshVertexData {
                vertex,
                faces: vec![face_idx],
                vertex_normal: None,
            });
        }
    }

    fn create_face3(
        &mut self,
        v1_idx: usize,
        v2_idx: usize,
        v3_idx: usize,
        normal: Option<Vec3>,
        data: Option<BV>,
    ) {
        let mut vertices = Vec::with_capacity(3);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);

        self.faces.push(Face {
            face_normal: normal,
            vertices,
            data,
        });
    }

    fn create_face4(
        &mut self,
        v1_idx: usize,
        v2_idx: usize,
        v3_idx: usize,
        v4_idx: usize,
        normal: Option<Vec3>,
        data: Option<BV>,
    ) {
        let mut vertices = Vec::with_capacity(4);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);
        vertices.push(v4_idx);

        self.faces.push(Face {
            face_normal: normal,
            vertices,
            data,
        });
    }

    fn add_vertex_face(&mut self, vertex_idx: usize, face_idx: usize) {
        if let Some(v) = self.vertices.get_mut(vertex_idx) {
            v.faces.push(face_idx);
        }
    }
}

impl<Idx, BV> ToBufferedGeometry for MeshGeometry<Idx, BV>
where
    Idx: VertexIndex,
    BV: BufferedVertexData + OverrideAttributesWith + Position3D,
{
    fn to_buffered_geometry(&self) -> BufferedGeometry {
        self.to_buffered_geometry_by_type(MeshBufferedGeometryType::NoNormals)
    }
}

#[cfg(test)]
mod tests;
