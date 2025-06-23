use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const STATE_CACHE_DIR: &'static str = "trivalibs_painter_dev_state";
const DEFAULT_STATE_FILE: &'static str = "app-state.json";

pub fn get_state_path(state_file_name: Option<&str>) -> PathBuf {
	let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
	path.push(STATE_CACHE_DIR);
	path.push(state_file_name.unwrap_or(DEFAULT_STATE_FILE));
	path
}

pub fn load_state<T: for<'de> Deserialize<'de>>(state_file_name: Option<&str>) -> Option<T> {
	let path = get_state_path(state_file_name);
	fs::read_to_string(path)
		.ok()
		.and_then(|json| serde_json::from_str(&json).ok())
}

pub fn save_state<T: Serialize>(state: &T, state_file_name: Option<&str>) -> std::io::Result<()> {
	let path = get_state_path(state_file_name);
	fs::create_dir_all(path.parent().unwrap())?;
	let json = serde_json::to_string_pretty(state)?;
	fs::write(path, json)
}

pub fn cleanup_state(state_file_name: Option<&str>) -> std::io::Result<()> {
	let path = get_state_path(state_file_name);
	if path.exists() {
		fs::remove_file(path)?;
	}
	Ok(())
}
