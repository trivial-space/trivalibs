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

struct App {
	canvas: Layer,
	shape_pool: Vec<Shape>,
	timer: f32,
}

const POOL_SIZE: usize = 20;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		// Create a single shade that all shapes will share
		let shade = p
			.shade(&[Float32x2])
			.with_bindings(&[BINDING_BUFFER_FRAG])
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
			let color = p.bind_const_vec3(rand_vec3());

			// Create shape
			let shape = p
				.shape(form, shade)
				.with_bindings(map! {
					0 => color
				})
				.create();

			shape_pool.push(shape);

			println!(
				"Created shape {}: pos=({:.2}, {:.2}), size={:.2}, color=({:.2}, {:.2}, {:.2})",
				i,
				pos.x,
				pos.y,
				size,
				rand_range(0.0, 1.0),
				rand_range(0.0, 1.0),
				rand_range(0.0, 1.0)
			);
		}

		// Create the canvas layer with no shapes initially
		let canvas = p
			.layer()
			.with_clear_color(wgpu::Color {
				r: 0.1,
				g: 0.1,
				b: 0.15,
				a: 1.0,
			})
			.with_multisampling()
			.create();

		// Start with a random selection of shapes
		let initial_count = rand_range(1.0, POOL_SIZE as f32) as usize;
		let mut initial_shapes = Vec::new();
		for _ in 0..initial_count {
			let idx = rand_range(0.0, POOL_SIZE as f32) as usize % POOL_SIZE;
			initial_shapes.push(shape_pool[idx]);
		}
		canvas.set_shapes(p, initial_shapes.clone());

		println!("\nInitial render: {} shapes", initial_shapes.len());

		Self {
			canvas,
			shape_pool,
			timer: 0.0,
		}
	}

	fn resize(&mut self, _p: &mut Painter, _width: u32, _height: u32) {}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.timer += tpf;

		// Every second, pick a random selection of shapes from the pool
		if self.timer >= 1.0 {
			self.timer -= 1.0;

			// Pick random count of shapes to display
			let count = rand_range(0.0, POOL_SIZE as f32) as usize;

			// Randomly select shapes from the pool (can include duplicates)
			let mut selected_shapes = Vec::new();
			for _ in 0..count {
				let idx = rand_range(0.0, POOL_SIZE as f32) as usize % POOL_SIZE;
				selected_shapes.push(self.shape_pool[idx]);
			}

			// Update the layer with the new selection
			self.canvas.set_shapes(p, selected_shapes.clone());

			println!("Frame update: {} shapes selected from pool", count);
		}

		// Request continuous rendering
		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint_and_show(self.canvas)
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
