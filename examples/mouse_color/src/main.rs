use trivalibs::painter::{wgpu, winit::event::WindowEvent, CanvasApp, Event, Painter};

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

	fn render(&self, p: &mut Painter) -> Result<(), wgpu::SurfaceError> {
		let frame = p.surface.get_current_texture()?;

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
				})],
				depth_stencil_attachment: None,
				timestamp_writes: None,
				occlusion_query_set: None,
			});
		}

		p.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn event(&mut self, event: Event<()>, p: &mut Painter) {
		match event {
			Event::WindowEvent(WindowEvent::CursorMoved {
				device_id: _,
				position,
			}) => {
				let size = p.canvas_size();
				self.color = wgpu::Color {
					r: position.x / size.width as f64,
					g: position.y / size.height as f64,
					b: 0.3,
					a: 1.0,
				};
				p.request_next_frame();
			}
			_ => {}
		}
	}

	fn resize(&mut self, _p: &mut Painter, _w: u32, _h: u32) {}
	fn update(&mut self, _p: &mut Painter, _tpf: f32) {}
}

pub fn main() {
	App::create().start();
}
