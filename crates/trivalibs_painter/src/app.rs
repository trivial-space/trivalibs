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
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::PhysicalPosition;
use winit::{
	application::ApplicationHandler,
	dpi::PhysicalSize,
	event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
	event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
	keyboard::PhysicalKey,
	window::{Window, WindowId},
};

// Re-export custom event types
pub use crate::events::{Event, KeyCode, PointerButton};

pub trait CanvasApp<UserEvent = (), DevState = ()> {
	fn init(painter: &mut Painter) -> Self;
	fn frame(&mut self, painter: &mut Painter, tpf: f32);
	fn resize(&mut self, _painter: &mut Painter, _width: u32, _height: u32) {}
	fn event(&mut self, _event: Event<UserEvent>, _painter: &mut Painter) {}

	/// Captures current state for persistence. Return default value to skip saving.
	fn save_dev_state(&self) -> DevState
	where
		DevState: serde::Serialize + Default,
	{
		Default::default()
	}

	/// Restores state after init(). Called only if state exists and loads successfully.
	fn load_dev_state(&mut self, _state: DevState)
	where
		DevState: for<'de> serde::Deserialize<'de>,
	{
	}

	fn create() -> CanvasAppStarter<UserEvent, Self, DevState>
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
			last_cursor: None,
		};

		CanvasAppStarter { runner, event_loop }
	}
}

enum WindowState<UserEvent, App: CanvasApp<UserEvent, DevState>, DevState> {
	Uninitialized,
	Initializing,
	Initialized(Painter, App),
	_PHANTOM(std::marker::PhantomData<(UserEvent, DevState)>),
}

pub enum CustomEvent<UserEvent> {
	StateInitializationEvent(Painter),
	UserEvent(UserEvent),
	ReloadShaders(String),
}

pub struct CanvasAppRunner<UserEvent, App, DevState = ()>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent, DevState>,
{
	state: WindowState<UserEvent, App, DevState>,
	event_loop_proxy: EventLoopProxy<CustomEvent<UserEvent>>,
	is_running: bool,
	is_resizing: bool,
	frame_count: u32,
	frame_time: f32,
	now: Instant,
	config: AppConfig,
	last_cursor: Option<(f64, f64)>,
}

impl<UserEvent, App, DevState> CanvasAppRunner<UserEvent, App, DevState>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent, DevState>,
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
	pub dev_state_key: &'static str,
	pub reload_dev_state: bool,
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
			dev_state_key: "",
			reload_dev_state: false,
		}
	}
}

pub struct CanvasAppStarter<UserEvent, App, DevState = ()>
where
	UserEvent: 'static,
	App: CanvasApp<UserEvent, DevState>,
{
	runner: CanvasAppRunner<UserEvent, App, DevState>,
	event_loop: EventLoop<CustomEvent<UserEvent>>,
}

impl<UserEvent, App, DevState> CanvasAppStarter<UserEvent, App, DevState>
where
	UserEvent: std::marker::Send,
	App: CanvasApp<UserEvent, DevState> + std::marker::Send + 'static,
	DevState: serde::Serialize + for<'de> serde::Deserialize<'de> + Default + 'static,
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

impl<UserEvent, App, DevState> CanvasAppRunner<UserEvent, App, DevState>
where
	App: CanvasApp<UserEvent, DevState>,
	DevState: serde::Serialize + for<'de> serde::Deserialize<'de> + Default,
{
	/// Saves dev state if enabled. Called before exiting the application.
	fn save_dev_state_before_exit(_config: &AppConfig, _app: &App) {
		#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
		{
			if _config.reload_dev_state && !_config.dev_state_key.is_empty() {
				let state = _app.save_dev_state();
				match serde_json::to_value(&state) {
					Ok(json) => {
						if let Err(e) =
							crate::dev_state::DevState::save(_config.dev_state_key, &json)
						{
							log::warn!("Failed to save dev state: {}", e);
						}
					}
					Err(e) => log::warn!("Failed to serialize dev state: {}", e),
				}
			}
		}
	}
}

impl<UserEvent, App, DevState> ApplicationHandler<CustomEvent<UserEvent>>
	for CanvasAppRunner<UserEvent, App, DevState>
