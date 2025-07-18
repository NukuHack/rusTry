
use std::path::PathBuf;
use crate::game::player;
use crate::ext::{ptr, config};
use crate::world;
use std::sync::atomic::Ordering;
use glam::Vec3;

#[allow(dead_code)]
pub struct GameState {
	worldname: String,
	player: player::Player,
	world: world::main::World, // lol main data storage :)
	save_path: std::path::PathBuf,
	is_running: bool,
}

pub fn make_world(save_path: PathBuf) {
	// has to make the error handling better , make the error quit from world
	let _ = config::ensure_save_dir();
	// Check and create directories if needed
	match std::fs::metadata(&save_path) {
		Ok(metadata) => {
			if !metadata.is_dir() {
				println!(
					"Save path {:?} exists but is not a directory", 
					save_path
				);
			}
			// Directory already exists, no need to create
		}
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
			// Directory doesn't exist, try to create it
			std::fs::create_dir_all(&save_path)
				.map_err(|e| {
					println!("Failed to create save directory at {:?}: {}", save_path, e)
				}).unwrap_or_else(|_| println!("Something failed"));
		}
		Err(e) => {
			// Other IO error (permission issues, etc.)
			println!(
				"Unexpected error accessing save path {:?}: {}", 
				save_path, e
			);
		}
	}
}

#[allow(dead_code)]
impl GameState {
	#[inline]
	pub fn new(worldname: &str) -> Self {
		let player = player::Player::new(player::CameraConfig::new(Vec3::new(0.5, 1.8, 2.0)));
		
		// Create the save path
		let save_path = config::get_save_path()
			.join("saves")
			.join(worldname);

		make_world(save_path.clone());

		match world::manager::update_world_data(&save_path) {
			Ok(_) => (), // Everything is fine, do nothing
			Err(e) => println!("Error updating world data: {}", e),
		}
		
		Self {
			worldname: worldname.to_string(),
			player,
			world: world::main::World::empty(),
			save_path,
			is_running: false,
		}
	}
	#[inline]
	pub fn world_mut(&mut self) -> &mut world::main::World {
		&mut self.world
	}
	#[inline]
	pub fn player_mut(&mut self) -> &mut player::Player {
		&mut self.player
	}
	#[inline]
	pub fn world_change(&mut self, world : world::main::World) {
		self.world = world;
	}

	#[inline]
	pub fn worldname(&self) -> &String {
		&self.worldname
	}
	#[inline]
	pub fn player(&self) -> &player::Player {
		&self.player
	}
	#[inline]
	pub fn world(&self) -> &world::main::World {
		&self.world
	}
	#[inline]
	pub fn save_path(&self) -> &std::path::PathBuf {
		&self.save_path
	}
	#[inline]
	pub fn is_running(&self) -> bool {
		self.is_running
	}
	#[inline]
	pub fn running(&mut self) -> &mut bool {
		&mut self.is_running
	}


	#[inline]
	pub fn seed(&self) -> &u32 {
		&0u32
	}
}

#[inline]
pub fn start_world(worldname: &str) {
	let game_state = GameState::new(worldname);
	ptr::GAMESTATE_PTR.store(Box::into_raw(Box::new(game_state)), Ordering::Release);
	
	// This will only execute when not in test configuration
	if !cfg!(test) {
		ptr::get_state().is_world_running = true;
	}
}