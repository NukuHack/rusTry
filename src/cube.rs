﻿
use super::geometry::Vertex;
use cgmath::{Rotation3, Vector3};
use std::collections::{HashMap, HashSet};

/// Stores position for X, Y, Z as 4-bit fields: [X:4, Y:4, Z:4, Empty:4]
/// Stores rotations for X, Y, Z as 5-bit fields: [X:5, Y:5, Z:5, Empty:1]
/// Stores 3x3x3 points as a 32-bit "array" [Points: 27, Empty: 5]
#[derive(Clone, Copy)]
pub struct Block {
    /// in case someone needs it (i do i'm stupid) 4 bits is 0-15 ; 5 bits is 0-32; this goes forever (i think u256 is the current max)
    pub position: u16,    // [X:4, Y:4, Z:4, Empty:4]
    pub material: u16,    // Material info (unused in current implementation)
    pub points: u32,      // 3x3x3 points (27 bits used)
    pub rotation: u16,    // [X:5, Y:5, Z:5, Empty:1]
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block")
            .field("position", &format_args!("{:?}", self.position))
            .field("material", &format_args!("{:?}", self.material))
            .field("points", &format_args!("{:?}", self.points))
            .field("rotation", &format_args!("{:?}", self.rotation))
            .finish()
    }
}
impl Block {
    pub const POS_MASK_X: u16 = 0xF << 8;
    pub const POS_SHIFT_X: u32 = 8;
    pub const POS_MASK_Y: u16 = 0xF << 4;
    pub const POS_SHIFT_Y: u32 = 4;
    pub const POS_MASK_Z: u16 = 0xF;
    pub const POS_SHIFT_Z: u32 = 0;

    pub const ROT_MASK_X: u16 = 0b11111;
    pub const ROT_SHIFT_X: u32 = 0;
    pub const ROT_MASK_Y: u16 = 0b11111 << 5;
    pub const ROT_SHIFT_Y: u32 = 5;
    pub const ROT_MASK_Z: u16 = 0b11111 << 10;
    pub const ROT_SHIFT_Z: u32 = 10;

    /// Creates a new cube with a specified position.
    #[inline]
    pub fn new(position: u16) -> Self {
        Self { position, material: 1, ..Self::default() }
    }
    #[inline]
    pub fn default() -> Self {
        Self { position:0, material:0, points:0, rotation:0}
    }

    /// Creates a new cube with a specified position and rotation.
    #[inline]
    pub fn new_rot(position: u16, rotation: u16) -> Self {
        Self {
            position,
            rotation,
            material: 1,
            ..Self::default()
        }
    }

    pub fn new_raw(
        position: cgmath::Vector3<i32>,
    ) -> Self {
        Self {
            position:vector_to_position(position),
            material: 1,
            ..Self::default()
        }
    }
    pub fn new_rot_raw(
        position: cgmath::Vector3<i32>,
        rotation: cgmath::Quaternion<f32>,
    ) -> Self {
        Self {
            position:vector_to_position(position),
            rotation:quaternion_to_rotation(rotation),
            material: 1,
            ..Self::default()
        }
    }

    /// Extract individual rotation components (0-3)
    #[inline]
    pub fn get_x_rotation(&self) -> u16 { (self.rotation & Self::ROT_MASK_X) >> Self::ROT_SHIFT_X }
    #[inline]
    pub fn get_y_rotation(&self) -> u16 { (self.rotation & Self::ROT_MASK_Y) >> Self::ROT_SHIFT_Y }
    #[inline]
    pub fn get_z_rotation(&self) -> u16 { (self.rotation & Self::ROT_MASK_Z) >> Self::ROT_SHIFT_Z }

    /// Rotation snapping and conversion to quaternion
    pub fn rotation_to_quaternion(&self) -> cgmath::Quaternion<f32> {
        let angles = [self.get_x_rotation(), self.get_y_rotation(), self.get_z_rotation()]
            .map(|r| cgmath::Deg(r as f32 * (360.0 / 32.0)));
        
        cgmath::Quaternion::from_angle_z(angles[2]) *
        cgmath::Quaternion::from_angle_y(angles[1]) *
        cgmath::Quaternion::from_angle_x(angles[0])
    }


