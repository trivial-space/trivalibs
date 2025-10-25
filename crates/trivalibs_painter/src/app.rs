use crate::layer::Layer;
#[cfg(not(target_arch = "wasm32"))]
use crate::window_dimensions::WindowDimensions;
use crate::{Painter, painter::PainterConfig};
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use notify::Watcher;
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use std::collections::BTreeMap;
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
use web_time::Instant;
use wgpu::SurfaceError;
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	keyboard::{KeyCode, PhysicalKey},
	window::{Window, WindowId},
};

#[derive(Debug)]
pub enum Event<UserEvent> {
	WindowEvent(WindowEvent),
	DeviceEvent(DeviceEvent),
	UserEvent(UserEvent),
	ShaderReloadEvent,
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
			let _ = console_log::init(); // Ignore error if already initialized
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
			frame_count: 0,
			frame_time: 0.0,
			now: Instant::now(),
			config: AppConfig::default(),
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
	frame_count: u32,
	frame_time: f32,
	now: Instant,
	config: AppConfig,
}

impl<UserEvent, App> CanvasAppRunner<UserEvent, App>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent>,
{
	pub fn pause(&mut self) {
		self.is_running = false;
	}

	pub fn play(&mut self) {
		self.is_running = true;
		self.now = Instant::now();
		if let WindowState::Initialized(painter, _) = &mut self.state {
			painter.request_next_frame();
		}
	}
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

#[derive(Debug)]
pub struct AppConfig {
	pub show_fps: bool,
	pub use_vsync: bool,
	pub remember_window_dimensions: bool,
	pub features: Option<wgpu::Features>,
	#[cfg(target_arch = "wasm32")]
	pub canvas: Option<web_sys::HtmlCanvasElement>,
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			show_fps: false,
			use_vsync: true,
			remember_window_dimensions: false,
			features: None,
			#[cfg(target_arch = "wasm32")]
			canvas: None,
		}
	}
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
		self.runner.config = config;
		self
	}

	pub fn start(self) {
		let event_loop = self.event_loop;
		let mut runner = self.runner;

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		let (tx, rx) = std::sync::mpsc::channel::<notify::Result<notify::Event>>();

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		let mut watcher = notify::recommended_watcher(tx).unwrap();

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		let path = std::env::current_dir().unwrap();

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		println!("Watching: {:?}", path);

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		watcher
			.watch(&path, notify::RecursiveMode::Recursive)
			.unwrap();

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		let proxy = runner.event_loop_proxy.clone();

		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		std::thread::spawn(move || {
			let mut current_shaders = BTreeMap::new();
			// Block forever, printing out events as they come in
			for res in rx {
				match res {
					Ok(event) => {
						if event.kind.is_modify() {
							let current_time = std::time::SystemTime::now();

							event.paths.iter().for_each(|path| {
								if let Some(ext) = path.extension() {
									if ext != "spv" {
										return;
									}

									if let Some(last_event_time) = current_shaders.get(path) {
										if current_time
											.duration_since(*last_event_time)
											.unwrap()
											.as_millis() < 500
										{
											return;
										}
									}

									proxy
										.send_event(CustomEvent::ReloadShaders(
											path.display().to_string(),
										))
										.unwrap_or_else(|_| {
											panic!("Failed to send shader reload event");
										});

									current_shaders.insert(path.clone(), current_time);
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

				#[cfg(not(target_arch = "wasm32"))]
				let mut window_attributes = Window::default_attributes();
				#[cfg(target_arch = "wasm32")]
				let window_attributes = Window::default_attributes();

				// Load and apply saved window state
				#[cfg(not(target_arch = "wasm32"))]
				if self.config.remember_window_dimensions {
					if let Some(state) = WindowDimensions::load() {
						window_attributes = window_attributes
							.with_inner_size(PhysicalSize::new(state.size.0, state.size.1));
						window_attributes = window_attributes.with_position(PhysicalPosition::new(
							state.position.0,
							state.position.1,
						));
					}
				} else {
					let _ = WindowDimensions::cleanup();
				}

				#[cfg(target_arch = "wasm32")]
				let window = {
					use winit::platform::web::WindowAttributesExtWebSys;

					if let Some(canvas) = &self.config.canvas {
						// Use the provided canvas
						let window = event_loop
							.create_window(window_attributes.with_canvas(Some(canvas.clone())))
							.unwrap();
						let window = Arc::new(window);

						// Set canvas attributes even for provided canvas
						canvas
							.set_attribute("tabindex", "0")
							.expect("failed to set tabindex");
						canvas.focus().expect("Unable to focus on canvas");

						window
					} else {
						// Create a new canvas as before
						let window = event_loop.create_window(window_attributes).unwrap();
						let window = Arc::new(window);

						use winit::platform::web::WindowExtWebSys;

						web_sys::window()
							.and_then(|win| win.document())
							.and_then(|doc| {
								let body = doc.body()?;
								let canvas = window.canvas().expect("Failed to get canvas");
								canvas
									.set_attribute("tabindex", "0")
									.expect("failed to set tabindex");
								// Set canvas size to fill the window
								canvas.style().set_property("width", "100%").ok();
								canvas.style().set_property("height", "100%").ok();
								canvas.style().set_property("display", "block").ok();
								body.append_child(&canvas).ok()?;
								canvas.focus().expect("Unable to focus on canvas");
								Some(())
							})
							.expect("Couldn't append canvas to document body.");

						window
					}
				};

				#[cfg(not(target_arch = "wasm32"))]
				let window = {
					let window = event_loop.create_window(window_attributes).unwrap();
					Arc::new(window)
				};

				let renderer_future = Painter::new(
					window,
					PainterConfig {
						use_vsync: self.config.use_vsync,
						features: self.config.features,
					},
				);

				#[cfg(target_arch = "wasm32")]
				{
					let event_loop_proxy = self.event_loop_proxy.clone();
					spawn_local(async move {
						let painter = renderer_future.await;

						event_loop_proxy
							.send_event(CustomEvent::StateInitializationEvent(painter))
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

				for i in 0..painter.layers.len() {
					Layer(i).init_layer_gpu_pipelines(&mut painter);
				}

				let size = painter.canvas_size();
				app.resize(&mut painter, size.width, size.height);
				painter.request_next_frame();
				self.state = WindowState::Initialized(painter, app);
			}
			CustomEvent::UserEvent(user_event) => {
				if let WindowState::Initialized(painter, app) = &mut self.state {
					if self.is_running {
						app.event(Event::UserEvent(user_event), painter);
					}
				}
			}
			CustomEvent::ReloadShaders(path) => {
				#[cfg(target_arch = "wasm32")]
				let _ = path; // Use the path if needed
				#[cfg(not(target_arch = "wasm32"))]
				#[cfg(debug_assertions)]
				{
					if let WindowState::Initialized(painter, app) = &mut self.state {
						painter.reload_shader(path);
						app.event(Event::ShaderReloadEvent, painter);
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

						#[cfg(not(target_arch = "wasm32"))]
						{
							let window = painter.window();
							if self.config.remember_window_dimensions {
								let dim = WindowDimensions::from_window(
									new_size,
									window.outer_position().unwrap_or_default(),
								);
								let _ = dim.save();
							}
						}
					}

					#[cfg(not(target_arch = "wasm32"))]
					WindowEvent::Moved(new_position) => {
						let window = painter.window();
						if self.config.remember_window_dimensions {
							let dim =
								WindowDimensions::from_window(window.inner_size(), new_position);
							let _ = dim.save();
						}
					}

					WindowEvent::RedrawRequested => {
						if self.is_running || self.is_resizing {
							let elapsed = self.now.elapsed().as_secs_f32();
							self.now = Instant::now();

							let elapsed = if self.is_running { elapsed } else { 0.0 };

							if self.config.show_fps && self.is_running {
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

								Err(other) => {
									log::error!("Other error: {:?}", other);
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
						if self.is_running {
							self.pause();
						} else {
							self.play();
						}
					}

					// TODO: make this configurable
					#[cfg(not(target_arch = "wasm32"))]
					WindowEvent::KeyboardInput {
						event:
							KeyEvent {
								state: ElementState::Released,
								physical_key: PhysicalKey::Code(KeyCode::Escape),
								..
							},
						..
					} => {
						event_loop.exit();
					}

					rest => {
						if self.is_running {
							app.event(Event::WindowEvent(rest), painter);
						}
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
			if self.is_running {
				app.event(Event::DeviceEvent(event), painter);
			}
		}
	}
}
