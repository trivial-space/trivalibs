use std::sync::Arc;
use wgpu::SurfaceError;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{DeviceEvent, DeviceId, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	window::{Window, WindowId},
};

pub mod painter;
pub use painter::Painter;

pub trait CanvasApp<UserEvent> {
	fn init(&mut self, painter: &mut Painter);
	fn render(&self, painter: &Painter) -> Result<(), SurfaceError>;
	fn window_event(&mut self, event: WindowEvent, painter: &Painter);
	fn device_event(&mut self, event: DeviceEvent, painter: &Painter);
	fn user_event(&mut self, event: UserEvent, painter: &Painter);
}

enum WindowState {
	Uninitialized,
	Initializing,
	Initialized(Painter),
}

pub enum CustomEvent<UserEvent> {
	StateInitializationEvent(Painter),
	UserEvent(UserEvent),
}

pub struct CanvasAppRunner<UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent>,
{
	state: WindowState,
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
	app: App,
}

pub struct CanvasHandle<UserEvent>
where
	UserEvent: 'static,
{
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
}

impl<UserEvent> CanvasHandle<UserEvent> {
	pub fn send_event(
		&self,
		event: UserEvent,
	) -> Result<(), winit::event_loop::EventLoopClosed<CustomEvent<UserEvent>>> {
		self.event_loop_proxy
			.send_event(CustomEvent::UserEvent(event))
	}
}

pub struct CanvasAppStarter<UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent>,
{
	app: CanvasAppRunner<UserEvent, App>,
	event_loop: EventLoop<CustomEvent<UserEvent>>,
}

impl<UserEvent, App> CanvasAppStarter<UserEvent, App>
where
	UserEvent: std::marker::Send,
	App: CanvasApp<UserEvent> + std::marker::Send + 'static,
{
	pub fn start(self) {
		let event_loop = self.event_loop;
		let mut app = self.app;
		let _ = event_loop.run_app(&mut app);
	}

	pub fn get_handle(&self) -> CanvasHandle<UserEvent> {
		CanvasHandle {
			event_loop_proxy: self.app.event_loop_proxy.clone(),
		}
	}
}

pub fn create_canvas_app<UserEvent, App: CanvasApp<UserEvent> + 'static>(
	app: App,
) -> CanvasAppStarter<UserEvent, App> {
	#[cfg(not(target_arch = "wasm32"))]
	env_logger::init();

	#[cfg(target_arch = "wasm32")]
	{
		std::panic::set_hook(Box::new(console_error_panic_hook::hook));
		console_log::init().expect("could not initialize logger");
	}

	let event_loop = EventLoop::<CustomEvent<UserEvent>>::with_user_event()
		.build()
		.unwrap();

	let event_loop_proxy = event_loop.create_proxy();

	let runner = CanvasAppRunner {
		state: WindowState::Uninitialized,
		event_loop_proxy,
		app,
	};

	return CanvasAppStarter {
		app: runner,
		event_loop,
	};
}

impl<UserEvent, App> ApplicationHandler<CustomEvent<UserEvent>> for CanvasAppRunner<UserEvent, App>
where
	App: CanvasApp<UserEvent>,
{
	// This is a common indicator that you can create a window.
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		match self.state {
			WindowState::Initializing | WindowState::Initialized(_) => return,
			WindowState::Uninitialized => {
				self.state = WindowState::Initializing;
			}
		}
		let window = event_loop
			.create_window(Window::default_attributes())
			.unwrap();

		let window = Arc::new(window);

		#[cfg(target_arch = "wasm32")]
		{
			// TODO: initialize canvas
			// web_sys::window()
			// 	.and_then(|win| win.document())
			// 	.and_then(|doc| {
			// 		let dst = doc.get_element_by_id("kloenk-wasm")?;
			// 		let canvas = window.canvas()?;
			// 		canvas
			// 			.set_attribute("tabindex", "0")
			// 			.expect("failed to set tabindex");
			// 		dst.append_child(&canvas).ok()?;
			// 		canvas.focus().expect("Unable to focus on canvas");
			// 		Some(())
			// 	})
			// 	.expect("Couldn't append canvas to document body.");
		}

		let renderer_future = Painter::new(window);

		#[cfg(target_arch = "wasm32")]
		{
			let event_loop_proxy = self.event_loop_proxy.clone();
			spawn_local(async move {
				let painter = renderer_future.await;

				event_loop_proxy
					.send_event(CustomEvent(painter))
					.unwrap_or_else(|_| {
						panic!("Failed to send initialization event");
					});
			});
		}

		#[cfg(not(target_arch = "wasm32"))]
		{
			let painter = pollster::block_on(renderer_future);

			self.event_loop_proxy
				.send_event(CustomEvent::StateInitializationEvent(painter))
				.unwrap_or_else(|_| {
					panic!("Failed to send initialization event");
				});
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: CustomEvent<UserEvent>) {
		match event {
			CustomEvent::StateInitializationEvent(mut painter) => {
				painter.redraw();
				self.app.init(&mut painter);
				self.state = WindowState::Initialized(painter);
			}
			CustomEvent::UserEvent(user_event) => {
				if let WindowState::Initialized(painter) = &self.state {
					self.app.user_event(user_event, painter);
				}
			}
		}
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: WindowId,
		event: WindowEvent,
	) {
		match &mut self.state {
			WindowState::Initialized(painter) => {
				match event {
					WindowEvent::Resized(new_size) => {
						// Reconfigure the surface with the new size
						painter.resize(new_size);
						// On macos the window needs to be redrawn manually after resizing
						painter.redraw();
					}

					WindowEvent::RedrawRequested => {
						match self.app.render(painter) {
							Ok(_) => {}
							// Reconfigure the surface if it's lost or outdated
							Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
								painter.resize(PhysicalSize {
									width: painter.config.width,
									height: painter.config.height,
								});
							}
							// The system is out of memory, we should probably quit
							Err(wgpu::SurfaceError::OutOfMemory) => {
								log::error!("OutOfMemory");
								event_loop.exit();
							}

							// This happens when the a frame takes too long to present
							Err(wgpu::SurfaceError::Timeout) => {
								log::warn!("Surface timeout")
							}
						}
					}

					WindowEvent::CloseRequested => event_loop.exit(),
					rest => {
						self.app.window_event(rest, painter);
					}
				};
			}
			_ => {}
		}
	}

	fn device_event(
		&mut self,
		_event_loop: &ActiveEventLoop,
		_device_id: DeviceId,
		event: DeviceEvent,
	) {
		if let WindowState::Initialized(painter) = &mut self.state {
			self.app.device_event(event, painter);
		}
	}
}
