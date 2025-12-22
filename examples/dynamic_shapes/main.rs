use trivalibs::{map, painter::prelude::*, prelude::*};

// Generate a simple square geometry centered at a position with a given size
fn generate_square(pos: Vec2, size: f32) -> Vec<Vec2> {
	let half_size = size * 0.5;
	vec![
		// Triangle 1
		vec2(pos.x - half_size, pos.y - half_size),
		vec2(pos.x + half_size, pos.y + half_size),
		vec2(pos.x - half_size, pos.y + half_size),
		// Triangle 2
		vec2(pos.x - half_size, pos.y - half_size),
		vec2(pos.x + half_size, pos.y - half_size),
		vec2(pos.x + half_size, pos.y + half_size),
	]
}

// Pick a random selection of shapes from the pool
// Randomly selects a count between 0 and POOL_SIZE
fn pick_random_shapes(shape_pool: &[Shape]) -> Vec<Shape> {
	let count = rand_range(0.0, POOL_SIZE as f32) as usize;
	let mut selected_shapes = Vec::new();
	for _ in 0..count {
		let idx = rand_range(0.0, shape_pool.len() as f32) as usize % shape_pool.len();
		selected_shapes.push(shape_pool[idx]);
	}
	selected_shapes
}

struct App {
	canvas: Layer,
	layer1: Layer,
	layer2: Layer,
	shape_pool: Vec<Shape>,
	timer: f32,
}

const POOL_SIZE: usize = 30;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		// Create a single shade that all shapes will share
		let shade = p
			.shade(&[Float32x2])
			.with_bindings([BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
			.create();
		load_vertex_shader!(shade, p, "./shader/vertex.spv");
		load_fragment_shader!(shade, p, "./shader/fragment.spv");

		// Create a pool of shapes, each with:
		// - A form (square at random position)
		// - A random color binding
		let mut shape_pool = Vec::with_capacity(POOL_SIZE);

		for i in 0..POOL_SIZE {
			// Random position in normalized device coordinates (-1 to 1)
			let pos = vec2(rand_range(-0.8, 0.8), rand_range(-0.8, 0.8));
			let size = rand_range(0.1, 0.3);

			// Create form for this square
			let vertices = generate_square(pos, size);
			let form = p.form(&vertices).create();

			// Create color binding with random color
			let color = rand_vec3();
			let color_binding = p.bind_const_vec3(color);

			// Create shape
			let shape = p
				.shape(form, shade)
				.with_bindings(map! {
					0 => color_binding
				})
				.create();

			shape_pool.push(shape);

			println!(
				"Created shape {}: pos=({:.2}, {:.2}), size={:.2}, color=({:.2}, {:.2}, {:.2})",
				i, pos.x, pos.y, size, color.x, color.y, color.z
			);
		}

		let is_vertical = p.bind_const_u32(1);
		let is_horizontal = p.bind_const_u32(0);

		// Create two intermediate layers with distinct random shapes
		let layer1 = p
			.layer()
			.with_bindings(vec![(1, is_vertical)])
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.with_multisampling()
			.create();

		let layer2 = p
			.layer()
			.with_bindings(vec![(1, is_horizontal)])
			.with_clear_color(wgpu::Color::TRANSPARENT)
			.with_multisampling()
			.create();

		// Pick distinct sets of shapes for each layer
		let shapes1 = pick_random_shapes(&shape_pool);
		let shapes2 = pick_random_shapes(&shape_pool);

		layer1.set_shapes(p, shapes1.clone());
		layer2.set_shapes(p, shapes2.clone());

		println!("\nLayer 1: {} shapes", shapes1.len());
		println!("Layer 2: {} shapes", shapes2.len());

		// Create effect shader for rendering textures
		let effect_shade = p
			.shade_effect()
			.with_bindings([BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(effect_shade, p, "./shader/effect_fragment.spv");

		// Create a default sampler
		let sampler = p.sampler_linear();

		// Create effect with two instances for the two layers
		let effect = p
			.effect(effect_shade)
			.with_instances(vec![
				InstanceBinding {
					layers: vec![(0, layer1.binding())],
					..default()
				},
				InstanceBinding {
					layers: vec![(0, layer2.binding())],
					..default()
				},
			])
			.with_blend_state(wgpu::BlendState::ALPHA_BLENDING)
			.create();

		// Create the canvas layer with the effect instances
		let canvas = p
			.layer()
			.with_effect(effect)
			.with_bindings(vec![(0, sampler.binding())])
			.with_clear_color(wgpu::Color {
				r: 0.1,
				g: 0.1,
				b: 0.15,
				a: 1.0,
			})
			.with_multisampling()
			.create();

		Self {
			canvas,
			layer1,
			layer2,
			shape_pool,
			timer: 0.0,
		}
	}

	fn resize(&mut self, _p: &mut Painter, _width: u32, _height: u32) {}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.timer += tpf;

		// Every second, pick a random selection of shapes from the pool for both layers
		if self.timer >= 1.0 {
			self.timer -= 1.0;

			// Randomly select shapes from the pool for each layer
			let shapes1 = pick_random_shapes(&self.shape_pool);
			let shapes2 = pick_random_shapes(&self.shape_pool);

			// Update both layers with new selections
			self.layer1.set_shapes(p, shapes1.clone());
			self.layer2.set_shapes(p, shapes2.clone());

			println!(
				"Frame update: layer1={} shapes, layer2={} shapes",
				shapes1.len(),
				shapes2.len()
			);
		}

		// Request continuous rendering
		p.request_next_frame();

		// render
		p.paint(self.layer1);
		p.paint(self.layer2);
		p.paint_and_show(self.canvas);
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
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
