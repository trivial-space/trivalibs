#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use serde::{Deserialize, Serialize};
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use std::{fs, path::PathBuf};

/// Helper for managing development state persistence
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
pub struct DevState;

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
impl DevState {
	/// Get the path where state should be stored for the given key
	pub fn get_state_path(storage_key: &str) -> PathBuf {
		let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
		path.push("rust-graphics");
		path.push("dev-state");
		path.push(format!("{}.json", storage_key));
		path
	}

	/// Load state from disk. Returns None if file doesn't exist or deserialization fails.
	pub fn load<T>(storage_key: &str) -> Option<T>
	where
		T: for<'de> Deserialize<'de>,
	{
		let path = Self::get_state_path(storage_key);
		match fs::read_to_string(&path) {
			Ok(json) => match serde_json::from_str(&json) {
				Ok(state) => {
					log::info!("Loaded dev state from: {:?}", path);
					Some(state)
				}
				Err(e) => {
					log::warn!("Failed to deserialize dev state from {:?}: {}", path, e);
					None
				}
			},
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				log::debug!("No dev state file found at: {:?}", path);
				None
			}
			Err(e) => {
				log::warn!("Failed to read dev state from {:?}: {}", path, e);
				None
			}
		}
	}

	/// Save state to disk
	pub fn save<T>(storage_key: &str, state: &T) -> std::io::Result<()>
	where
		T: Serialize,
	{
		let path = Self::get_state_path(storage_key);
		fs::create_dir_all(path.parent().unwrap())?;
		let json = serde_json::to_string_pretty(state)?;
		fs::write(&path, json)?;
		log::info!("Saved dev state to: {:?}", path);
		Ok(())
	}

	/// Remove state file if it exists
	pub fn cleanup(storage_key: &str) -> std::io::Result<()> {
		let path = Self::get_state_path(storage_key);
		if path.exists() {
			fs::remove_file(&path)?;
			log::info!("Cleaned up dev state at: {:?}", path);
		}
		Ok(())
	}
}
