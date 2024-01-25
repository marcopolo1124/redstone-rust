pub use super::*;

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
                return Some(blk)
            } else{
                return None
            }
        } else {
            return None
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

    update_dust_ports(chunks, x, y);
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        update_dust_ports(chunks, next_x, next_y);
        update_entity(commands, &mut chunks, next_x, next_y, image_assets, query);
    }

    // chunks.print_chunks();

    let prev_redstone = get_max_prev(chunks, x, y);
    let (from_port, previous_signal, prev_signal_type) = prev_redstone;
    // println!("{:?}", prev_redstone);
    propagate_signal_at(
        chunks,
        x,
        y,
        from_port,
        previous_signal,
        previous_signal,
        prev_signal_type,
        listeners
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
            let transmitted_signal = if curr_signal < 1 { 0 } else { curr_signal - 1 };

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
                        transmitted_signal,
                        curr_signal_type,
                        listeners
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
        update_dust_ports(chunks, next_x, next_y);
        update_entity(commands, &mut chunks, next_x, next_y, image_assets, query);
    }
    return true;
}

pub fn get_max_prev(
    chunks: &mut Chunks,
    x: i128,
    y: i128
) -> (Option<Orientation>, u8, Option<SignalType>) {
    let chunk_blk = chunks.get_maybe_block(x, y);
    let curr_blk = if let Some(curr_blk) = chunk_blk {
        curr_blk
    } else {
        return (None, 0, None);
    };

    let (signal, input_ports, signal_type) = match curr_blk {
        Some(Block { redstone: Some(Redstone { signal, input_ports, signal_type, .. }), .. }) =>
            (*signal, input_ports.clone(), *signal_type),
        _ => {
            return (None, 0, None);
        }
    };
    let mut max_signal = signal;
    let mut max_signal_loc: Option<Orientation> = None;
    let mut max_signal_type = signal_type;

    for (idx, port) in input_ports.iter().enumerate() {
        if *port {
            let port_orientation = Orientation::port_idx_to_orientation(idx);
            let (next_x, next_y) = port_orientation.get_next_coord(x, y);
            let next_blk = chunks.get_maybe_block(next_x, next_y);
            if let Some(mutref) = next_blk {
                if let Some(blk) = mutref {
                    if let Some(Redstone { signal, output_ports, signal_type, .. }) = blk.redstone {
                        if
                            output_ports[port_orientation.get_opposing().to_port_idx()] &&
                            (signal > max_signal + 1 || (max_signal_loc == None && signal > 0))
                        {
                            max_signal = signal - 1;
                            max_signal_loc = Some(port_orientation);
                            max_signal_type = signal_type;
                        }
                    }
                }
            }
        }
    }
    (max_signal_loc, max_signal, max_signal_type)
}

