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

#[test]
fn cleanup_vertices() {
	let line1 = Line::from_vecs(
		2.0,
		[
			vec2(0.0, 0.0),
			vec2(1.0, 0.0),
			vec2(2.0, 0.0),
			vec2(3.0, 0.0),
		],
	);

	let cleaned1 = line1.cleanup_vertices(0.5, 0.001, 0.001);
	assert_eq!(cleaned1.vert_count(), 2);
	assert_eq!(cleaned1.get(0).pos, vec2(0.0, 0.0));
	assert_eq!(cleaned1.get(1).pos, vec2(3.0, 0.0));

	let line2 = Line::from_vecs(
		10.0,
		[
			vec2(0.0, 0.0),
			vec2(1.0, 0.0),
			vec2(2.0, 1.0),
			vec2(3.0, 0.0),
			vec2(4.0, 1.0),
			vec2(5.0, 0.0),
		],
	);

	let cleaned2_1 = line2.cleanup_vertices(1.0, 0.001, 0.001);
	assert_eq!(cleaned2_1.vert_count(), 2);

	let cleaned2_2 = line2.cleanup_vertices(0.5, 0.001, 0.001);
	assert_eq!(cleaned2_2.vert_count(), 3);

	let cleaned2_3 = line2.cleanup_vertices(0.2, 0.001, 0.001);
	assert_eq!(cleaned2_3.vert_count(), 5);

	let cleaned2_4 = line2.cleanup_vertices(0.1, 0.001, 0.001);
	assert_eq!(cleaned2_4.vert_count(), 6);
}
