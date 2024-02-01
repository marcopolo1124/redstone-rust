pub use super::*;
use std::collections::VecDeque;

fn get_extended(chunks: &mut Chunks, x: i128, y: i128) -> Option<&mut bool> {
    let blk = chunks.get_block(x, y);
    if
        let Some(Block { mechanism: Some(MechanismKind::Piston { ref mut extended, .. }), .. }) =
            blk
    {
        Some(extended)
    } else {
        None
    }
}

fn get_movable(chunks: &mut Chunks, x: i128, y: i128) -> Option<&mut bool> {
    let blk = chunks.get_block(x, y);
    if let Some(Block { movable, .. }) = blk {
        Some(movable)
    } else {
        None
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
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32,
    texture_to_block_map: &HashMap<TextureName, Block>
) {
    let maybe_blk = chunks.get_block(x, y);
    let blk = if let Some(blk) = maybe_blk {
        blk
    } else {
        return;
    };

    let Block { orientation, mechanism, redstone, movable, .. } = blk;
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
                propagate_signal_at(
                    chunks,
                    x,
                    y,
                    None,
                    0,
                    17,
                    None,
                    listeners,
                    propagation_queue,
                    calculations
                )
            } else if !on && signal <= 0 {
                propagate_signal_at(
                    chunks,
                    x,
                    y,
                    None,
                    16,
                    16,
                    None,
                    listeners,
                    propagation_queue,
                    calculations
                )
            }
        }
        MechanismKind::Piston { ref mut extended, sticky } => {
            let is_sticky = *sticky;
            let piston_head = if is_sticky { STICKY_PISTON_HEAD } else { PISTON_HEAD };
            let mut traversed = HashSet::new();
            *movable = false;
            if !*extended && on {
                let (next_x, next_y) = orientation.get_next_coord(x, y);
                let affected_blocks = get_power(chunks, next_x, next_y, orientation, 12);
                let moved = move_blocks(
                    chunks,
                    next_x,
                    next_y,
                    orientation,
                    &affected_blocks,
                    listeners,
                    &mut commands,
                    &image_assets,
                    query,
                    propagation_queue,
                    calculations,
                    &mut traversed,
                    orientation.get_opposing(),
                    texture_to_block_map
                );
                if moved {
                    if let Some(extended) = get_extended(chunks, x, y) {
                        *extended = true;
                    }

                    place(
                        chunks,
                        piston_head,
                        orientation,
                        next_x,
                        next_y,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query,
                        propagation_queue,
                        calculations,
                        texture_to_block_map
                    );
                    listeners.update_entity(x, y);
                } else {
                    if let Some(movable) = get_movable(chunks, x, y) {
                        *movable = true;
                    }
                    listeners.turn_mechanism_on(x, y);
                }
            } else if *extended && !on {
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
                            query,
                            propagation_queue,
                            calculations
                        );
                    }
                }
                if is_sticky {
                    let (next_next_x, next_next_y) = orientation.get_next_coord(next_x, next_y);
                    let pull_dir = orientation.get_opposing();
                    let affected_blocks = get_power(chunks, next_next_x, next_next_y, pull_dir, 12);
                    move_blocks(
                        chunks,
                        next_next_x,
                        next_next_y,
                        pull_dir,
                        &affected_blocks,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query,
                        propagation_queue,
                        calculations,
                        &mut traversed,
                        pull_dir,
                        texture_to_block_map
                    );
                }
                if let Some(movable) = get_movable(chunks, x, y) {
                    *movable = true;
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
                if signal <= 0 && on {
                    propagate_signal_at(
                        chunks,
                        x,
                        y,
                        None,
                        16,
                        16,
                        None,
                        listeners,
                        propagation_queue,
                        calculations
                    );
                } else if signal > 0 && !on{
                    propagate_signal_at(
                        chunks,
                        x,
                        y,
                        None,
                        0,
                        17,
                        None,
                        listeners,
                        propagation_queue,
                        calculations
                    );
                }
            }
        }
        MechanismKind::Observer => {
            if on {
                propagate_signal_at(
                    chunks,
                    x,
                    y,
                    None,
                    16,
                    16,
                    None,
                    listeners,
                    propagation_queue,
                    calculations
                );

                listeners.turn_mechanism_off(x, y);
            } else {
                propagate_signal_at(
                    chunks,
                    x,
                    y,
                    None,
                    0,
                    17,
                    None,
                    listeners,
                    propagation_queue,
                    calculations
                );
            }
        }
    }
}