pub fn propagate_signal_at(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    from_port: Option<Orientation>,
    input_signal: u8,
    previous_signal: u8,
    prev_signal_type: Option<SignalType>,
    listeners: &mut EventListeners
) {
    if input_signal <= 0 && previous_signal <= 1 {
        return;
    }

    let curr_blk = chunks.get_block(x, y);

    let (signal, signal_type, kind, input_ports, output_ports) = match *curr_blk {
        Some(
            Block {
                redstone: Some(
                    Redstone {
                        ref mut signal,
                        ref mut signal_type,
                        kind,
                        input_ports,
                        output_ports,
                    },
                ),
                ..
            },
        ) => (signal, signal_type, kind, input_ports, output_ports),
        _ => {
            return;
        }
    };

    // if there is an input signal, filter out all signals that will not continue propagation
    // Cases are: Weak signal from opaque block going to redstone dust
    // triggering a mechanism. In cases of redstone torch it will propagate on the next tick
    if let Some(from_port) = from_port {
        if !input_ports[from_port.to_port_idx()] {
            return;
        }
        match kind {
            Some(RedstoneKind::Dust) => {
                // this doesn't return and will continue
                if let Some(SignalType::Weak(false)) = prev_signal_type {
                    return;
                } else if let None = prev_signal_type {
                    return;
                }
            }
            Some(RedstoneKind::Mechanism) => {
                if input_signal > 0 {
                    listeners.turn_mechanism_on(x, y);
                } else {
                    listeners.turn_mechanism_off(x, y);
                }
            }
            None => {
                match prev_signal_type {
                    Some(SignalType::Weak(false)) | Some(SignalType::Strong(false)) => {
                        return;
                    }
                    _ => {}
                }
            }
            _ => {
                return;
            }
        }
    }

    //input signal > current signal -> turning on
    // previous signal > current signal -> can be either turning on or off
    if input_signal >= *signal || (previous_signal >= *signal && *signal > 0) {
        let output_signal_type = match *signal_type {
            Some(curr_signal_type) => {
                // redstone sources can only have their signal set externally and no
                if let SignalType::Strong(true) = curr_signal_type {
                    if let Some(_) = from_port {
                        return;
                    }
                }

                if
                    input_signal == 0 &&
                    ((curr_signal_type == SignalType::Weak(false) &&
                        prev_signal_type == Some(SignalType::Weak(true))) ||
                        (curr_signal_type == SignalType::Strong(false) &&
                            prev_signal_type == Some(SignalType::Strong(true))))
                {
                    *signal_type = None;
                }
                curr_signal_type
            }
            None => {
                match prev_signal_type {
                    Some(SignalType::Strong(true)) => {
                        *signal_type = Some(SignalType::Strong(false));
                        SignalType::Strong(false)
                    }
                    Some(SignalType::Weak(true)) => {
                        *signal_type = Some(SignalType::Weak(false));
                        SignalType::Weak(false)
                    }
                    _ => {
                        return;
                    }
                }
            }
        };
        // println!("output type {:?} {input_signal} {previous_signal} {}", output_signal_type, *signal);
        // the equality is only valid for cases where the entity is lit up by the system
        // The cases are when a mechanism can propagate siganl and is lit up by external circumstances
        // None in from_port means that the entity is lighting itself up
        if let Some(_) = from_port {
            if input_signal == *signal {
                return;
            }
        }
        let current_signal = *signal;
        *signal = input_signal;
        let transmitted_signal = if *signal > 0 { *signal - 1 } else { 0 };

        listeners.entity_map_update.insert((x, y));
        for (idx, port) in output_ports.iter().enumerate() {
            if *port {
                let port_orientation = Orientation::port_idx_to_orientation(idx);
                let input_port_orientation = port_orientation.get_opposing();
                let (next_x, next_y) = port_orientation.get_next_coord(x, y);
                propagate_signal_at(
                    chunks,
                    next_x,
                    next_y,
                    Some(input_port_orientation),
                    transmitted_signal,
                    current_signal,
                    Some(output_signal_type),
                    listeners
                );
            }
        }
    }

    if input_signal == 0 {
        let prev_redstone = get_max_prev(chunks, x, y);
        // println!("back {:?}", prev_redstone);
        let (from_port, previous_signal, prev_signal_type) = prev_redstone;
        propagate_signal_at(
            chunks,
            x,
            y,
            from_port,
            previous_signal,
            previous_signal,
            prev_signal_type,
            listeners
        );
    }
}

pub fn execute_mechanism(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    on: bool,
    listeners: &mut EventListeners,
    mut commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    let maybe_blk = chunks.get_block(x, y);
    let blk = if let Some(blk) = maybe_blk {
        blk
    } else {
        return;
    };

    let Block { orientation, mechanism, redstone, .. } = blk;
    let mechanism_kind = if let Some(mechanism_kind) = mechanism {
        mechanism_kind
    } else {
        return;
    };

    let orientation = *orientation;
    let redstone = *redstone;

    match mechanism_kind {
        MechanismKind::RedstoneTorch => {
            let signal = if let Some(Redstone { signal, .. }) = redstone {
                signal
            } else {
                return;
            };
            if on && signal > 0 {
                // println!("turning off");
                propagate_signal_at(chunks, x, y, None, 0, 16, None, listeners)
            } else if !on && signal <= 0 {
                // println!("turning on");
                propagate_signal_at(chunks, x, y, None, 16, 16, None, listeners)
            }
        }
        MechanismKind::Piston { ref mut extended, sticky } => {
            let is_sticky = *sticky;
            let piston_head = if is_sticky { STICKY_PISTON_HEAD } else { PISTON_HEAD };
            if !*extended && on {
                // println!("moved");
                let (next_x, next_y) = orientation.get_next_coord(x, y);
                *extended = true;
                if
                    move_blocks(
                        chunks,
                        next_x,
                        next_y,
                        orientation,
                        20,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query
                    )
                {
                    place(
                        chunks,
                        piston_head,
                        orientation,
                        next_x,
                        next_y,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query
                    );
                    listeners.update_entity(x, y);
                }
            } else if *extended && !on {
                // println!("retract here");
                *extended = false;
                listeners.update_entity(x, y);
                let (next_x, next_y) = orientation.get_next_coord(x, y);
                let next_block = chunks.get_block(next_x, next_y);
                if let Some(blk) = next_block {
                    if blk.texture_name == piston_head.texture_name {
                        destroy(
                            chunks,
                            next_x,
                            next_y,
                            listeners,
                            &mut commands,
                            &image_assets,
                            query
                        );
                    }
                }
                if is_sticky {
                    let (next_next_x, next_next_y) = orientation.get_next_coord(next_x, next_y);
                    let pull_dir = orientation.get_opposing();
                    move_blocks(
                        chunks,
                        next_next_x,
                        next_next_y,
                        pull_dir,
                        20,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query
                    );
                }
            }
        }
        MechanismKind::Repeater { countdown, tick } => {
            let signal = if let Some(Redstone { signal, .. }) = redstone {
                signal
            } else {
                return;
            };

            if *countdown < 0 {
                if (on && signal <= 0) || (!on && signal > 0) {
                    *countdown = *tick;
                }
            }

            if *countdown > 0 {
                *countdown -= 1;
                if on {
                    listeners.turn_mechanism_on(x, y);
                } else {
                    listeners.turn_mechanism_off(x, y);
                }
            } else if *countdown == 0 {
                *countdown -= 1;
                if signal <= 0 {
                    propagate_signal_at(chunks, x, y, None, 16, 16, None, listeners);
                    if !on {
                        listeners.turn_mechanism_off(x, y)
                    }
                } else {
                    propagate_signal_at(chunks, x, y, None, 0, 16, None, listeners);
                    if on {
                        listeners.turn_mechanism_on(x, y)
                    }
                }
            }
        }
    }
}

