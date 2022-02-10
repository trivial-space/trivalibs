use crate::geometry::{
    mesh_geometry_3d::{Face::Face3, MeshGeometry3D, VertexPosition3D},
    vertex_index::{VertIdx3f, WithVertexIndex},
};
use glam::{vec3, Vec3};

#[derive(Debug, PartialEq, Clone, Copy)]
struct Vert {
    pos: Vec3,
}
impl VertexPosition3D for Vert {
    fn position(&self) -> Vec3 {
        self.pos
    }
}
impl WithVertexIndex<VertIdx3f> for Vert {
    fn vertex_index(&self) -> VertIdx3f {
        VertIdx3f::from(self.pos)
    }
}
fn vert(x: f32, y: f32, z: f32) -> Vert {
    Vert { pos: vec3(x, y, z) }
}

#[test]
fn generate_geometry() {
    let mut geom = MeshGeometry3D::new();
    let v1 = vert(0.0, 0.0, 0.0);
    let v2 = vert(1.0, 0.0, 0.0);
    let v3 = vert(0.0, 1.0, 0.0);

    geom.add_face3(v1, v2, v3);

    assert_eq!(geom.vertex(0).vertex, v1);
    assert_eq!(geom.vertex(1).vertex, v2);
    assert_eq!(geom.vertex(2).vertex, v3);

    if let Face3 { vertices, .. } = geom.face(0) {
        assert_eq!(vertices, &[0, 1, 2]);
    } else {
        panic!("not a face3!!")
    }

    assert_eq!(geom.faces.len(), 1);
    assert_eq!(geom.vertices.len(), 3);
}
