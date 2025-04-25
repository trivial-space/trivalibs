use crate::{
	app::Event,
	winit::{
		event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
		keyboard::{KeyCode, PhysicalKey},
	},
};
use std::collections::BTreeSet;

pub struct DraggingState {
	pub delta_x: f32,
	pub delta_y: f32,
}

pub struct InputState {
	pub pressed_keys: BTreeSet<KeyCode>,
	pub pressed_pointer_buttons: BTreeSet<MouseButton>,
	pub dragging: Option<DraggingState>,
}

impl Default for InputState {
	fn default() -> Self {
		Self {
			pressed_keys: BTreeSet::new(),
			pressed_pointer_buttons: BTreeSet::new(),
			dragging: None,
		}
	}
}

impl InputState {
	pub fn process_event<U>(&mut self, event: Event<U>) {
		match event {
			Event::WindowEvent(WindowEvent::KeyboardInput {
				event:
					KeyEvent {
						state,
						physical_key: PhysicalKey::Code(code),
						..
					},
				..
			}) => {
				if state == ElementState::Pressed {
					self.pressed_keys.insert(code);
				} else {
					self.pressed_keys.remove(&code);
				}
			}

			Event::WindowEvent(WindowEvent::MouseInput { state, button, .. }) => {
				if state == ElementState::Pressed {
					self.pressed_pointer_buttons.insert(button);
				} else {
					self.pressed_pointer_buttons.remove(&button);
				}

				if self.pressed_pointer_buttons.is_empty() {
					if self.dragging.is_some() {
						self.dragging = None;
					}
				} else {
					if self.dragging.is_none() {
						self.dragging = Some(DraggingState {
							delta_x: 0.0,
							delta_y: 0.0,
						});
					}
				}
			}

			Event::DeviceEvent(DeviceEvent::MouseMotion { delta }) => {
				if let Some(dragging) = &mut self.dragging {
					dragging.delta_x += delta.0 as f32;
					dragging.delta_y += delta.1 as f32;
				}
			}

			_ => {}
		}
	}
}
