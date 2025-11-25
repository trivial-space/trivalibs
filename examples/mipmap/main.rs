use trivalibs::{map, painter::prelude::*, prelude::*};

struct App {
	time: f32,

	u_time: BindingBuffer<f32>,
	u_size: BindingBuffer<UVec2>,
	u_mip_levels: BindingBuffer<f32>,

	image: Layer,
	canvas: Layer,
}

impl CanvasApp<()> for App {
	fn init(p: &mut Painter) -> Self {
		let image_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_BUFFER_FRAG])
			.create();
		load_fragment_shader!(image_shade, p, "./shader/image.spv");

		let u_time = p.bind_f32();
		let u_size = p.bind_uvec2();
		let u_mip_levels = p.bind_f32();

		let image_effect = p
			.effect(image_shade)
			.with_bindings(map! {
				0 => u_size.binding(),
				1 => u_time.binding()
			})
			.create();

		let image = p.layer().with_effect(image_effect).with_mips().create();

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

		let sampler = p
			.sampler()
			.with_filters(wgpu::FilterMode::Linear)
			.with_mipmap_filter(wgpu::FilterMode::Linear)
			.create();

		let sample_effect = p
			.effect(sample_shade)
			.with_bindings(map! {
				0 => u_time.binding(),
				1 => u_mip_levels.binding(),
				2 => sampler.binding()
			})
			.with_layers(map! { 1 => image.binding() })
			.create();

		let effect_shade = p
			.shade_effect()
			.with_bindings(&[BINDING_BUFFER_FRAG, BINDING_SAMPLER_FRAG])
			.with_layer()
			.create();
		load_fragment_shader!(effect_shade, p, "./shader/wave_effect.spv");

		let effect = p
			.effect(effect_shade)
			.with_bindings(map! {
				0 => u_time.binding(),
				1 => sampler.binding()
			})
			.create();

		let canvas = p.layer().with_effects(vec![sample_effect, effect]).create();

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

	fn frame(&mut self, p: &mut Painter, tpf: f32) {
		self.time += tpf;
		self.u_time.update(p, self.time);

		p.paint(self.image);
		p.paint_and_show(self.canvas);

		p.request_next_frame();
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
