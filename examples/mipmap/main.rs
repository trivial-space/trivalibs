use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	time: f32,

	u_time: UniformBuffer<f32>,
	u_size: UniformBuffer<UVec2>,
	u_mip_levels: UniformBuffer<f32>,

	image: Layer,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let image_shade = p
			.shade_effect()
			.with_uniforms(&[UNIFORM_BUFFER_FRAG, UNIFORM_BUFFER_FRAG])
			.create();
		load_fragment_shader!(image_shade, p, "./shader/image.spv");

		let u_time = p.uniform_f32();
		let u_size = p.uniform_uvec2();
		let u_mip_levels = p.uniform_f32();

		let image_effect = p
			.effect(image_shade)
			.with_uniforms(map! {
				0 => u_size.uniform(),
				1 => u_time.uniform()
			})
			.create();

		let image = p.layer().with_effect(image_effect).with_mips().create();

		let sample_shade = p
			.shade_effect()
			.with_uniforms(&[
				UNIFORM_BUFFER_FRAG,
				UNIFORM_BUFFER_FRAG,
				UNIFORM_SAMPLER_FRAG,
			])
			.with_effect_layer()
			.create();
		load_fragment_shader!(sample_shade, p, "./shader/mip_sampling.spv");

		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_mipmap_filter(wgpu::FilterMode::Linear)
			.create();

		let sample_effect = p
			.effect(sample_shade)
			.with_uniforms(map! {
				0 => u_time.uniform(),
				1 => u_mip_levels.uniform(),
				2 => sampler.uniform()
			})
			.with_effect_layers(map! { 1 => image })
			.create();

		let canvas = p.layer().with_effect(sample_effect).create();

		Self {
			time: 0.0,

			u_time,
			u_size,
			u_mip_levels,

			canvas,
			image,
		}
	}

	fn resize(&mut self, p: &mut Painter, width: u32, height: u32) {
		self.u_size.update(p, uvec2(width, height));

		let mips = self.image.get_mip_levels_count(p);
		println!("mip levels: {}", mips);
		self.u_mip_levels.update(p, mips as f32);

		let canvas_mips = self.canvas.get_mip_levels_count(p);
		println!("canvas mip levels: {}", canvas_mips);
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
			keep_window_dimensions: true,
			..default()
		})
		.start();
}
