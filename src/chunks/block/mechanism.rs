pub use super::*;

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
                    calculations,
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
                    calculations,
                )
            }
        }
        MechanismKind::Piston { ref mut extended, sticky } => {
            let is_sticky = *sticky;
            let piston_head = if is_sticky { STICKY_PISTON_HEAD } else { PISTON_HEAD };
            if !*extended && on {
                let (next_x, next_y) = orientation.get_next_coord(x, y);
                let moved = move_blocks(
                    chunks,
                    next_x,
                    next_y,
                    orientation,
                    20,
                    listeners,
                    &mut commands,
                    &image_assets,
                    query,
                    propagation_queue,
                    calculations,
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
                    );
                    listeners.update_entity(x, y);
                } else {
                    listeners.turn_mechanism_on(x, y)
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
                            calculations,
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
                        query,
                        propagation_queue,
                        calculations,
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
                        calculations,
                    );
                    if !on {
                        listeners.turn_mechanism_off(x, y)
                    }
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
                        calculations,
                    );
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
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32,
) -> bool {
    if strength <= 0 {
        return false;
    }

    let maybe_blk = chunks.get_block(x, y);
    let mut blk = if let Some(blk) = maybe_blk {
        if blk.movable {
            *blk
        } else {
            return false;
        }
    } else {
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
        query,
        propagation_queue,
        calculations,
    );
    if moved {
        if
            let Block { redstone: Some(Redstone { ref mut signal, ref mut signal_type, .. }), .. } =
                blk
        {
            if
                *signal_type == Some(SignalType::Strong(false)) ||
                *signal_type == Some(SignalType::Weak(false))
            {
                *signal = 0;
                *signal_type = None;
            }
        }

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
        );
        destroy(
            chunks,
            x,
            y,
            listeners,
            &mut commands,
            &image_assets,
            query,
            propagation_queue,
            calculations,
        );
        listeners.update_entity(x, y);
    }

    moved
}
