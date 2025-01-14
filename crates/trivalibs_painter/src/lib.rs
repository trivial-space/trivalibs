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
pub mod binding;
pub mod effect;
pub mod form;
pub mod layer;
pub mod shade;
pub mod shaders;
pub mod shape;
pub mod texture;
pub mod uniform;

#[derive(Debug)]
pub enum Event<UserEvent> {
	WindowEvent(WindowEvent),
	DeviceEvent(DeviceEvent),
	UserEvent(UserEvent),
}

pub trait CanvasApp<UserEvent> {
	fn init(painter: &mut Painter) -> Self;
	fn resize(&mut self, painter: &mut Painter, width: u32, height: u32);
	fn update(&mut self, painter: &mut Painter, tpf: f32);
	fn render(&self, painter: &mut Painter) -> Result<(), SurfaceError>;
	fn event(&mut self, event: Event<UserEvent>, painter: &mut Painter);

	fn create() -> CanvasAppStarter<UserEvent, Self>
	where
		Self: Sized,
	{
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
			is_running: true,
			is_resizing: false,
			show_fps: false,
			frame_count: 0,
			frame_time: 0.0,
			now: Instant::now(),
		};

		CanvasAppStarter { runner, event_loop }
	}
}

enum WindowState<UserEvent, App: CanvasApp<UserEvent>> {
	Uninitialized,
	Initializing,
	Initialized(Painter, App),
	_PHANTOM(std::marker::PhantomData<UserEvent>),
}

pub enum CustomEvent<UserEvent> {
	StateInitializationEvent(Painter),
	UserEvent(UserEvent),
	ReloadShaders(String),
}

pub struct CanvasAppRunner<UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent>,
{
	state: WindowState<UserEvent, App>,
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
	is_running: bool,
	is_resizing: bool,
	show_fps: bool,
	frame_count: u32,
	frame_time: f32,
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

#[derive(Debug, Default)]
pub struct AppConfig {
	pub show_fps: bool,
}

pub struct CanvasAppStarter<UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent>,
{
	runner: CanvasAppRunner<UserEvent, App>,
	event_loop: EventLoop<CustomEvent<UserEvent>>,
}

impl<UserEvent, App> CanvasAppStarter<UserEvent, App>
where
	UserEvent: std::marker::Send,
	App: CanvasApp<UserEvent> + std::marker::Send + 'static,
{
	pub fn config(mut self, config: AppConfig) -> Self {
		self.runner.show_fps = config.show_fps;
		self
	}

	pub fn start(self) {
		let event_loop = self.event_loop;
		let mut runner = self.runner;

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
		let proxy = runner.event_loop_proxy.clone();

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

		let _ = event_loop.run_app(&mut runner);
	}

	pub fn get_handle(&self) -> CanvasHandle<UserEvent> {
		CanvasHandle {
			event_loop_proxy: self.runner.event_loop_proxy.clone(),
		}
	}
}

impl<UserEvent, App> ApplicationHandler<CustomEvent<UserEvent>> for CanvasAppRunner<UserEvent, App>
where
	App: CanvasApp<UserEvent>,
{
	// This is a common indicator that you can create a window.
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		match self.state {
			WindowState::Uninitialized => {
				self.state = WindowState::Initializing;

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
			_ => {}
		}
	}

	fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: CustomEvent<UserEvent>) {
		match event {
			CustomEvent::StateInitializationEvent(mut painter) => {
				let mut app = App::init(&mut painter);
				let size = painter.canvas_size();
				app.resize(&mut painter, size.width, size.height);
				painter.request_next_frame();
				self.state = WindowState::Initialized(painter, app);
			}
			CustomEvent::UserEvent(user_event) => {
				if let WindowState::Initialized(painter, app) = &mut self.state {
					app.event(Event::UserEvent(user_event), painter);
				}
			}
			CustomEvent::ReloadShaders(path) => {
				#[cfg(debug_assertions)]
				{
					if let WindowState::Initialized(painter, _) = &mut self.state {
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
			WindowState::Initialized(painter, app) => {
				match event {
					WindowEvent::Resized(new_size) => {
						// Reconfigure the surface with the new size
						painter.resize(new_size);
						app.resize(painter, new_size.width, new_size.height);
						// On macos the window needs to be redrawn manually after resizing
						painter.request_next_frame();
						self.is_resizing = true;
					}

					WindowEvent::RedrawRequested => {
						if self.is_running || self.is_resizing {
							let elapsed = self.now.elapsed().as_secs_f32();
							self.now = Instant::now();

							let elapsed = if self.is_running { elapsed } else { 0.0 };

							if self.show_fps && self.is_running {
								self.frame_count += 1;
								self.frame_time += elapsed;
								if self.frame_time >= 2.0 {
									// TODO: setup logger
									println!("FPS: {}", self.frame_count as f32 / self.frame_time);
									self.frame_count = 0;
									self.frame_time = 0.0;
								}
							}

							app.update(painter, elapsed);

							match app.render(painter) {
								Ok(_) => {}
								// Reconfigure the surface if it's lost or outdated
								Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
									painter.resize(PhysicalSize {
										width: painter.config.width,
										height: painter.config.height,
									});
									app.resize(
										painter,
										painter.config.width,
										painter.config.height,
									);
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
						app.event(Event::WindowEvent(rest), painter);
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
		if let WindowState::Initialized(painter, app) = &mut self.state {
			app.event(Event::DeviceEvent(event), painter);
		}
	}
}
