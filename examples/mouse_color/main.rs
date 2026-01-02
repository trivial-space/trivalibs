use trivalibs::painter::{
	Painter,
	app::{CanvasApp, Event},
	wgpu,
};

struct App {
	color: wgpu::Color,
}

impl CanvasApp<()> for App {
	fn init(_p: &mut Painter) -> Self {
		Self {
			color: wgpu::Color {
				r: 0.3,
				g: 0.3,
				b: 0.3,
				a: 1.0,
			},
		}
	}

	fn frame(&mut self, p: &mut Painter, _tpf: f32) {
		let frame = p.surface.get_current_texture().unwrap();

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = p
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
		{
			encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
		}

		p.queue.submit(Some(encoder.finish()));
		frame.present();
	}

	fn event(&mut self, event: Event<()>, p: &mut Painter) {
		match event {
			Event::PointerMove { x, y, .. } => {
				let size = p.canvas_size();
				self.color = wgpu::Color {
					r: x / size.width as f64,
					g: y / size.height as f64,
					b: 0.3,
					a: 1.0,
				};
				p.request_next_frame();
			}
			_ => {}
		}
	}
}

pub fn main() {
	App::create().start();
}
