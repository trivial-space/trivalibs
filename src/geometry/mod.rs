use std::collections::HashMap;
use std::hash::Hash;

use glam::Vec3;

enum Face3D {
    Face3 {
        vertices: [usize; 3],
        face_normal: Option<Vec3>,
    },

    Face4 {
        vertices: [usize; 4],
        face_normal: Option<Vec3>,
    },
}

pub trait VertexPosition3D {
    fn position(&self) -> Vec3;
}

struct VertexData3D<T: VertexPosition3D> {
    vertex: T,
    faces: Vec<usize>,
}

#[derive(PartialEq)]
struct Pos(f32, f32, f32);

impl Eq for Pos {}
impl From<Vec3> for Pos {
    fn from(v: Vec3) -> Self {
        Self(v.x, v.y, v.z)
    }
}

impl Hash for Pos {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash_val: f64 =
            self.0 as f64 + self.1 as f64 * 1_000_000_f64 + self.2 as f64 * 1_000_000_000_000_f64;
        hash_val.to_ne_bytes().hash(state)
    }
}

pub struct Geometry3D<T: VertexPosition3D> {
    next_index: usize,
    vertex_indices: HashMap<Pos, usize>,
    vertices: Vec<VertexData3D<T>>,
    faces: Vec<Face3D>,
}

impl<T: VertexPosition3D> Geometry3D<T> {
    pub fn new() -> Self {
        Self {
            next_index: 0,
            faces: vec![],
            vertices: vec![],
            vertex_indices: HashMap::new(),
        }
    }

    pub fn add_face3(&mut self, v1: T, v2: T, v3: T) {
        let v1_idx = self.get_index(&v1);
        let v2_idx = self.get_index(&v2);
        let v3_idx = self.get_index(&v3);

        self.faces.push(Face3D::Face3 {
            face_normal: None,
            vertices: [v1_idx, v2_idx, v3_idx],
        });
        let face_idx = self.faces.len() - 1;

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
    }

    pub fn add_face4(&mut self, v1: T, v2: T, v3: T, v4: T) {
        let v1_idx = self.get_index(&v1);
        let v2_idx = self.get_index(&v2);
        let v3_idx = self.get_index(&v3);
        let v4_idx = self.get_index(&v4);

        self.faces.push(Face3D::Face4 {
            face_normal: None,
            vertices: [v1_idx, v2_idx, v3_idx, v4_idx],
        });

        let face_idx = self.faces.len() - 1;

        self.add_vertex(v1_idx, face_idx, v1);
        self.add_vertex(v2_idx, face_idx, v2);
        self.add_vertex(v3_idx, face_idx, v3);
        self.add_vertex(v4_idx, face_idx, v4);
    }

    fn add_vertex(&mut self, vertex_idx: usize, face_idx: usize, vertex: T) {
        if let Some(data) = self.vertices.get(vertex_idx) {
            let mut faces = data.faces.to_vec();
            faces.push(face_idx);
            self.vertices
                .insert(vertex_idx, VertexData3D { vertex, faces });
        } else {
            self.vertices.insert(
                vertex_idx,
                VertexData3D {
                    vertex,
                    faces: vec![face_idx],
                },
            );
        }
    }

    fn get_index(&mut self, v: &T) -> usize {
        let pos = Pos::from(v.position());
        if let Some(idx) = self.vertex_indices.get(&pos) {
            *idx
        } else {
            let idx = self.next_index;
            self.vertex_indices.insert(pos, idx);
            self.next_index = idx + 1;
            idx
        }
    }
}
