use std::{collections::HashMap, marker::PhantomData};

use glam::{vec3, Vec3};

use crate::rendering::buffered_geometry::{
    create_buffered_geometry_layout, BufferedGeometry, BufferedVertexData, ToBufferedGeometry,
    ToBufferedVertexData, VertexFormat, VertexType,
};

use super::vertex_index::{VertexIndex, WithVertexIndex};

#[derive(Debug)]
pub struct Face {
    pub vertices: Vec<usize>,
    pub face_normal: Option<Vec3>,
}

pub trait VertexPosition {
    fn position(&self) -> Vec3;
}

pub trait MeshVertex<Idx, BV>:
    VertexPosition + ToBufferedVertexData<BV> + WithVertexIndex<Idx>
where
    Idx: VertexIndex,
    BV: BufferedVertexData,
{
}

pub enum MeshBufferedGeometryType {
    NoNormals,
    VertexNormals,
    FaceNormals,
}

pub struct MeshVertexData<Idx, BV, V>
where
    Idx: VertexIndex,
    BV: BufferedVertexData,
    V: MeshVertex<Idx, BV>,
{
    _idx: PhantomData<Idx>,
    _bv: PhantomData<BV>,
    pub vertex: V,
    pub faces: Vec<usize>,
    pub vertex_normal: Option<Vec3>,
}

pub struct MeshGeometry<Idx, BV, V>
where
    Idx: VertexIndex,
    BV: BufferedVertexData,
    V: MeshVertex<Idx, BV>,
{
    next_index: usize,
    vertex_indices: HashMap<Idx, usize>,
    pub vertices: Vec<MeshVertexData<Idx, BV, V>>,
    pub faces: Vec<Face>,
}