    /// Sets the position of the cube in 3D space.
    #[inline]
    pub fn set_pos(&mut self, x: u16, y: u16, z: u16) {
        self.position = (x << 8) | (y << 4) | z;
    }
    /// Convert packed position to world coordinates
    pub fn get_pos(&self) -> cgmath::Vector3<i32> {
        cgmath::Vector3::new(
            ((self.position >> 8) & 0xF) as i32,
            ((self.position >> 4) & 0xF) as i32,
            (self.position & 0xF) as i32
        )
    }

    #[inline]
    pub fn is_empty(&self) -> bool { self.material == 0 }

    /// Full conversion to Instance
    pub fn to_instance(&self) -> super::geometry::Instance {
        super::geometry::Instance {
            position: vec3_i32_to_f32(self.get_pos()),
            rotation: self.rotation_to_quaternion()
        }
    }
    pub fn to_world_instance(&self, chunk_pos: ChunkCoord) -> super::geometry::Instance {
        super::geometry::Instance {
            position: vec3_i32_to_f32(self.get_pos() + chunk_pos.to_world_pos()),
            rotation: self.rotation_to_quaternion()
        }
    }

    // Here is the marching cube stuff, what i did not finish so it's commented out currently
    pub fn rotate(&mut self, axis: char, steps: u16) {
        let (current, mask, shift) = match axis {
            'x' => (self.get_x_rotation(), Self::ROT_MASK_X, Self::ROT_SHIFT_X),
            'y' => (self.get_y_rotation(), Self::ROT_MASK_Y, Self::ROT_SHIFT_Y),
            'z' => (self.get_z_rotation(), Self::ROT_MASK_Z, Self::ROT_SHIFT_Z),
            _ => unreachable!(),
        };
        
        let new_rot = (current + steps) % 32;
        self.rotation = (self.rotation & !mask) | (new_rot << shift);
    }
    // More type-safe rotation
    pub fn set_rotation(&mut self, x: u16, y: u16, z: u16) {
        self.rotation = (x & 0x1F) 
            | ((y & 0x1F) << 5) 
            | ((z & 0x1F) << 10);
    }
}


/// Convert a quaternion to the packed u16 rotation format
/// Convert a quaternion to the packed u16 rotation format
pub fn quaternion_to_rotation(rotation: cgmath::Quaternion<f32>) -> u16 {
    let angles = [
        (2.0 * (rotation.s * rotation.v.x + rotation.v.y * rotation.v.z)).atan2(1.0 - 2.0 * (rotation.v.x.powi(2) + rotation.v.y.powi(2))),
        (2.0 * (rotation.s * rotation.v.y - rotation.v.z * rotation.v.x)).asin(),
        (2.0 * (rotation.s * rotation.v.z + rotation.v.x * rotation.v.y)).atan2(1.0 - 2.0 * (rotation.v.y.powi(2) + rotation.v.z.powi(2)))
    ];

    const SCALE: f32 = 31.0 / (2.0 * std::f32::consts::PI);
    let bits: [u16; 3] = angles.map(|a| ((a.rem_euclid(2.0 * std::f32::consts::PI) * SCALE).round() as u16 & 0x1F));
    
    bits[0] | (bits[1] << 5) | (bits[2] << 10)
}

