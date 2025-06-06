use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	time: f32,

	u_time: BindingBuffer<f32>,
	u_size: BindingBuffer<UVec2>,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let shade = p
			.shade_effect()
			.with_uniforms(&[BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
			.create();
		load_fragment_shader!(shade, p, "./shader/main.spv");

		let u_time = p.uniform_f32();
		let u_size = p.uniform_uvec2();

		let effect = p
			.effect(shade)
			.with_bindings(map! {
				0 => u_size.binding(),
				1 => u_time.binding()
			})
			.create();

		let canvas = p.layer().with_effect(effect).create();

		Self {
			time: 0.0,

			u_time,
			u_size,
			canvas,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);

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
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