fn move_blocks(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    orientation: Orientation,
    affected_blocks: &HashSet<(i128, i128)>,
    listeners: &mut EventListeners,
    mut commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32,
    traversed: &mut HashSet<(i128, i128)>,
    from: Orientation,
    texture_to_block_map: &HashMap<TextureName, Block>
) -> bool {
    let maybe_blk = chunks.get_block(x, y);
    let blk = if let Some(blk) = maybe_blk {
        if blk.movable {
            *blk
        } else {
            return false;
        }
    } else {
        return true;
    };

    if !affected_blocks.contains(&(x, y)) {
        return false;
    }

    if traversed.contains(&(x, y)) {
        return false;
    }

    let (next_x, next_y) = orientation.get_next_coord(x, y);
    let moved = move_blocks(
        chunks,
        next_x,
        next_y,
        orientation,
        affected_blocks,
        listeners,
        &mut commands,
        &image_assets,
        query,
        propagation_queue,
        calculations,
        traversed,
        orientation.get_opposing(),
        texture_to_block_map,
    );

    if moved {
        if 
            place(
                chunks,
                blk,
                blk.orientation,
                next_x,
                next_y,
                listeners,
                &mut commands,
                &image_assets,
                query,
                propagation_queue,
                calculations,
                texture_to_block_map
            )
        {
            destroy(
                chunks,
                x,
                y,
                listeners,
                &mut commands,
                &image_assets,
                query,
                propagation_queue,
                calculations
            );
        }

        traversed.insert((next_x, next_y));
        traversed.insert((x, y));

        if blk.sticky {
            for neighbor_orientation in Orientation::iter() {
                let (next_x, next_y) = neighbor_orientation.get_next_coord(x, y);
                if
                    neighbor_orientation != orientation &&
                    neighbor_orientation != from &&
                    !traversed.contains(&(next_x, next_y))
                {
                    move_blocks(
                        chunks,
                        next_x,
                        next_y,
                        orientation,
                        affected_blocks,
                        listeners,
                        &mut commands,
                        &image_assets,
                        query,
                        propagation_queue,
                        calculations,
                        traversed,
                        neighbor_orientation.get_opposing(),
                        texture_to_block_map
                    );
                }
            }
        }
    }

    moved
}

fn get_power(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    orientation: Orientation,
    strength: u32
) -> HashSet<(i128, i128)> {
    let mut queue = VecDeque::from([(x, y, strength)]);
    let mut traversed: HashSet<(i128, i128)> = HashSet::new();

    while queue.len() > 0 {
        let (x, y, strength) = queue.pop_front().unwrap();
        traversed.insert((x, y));
        if let Some(Block { movable: true, sticky, .. }) = chunks.get_block_ref(x, y) {
            let (next_x, next_y) = orientation.get_next_coord(x, y);
            if let Some(_) = chunks.get_block_ref(next_x, next_y) {
                if !traversed.contains(&(next_x, next_y)) && strength > 0 {
                    queue.push_back((next_x, next_y, strength - 1));
                }
            }
            if *sticky {
                for adj in Orientation::iter() {
                    let (next_x, next_y) = adj.get_next_coord(x, y);
                    if let Some(_) = chunks.get_block_ref(next_x, next_y) {
                        if !traversed.contains(&(next_x, next_y)) && strength > 0 {
                            queue.push_back((next_x, next_y, strength - 1));
                        }
                    }
                }
            }
        }
    }

    if traversed.len() > 12 {
        return HashSet::new();
    }

    return traversed;
}
