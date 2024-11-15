use glam::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use js_sys::Float32Array;

pub fn mat2_to_js(mat: &Mat2) -> Float32Array {
	js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn mat3_to_js(mat: &Mat3) -> Float32Array {
	js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn mat4_to_js(mat: &Mat4) -> Float32Array {
	js_sys::Float32Array::from(mat.to_cols_array().as_slice())
}

pub fn vec2_to_js(vec: &Vec2) -> Float32Array {
	js_sys::Float32Array::from(vec.to_array().as_slice())
}

pub fn vec3_to_js(vec: &Vec3) -> Float32Array {
	js_sys::Float32Array::from(vec.to_array().as_slice())
}

pub fn vec4_to_js(vec: &Vec4) -> Float32Array {
	js_sys::Float32Array::from(vec.to_array().as_slice())
}

#[macro_export]
macro_rules! setup_camera_interactions {
	($state_struct: ident, $camera_identifier:ident) => {
		#[wasm_bindgen]
		pub fn update_screen(width: f32, height: f32) {
			$state_struct::mutate(|s| s.$camera_identifier.set_aspect_ratio(width / height))
		}

		#[derive(Serialize)]
		struct CameraTransform {
			pos: Vec3,
			rot_horizontal: f32,
			rot_vertical: f32,
		}

		#[wasm_bindgen]
		pub fn update_camera(forward: f32, left: f32, up: f32, rot_y: f32, rot_x: f32) -> JsValue {
			$state_struct::mutate(|s| {
				s.$camera_identifier
					.update_transform(forward, left, up, rot_y, rot_x)
			});
			let s = $state_struct::read();
			serde_wasm_bindgen::to_value(&CameraTransform {
				pos: s.$camera_identifier.translation,
				rot_horizontal: s.$camera_identifier.rot_horizontal,
				rot_vertical: s.$camera_identifier.rot_vertical,
			})
			.unwrap()
		}

		#[wasm_bindgen]
		pub fn reset_camera(x: f32, y: f32, z: f32, rot_horizontal: f32, rot_vertical: f32) {
			$state_struct::mutate(|s| {
				s.$camera_identifier.reset_transform(
					vec3(x as f32, y as f32, z as f32),
					rot_horizontal,
					rot_vertical,
				)
			})
		}
	};
}
