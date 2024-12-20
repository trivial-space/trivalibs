use trivalibs::painter::{
	create_canvas_app, wgpu, winit::event::WindowEvent, CanvasApp, Event, Painter,
};

struct App {
	color: wgpu::Color,
}

impl App {
	fn new() -> Self {
		Self {
			color: wgpu::Color {
				r: 0.3,
				g: 0.3,
				b: 0.3,
				a: 1.0,
			},
		}
	}
}

impl CanvasApp<(), ()> for App {
	fn render(&self, painter: &mut Painter, _state: &()) -> Result<(), wgpu::SurfaceError> {
		let frame = painter.surface.get_current_texture()?;

		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = painter
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

		painter.queue.submit(Some(encoder.finish()));
		frame.present();

		Ok(())
	}

	fn event(&mut self, event: Event<()>, painter: &Painter) {
		match event {
			Event::WindowEvent(WindowEvent::CursorMoved {
				device_id: _,
				position,
			}) => {
				let size = painter.canvas_size();
				self.color = wgpu::Color {
					r: position.x / size.width as f64,
					g: position.y / size.height as f64,
					b: 0.3,
					a: 1.0,
				};
				painter.request_next_frame();
			}
			_ => {}
		}
	}

	fn init(&self, _painter: &mut Painter) {}
	fn resize(&mut self, _painter: &mut Painter, _r: &mut ()) {}
	fn update(&mut self, _painter: &mut Painter, _render_state: &mut (), _tpf: f32) {}
}

pub fn main() {
	create_canvas_app(App::new()).start();
}
