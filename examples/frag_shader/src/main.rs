use trivalibs::{
	painter::{
		effect::EffectProps,
		layer::{Layer, LayerProps},
		load_fragment_shader,
		shade::ShadeEffectProps,
		uniform::UniformBuffer,
		wgpu::SurfaceError,
		CanvasApp, Event, Painter,
	},
	prelude::*,
};

struct App {
	time: f32,

	u_time: UniformBuffer<f32>,
	u_size: UniformBuffer<UVec2>,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let u_type = p.uniform_type_buffered_frag();

		let shade = p.shade_create_effect(ShadeEffectProps {
			uniform_types: &[u_type, u_type],
		});
		load_fragment_shader!(shade, p, "../shader/main.spv");

		let u_time = u_type.create_f32(p);
		let u_size = u_type.create_uvec2(p);

		let effect = p.effect_create(
			shade,
			EffectProps {
				uniforms: vec![(0, u_size.uniform), (1, u_time.uniform)],
				..default()
			},
		);

		let canvas = p.layer_create(LayerProps {
			effects: vec![effect],
			..default()
		});

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
		p.paint(self.canvas)?;
		p.show(self.canvas)
	}

	fn event(&mut self, _e: Event<()>, _p: &mut Painter) {}
}

pub fn main() {
	App::create().start();
}