#[inline]
pub fn vector_to_position(position: cgmath::Vector3<i32>) -> u16 {
    ((position.x as u16 & 0xF) << 8) | 
    ((position.y as u16 & 0xF) << 4) | 
    (position.z as u16 & 0xF)
}
// Utility functions for vector type conversion
#[inline]
pub fn vec3_f32_to_i32(v: cgmath::Vector3<f32>) -> cgmath::Vector3<i32> {
    cgmath::Vector3::new(v.x as i32, v.y as i32, v.z as i32)
}
#[inline]
pub fn vec3_i32_to_f32(v: cgmath::Vector3<i32>) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(v.x as f32, v.y as f32, v.z as f32)
}
// converting from i32 to u32 never happens outside chunk/block structs so it's not needed

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn to_world_pos(&self) -> Vector3<i32> {
        Vector3::new(
            self.x * Chunk::CHUNK_SIZE_I,
            self.y * Chunk::CHUNK_SIZE_I,
            self.z * Chunk::CHUNK_SIZE_I,
        )
    }

    #[inline]
    pub fn from_world_pos(world_pos: Vector3<i32>) -> Self {
        Self {
            x: world_pos.x.div_euclid(Chunk::CHUNK_SIZE_I),
            y: world_pos.y.div_euclid(Chunk::CHUNK_SIZE_I),
            z: world_pos.z.div_euclid(Chunk::CHUNK_SIZE_I),
        }
    }
}


#[derive(Clone, Debug)]
pub struct Chunk {
    pub position: ChunkCoord,  // World coordinates of chunk
    pub blocks: [Block; 4096],  // Array of blocks in the chunk basically 16*16*16
    pub dirty: bool,  // For mesh regeneration
    pub mesh: Option<super::geometry::GeometryBuffer>,
}
impl Chunk {
    pub const CHUNK_SIZE: usize = 16;
    pub const CHUNK_SIZE_U: u32 = Self::CHUNK_SIZE as u32;
    pub const CHUNK_SIZE_I: i32 = Self::CHUNK_SIZE as i32;
    pub const CUBES_PER_CHUNK: usize = Self::CHUNK_SIZE.pow(3);

    /// Creates a new empty chunk at the specified chunk coordinates
    pub fn empty(chunk_coord: ChunkCoord) -> Self {
        Self {
            position: chunk_coord,
            blocks: [Block::default(); Self::CUBES_PER_CHUNK],
            dirty: false,
            mesh: None,
        }
    }

    /// Creates a new filled chunk at the specified chunk coordinates
    pub fn new(chunk_coord: ChunkCoord) -> Self {
        // Precompute all possible packed positions for this chunk
        let mut precomputed_positions = [[[0u16; Self::CHUNK_SIZE]; Self::CHUNK_SIZE]; Self::CHUNK_SIZE];
        
        for x in 0..Self::CHUNK_SIZE {
            for y in 0..Self::CHUNK_SIZE {
                for z in 0..Self::CHUNK_SIZE {
                    precomputed_positions[x][y][z] = ((x as u16) << 8) | ((y as u16) << 4) | z as u16;
                }
            }
        }

        Chunk { 
            position: chunk_coord,
            blocks: std::array::from_fn(|i| {
                let (x, y, z) = (
                    (i / (Self::CHUNK_SIZE * Self::CHUNK_SIZE)) % Self::CHUNK_SIZE,
                    (i / Self::CHUNK_SIZE) % Self::CHUNK_SIZE,
                    i % Self::CHUNK_SIZE
                );
                Block::new(precomputed_positions[x][y][z])
            }),
            dirty: false,
            mesh: None,
        }
    }

    #[inline]
    pub fn load(chunk_coord: ChunkCoord) -> core::option::Option<Self> {
        Some(Self::new(chunk_coord))
    }

    // Add chunk neighbors awareness
    pub fn get_adjacent_chunk_coords(&self) -> [ChunkCoord; 6] {
        [
            ChunkCoord::new(self.position.x - 1, self.position.y, self.position.z),
            ChunkCoord::new(self.position.x + 1, self.position.y, self.position.z),
            ChunkCoord::new(self.position.x, self.position.y - 1, self.position.z),
            ChunkCoord::new(self.position.x, self.position.y + 1, self.position.z),
            ChunkCoord::new(self.position.x, self.position.y, self.position.z - 1),
            ChunkCoord::new(self.position.x, self.position.y, self.position.z + 1),
        ]
    }

    pub fn get_block(&self, local_pos: Vector3<u32>) -> Option<&Block> {
        self.blocks.get(Self::local_to_index(local_pos) as usize).filter(|b| !b.is_empty())
    }

