use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::{fs, path::PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::{PhysicalPosition, PhysicalSize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowDimensions {
	pub size: (u32, u32),
	pub position: (i32, i32),
}

#[cfg(not(target_arch = "wasm32"))]
impl WindowDimensions {
	pub fn get_state_path() -> PathBuf {
		let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
		path.push("rust-graphics");
		path.push("window-state.json");
		path
	}

	pub fn load() -> Option<Self> {
		let path = Self::get_state_path();
		fs::read_to_string(path)
			.ok()
			.and_then(|json| serde_json::from_str(&json).ok())
	}

	pub fn save(&self) -> std::io::Result<()> {
		let path = Self::get_state_path();
		fs::create_dir_all(path.parent().unwrap())?;
		let json = serde_json::to_string(self)?;
		fs::write(path, json)
	}

	pub fn from_window(size: PhysicalSize<u32>, position: PhysicalPosition<i32>) -> Self {
		Self {
			size: (size.width, size.height),
			position: (position.x, position.y),
		}
	}

	pub fn cleanup() -> std::io::Result<()> {
		let path = Self::get_state_path();
		if path.exists() {
			fs::remove_file(path)?;
		}
		Ok(())
	}
}
