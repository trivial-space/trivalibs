use crate::app::{Event, KeyCode, PointerButton};
use std::collections::BTreeSet;

pub struct DraggingState {
	pub delta_x: f32,
	pub delta_y: f32,
}

pub struct InputState {
	pub pressed_keys: BTreeSet<KeyCode>,
	pub pressed_pointer_buttons: BTreeSet<PointerButton>,
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
			Event::KeyDown { key } => {
				self.pressed_keys.insert(key);
			}

			Event::KeyUp { key } => {
				self.pressed_keys.remove(&key);
			}

			Event::PointerDown { button, .. } => {
				self.pressed_pointer_buttons.insert(button);

				// Start dragging when any button is pressed
				if self.dragging.is_none() {
					self.dragging = Some(DraggingState {
						delta_x: 0.0,
						delta_y: 0.0,
					});
				}
			}

			Event::PointerUp { button, .. } => {
				self.pressed_pointer_buttons.remove(&button);

				// Stop dragging when all buttons are released
				if self.pressed_pointer_buttons.is_empty() {
					self.dragging = None;
				}
			}

			Event::PointerMove {
				delta_x,
				delta_y,
				mouse_lock,
				..
			} => {
				// Accumulate deltas while dragging
				// Use mouse_lock events for raw motion (FPS-style controls)
				if let Some(dragging) = &mut self.dragging {
					dragging.delta_x += delta_x as f32;
					dragging.delta_y += delta_y as f32;
				} else if mouse_lock {
					// Even without dragging, raw motion might be useful for FPS controls
					// For now, we only track it when dragging
				}
			}

			_ => {}
		}
	}

	/// Check if a key is currently pressed
	pub fn is_key_pressed(&self, key: KeyCode) -> bool {
		self.pressed_keys.contains(&key)
	}

	/// Check if a pointer button is currently pressed
	pub fn is_button_pressed(&self, button: PointerButton) -> bool {
		self.pressed_pointer_buttons.contains(&button)
	}
}
