use notify::Watcher;
use std::{sync::Arc, time::Instant};
use wgpu::SurfaceError;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	keyboard::{KeyCode, PhysicalKey},
	window::{Window, WindowId},
};

pub use wgpu;
pub use winit;

pub mod painter;
pub use painter::Painter;
pub mod effect;
pub mod form;
pub mod layer;
pub mod shade;
pub mod shaders;
pub mod sketch;
pub mod texture;
pub mod uniform;
pub use uniform::UniformType;

pub trait CanvasApp<RenderState, UserEvent> {
	fn init(&self, painter: &mut Painter) -> RenderState;
	fn resize(&mut self, painter: &mut Painter, render_state: &mut RenderState);
	fn update(&mut self, painter: &mut Painter, render_state: &mut RenderState, tpf: f32);
	fn render(&self, painter: &mut Painter, render_state: &RenderState)
		-> Result<(), SurfaceError>;
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
	ReloadShaders(String),
}

pub struct CanvasAppRunner<RenderState, UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<RenderState, UserEvent>,
{
	state: WindowState,
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
	app: App,
	render_state: Option<RenderState>,
	is_running: bool,
	is_resizing: bool,
	now: Instant,
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

pub struct CanvasAppStarter<RenderState, UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<RenderState, UserEvent>,
{
	app: CanvasAppRunner<RenderState, UserEvent, App>,
	event_loop: EventLoop<CustomEvent<UserEvent>>,
}

impl<RenderState, UserEvent, App> CanvasAppStarter<RenderState, UserEvent, App>
where
	UserEvent: std::marker::Send,
	App: CanvasApp<RenderState, UserEvent> + std::marker::Send + 'static,
{
	pub fn start(self) {
		let event_loop = self.event_loop;
		let mut app = self.app;

		#[cfg(debug_assertions)]
		let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();

		#[cfg(debug_assertions)]
		let mut watcher = notify::recommended_watcher(tx).unwrap();

		#[cfg(debug_assertions)]
		let path = std::env::current_dir().unwrap();

		#[cfg(debug_assertions)]
		println!("Watching: {:?}", path);

		#[cfg(debug_assertions)]
		watcher
			.watch(&path, notify::RecursiveMode::Recursive)
			.unwrap();

		#[cfg(debug_assertions)]
		let proxy = app.event_loop_proxy.clone();

		#[cfg(debug_assertions)]
		std::thread::spawn(move || {
			// Block forever, printing out events as they come in
			for res in rx {
				match res {
					Ok(event) => {
						if event.kind.is_modify() {
							event.paths.iter().for_each(|path| {
								if let Some(ext) = path.extension() {
									if ext != "spv" {
										return;
									}
									proxy
										.send_event(CustomEvent::ReloadShaders(
											path.display().to_string(),
										))
										.unwrap_or_else(|_| {
											panic!("Failed to send shader reload event");
										});
								}
							});
						}
					}
					Err(e) => println!("watch error: {:?}", e),
				}
			}
		});

		let _ = event_loop.run_app(&mut app);
	}

	pub fn get_handle(&self) -> CanvasHandle<UserEvent> {
		CanvasHandle {
			event_loop_proxy: self.app.event_loop_proxy.clone(),
		}
	}
}

pub fn create_canvas_app<
	RenderState,
	UserEvent,
	App: CanvasApp<RenderState, UserEvent> + 'static,
>(
	app: App,
) -> CanvasAppStarter<RenderState, UserEvent, App> {
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
		render_state: None,
		event_loop_proxy,
		app,
		is_running: true,
		is_resizing: false,
		now: Instant::now(),
	};

	return CanvasAppStarter {
		app: runner,
		event_loop,
	};
}

impl<RenderState, UserEvent, App> ApplicationHandler<CustomEvent<UserEvent>>
	for CanvasAppRunner<RenderState, UserEvent, App>
where
	App: CanvasApp<RenderState, UserEvent>,
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
				self.render_state = Some(self.app.init(&mut painter));
				self.app
					.resize(&mut painter, self.render_state.as_mut().unwrap());
				painter.request_next_frame();
				self.state = WindowState::Initialized(painter);
			}
			CustomEvent::UserEvent(user_event) => {
				if let WindowState::Initialized(painter) = &self.state {
					self.app.user_event(user_event, painter);
				}
			}
			CustomEvent::ReloadShaders(path) => {
				#[cfg(debug_assertions)]
				{
					if let WindowState::Initialized(painter) = &mut self.state {
						painter.reload_shader(path);
					}
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
						self.app
							.resize(painter, self.render_state.as_mut().unwrap());
						// On macos the window needs to be redrawn manually after resizing
						painter.request_next_frame();
						self.is_resizing = true;
					}

					WindowEvent::RedrawRequested => {
						if self.is_running || self.is_resizing {
							let elapsed = self.now.elapsed().as_secs_f32();
							self.now = Instant::now();

							let elapsed = if self.is_running { elapsed } else { 0.0 };

							let render_state = &mut self.render_state.as_mut().unwrap();

							self.app.update(painter, render_state, elapsed);

							match self.app.render(painter, render_state) {
								Ok(_) => {}
								// Reconfigure the surface if it's lost or outdated
								Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
									painter.resize(PhysicalSize {
										width: painter.config.width,
										height: painter.config.height,
									});
									self.app.resize(painter, render_state);
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

							self.is_resizing = false;
						}
					}

					WindowEvent::CloseRequested => event_loop.exit(),

					WindowEvent::KeyboardInput {
						event:
							KeyEvent {
								state: ElementState::Released,
								physical_key: PhysicalKey::Code(KeyCode::Space),
								..
							},
						..
					} => {
						self.is_running = !self.is_running;
						if self.is_running {
							self.now = Instant::now();
							painter.request_next_frame();
						}
					}

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
