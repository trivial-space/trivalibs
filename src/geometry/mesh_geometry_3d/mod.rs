use std::collections::HashMap;

use glam::Vec3;

use super::vertex_index::{VertexIndex, WithVertexIndex};

pub struct Face {
    pub vertices: Vec<usize>,
    pub face_normal: Option<Vec3>,
}

pub trait VertexPosition3D {
    fn position(&self) -> Vec3;
}
pub struct VertexData<T: VertexPosition3D> {
    pub vertex: T,
    pub faces: Vec<usize>,
    pub vertex_normal: Option<Vec3>,
}

pub struct MeshGeometry3D<Idx: VertexIndex, T: VertexPosition3D + WithVertexIndex<Idx>> {
    next_index: usize,
    vertex_indices: HashMap<Idx, usize>,
    vertices: Vec<VertexData<T>>,
    faces: Vec<Face>,
}

impl<Idx: VertexIndex, T: VertexPosition3D + WithVertexIndex<Idx>> MeshGeometry3D<Idx, T> {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            faces: vec![],
            vertices: vec![],
            vertex_indices: HashMap::new(),
        }
    }

    pub fn add_face3(&mut self, v1: T, v2: T, v3: T) {
        let v1_idx = self.get_index(v1.vertex_index());
        let v2_idx = self.get_index(v2.vertex_index());
        let v3_idx = self.get_index(v3.vertex_index());

        let mut vertices = Vec::with_capacity(3);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);

        self.faces.push(Face {
            face_normal: None,
            vertices,
        });

        let face_idx = self.faces.len() - 1;

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
    }

    pub fn add_face4(&mut self, v1: T, v2: T, v3: T, v4: T) {
        let v1_idx = self.get_index(v1.vertex_index());
        let v2_idx = self.get_index(v2.vertex_index());
        let v3_idx = self.get_index(v3.vertex_index());
        let v4_idx = self.get_index(v4.vertex_index());

        let mut vertices = Vec::with_capacity(4);
        vertices.push(v1_idx);
        vertices.push(v2_idx);
        vertices.push(v3_idx);
        vertices.push(v4_idx);

        self.faces.push(Face {
            face_normal: None,
            vertices,
        });

        let face_idx = self.faces.len() - 1;

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
        self.add_vertex(v4_idx, face_idx, v4);
    }

    pub fn vertex(&self, i: usize) -> &VertexData<T> {
        &self.vertices[i]
    }

    pub fn face(&self, i: usize) -> &Face {
        &self.faces[i]
    }

    pub fn triangulate(&self) {}

    pub fn get_index(&mut self, vert_idx: Idx) -> usize {
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

        // let len = self.faces.len();
        // if len > 0 {
        //     let new_face = &self.faces[face_idx];
        //     for v in &new_face.vertices {
        //         let vert = self.vertices.get_mut(*v).unwrap();
        //         let v_f_idx = vert.faces.iter().position(|i| *i == len).unwrap();
        //         vert.faces[v_f_idx] = face_idx;
        //     }
        // }
    }

    fn add_vertex(&mut self, vertex_idx: usize, face_idx: usize, vertex: T) {
        // println!(
        //     "inserting vertex! idx: {0}, position: {1:?}",
        //     vertex_idx,
        //     vertex.position()
        // );
        if let Some(data) = self.vertices.get(vertex_idx) {
            let mut faces = data.faces.to_vec();
            faces.push(face_idx);
            self.vertices[vertex_idx] = VertexData {
                vertex,
                faces,
                vertex_normal: None,
            };
        } else {
            self.vertices.push(VertexData {
                vertex,
                faces: vec![face_idx],
                vertex_normal: None,
            });
        }
    }
}

#[cfg(test)]
mod tests;