impl<Idx, BV, V> MeshGeometry<Idx, BV, V>
where
    Idx: VertexIndex,
    BV: BufferedVertexData,
    V: MeshVertex<Idx, BV>,
{
    pub fn new() -> Self {
        Self {
            next_index: 0,
            faces: vec![],
            vertices: vec![],
            vertex_indices: HashMap::new(),
        }
    }

    pub fn add_face3(&mut self, v1: V, v2: V, v3: V) {
        let v1_idx = self.get_vertex_index(v1.vertex_index());
        let v2_idx = self.get_vertex_index(v2.vertex_index());
        let v3_idx = self.get_vertex_index(v3.vertex_index());

        let face_idx = self.faces.len();

        self.create_face3(v1_idx, v2_idx, v3_idx, None);

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
    }

    pub fn add_face4(&mut self, v1: V, v2: V, v3: V, v4: V) {
        let v1_idx = self.get_vertex_index(v1.vertex_index());
        let v2_idx = self.get_vertex_index(v2.vertex_index());
        let v3_idx = self.get_vertex_index(v3.vertex_index());
        let v4_idx = self.get_vertex_index(v4.vertex_index());

        let face_idx = self.faces.len();
        self.create_face4(v1_idx, v2_idx, v3_idx, v4_idx, None);

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
        self.add_vertex(v4_idx, face_idx, v4);
    }

    pub fn vertex(&self, i: usize) -> &MeshVertexData<Idx, BV, V> {
        &self.vertices[i]
    }

    pub fn face(&self, i: usize) -> &Face {
        &self.faces[i]
    }

    pub fn triangulate(&mut self) {
        let quads = self
            .faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.vertices.len() == 4)
            .map(|(i, f)| (i, f.vertices.clone(), f.face_normal))
            .rev()
            .collect::<Vec<_>>();

        for (i, verts, normal) in quads {
            self.remove_face(i);

            let face_idx1 = self.faces.len();
            self.create_face3(verts[0], verts[1], verts[2], normal);
            self.update_vertex_face(verts[0], face_idx1);
            self.update_vertex_face(verts[1], face_idx1);
            self.update_vertex_face(verts[2], face_idx1);

            let face_idx2 = self.faces.len();
            self.create_face3(verts[0], verts[2], verts[3], normal);
            self.update_vertex_face(verts[0], face_idx2);
            self.update_vertex_face(verts[2], face_idx2);
            self.update_vertex_face(verts[3], face_idx2);
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
            let mut vert = self.vertices.get_mut(i).unwrap();
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
                let vert = self.vertices.get_mut(*v).unwrap();
                let v_f_idx = vert.faces.iter().position(|i| *i == len).unwrap();
                vert.faces[v_f_idx] = face_idx;
            }
        }
    }

    pub fn set_vertex(&mut self, vertex_idx: usize, vertex: V) {
        if let Some(data) = self.vertices.get_mut(vertex_idx) {
            data.vertex = vertex
        }
    }

    pub fn generate_face_normals(&mut self) {
        for face in self.faces.iter_mut() {
            let verts = &face.vertices;
            let pos0 = self.vertices[verts[0]].vertex.position();
            let pos1 = self.vertices[verts[1]].vertex.position();
            let pos2 = self.vertices[verts[2]].vertex.position();
            let normal = (pos1 - pos0).cross(pos2 - pos0);
            face.face_normal = Some(normal);
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
        mut layout: Vec<VertexType>,
    ) -> BufferedGeometry {
        let mut buffer = vec![];
        let mut indices = vec![];

        match geom_type {
            MeshBufferedGeometryType::NoNormals => {
                for vert_data in self.vertices.iter() {
                    buffer.extend(bytemuck::bytes_of(
                        &vert_data.vertex.to_buffered_vertex_data(),
                    ));
                }
                self.fill_buffered_geometry_indices(&mut indices);
            }
            MeshBufferedGeometryType::VertexNormals => {
                for vert_data in self.vertices.iter() {
                    let normal = match vert_data.vertex_normal {
                        Some(normal) => normal,
                        None => panic!("generate vertex normals before generating buffers"),
                    };
                    buffer.extend(bytemuck::bytes_of(
                        &vert_data.vertex.to_buffered_vertex_data(),
                    ));
                    buffer.extend(bytemuck::bytes_of(&normal));
                }
                self.fill_buffered_geometry_indices(&mut indices);
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
                        buffer.extend(bytemuck::bytes_of(
                            &self.vertices[*v].vertex.to_buffered_vertex_data(),
                        ));
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

        BufferedGeometry {
            buffer,
            indices: if indices.len() == 0 {
                None
            } else {
                Some(indices)
            },
            vertex_size: geom_layout.vertex_size,
            vertex_layout: geom_layout.vertex_layout,
        }
    }

    fn fill_buffered_geometry_indices(&self, indices: &mut Vec<u32>) {
        for face in self.faces.iter() {
            if face.vertices.len() != 3 {
                panic!("triangulate the geometry before generating buffers");
            }
            for v in &face.vertices {
                indices.push(*v as u32)
            }
        }
    }

    fn add_vertex(&mut self, vertex_idx: usize, face_idx: usize, vertex: V) {
        if let Some(data) = self.vertices.get_mut(vertex_idx) {
            data.vertex = vertex;
        } else {
            self.vertices.push(MeshVertexData {
                vertex,
                faces: vec![],
                vertex_normal: None,
                _idx: PhantomData,
                _bv: PhantomData,
            });
        }
        self.update_vertex_face(vertex_idx, face_idx);
    }

    fn create_face3(&mut self, v1_idx: usize, v2_idx: usize, v3_idx: usize, normal: Option<Vec3>) {
        let mut vertices = Vec::with_capacity(3);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);

        self.faces.push(Face {
            face_normal: normal,
            vertices,
        });
    }

    fn create_face4(
        &mut self,
        v1_idx: usize,
        v2_idx: usize,
        v3_idx: usize,
        v4_idx: usize,
        normal: Option<Vec3>,
    ) {
        let mut vertices = Vec::with_capacity(4);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);
        vertices.push(v4_idx);

        self.faces.push(Face {
            face_normal: normal,
            vertices,
        });
    }

    fn update_vertex_face(&mut self, vertex_idx: usize, face_idx: usize) {
        if let Some(v) = self.vertices.get_mut(vertex_idx) {
            v.faces.push(face_idx);
        }
    }
}

impl<Idx, BV, V> ToBufferedGeometry for MeshGeometry<Idx, BV, V>
where
    Idx: VertexIndex,
    BV: BufferedVertexData,
    V: MeshVertex<Idx, BV>,
{
    fn to_buffered_geometry(&self, layout: Vec<VertexType>) -> BufferedGeometry {
        self.to_buffered_geometry_by_type(MeshBufferedGeometryType::NoNormals, layout)
    }
}

#[cfg(test)]
mod tests;
