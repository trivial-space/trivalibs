use super::LineVertex;
use glam::vec2;

#[test]
fn vert_point_to() {
    let mut vert = LineVertex::default();
    vert.point_to(&vec2(2.0, 0.0));

    assert_eq!(vert.len, 2.0);
    assert_eq!(vert.dir, vec2(1.0, 0.0));

    vert.point_to(&vec2(0.0, 3.0));

    assert_eq!(vert.len, 3.0);
    assert_eq!(vert.dir, vec2(0.0, 1.0));

    vert.point_to(&vec2(0.0, -4.0));

    assert_eq!(vert.len, 4.0);
    assert_eq!(vert.dir, vec2(0.0, -1.0));
}
