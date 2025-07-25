
use include_dir::{DirEntry, include_dir};
use std::path::Path;

// Include the assets directory at compile time
pub const RESOURCE_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/comp_resources");

// Macro to get raw bytes from included resources (equivalent to your old get_bytes!)
#[macro_export]
macro_rules! get_raw_data {
	($path:expr) => {{
		use crate::fs::rs::RESOURCE_DIR;
		RESOURCE_DIR
			.get_file($path.clone())
			.map(|file| file.contents())
			.unwrap_or_else(|| panic!("File {} not found in embedded resources", $path))
	}};
}

// Updated get_bytes! macro that adds compression support while maintaining backward compatibility
#[macro_export]
macro_rules! get_bytes {
	($path:expr) => {{
		use crate::fs::rs::RESOURCE_DIR;
		// First try to find a compressed version
		if let Some(file) = RESOURCE_DIR.get_file(format!("{}{}",$path, ".lz4")) {
			lz4_flex::decompress_size_prepended(file.contents())
				.unwrap_or_else(|e| panic!("Failed to decompress {}: {}", $path, e))
		} else {
			// Fall back to uncompressed version (original behavior)
			crate::get_raw_data!($path).to_vec()
		}
	}};
}

// Updated get_string! macro that works with both compressed and uncompressed resources
#[macro_export]
macro_rules! get_string {
	($path:expr) => {{
		let bytes = $crate::get_bytes!($path);
		String::from_utf8(bytes)
			.unwrap_or_else(|e| panic!("File {} is not valid UTF-8: {}", $path, e))
	}};
}

#[macro_export]
macro_rules! get_nth_file {
	($index:expr, $subdir:expr) => {{
		use crate::fs::rs::RESOURCE_DIR;
		let subdir_path = std::path::Path::new($subdir);
		let dir = RESOURCE_DIR.get_dir(subdir_path)
			.unwrap_or_else(|| panic!("Subdirectory '{}' not found in embedded resources", $subdir));
			
		let mut entries: Vec<_> = dir.files().collect();
		entries.sort_by(|a, b| a.path().cmp(b.path()));
		
		entries.get($index)
			.map(|file| {
				let mut path = file.path().to_path_buf();
				// Remove the .lz4 extension if present
				if path.extension().and_then(|e| e.to_str()) == Some("lz4") {
					path.set_extension("");
				}
				path
			})
			.unwrap_or_else(|| panic!("Index {} out of bounds for files in '{}'", $index, $subdir))
	}};
}

use image::ImageReader;
use std::io::Cursor;
use winit::window::Icon;
#[inline]
pub fn load_main_icon() -> Option<Icon> {
	let Some((rgba,w,h)) = load_image_asset_from_path("rusticubes.png".to_string()) else { panic!() };

	match Icon::from_rgba(rgba, w, h) {
		Ok(icon) => Some(icon),
		Err(e) => {
			println!("Failed to create icon from RGBA data: {}", e);
			None
		}
	}
}


/// Scans a subdirectory of the included resources directory for PNG files
/// (including compressed variants) and returns their paths as Strings relative
/// to the resource directory.
pub fn find_png_resources(subdir: &str) -> Vec<String> {
	let mut png_paths = Vec::new();
	let subdir_path = Path::new(subdir);

	// Get the subdirectory within RESOURCE_DIR
	let target_dir = match RESOURCE_DIR.get_dir(subdir_path) {
		Some(dir) => dir,
		None => {
			println!("Subdirectory '{}' not found in resources", subdir);
			return png_paths;
		}
	};

	// Iterate through the target directory entries
	for entry in target_dir.entries() {
		if let DirEntry::File(file) = entry {
			let path = file.path();
			
			// Get the full extension (e.g., "png.lz4")
			let full_ext = path.extension()
				.and_then(|e| e.to_str())
				.unwrap_or("")
				.to_lowercase();

			// Get the file stem (name without extensions)
			let stem = path.file_stem()
				.and_then(|s| s.to_str())
				.unwrap_or("")
				.to_lowercase();

			// Check for either:
			// 1. Direct .png files (full_ext == "png")
			// 2. .png.lz4 files (full_ext == "lz4" and stem ends with ".png")
			let is_png = full_ext == "png" || 
				(full_ext == "lz4" && stem.ends_with(".png"));

			if is_png {
				// Convert to relative path string
				if let Some(path_str) = path.to_str() {
					// Remove .lz4 suffix if present to get the base path
					let clean_path = if full_ext == "lz4" {
						path_str.trim_end_matches(".lz4")
					} else {
						path_str
					};
					png_paths.push(clean_path.to_string());
				}
			}
		}
	}

	png_paths
}

pub fn load_image_from_path<T: Into<String>>(path: T) -> Option<(Vec<u8>,u32,u32)> {
	let path:String = path.into();
	// Create a cursor to read from memory
	let reader_rgba = match ImageReader::new(Cursor::new(crate::get_bytes!(path.clone())))
		.with_guessed_format()
		.expect("Failed to guess format")
		.decode() 
	{
		Ok(img) => img.to_rgba8(),
		Err(e) => {
			println!("Failed to decode image: {}", e);
			return None;
		}
	};

	let (width, height) = reader_rgba.dimensions();

	// Convert to RGBA8 (16×16 = 1024 bytes)
	Some((reader_rgba.into_raw(),width,height))
}
pub fn load_image_asset_from_path<T: Into<String>>(path: T) -> Option<(Vec<u8>,u32,u32)> {
	let path:String = path.into();

	// Create a cursor to read from memory
	let reader_rgba = match ImageReader::new(Cursor::new(crate::get_asset_bytes!(path)))
		.with_guessed_format()
		.expect("Failed to guess format")
		.decode() 
	{
		Ok(img) => img.to_rgba8(),
		Err(e) => {
			println!("Failed to decode image: {}", e);
			return None;
		}
	};

	let (width, height) = reader_rgba.dimensions();

	// Convert to RGBA8 (16×16 = 1024 bytes)
	Some((reader_rgba.into_raw(),width,height))
}

pub const ASSET_DIR: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/comp_assets");

// Macro to get raw bytes from included resources (equivalent to your old get_bytes!)
#[macro_export]
macro_rules! get_asset_raw_data {
	($path:expr) => {{
		use crate::fs::rs::ASSET_DIR;
		ASSET_DIR
			.get_file($path.clone())
			.map(|file| file.contents())
			.unwrap_or_else(|| panic!("File {} not found in embedded resources", $path))
	}};
}

// Updated get_bytes! macro that adds compression support while maintaining backward compatibility
#[macro_export]
macro_rules! get_asset_bytes {
	($path:expr) => {{
		use crate::fs::rs::ASSET_DIR;
		// First try to find a compressed version
		if let Some(file) = ASSET_DIR.get_file(format!("{}{}",$path, ".lz4")) {
			lz4_flex::decompress_size_prepended(file.contents())
				.unwrap_or_else(|e| panic!("Failed to decompress {}: {}", $path, e))
		} else {
			// Fall back to uncompressed version (original behavior)
			crate::get_asset_raw_data!($path).to_vec()
		}
	}};
}