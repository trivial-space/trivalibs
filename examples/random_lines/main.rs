use trivalibs::{
	map,
	painter::{form::Form, prelude::*},
	prelude::*,
};

struct App {
	canvas: Layer,
	form: Form,
	color_binding: BindingBuffer<Vec3U>,
	timer: f32,
}

// Generate a single line made of multiple quad segments
fn generate_line_geometry(start: Vec2, end: Vec2, width: f32, segments: usize) -> Vec<Vec2> {
	let mut vertices = Vec::new();

	let dir = (end - start).normalize();
	let perp = vec2(-dir.y, dir.x) * (width * 0.5);

	for i in 0..segments {
		let t0 = i as f32 / segments as f32;
		let t1 = (i + 1) as f32 / segments as f32;

		let p0 = start + dir * (t0 * (end - start).length());
		let p1 = start + dir * (t1 * (end - start).length());

		// Create quad for this segment (two triangles)
		// Triangle 1
		vertices.push(vec2(p0.x - perp.x, p0.y - perp.y));
		vertices.push(vec2(p1.x + perp.x, p1.y + perp.y));
		vertices.push(vec2(p0.x + perp.x, p0.y + perp.y));

		// Triangle 2
		vertices.push(vec2(p0.x - perp.x, p0.y - perp.y));
		vertices.push(vec2(p1.x - perp.x, p1.y - perp.y));
		vertices.push(vec2(p1.x + perp.x, p1.y + perp.y));
	}

	vertices
}

// Generate random lines with varying segment counts - each line as separate vector
fn generate_all_lines() -> Vec<Vec<Vec2>> {
	let line_count = rand_range(1.0, 10.0) as usize;
	let mut all_lines = Vec::new();

	for _ in 0..line_count {
		let start = vec2(rand_range(-0.9, 0.9), rand_range(-0.9, 0.9));
		let end = vec2(rand_range(-0.9, 0.9), rand_range(-0.9, 0.9));
		let segments = rand_range(1.0, 10.0) as usize;

		// Width in normalized device coordinates (60px relative to screen)
		let width = 0.06;

		let line_verts = generate_line_geometry(start, end, width, segments);
		all_lines.push(line_verts);
	}

	all_lines
}

impl CanvasApp for App {
	fn init(p: &mut Painter) -> Self {
		// Generate initial geometry - each line gets its own buffer
		let initial_lines = generate_all_lines();

		// Create form with initial data using FormBuilder with_buffers
		// Pass the Vec directly - each Vec<Vec3> converts to FormBuffer automatically
		let form = p.form_builder().with_buffers(&initial_lines).create();

		// Create shade with vec3 position and vec3 color uniform
		let shade = p
			.shade([Float32x2])
			.with_bindings([BINDING_BUFFER_FRAG])
			.create();

		// Load shaders
		load_vertex_shader!(shade, p, "./shader/vertex.spv");
		load_fragment_shader!(shade, p, "./shader/fragment.spv");

		// Create color binding with random initial color
		let color_binding = p.bind_vec3();
		color_binding.update_vec3(p, rand_vec3());

		// Create shape
		let shape = p
			.shape(form, shade)
			.with_bindings(map! {
				0 => color_binding.binding(),
			})
			.create();

		let canvas = p
			.layer()
			.with_shape(shape)
			.with_clear_color(wgpu::Color::WHITE)
			.with_multisampling()
			.create();

		Self {
			canvas,
			form,
			color_binding,
			timer: 0.0,
		}
	}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.timer += tpf;

		// Every second, regenerate geometry and color
		if self.timer >= 1.0 {
			self.timer -= 1.0;

			// Generate new random lines - each line as separate buffer
			let new_lines = generate_all_lines();

			// Update form with all new geometries using update_all
			// Pass the Vec directly - each Vec<Vec2> converts to FormBuffer automatically
			self.form.update_all(p, &new_lines);

			// Update color with new random color
			self.color_binding.update_vec3(p, rand_vec3());

			let total_vertices: usize = new_lines.iter().map(|l| l.len()).sum();
			println!(
				"\nGenerated {} lines with {} total vertices",
				new_lines.len(),
				total_vertices
			);
		}

		p.paint_and_show(self.canvas);

		// Request continuous rendering
		p.request_next_frame();
	}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			use_vsync: false,
			remember_window_dimensions: true,
			..default()
		})
		.start();
}
