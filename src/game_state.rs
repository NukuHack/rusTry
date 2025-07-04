
use std::sync::atomic::Ordering;
use glam::Vec3;
use crate::player;
use crate::world;

#[allow(dead_code)]
pub struct GameState {
    worldname: String,
    player: player::Player,
    world: world::main::World, // lol main data storage :)
    save_path: std::path::PathBuf,
}
#[allow(dead_code)]
impl GameState {
    #[inline]
    pub fn new(worldname: &str) -> Self {
        let player = super::player::Player::new(super::player::CameraConfig::new(Vec3::new(0.5, 1.8, 2.0)));
        
        // Create the save path
        let save_path = std::path::PathBuf::from(&super::config::get_state().save_path)
            .join("saves")
            .join(worldname);

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
        
        match world::manager::update_world_data(&save_path) {
            Ok(_) => (), // Everything is fine, do nothing
            Err(e) => println!("Error updating world data: {}", e),
        }
        
        Self {
            worldname: worldname.to_string(),
            player,
            world: world::main::World::empty(),
            save_path,
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
}

#[inline]
pub fn start_world(worldname: &str) {
    let game_state = GameState::new(worldname);
    super::config::GAMESTATE_PTR.store(Box::into_raw(Box::new(game_state)), Ordering::Release);
    super::config::get_state().is_world_running= true;
}