    pub fn get_block_mut(&mut self, local_pos: Vector3<u32>) -> Option<&mut Block> {
        self.blocks.get_mut(Self::local_to_index(local_pos) as usize).filter(|b| !b.is_empty())
    }

    pub fn set_block(&mut self, local_pos: Vector3<u32>, cube: Block) {
        if let Some(b) = self.blocks.get_mut(Self::local_to_index(local_pos) as usize) {
            *b = cube;
            self.dirty = true;
        }
    }

    /// Convert local chunk coordinates to world position
    #[inline]
    pub fn local_to_world_pos(&self, local_pos: Vector3<u32>) -> Vector3<i32> {
        Vector3::new(
            self.position.x * Self::CHUNK_SIZE_I + local_pos.x as i32,
            self.position.y * Self::CHUNK_SIZE_I + local_pos.y as i32,
            self.position.z * Self::CHUNK_SIZE_I + local_pos.z as i32,
        )
    }

    /// Convert world position to local chunk coordinates
    #[inline]
    pub fn world_to_local_pos(world_pos: Vector3<i32>) -> Vector3<u32> {
        Vector3::new(
            world_pos.x.rem_euclid(Self::CHUNK_SIZE_I) as u32,
            world_pos.y.rem_euclid(Self::CHUNK_SIZE_I) as u32,
            world_pos.z.rem_euclid(Self::CHUNK_SIZE_I) as u32,
        )
    }

    /// Convert local coordinates to array index
    #[inline]
    pub fn local_to_index(local_pos: Vector3<u32>) -> u32 {
        local_pos.z * Self::CHUNK_SIZE_U.pow(2) + local_pos.y * Self::CHUNK_SIZE_U + local_pos.x
    }

    /// Convert array index to local coordinates
    #[inline]
    pub fn index_to_local(index: u32) -> Vector3<u32> {
        Vector3::new(
            index % Self::CHUNK_SIZE_U,
            (index / Self::CHUNK_SIZE_U) % Self::CHUNK_SIZE_U,
            index / Self::CHUNK_SIZE_U.pow(2),
        )
    }

    /// Check if a world position is within this chunk
    #[inline]
    pub fn contains_world_pos(&self, world_pos: Vector3<i32>) -> bool {
        ChunkCoord::from_world_pos(world_pos) == self.position
    }

    /// Get block at world position if it's in this chunk
    pub fn get_block_at_world_pos(&self, world_pos: Vector3<i32>) -> Option<&Block> {
        if self.contains_world_pos(world_pos) {
            let local = Self::world_to_local_pos(world_pos);
            self.get_block(local)
        } else {
            None
        }
    }

    #[inline]
    pub fn position(&self) -> Vector3<i32> {
        self.position.to_world_pos()
    }

    pub fn make_mesh(&self) {
        todo!();
        // self.mesh = "stuff"
    }
}

pub struct BlockBuffer;

impl BlockBuffer {
    pub fn new(device: &wgpu::Device) -> super::geometry::GeometryBuffer {
        super::geometry::GeometryBuffer::new(device, &INDICES, &VERTICES)
    }
}


#[derive(Debug, Clone)]
pub struct World {
    pub chunks: HashMap<ChunkCoord, Chunk>,  // Position is now solely in the key
    
    // Spatial indexing system
    spatial_grid: HashMap<(i32, i32, i32), HashSet<ChunkCoord>>,
    grid_cell_size: i32,
}
#[allow(dead_code, unused)]
impl World {
    /// Create an empty world with no chunks
    #[inline]
    pub fn empty() -> Self {
        Self {
            chunks: HashMap::new(),

            spatial_grid: HashMap::new(),
            grid_cell_size: 1.max(1),
        }
    }
    // Helper to get grid cell coordinates for a chunk
    #[inline]
    fn get_grid_cell(&self, coord: &ChunkCoord) -> (i32, i32, i32) {
        (
            coord.x.div_euclid(self.grid_cell_size),
            coord.y.div_euclid(self.grid_cell_size),
            coord.z.div_euclid(self.grid_cell_size),
        )
    }
    #[inline]
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    #[inline]
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    pub fn get_block(&self, world_pos: Vector3<i32>) -> Option<&Block> {
        let chunk_coord = ChunkCoord::from_world_pos(world_pos);
        self.chunks.get(&chunk_coord)
            .and_then(|chunk| chunk.get_block_at_world_pos(world_pos))
    }
    pub fn get_block_mut(&mut self, world_pos: Vector3<i32>) -> Option<&mut Block> {
        let chunk_coord = ChunkCoord::from_world_pos(world_pos);
        self.chunks.get_mut(&chunk_coord)
            .and_then(|chunk| {
                let local = Chunk::world_to_local_pos(world_pos);
                chunk.get_block_mut(local)
            })
    }

