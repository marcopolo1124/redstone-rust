pub use super::*;

mod block;
pub use block::*;

use bevy::utils::HashMap;

pub const CHUNK_SIZE: (i128, i128) = (16, 16);

pub type Map = [[Option<Block>; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize];
pub type EntityMap = [[Option<Entity>; CHUNK_SIZE.1 as usize]; CHUNK_SIZE.0 as usize];
#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub map: Map,
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
pub struct Chunks(pub HashMap<(i128, i128), Chunk>);

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
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32,
    texture_to_block_map: &HashMap<TextureName, Block>
) -> bool {
    let curr = chunks.get_block(x, y);
    if let Some(_) = curr {
        return false;
    }

    let mut blk_clone = *texture_to_block_map.get(&blk.texture_name).unwrap();
    blk_clone.mechanism = blk.mechanism;

    *calculations = 0;

    if blk.symmetric {
        orientation = Orientation::Up;
    }

    let redstone = if let Some(redstone) = blk_clone.redstone {
        Some(Redstone {
            input_ports: orientation.rotate_ports(redstone.input_ports),
            output_ports: orientation.rotate_ports(redstone.output_ports),
            signal_type_port_mapping: orientation.rotate_ports(redstone.signal_type_port_mapping),
            ..redstone
        })
    } else {
        None
    };

    *curr = Some(Block {
        orientation,
        redstone,
        ..blk_clone
    });

    if let Some(rs) = redstone {
        match rs.signal_type {
            Some(SignalType::Strong(true) | SignalType::Weak(true)) => {
                update_dust_ports(chunks, x, y, listeners, propagation_queue, calculations);

                for orientation in Orientation::iter() {
                    let (next_x, next_y) = orientation.get_next_coord(x, y);
                    update_dust_ports(
                        chunks,
                        next_x,
                        next_y,
                        listeners,
                        propagation_queue,
                        calculations
                    );
                    listeners.update_entity(next_x, next_y);
                    alert_neighbours(x, y, &chunks, listeners);
                }
            }
            _ => {}
        }

        let prev_redstone = get_max_prev(chunks, x, y);
        let (from_port, previous_signal, prev_signal_type) = prev_redstone;
        let transmitted_signal = if previous_signal > 0 { previous_signal - 1 } else { 0 };

        propagate_signal_at(
            chunks,
            x,
            y,
            from_port,
            transmitted_signal,
            previous_signal,
            prev_signal_type,
            listeners,
            propagation_queue,
            calculations
        );
    }

    update_entity(commands, &mut chunks, x, y, image_assets, query);
    alert_neighbours(x, y, &chunks, listeners);

    return true;
}

pub fn destroy(
    mut chunks: &mut Chunks,
    x: i128,
    y: i128,
    listeners: &mut EventListeners,
    commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32
) -> bool {
    let curr_blk = chunks.get_maybe_block(x, y);
    *calculations = 0;
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
            update_entity(commands, &mut chunks, x, y, image_assets, query);
            alert_neighbours(x, y, &chunks, listeners);

            for (idx, port) in curr_output_ports.iter().enumerate() {
                if *port {
                    let output_port_orientation = Orientation::port_idx_to_orientation(idx);
                    let (next_x, next_y) = output_port_orientation.get_next_coord(x, y);
                    let input_port_orientation = output_port_orientation.get_opposing();
                    // propagation_queue.append(
                    //     next_x,
                    //     next_y,
                    //     0,
                    //     Some(input_port_orientation),
                    //     curr_signal,
                    //     curr_signal_type
                    // );
                    propagate_signal_at(
                        chunks,
                        next_x,
                        next_y,
                        Some(input_port_orientation),
                        0,
                        curr_signal,
                        curr_signal_type,
                        listeners,
                        propagation_queue,
                        calculations
                    );
                }
            }

            if
                curr_signal_type == Some(SignalType::Strong(true)) ||
                curr_signal_type == Some(SignalType::Weak(true))
            {
                for orientation in Orientation::iter() {
                    let (next_x, next_y) = orientation.get_next_coord(x, y);
                    update_dust_ports(
                        chunks,
                        next_x,
                        next_y,
                        listeners,
                        propagation_queue,
                        calculations
                    );
                    listeners.update_entity(next_x, next_y);
                    alert_neighbours(x, y, &chunks, listeners);
                }
            }
        } else {
            *mutref = None;
            update_entity(commands, &mut chunks, x, y, image_assets, query);
            alert_neighbours(x, y, &chunks, listeners);
        }
    }

    listeners.remove_mechanism(x, y);

    return true;
}
