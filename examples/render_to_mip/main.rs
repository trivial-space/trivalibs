use trivalibs::{map, painter::prelude::*, prelude::*};
use trivalibs_nostd::color::hsv2rgb;

struct App {
	time: f32,

	u_time: BindingBuffer<f32>,
	u_mip_levels: BindingBuffer<f32>,

	image: Layer,
	canvas: Layer,
}

const MIP_EFFECT_COUNT: u32 = 8;

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let image_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(image_shade, p, "./shader/image.spv");

		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Nearest)
			.with_mipmap_filter(wgpu::FilterMode::Linear)
			.create();

		let mut effects = vec![];

		for i in 1..MIP_EFFECT_COUNT {
			let color_shift = i as f32 / MIP_EFFECT_COUNT as f32;
			let color = hsv2rgb(vec3(color_shift, 1.0, 0.7));
			let color_binding = p.bind_const_vec3(color);
			let i = MIP_EFFECT_COUNT - i;
			effects.push(
				p.effect(image_shade)
					.with_bindings(map! {
						0 => color_binding,
						1 => sampler.binding(),
					})
					.with_mip_source(i)
					.with_mip_target(i - 1)
					.create(),
			);
		}

		let image = p
			.layer()
			.with_effects(effects)
			.with_mips_max(MIP_EFFECT_COUNT)
			.create();

		let u_time = p.bind_f32();
		let u_mip_levels = p.bind_f32();

		let sample_shade = p
			.shade_effect()
			.with_bindings(&[
				BINDING_BUFFER_FRAG,
				BINDING_BUFFER_FRAG,
				BINDING_SAMPLER_FRAG,
			])
			.with_layer()
			.create();
		load_fragment_shader!(sample_shade, p, "./shader/mip_sampling.spv");

		let sample_effect = p
			.effect(sample_shade)
			.with_bindings(map! {
				0 => u_time.binding(),
				1 => u_mip_levels.binding(),
				2 => sampler.binding()
			})
			.with_layers(map! { 1 => image.binding() })
			.create();

		let canvas = p.layer().with_effects(vec![sample_effect]).create();

		Self {
			time: 0.0,

			u_time,
			u_mip_levels,

			canvas,
			image,
		}
	}

	fn resize(&mut self, p: &mut Painter, _width: u32, _height: u32) {
		let mips = self.image.get_mip_levels_count(p);
		self.u_mip_levels.update(p, mips as f32);
	}

	fn update(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);

		p.request_next_frame();
	}

	fn render(&self, p: &mut Painter) -> Result<(), SurfaceError> {
		p.paint(self.image)?;
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
