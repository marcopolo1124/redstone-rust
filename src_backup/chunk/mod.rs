pub use super::*;

use bevy::{ prelude::*, utils::HashMap };

pub const CHUNK_SIZE: (i128, i128) = (16, 16);

pub type Map = [[Option<Block>; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize];
pub type EntityMap = [[Option<Entity>; CHUNK_SIZE.1 as usize]; CHUNK_SIZE.0 as usize];

#[derive(Debug)]
pub struct Chunk {
    map: Map,
    entity_map: EntityMap,
}

impl Chunk {
    fn new() -> Chunk {
        Chunk {
            map: [[None; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize],
            entity_map: [[None; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize],
        }
    }
}

#[derive(Debug, Resource)]
pub struct Chunks(HashMap<(i128, i128), Chunk>);

impl Chunks {
    pub fn new() -> Chunks {
        Chunks(HashMap::new())
    }

    pub fn from_world_coord(x: i128, y: i128) -> ((i128, i128), (usize, usize)) {
        return (
            (x / CHUNK_SIZE.0, y / CHUNK_SIZE.1),
            (x.rem_euclid(CHUNK_SIZE.0) as usize, y.rem_euclid(CHUNK_SIZE.1) as usize),
        );
    }

    pub fn create_chunk_at(&mut self, chunk_x: i128, chunk_y: i128) {
        let chunk  = self.0.get(&(chunk_x, chunk_y));
        if let None = chunk {
            self.0.insert((chunk_x, chunk_y), Chunk::new());
        }
    }

    pub fn create_chunk_at_world(&mut self, x: i128, y: i128){
        let ((chunk_x, chunk_y), _) = Chunks::from_world_coord(x, y);
        self.create_chunk_at(chunk_x, chunk_y);
    }


    pub fn get_block(&mut self, x: i128, y: i128) -> &mut Option<Block> {
        self.create_chunk_at_world(x, y);
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get_mut(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &mut chk.map;
            return &mut map[u][v]
        } else{
            panic!("Chunk should exist");
        }
    }

    pub fn get_entity(&mut self, x: i128, y: i128) -> &mut Option<Entity> {
        self.create_chunk_at_world(x, y);
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get_mut(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &mut chk.entity_map;
            return &mut map[u][v]
        } else{
            panic!("Chunk should exist");
        }
    }

    pub fn draw_chunk(&mut self, x: i128, y: i128, commands: Commands, image_assets: ImageAssets){
        let chunk = self.0.get_mut(&(x, y));
        if let Some(mutref) = chunk {
            let map = mutref.map.clone();
            let entity_map = mutref.entity_map.clone();

            for (u, row) in map.iter().enumerate() {
                for (v, blk) in row.iter().enumerate() {
                    let sprite_bundle = get_sprite(u, v, x, y, blk, &image_assets);
                    commands.spawn(get_sprite(u, v, x, y, &None, image_assets.as_ref()));
                    let entity = commands.spawn((sprite_bundle, BlockComponent)).id();
        
                    entity_map[u][v] = Some(entity);
                }
            }
        }

    }
}