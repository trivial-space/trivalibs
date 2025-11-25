use trivalibs::painter::{
	Painter,
	app::{CanvasApp, Event},
	wgpu::{self, include_spirv},
};

struct App {
	color: wgpu::Color,
	pipeline: wgpu::RenderPipeline,
}

struct UserEvent(wgpu::Color);

impl CanvasApp<UserEvent> for App {
	fn init(p: &mut Painter) -> Self {
		// Initialize the app

		let pipeline_layout = p
			.device
			.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: None,
				bind_group_layouts: &[],
				push_constant_ranges: &[],
			});

		// let capabilities = painter.surface.get_capabilities(&painter.adapter);
		// let format = capabilities.formats[0];

		// Load the shaders from disk
		let vert_shader = p
			.device
			.create_shader_module(include_spirv!("./shader/vertex.spv"));
		let frag_shader = p
			.device
			.create_shader_module(include_spirv!("./shader/fragment.spv"));

		let pipeline = p
			.device
			.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: None,
				layout: Some(&pipeline_layout),
				vertex: wgpu::VertexState {
					module: &vert_shader,
					entry_point: None,
					buffers: &[],
					compilation_options: Default::default(),
				},
				fragment: Some(wgpu::FragmentState {
					module: &frag_shader,
					entry_point: None,
					compilation_options: Default::default(),
					targets: &[Some(wgpu::ColorTargetState {
						format: p.config.format, // for direct rendering into te surface
						blend: Some(wgpu::BlendState::REPLACE),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: wgpu::PrimitiveState::default(),
				depth_stencil: None,
				multisample: wgpu::MultisampleState::default(),
				multiview: None,
				cache: None,
			});

		Self {
			color: wgpu::Color::BLUE,
			pipeline,
		}
	}

	fn render(&self, painter: &mut Painter) {
		let frame = painter.surface.get_current_texture().unwrap();

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = painter
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: None,
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
					view: &view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(self.color),
						store: wgpu::StoreOp::Store,
					},
					depth_slice: None,
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
			rpass.set_pipeline(&self.pipeline);
			rpass.draw(0..3, 0..1);
		}

		painter.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn event(&mut self, event: Event<UserEvent>, painter: &mut Painter) {
		match event {
			Event::UserEvent(event) => {
				self.color = event.0;
				painter.request_next_frame();
			}
			_ => {}
		}
	}

	fn resize(&mut self, _p: &mut Painter, _w: u32, _h: u32) {}
	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
}

pub fn main() {
	let app = App::create();
	let handle = app.get_handle();

	std::thread::spawn(move || {
		loop {
			std::thread::sleep(std::time::Duration::from_secs(2));
			let _ = handle.send_event(UserEvent(wgpu::Color::RED));
			std::thread::sleep(std::time::Duration::from_secs(2));
			let _ = handle.send_event(UserEvent(wgpu::Color::GREEN));
			std::thread::sleep(std::time::Duration::from_secs(2));
			let _ = handle.send_event(UserEvent(wgpu::Color::BLUE));
		}
	});

	app.start();
}