    #[inline]
    pub fn set_chunk(&mut self, chunk_coord: ChunkCoord, chunk: Chunk) {
        let grid_cell = self.get_grid_cell(&chunk_coord);
        
        // Update spatial index
        self.spatial_grid.entry(grid_cell)
            .or_default()
            .insert(chunk_coord);
        
        // Store chunk
        self.chunks.insert(chunk_coord, chunk);
    }
    pub fn set_block(&mut self, world_pos: Vector3<i32>, block: Block) {
        let chunk_coord = ChunkCoord::from_world_pos(world_pos);
        
        if !self.chunks.contains_key(&chunk_coord) {
            self.set_chunk(chunk_coord, Chunk::empty(chunk_coord));
        }
        
        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            let local = Chunk::world_to_local_pos(world_pos);
            chunk.set_block(local, block);
        }
    }

    #[inline]
    pub fn load_chunk(&mut self, chunk_coord: ChunkCoord) -> bool {
        let chunk: Option<Chunk> = Chunk::load(chunk_coord); // currently just makes a full chunk - will change this to an actual load/generate function later
        if chunk.is_some() {
            self.set_chunk(chunk_coord, chunk.unwrap());
        } else {
            return false;
        };

        true
    }

    #[inline]
    pub fn unload_chunk(&mut self, chunk_coord: ChunkCoord) {
        if let Some(grid_cell) = self.chunks.get(&chunk_coord)
            .map(|_| self.get_grid_cell(&chunk_coord))
        {
            if let Some(cell_set) = self.spatial_grid.get_mut(&grid_cell) {
                cell_set.remove(&chunk_coord);
                if cell_set.is_empty() {
                    self.spatial_grid.remove(&grid_cell);
                }
            }
        }
        self.chunks.remove(&chunk_coord);
    }


    // Add chunk loading/unloading strategies
    pub fn update_loaded_chunks(&mut self, center: Vector3<i32>, radius: u32) {
        let radius_i32 = radius as i32;
        let radius_sq = (radius * radius) as i32;

        // Precompute the bounds of the sphere in chunk coordinates
        let min_x = center.x - radius_i32;
        let max_x = center.x + radius_i32;
        let min_y = center.y - radius_i32;
        let max_y = center.y + radius_i32;
        let min_z = center.z - radius_i32;
        let max_z = center.z + radius_i32;

        // Track chunks we want to keep
        let mut chunks_to_keep = HashSet::with_capacity((radius * radius * radius * 4) as usize);

        // OPTIMIZATION 1: Precompute grid cell ranges and only check cells that could contain chunks in range
        let min_cell_x = min_x.div_euclid(self.grid_cell_size);
        let max_cell_x = max_x.div_euclid(self.grid_cell_size);
        let min_cell_y = min_y.div_euclid(self.grid_cell_size);
        let max_cell_y = max_y.div_euclid(self.grid_cell_size);
        let min_cell_z = min_z.div_euclid(self.grid_cell_size);
        let max_cell_z = max_z.div_euclid(self.grid_cell_size);

        // ^^^ this part took around 8 micro seconds so it's basically nothing
        //let start = std::time::Instant::now();

        // OPTIMIZATION 2: Use a single loop with early continues for better branch prediction
        for cell_x in min_cell_x..=max_cell_x {
            let cell_x_world = cell_x * self.grid_cell_size;
            let cell_min_x = cell_x_world;
            let cell_max_x = cell_x_world + self.grid_cell_size - 1;
            
            // Skip if this cell is completely outside the X bounds
            if cell_max_x < min_x || cell_min_x > max_x {
                continue;
            }

            for cell_y in min_cell_y..=max_cell_y {
                let cell_y_world = cell_y * self.grid_cell_size;
                let cell_min_y = cell_y_world;
                let cell_max_y = cell_y_world + self.grid_cell_size - 1;
                
                // Skip if this cell is completely outside the Y bounds
                if cell_max_y < min_y || cell_min_y > max_y {
                    continue;
                }

                for cell_z in min_cell_z..=max_cell_z {
                    let cell_z_world = cell_z * self.grid_cell_size;
                    let cell_min_z = cell_z_world;
                    let cell_max_z = cell_z_world + self.grid_cell_size - 1;
                    
                    // Skip if this cell is completely outside the Z bounds
                    if cell_max_z < min_z || cell_min_z > max_z {
                        continue;
                    }

                    // Only proceed if this cell could potentially contain chunks in range
                    if let Some(cell_chunks) = self.spatial_grid.get(&(cell_x, cell_y, cell_z)) {
                        for &coord in cell_chunks {
                            // Check if chunk is within the sphere
                            let dx = coord.x - center.x;
                            let dy = coord.y - center.y;
                            let dz = coord.z - center.z;
                            
                            if dx * dx + dy * dy + dz * dz <= radius_sq {
                                chunks_to_keep.insert(coord);
                            }
                        }
                    }
                }
            }
        }

        //println!("Chunk checking (3d loops) took: {:?}", start.elapsed()); // this is soo slow takes around 7 millisec (while empty it takes 200 micro)
        //let start = std::time::Instant::now();

        // OPTIMIZATION 3: Batch unload operations
        let to_unload: Vec<ChunkCoord> = self.chunks.keys()
            .filter(|&&coord| !chunks_to_keep.contains(&coord))
            .cloned()
            .collect();
        
        for coord in to_unload {
            self.unload_chunk(coord);
        }

        //println!("Chunk unloading took: {:?}", start.elapsed()); // this is slow - takes around 2 millisec (while empty it takes 2 micro)
        //let start = std::time::Instant::now();


        // OPTIMIZATION 4: Use a more computer-friendly early exit loops

        // First collect all chunks that need loading
        for x in -radius_i32..=radius_i32 {
            let x_sq = x * x;
            if x_sq > radius_sq {
                continue;
            }
            
            for y in -radius_i32..=radius_i32 {
                let y_sq = y * y;
                let xy_sq = x_sq + y_sq;
                if xy_sq > radius_sq {
                    continue;
                }
                
                for z in -radius_i32..=radius_i32 {
                    if xy_sq + z * z > radius_sq {
                        continue;
                    }
                    
                    let chunk_coord = ChunkCoord::new(center.x + x, center.y + y, center.z + z);
                    if !self.chunks.contains_key(&chunk_coord) {
                        self.load_chunk(chunk_coord);
                    }
                }
            }
        }

        //println!("Chunk loading took: {:?}", start.elapsed()); // this is slow around 2.5 millisec but when empty it takes up to 1 Sec to complete
    }
}

const VERTICES: [Vertex; 8] = [
    Vertex { position: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] },
    Vertex { position: [0.0, 1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0] },
    Vertex { position: [1.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
    Vertex { position: [0.0, 0.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
    Vertex { position: [0.0, 1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
    Vertex { position: [1.0, 1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [1.0, 0.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
];

const INDICES: [u32; 36] = [
    1, 0, 2, 3, 2, 0, // Front face (z=0)
    4, 5, 6, 6, 7, 4, // Back face (z=-1)
    0, 4, 7, 3, 0, 7, // Bottom (y=0)
    5, 1, 6, 1, 2, 6, // Top (y=1)
    6, 2, 7, 2, 3, 7, // Right (x=1)
    4, 0, 5, 0, 1, 5, // Left (x=0)
];