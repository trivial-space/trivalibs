use trivalibs_core::rendering::camera::PerspectiveCamera;
use trivalibs_painter::{
	app::{KeyCode, PointerButton},
	utils::input_state::InputState,
};

pub struct BasicFirstPersonCameraController {
	screen_width: u32,
	screen_height: u32,

	move_speed: f32,
	look_speed: f32,

	old_drag_x: f32,
	old_drag_y: f32,

	holding: bool,
	hold_cancelled: bool,
	hold_time: f32,
}

impl BasicFirstPersonCameraController {
	pub fn new(move_speed: f32, look_speed: f32) -> Self {
		Self {
			screen_width: 1,
			screen_height: 1,

			move_speed,
			look_speed,

			old_drag_x: 0.0,
			old_drag_y: 0.0,

			holding: false,
			hold_cancelled: false,
			hold_time: 0.0,
		}
	}

	pub fn set_move_speed(&mut self, speed: f32) {
		self.move_speed = speed;
	}
	pub fn set_look_speed(&mut self, speed: f32) {
		self.look_speed = speed;
	}

	pub fn set_screen_size(&mut self, width: u32, height: u32) {
		self.screen_width = width;
		self.screen_height = height;
	}

	pub fn update_camera(
		&mut self,
		camera: &mut PerspectiveCamera,
		input: &InputState,
		delta_time: f32,
	) {
		let mut left = 0.0;
		let mut forward = 0.0;
		let mut rot_x = 0.0;
		let mut rot_y = 0.0;

		if let Some(dragging) = &input.dragging {
			let delta_x = self.old_drag_x - dragging.delta_x;
			let delta_y = self.old_drag_y - dragging.delta_y;

			self.old_drag_x = dragging.delta_x;
			self.old_drag_y = dragging.delta_y;

			let slow_down = self.screen_width.max(self.screen_height) as f32;

			rot_x += delta_x * self.look_speed / slow_down;
			rot_y += delta_y * self.look_speed / slow_down;

			if !self.holding
				&& !self.hold_cancelled
				&& dragging.delta_x.abs() < 20.0
				&& dragging.delta_y.abs() < 20.0
			{
				self.hold_time += delta_time;
				if self.hold_time > 0.4 {
					self.holding = true;
				}
			} else {
				self.hold_cancelled = true;
			}
		} else {
			self.old_drag_x = 0.0;
			self.old_drag_y = 0.0;

			self.holding = false;
			self.hold_cancelled = false;
			self.hold_time = 0.0;
		}

		let move_distance = self.move_speed * delta_time;

		if input.pressed_keys.contains(&KeyCode::KeyW)
			|| input.pressed_keys.contains(&KeyCode::ArrowUp)
			|| (self.holding
				&& !input
					.pressed_pointer_buttons
					.contains(&PointerButton::Secondary))
		{
			forward += move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyS)
			|| input.pressed_keys.contains(&KeyCode::ArrowDown)
			|| input
				.pressed_pointer_buttons
				.contains(&PointerButton::Secondary)
		{
			forward -= move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyA)
			|| input.pressed_keys.contains(&KeyCode::ArrowLeft)
		{
			left += move_distance;
		}
		if input.pressed_keys.contains(&KeyCode::KeyD)
			|| input.pressed_keys.contains(&KeyCode::ArrowRight)
		{
			left -= move_distance;
		}

		if left != 0.0 || forward != 0.0 || rot_x != 0.0 || rot_y != 0.0 {
			camera.update_transform(forward, left, 0.0, rot_x, rot_y);
		}
	}
}
