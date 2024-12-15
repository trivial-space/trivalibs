pub fn main() {
	let this_file = file!();
	let current_dir = std::path::Path::new(this_file).parent().unwrap();
	let full_path = current_dir.join("../../ball/Cargo.toml");
	let full_path = std::fs::canonicalize(full_path).unwrap();
	println!(
		"defined in file: {full_path}",
		full_path = full_path.display()
	);
}
