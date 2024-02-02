pub use super::*;

pub fn propagate_signal_at(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    from_port: Option<Orientation>,
    input_signal: u8,
    previous_signal: u8,
    prev_signal_type: Option<SignalType>,
    listeners: &mut EventListeners,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32
) {
    if (input_signal <= 0 && previous_signal <= 1) || input_signal == 1 {
        return;
    }

    let curr_blk = chunks.get_block(x, y);

    let (
        signal,
        signal_type,
        kind,
        input_ports,
        output_ports,
        signal_type_port_mapping,
        mechanism,
    ) = match *curr_blk {
        Some(
            Block {
                redstone: Some(
                    Redstone {
                        ref mut signal,
                        ref mut signal_type,
                        kind,
                        input_ports,
                        output_ports,
                        signal_type_port_mapping,
                    },
                ),
                mechanism,
                ..
            },
        ) =>
            (
                signal,
                signal_type,
                kind,
                input_ports,
                output_ports,
                signal_type_port_mapping,
                mechanism,
            ),
        _ => {
            return;
        }
    };

    if let Some(from_port) = from_port {
        if !input_ports[from_port.to_port_idx()] {
            return;
        }

        match kind {
            Some(RedstoneKind::Dust) => {
                if let Some(SignalType::Weak(false)) = prev_signal_type {
                    return;
                } else if let None = prev_signal_type {
                    return;
                }
            }
            Some(RedstoneKind::Mechanism) => {
                let is_redstone = match mechanism {
                    Some(MechanismKind::RedstoneTorch) | Some(MechanismKind::Repeater { .. }) =>
                        true,
                    _ => false,
                };
                if input_signal > 0 {
                    listeners.turn_mechanism_on(x, y, is_redstone);
                } else {
                    //  println!("turn off {x} {y}");
                    listeners.turn_mechanism_off(x, y, is_redstone);
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

    *calculations += 1;
    if *calculations > 10000 {
        propagation_queue.append(x, y, input_signal, from_port, previous_signal, prev_signal_type);
        return;
    }

    if
        input_signal >= *signal ||
        (previous_signal == *signal + 1 && *signal > 0 && input_signal == 0)
    {
        if let Some(_) = from_port {
            if input_signal == *signal && input_signal > 0 {
                return;
            }
        }

        let output_signal_type = match *signal_type {
            Some(curr_signal_type) => {
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
                        if input_signal > 0 {
                            *signal_type = Some(SignalType::Strong(false));
                        }

                        SignalType::Strong(false)
                    }
                    Some(SignalType::Weak(true)) => {
                        if input_signal > 0 {
                            *signal_type = Some(SignalType::Weak(false));
                        }

                        SignalType::Weak(false)
                    }
                    _ => {
                        return;
                    }
                }
            }
        };

        let current_signal = *signal;
        *signal = input_signal;
        let transmitted_signal = if input_signal > 0 { input_signal - 1 } else { 0 };

        listeners.entity_map_update.insert((x, y));
        for (idx, port) in output_ports.iter().enumerate() {
            if *port {
                let port_orientation = Orientation::port_idx_to_orientation(idx);
                let input_port_orientation = port_orientation.get_opposing();
                let (next_x, next_y) = port_orientation.get_next_coord(x, y);
                let port_signal_type = signal_type_port_mapping[idx];
                let mut port_output_signal_type = output_signal_type;
                if let Some(signal_type) = port_signal_type {
                    port_output_signal_type = signal_type;
                }

                propagate_signal_at(
                    chunks,
                    next_x,
                    next_y,
                    Some(input_port_orientation),
                    transmitted_signal,
                    current_signal,
                    Some(port_output_signal_type),
                    listeners,
                    propagation_queue,
                    calculations
                );
            }
        }
    }
    if input_signal == 0 {
        listeners.repropagate(x, y)
    }
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

    let mut max_signal = signal + 1;
    let mut max_signal_loc: Option<Orientation> = None;
    let mut max_signal_type = signal_type;

    for (idx, port) in input_ports.iter().enumerate() {
        if *port {
            let port_orientation = Orientation::port_idx_to_orientation(idx);
            let (next_x, next_y) = port_orientation.get_next_coord(x, y);
            let next_blk = chunks.get_maybe_block(next_x, next_y);
            if
                let Some(
                    Some(
                        Block {
                            redstone: Some(
                                Redstone {
                                    signal,
                                    output_ports,
                                    signal_type,
                                    signal_type_port_mapping,
                                    ..
                                },
                            ),
                            ..
                        },
                    ),
                ) = next_blk
            {
                if
                    output_ports[port_orientation.get_opposing().to_port_idx()] &&
                    (*signal >= max_signal || (max_signal_loc == None && *signal > 0))
                {
                    max_signal = *signal;
                    max_signal_loc = Some(port_orientation);
                    let output_port_signal_type =
                        signal_type_port_mapping[port_orientation.get_opposing().to_port_idx()];
                    let mut signal_type = *signal_type;
                    if let Some(sig_type) = output_port_signal_type {
                        signal_type = Some(sig_type);
                    }

                    let max_type_value = match max_signal_type {
                        Some(SignalType::Strong(true)) => 10,
                        Some(SignalType::Strong(false)) => 9,
                        Some(SignalType::Weak(true)) => 8,
                        Some(SignalType::Weak(false)) => 7,
                        None => 0,
                    };

                    let type_value = match signal_type {
                        Some(SignalType::Strong(true)) => 10,
                        Some(SignalType::Strong(false)) => 9,
                        Some(SignalType::Weak(true)) => 8,
                        Some(SignalType::Weak(false)) => 7,
                        None => 0,
                    };

                    if type_value > max_type_value {
                        max_signal_type = signal_type;
                    }
                }
            }
        }
    }
    (max_signal_loc, max_signal, max_signal_type)
}

pub fn is_redstone(chunks: &Chunks, x: i128, y: i128, input_port: Orientation) -> bool {
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
    if
        (redstone.input_ports[input_port.to_port_idx()] ||
            redstone.kind == Some(RedstoneKind::Dust) ||
            redstone.output_ports[input_port.to_port_idx()]) &&
        (redstone.signal_type == Some(SignalType::Strong(true)) ||
            redstone.signal_type == Some(SignalType::Weak(true)))
    {
        true
    } else {
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

pub fn update_dust_ports(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    listeners: &mut EventListeners,
    propagation_queue: &mut PropagationQueue,
    calculations: &mut u32
) {
    let mut last_orientation = Orientation::Up;
    let mut count = 0;
    let mut changed: bool = false;
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        let is_dust_and_open = is_redstone(chunks, next_x, next_y, orientation.get_opposing());
        let redstone = get_redstone_dust(chunks, x, y);
        let redstone_dust = if let Some(redstone_dust) = redstone {
            redstone_dust
        } else {
            return;
        };
        let initial = redstone_dust.output_ports[orientation.to_port_idx()];
        toggle_port(redstone_dust, orientation, false);
        let mut current = false;

        if is_dust_and_open {
            current = true;
            toggle_port(redstone_dust, orientation, true);
            last_orientation = orientation;
            count += 1;
        }

        if current != initial {
            changed = true;
        }
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        let signal = redstone_dust.signal;
        let signal_type = redstone_dust.signal_type;

        propagate_signal_at(
            chunks,
            next_x,
            next_y,
            Some(orientation.get_opposing()),
            0,
            signal,
            signal_type,
            listeners,
            propagation_queue,
            calculations
        );
    }

    if count == 1 {
        let redstone = get_redstone_dust(chunks, x, y);
        let redstone_dust = if let Some(redstone_dust) = redstone {
            redstone_dust
        } else {
            return;
        };
        let opposing_port = last_orientation.get_opposing();

        toggle_port(redstone_dust, opposing_port, true);
    } else if count == 0 {
        for orientation in Orientation::iter() {
            let redstone = get_redstone_dust(chunks, x, y);
            let redstone_dust = if let Some(redstone_dust) = redstone {
                redstone_dust
            } else {
                return;
            };
            toggle_port(redstone_dust, orientation, true);
        }
    }

    if changed {
        let redstone = get_redstone_dust(chunks, x, y);
        let dust = if let Some(rs) = redstone {
            rs
        } else {
            return;
        };
        let prev_signal = dust.signal + 1;
        propagate_signal_at(
            chunks,
            x,
            y,
            None,
            0,
            prev_signal,
            None,
            listeners,
            propagation_queue,
            calculations
        );
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
}
