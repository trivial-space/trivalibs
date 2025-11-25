use trivalibs::{
	map,
	painter::{
		prelude::*,
		winit::event::{ElementState, MouseButton, WindowEvent},
	},
	prelude::*,
};

#[derive(Copy, Clone)]
struct Canvas {
	layer: Layer,
	animated: bool,
}

struct App {
	time: f32,
	u_size: BindingBuffer<UVec2>,
	u_time: BindingBuffer<f32>,

	canvases: Vec<Canvas>,
	current_canvas: usize,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let u_size = p.bind_uvec2();
		let u_time = p.bind_f32();

		let shade_canvas = |p: &mut Painter, animated: bool| {
			let s = p
				.shade_effect()
				.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
				.create();

			let e = p.effect(s).create();
			let layer = p
				.layer()
				.with_effect(e)
				.with_bindings(map! {
					0 => u_size.binding(),
					1 => u_time.binding()
				})
				.create();

			(s, Canvas { layer, animated })
		};

		let (s, simplex_2d_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/simplex_2d_shader.spv");

		let (s, simplex_3d_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/simplex_3d_shader.spv");

		let (s, simplex_4d_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/simplex_4d_shader.spv");

		let (s, tiling_simplex_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/tiling_simplex_shader.spv");

		let (s, tiling_noise_2d_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/tiling_noise_2d_shader.spv");

		let (s, tiling_noise_3d_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/tiling_noise_3d_shader.spv");

		let (s, hash_test) = shade_canvas(p, true);
		load_fragment_shader!(s, p, "./shader/hash_shader.spv");

		// return App

		Self {
			time: 0.0,
			u_size,
			u_time,

			canvases: vec![
				hash_test,
				simplex_2d_test,
				simplex_3d_test,
				simplex_4d_test,
				tiling_simplex_test,
				tiling_noise_2d_test,
				tiling_noise_3d_test,
			],
			current_canvas: 0,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));
	}

	fn render(&self, p: &mut Painter)  {
		let c = &self.canvases[self.current_canvas];
		if c.animated {
			p.request_next_frame();
		}
		p.paint_and_show(c.layer)
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);
	}

	fn event(&mut self, e: Event<()>, p: &mut Painter) {
		match e {
			Event::ShaderReloadEvent => {
				p.request_next_frame();
			}
			Event::WindowEvent(WindowEvent::MouseInput { state, button, .. }) => {
				if state == ElementState::Released {
					p.request_next_frame();
					match button {
						MouseButton::Left => {
							self.current_canvas = (self.current_canvas + 1) % self.canvases.len();
						}
						_ => {
							self.current_canvas = (self.current_canvas + self.canvases.len() - 1)
								% self.canvases.len();
						}
					}
				}
			}
			_ => {}
		}
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
