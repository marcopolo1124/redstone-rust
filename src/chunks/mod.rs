pub use super::*;

mod block;
pub use block::*;


use bevy::utils::HashMap;

pub const CHUNK_SIZE: (i128, i128) = (16, 16);

pub type Map = [[Option<Block>; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize];
pub type EntityMap = [[Option<Entity>; CHUNK_SIZE.1 as usize]; CHUNK_SIZE.0 as usize];

#[derive(Debug, Clone, Copy)]
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

    fn print_chunk(&self) {
        for row in self.map {
            let mut debug = [0; CHUNK_SIZE.0 as usize];
            for (idx, x) in row.iter().enumerate() {
                if let Some(_) = x {
                    debug[idx] = 1;
                }
            }
            // println!("{:?}", debug);
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
        let chunk_x = x.div_euclid(CHUNK_SIZE.0);
        let chunk_y = y.div_euclid(CHUNK_SIZE.1);
        let u = x.rem_euclid(CHUNK_SIZE.0) as usize;
        let v = y.rem_euclid(CHUNK_SIZE.1) as usize;

        return ((chunk_x, chunk_y), (u, v));
    }

    pub fn get_chunk(&self, x: i128, y: i128) -> Option<&Chunk> {
        let ((chunk_x, chunk_y), _) = Chunks::from_world_coord(x, y);
        return self.0.get(&(chunk_x, chunk_y));
    }

    pub fn delete_chunk(&mut self, x: i128, y: i128) {
        let ((chunk_x, chunk_y), _) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get(&(chunk_x, chunk_y));
        if let Some(chk) = chunk {
            for r in chk.map {
                for blk in r {
                    if let Some(_) = blk {
                        return;
                    }
                }
            }
            self.0.remove(&(chunk_x, chunk_y));
        }
    }

    pub fn create_chunk_at(&mut self, chunk_x: i128, chunk_y: i128) {
        let chunk = self.0.get(&(chunk_x, chunk_y));
        if let None = chunk {
            self.0.insert((chunk_x, chunk_y), Chunk::new());
        }
    }

    pub fn create_chunk_at_world(&mut self, x: i128, y: i128) {
        let ((chunk_x, chunk_y), _) = Chunks::from_world_coord(x, y);
        self.create_chunk_at(chunk_x, chunk_y);
    }

    pub fn get_block(&mut self, x: i128, y: i128) -> &mut Option<Block> {
        self.create_chunk_at_world(x, y);
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get_mut(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &mut chk.map;
            return &mut map[u][v];
        } else {
            panic!("Chunk should exist");
        }
    }

    pub fn get_block_ref(&self, x: i128, y: i128) -> Option<&Block> {
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &chk.map;
            if let Some(blk) = &map[u][v] {
                return Some(blk);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    pub fn get_maybe_block(&mut self, x: i128, y: i128) -> Option<&mut Option<Block>> {
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get_mut(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &mut chk.map;
            return Some(&mut map[u][v]);
        } else {
            None
        }
    }

    pub fn get_entity(&mut self, x: i128, y: i128) -> &mut Option<Entity> {
        self.create_chunk_at_world(x, y);
        let (chunk_coord, (u, v)) = Chunks::from_world_coord(x, y);
        let chunk = self.0.get_mut(&chunk_coord);

        if let Some(chk) = chunk {
            let map = &mut chk.entity_map;
            return &mut map[u][v];
        } else {
            panic!("Chunk should exist");
        }
    }

    pub fn print_chunks(&self) {
        for (_, v) in self.0.iter() {
            // println!("chunk at {:?}", k);
            v.print_chunk();
        }
    }
}

pub fn place(
    mut chunks: &mut Chunks,
    blk: Block,
    mut orientation: Orientation,
    x: i128,
    y: i128,
    listeners: &mut EventListeners,
    commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) -> bool {
    // println!("");
    // println!("place");
    let curr = chunks.get_block(x, y);
    if let Some(_) = curr {
        return false;
    }

    if blk.symmetric {
        orientation = Orientation::Up;
    }

    let redstone = if let Some(redstone) = blk.redstone {
        Some(Redstone {
            input_ports: orientation.rotate_ports(redstone.input_ports),
            output_ports: orientation.rotate_ports(redstone.output_ports),
            ..redstone
        })
    } else {
        None
    };

    *curr = Some(Block {
        orientation,
        redstone,
        ..blk
    });

    update_dust_ports(chunks, x, y, listeners);
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        update_dust_ports(chunks, next_x, next_y, listeners);
        update_entity(commands, &mut chunks, next_x, next_y, image_assets, query);
    }

    // chunks.print_chunks();

    let prev_redstone = get_max_prev(chunks, x, y);
    let (from_port, previous_signal, prev_signal_type) = prev_redstone;
    let transmitted_signal = if previous_signal > 0 { previous_signal - 1 } else { 0 };
    // println!("prev redstone 2{:?}", prev_redstone);
    propagate_signal_at(
        chunks,
        x,
        y,
        from_port,
        transmitted_signal,
        previous_signal,
        prev_signal_type,
        listeners,
        false
    );
    update_entity(commands, &mut chunks, x, y, image_assets, query);
    return true;
}

pub fn destroy(
    mut chunks: &mut Chunks,
    x: i128,
    y: i128,
    listeners: &mut EventListeners,
    commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) -> bool {
    // println!("");
    let curr_blk = chunks.get_maybe_block(x, y);
    if let Some(mutref) = curr_blk {
        if
            let Some(
                Block { redstone: Some(Redstone { output_ports, signal, signal_type, .. }), .. },
            ) = mutref
        {
            let curr_signal = *signal;
            let curr_signal_type = *signal_type;
            let curr_output_ports = *output_ports;
            *mutref = None;
            // chunks.print_chunks();
            // let transmitted_signal = if curr_signal < 1 { 0 } else { curr_signal - 1 };

            for (idx, port) in curr_output_ports.iter().enumerate() {
                if *port {
                    let output_port_orientation = Orientation::port_idx_to_orientation(idx);
                    let (next_x, next_y) = output_port_orientation.get_next_coord(x, y);
                    let input_port_orientation = output_port_orientation.get_opposing();
                    propagate_signal_at(
                        chunks,
                        next_x,
                        next_y,
                        Some(input_port_orientation),
                        0,
                        curr_signal,
                        curr_signal_type,
                        listeners,
                        false
                    );
                }
            }
        } else {
            *mutref = None;
        }
    }

    listeners.remove_mechanism(x, y);
    update_entity(commands, &mut chunks, x, y, image_assets, query);
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        update_dust_ports(chunks, next_x, next_y, listeners);
        update_entity(commands, &mut chunks, next_x, next_y, image_assets, query);
    }
    return true;
}




