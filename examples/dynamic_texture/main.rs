use trivalibs::{painter::prelude::*, prelude::*};

struct App {
	red_layer: Layer,
	blue_layer: Layer,
	display_layer: Layer,
	time: f32,
	current_is_red: bool,
	last_toggle_time: f32,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		// Create color shader for red/blue layers
		let color_shader = p
			.shade_effect()
			.with_bindings([BINDING_BUFFER_FRAG])
			.create();
		load_fragment_shader!(color_shader, p, "./shader/col_fs.spv");

		// Create texture shader for display layer
		let texture_shader = p
			.shade_effect()
			.with_bindings([BINDING_SAMPLER_FRAG])
			.with_layers([BINDING_LAYER_FRAG])
			.create();
		load_fragment_shader!(texture_shader, p, "./shader/tex_fs.spv");

		// Create color uniform bindings
		let red_color = p.bind_const_vec4(vec4(1.0, 0.0, 0.0, 1.0));
		let blue_color = p.bind_const_vec4(vec4(0.0, 0.0, 1.0, 1.0));

		// Create red layer - renders solid red color
		let red_layer = p
			.single_effect_layer(color_shader)
			.with_bindings(vec![(0, red_color)])
			.with_size(4, 4)
			.create();

		p.init_and_paint(red_layer);

		// Create blue layer - renders solid blue color
		let blue_layer = p
			.single_effect_layer(color_shader)
			.with_bindings(vec![(0, blue_color)])
			.with_size(4, 4)
			.create();

		p.init_and_paint(blue_layer);

		// Create sampler for texture sampling
		let sampler = p.sampler_nearest();

		// Create display layer - samples from red/blue layers
		// Initially bound to red layer
		let display_layer = p
			.single_effect_layer(texture_shader)
			.with_bindings(vec![(0, sampler.binding())])
			.with_layers(vec![(0, red_layer.binding())])
			.create();

		Self {
			red_layer,
			blue_layer,
			display_layer,
			time: 0.0,
			current_is_red: true,
			last_toggle_time: 0.0,
		}
	}

	fn resize(&mut self, _p: &mut Painter, _width: u32, _height: u32) {}

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;

		// Toggle every second
		if self.time - self.last_toggle_time >= 1.0 {
			self.last_toggle_time = self.time;
			self.current_is_red = !self.current_is_red;

			// This is the key part - dynamically switch texture binding!
			if self.current_is_red {
				println!("Switching to RED ({:.2}s)", self.time);
				self.display_layer
					.set_layer_binding(p, 0, self.red_layer.binding());
			} else {
				println!("Switching to BLUE ({:.2}s)", self.time);
				self.display_layer
					.set_layer_binding(p, 0, self.blue_layer.binding());
			}
		}

		// Render and show display layer (which samples from red or blue)
		p.paint_and_show(self.display_layer);

		p.request_next_frame();
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create()
		.config(AppConfig {
			show_fps: true,
			..default()
		})
		.start();
}
