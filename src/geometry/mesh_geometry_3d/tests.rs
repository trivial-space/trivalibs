use crate::geometry::{
    mesh_geometry_3d::{Face, MeshGeometry3D, VertexPosition3D},
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

    let Face { vertices, .. } = geom.face(0);
    assert_eq!(vertices, &[0, 1, 2]);

    assert_eq!(geom.faces.len(), 1);
    assert_eq!(geom.vertices.len(), 3);

    assert_eq!(geom.get_index(v2.vertex_index()), 1);
    assert_eq!(geom.get_index(v3.vertex_index()), 2);
}

#[test]
fn remove_face() {
    let mut geom = MeshGeometry3D::new();
    let v = vert(1.0, 1.0, 0.0);

    geom.add_face3(v, vert(2.0, 0.0, 0.0), vert(0.0, 0.0, 0.0));
    geom.add_face3(v, vert(2.0, 2.0, 0.0), vert(2.0, 0.0, 0.0));
    geom.add_face3(v, vert(0.0, 2.0, 0.0), vert(2.0, 2.0, 0.0));
    geom.add_face3(v, vert(0.0, 0.0, 0.0), vert(0.0, 2.0, 0.0));

    assert_eq!(geom.faces.len(), 4);
    assert_eq!(geom.next_index, 5);
    assert_eq!(geom.vertices.len(), 5);
    assert_eq!(geom.face(1).vertices, vec![0, 3, 1]);
    assert_eq!(geom.face(3).vertices, vec![0, 2, 4]);
    assert_eq!(geom.vertex(0).faces.len(), 4);

    geom.remove_face(1);

    assert_eq!(geom.faces.len(), 3);
    assert_eq!(geom.vertices.len(), 5);
    assert_eq!(geom.face(1).vertices, vec![0, 2, 4]);
    assert_eq!(geom.vertex(0).faces.len(), 3);
}