where
	App: CanvasApp<UserEvent, DevState>,
	DevState: serde::Serialize + for<'de> serde::Deserialize<'de> + Default,
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
					if let Some(state) = WindowDimensions::load(self.config.dev_state_key) {
						window_attributes = window_attributes
							.with_inner_size(PhysicalSize::new(state.size.0, state.size.1));
						window_attributes = window_attributes.with_position(PhysicalPosition::new(
							state.position.0,
							state.position.1,
						));
					}
				} else {
					let _ = WindowDimensions::cleanup(self.config.dev_state_key);
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
					Layer(i).init_gpu_pipelines(&mut painter);
				}

				let size = painter.canvas_size();
				app.resize(&mut painter, size.width, size.height);

				// Try to restore dev state if enabled
				#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
				{
					if !self.config.dev_state_key.is_empty() {
						if self.config.reload_dev_state {
							// Load JSON and deserialize to app's DevState type
							if let Some(json) = crate::dev_state::DevState::load::<serde_json::Value>(
								self.config.dev_state_key,
							) {
								// Try to deserialize into the app's state type
								match serde_json::from_value(json) {
									Ok(state) => app.load_dev_state(state),
									Err(e) => log::warn!("Failed to deserialize dev state: {}", e),
								}
							}
						} else {
							if let Err(e) =
								crate::dev_state::DevState::cleanup(self.config.dev_state_key)
							{
								log::warn!("Failed to cleanup dev state: {}", e);
							}
						}
					}
				}

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
			CustomEvent::ReloadShaders(_path) => {
				#[cfg(all(not(target_arch = "wasm32"), debug_assertions))]
				{
					if let WindowState::Initialized(painter, app) = &mut self.state {
						painter.reload_shader(_path);
						app.event(Event::ShaderReloadEvent, painter);
						app.frame(painter, 0.0);
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
								let _ = dim.save(self.config.dev_state_key);
							}
						}
					}

					#[cfg(not(target_arch = "wasm32"))]
					WindowEvent::Moved(new_position) => {
						let window = painter.window();
						if self.config.remember_window_dimensions {
							let dim =
								WindowDimensions::from_window(window.inner_size(), new_position);
							let _ = dim.save(self.config.dev_state_key);
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

							app.frame(painter, elapsed);

							if let Some(err) = &painter.surface_error {
								match err {
									wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
										painter.resize(PhysicalSize {
											width: painter.config.width,
											height: painter.config.height,
										});
										app.resize(
											painter,
											painter.config.width,
											painter.config.height,
										);
										log::error!("Surface lost or outdated, resizing");
									}
									// The system is out of memory, we should probably quit
									wgpu::SurfaceError::OutOfMemory => {
										log::error!("OutOfMemory");
										Self::save_dev_state_before_exit(&self.config, app);
										event_loop.exit();
									}

									// This happens when the a frame takes too long to present
									wgpu::SurfaceError::Timeout => {
										log::warn!("Surface timeout")
									}

									other => {
										log::error!("Other error: {:?}", other);
									}
								}
								painter.surface_error = None;
							}

							self.is_resizing = false;
						}
					}

					WindowEvent::CloseRequested => {
						Self::save_dev_state_before_exit(&self.config, app);
						event_loop.exit()
					}

					WindowEvent::CursorMoved { position, .. } => {
						let x = position.x;
						let y = position.y;
						let (delta_x, delta_y) = if let Some((last_x, last_y)) = self.last_cursor {
							(x - last_x, y - last_y)
						} else {
							(0.0, 0.0)
						};
						self.last_cursor = Some((x, y));

						if self.is_running {
							app.event(
								Event::PointerMove {
									x,
									y,
									delta_x,
									delta_y,
									mouse_lock: false,
								},
								painter,
							);
						}
					}

					WindowEvent::MouseInput { state, button, .. } => {
						let button = PointerButton::from(button);
						let (x, y) = self.last_cursor.unwrap_or((0.0, 0.0));

						if self.is_running {
							match state {
								ElementState::Pressed => {
									app.event(Event::PointerDown { button, x, y }, painter);
								}
								ElementState::Released => {
									app.event(Event::PointerUp { button, x, y }, painter);
								}
							}
						}
					}

					WindowEvent::KeyboardInput {
						event:
							KeyEvent {
								state: ElementState::Released,
								physical_key: PhysicalKey::Code(code),
								..
							},
						..
					} => {
						let key = KeyCode::from(code);

						// Handle exit with Escape on native
						#[cfg(not(target_arch = "wasm32"))]
						if matches!(key, KeyCode::Escape) {
							Self::save_dev_state_before_exit(&self.config, app);
							event_loop.exit();
						}

						if self.is_running {
							app.event(Event::KeyUp { key }, painter);
						}

						// Handle internal pause/play with Space after event processing
						if matches!(key, KeyCode::Space) {
							if self.is_running {
								self.is_running = false;
							} else {
								self.is_running = true;
								self.now = Instant::now();
								painter.request_next_frame();
							}
						}
					}

					WindowEvent::KeyboardInput {
						event:
							KeyEvent {
								state: ElementState::Pressed,
								physical_key: PhysicalKey::Code(code),
								..
							},
						..
					} => {
						if self.is_running {
							let key = KeyCode::from(code);
							app.event(Event::KeyDown { key }, painter);
						}
					}

					_ => {
						// Ignore other window events (focus, hover, etc.)
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
				match event {
					DeviceEvent::MouseMotion { delta } => {
						// Raw mouse motion - typically from mouse lock / FPS mode
						app.event(
							Event::PointerMove {
								x: 0.0,
								y: 0.0,
								delta_x: delta.0,
								delta_y: delta.1,
								mouse_lock: true,
							},
							painter,
						);
					}
					_ => {
						// Ignore other device events for now
					}
				}
			}
		}
	}
}