fn move_blocks(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    orientation: Orientation,
    strength: usize,
    listeners: &mut EventListeners,
    mut commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) -> bool {
    // println!("called moved blocks");
    if strength <= 0 {
        // println!("no strength");
        return false;
    }

    let maybe_blk = chunks.get_block(x, y);
    let blk = if let Some(blk) = maybe_blk {
        if blk.movable {
            *blk
        } else {
            // println!("block not movable");
            return false;
        }
    } else {
        // println!("none");
        return true;
    };

    let (next_x, next_y) = orientation.get_next_coord(x, y);
    let moved = move_blocks(
        chunks,
        next_x,
        next_y,
        orientation,
        strength - 1,
        listeners,
        &mut commands,
        &image_assets,
        query
    );
    if moved {
        // println!("place and destroy");
        place(
            chunks,
            blk,
            blk.orientation,
            next_x,
            next_y,
            listeners,
            &mut commands,
            &image_assets,
            query
        );
        destroy(chunks, x, y, listeners, &mut commands, &image_assets, query);
        listeners.update_entity(x, y);
    }

    moved
}

pub fn is_redstone(chunks: &Chunks, x: i128, y: i128) -> bool{
    let maybe_blk = chunks.get_block_ref(x, y);
    let blk = if let Some(blk) = maybe_blk {
        blk
    } else {
        return false;
    };

    let redstone = if let Block { redstone: Some(redstone), .. } = blk {
        redstone
    } else {
        return false;
    };
    if redstone.signal_type == Some(SignalType::Strong(true)) || redstone.signal_type == Some(SignalType::Weak(true)){
        true
    } else{
        false
    }
}

pub fn get_redstone_dust(chunks: &mut Chunks, x: i128, y: i128) -> Option<&mut Redstone> {
    let maybe_blk = chunks.get_block(x, y);
    let blk = if let Some(blk) = maybe_blk {
        blk
    } else {
        return None;
    };

    let redstone = if let Block { redstone: Some(redstone), .. } = blk {
        redstone
    } else {
        return None;
    };

    if redstone.kind != Some(RedstoneKind::Dust) {
        return None;
    }

    Some(redstone)
}

fn update_dust_ports(chunks: &mut Chunks, x: i128, y: i128) {
    let mut last_orientation = Orientation::Up;
    let mut count = 0;
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        let is_dust = is_redstone(chunks, next_x, next_y);
        let redstone = get_redstone_dust(chunks, x, y);
        let redstone_dust = if let Some(redstone_dust) = redstone {
            redstone_dust
        } else {
            return;
        };
        toggle_port(redstone_dust, orientation, false);
        
        if is_dust {
            toggle_port(redstone_dust, orientation, true);
            last_orientation = orientation;
            count += 1
        }
    }

    if count == 1{
        let redstone = get_redstone_dust(chunks, x, y);
        let redstone_dust = if let Some(redstone_dust) = redstone {
            redstone_dust
        } else {
            return;
        };
        toggle_port(redstone_dust, last_orientation.get_opposing(), true);
    }
}
