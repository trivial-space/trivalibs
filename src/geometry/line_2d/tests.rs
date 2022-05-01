use super::{Line, LineVertex};
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

#[test]
fn line_length() {
    let mut line = Line::new(10.0);

    line.add(vec2(0.0, 0.0));
    line.add(vec2(2.0, 0.0));
    line.add(vec2(2.0, 1.0));
    line.add(vec2(2.0, 3.0));

    assert_eq!(line.line_length(), 5.0);
    assert_eq!(line.vert_count(), 4);
    assert_eq!(line.last().dir, vec2(0.0, 1.0));
    assert_eq!(line.last().len, 0.0);
}

#[test]
fn from_vecs() {
    let line = Line::from_vecs(
        2.0,
        [
            vec2(0.0, 0.0),
            vec2(2.0, 0.0),
            vec2(2.0, 1.0),
            vec2(2.0, 3.0),
        ],
    );

    assert_eq!(line.line_length(), 5.0);
    assert_eq!(line.vert_count(), 4);
}
