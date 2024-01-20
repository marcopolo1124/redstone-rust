mod repeater;
mod torch;


pub use super::*;
pub use repeater::*;
pub use torch::*;
use std::cmp;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RedstoneKind {
    Torch,
    Repeater {
        tick: i16,
        countdown: i16,
    },
    Block,
}

pub fn get_signal_type(kind: RedstoneKind) -> SignalType {
    match kind {
        RedstoneKind::Torch | RedstoneKind::Repeater { .. } => SignalType::Strong(true),
        RedstoneKind::Block => SignalType::Weak(true),
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Redstone {
    pub signal: u8,
    pub input_ports: Ports,
    pub output_ports: Ports,
    pub kind: RedstoneKind,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum SignalType {
    Weak(bool),
    Strong(bool),
}

pub fn place_redstone(
    signal: u8,
    orientation: Orientation,
    kind: RedstoneKind,
    input_ports: Ports,
    output_ports: Ports
) -> Redstone {
    Redstone {
        signal,
        input_ports: orient_port(orientation, input_ports),
        output_ports: orient_port(orientation, output_ports),
        kind,
    }
}

pub fn set_power_to_0(
    map: &mut Map,
    x: usize,
    y: usize,
    signal_type: Option<SignalType>,
    prev_signal: u8,
    listeners: &mut EventListener,
) {
    let blk: &mut Option<Block> = &mut map[x][y];
    let values = match *blk {
        Some(
            Block {
                kind: BlockKind::Redstone(
                    Redstone { ref mut signal, ref mut kind, output_ports, input_ports, .. },
                ),
                ref mut texture_name,
                ..
            },
        ) => {
            let update_values = update_redstone_signal_to_0(
                x,
                y,
                signal_type,
                kind,
                texture_name,
                signal,
                prev_signal,
                listeners
            );
            match update_values {
                None => None,
                Some((curr_signal, signal_type)) =>
                    Some((output_ports, input_ports, curr_signal, signal_type)),
            }
        }
        Some(
            Block { kind: BlockKind::Opaque { ref mut strong_signal, ref mut weak_signal }, .. },
        ) => {
            match signal_type {
                Some(SignalType::Weak(true)) => {
                    let curr_signal = *weak_signal;
                    if prev_signal > curr_signal {
                        *weak_signal = 0;
                    }
                    Some((
                        [true, true, true, true],
                        [true, true, true, true],
                        curr_signal,
                        Some(SignalType::Weak(false)),
                    ))
                }
                Some(SignalType::Strong(true)) => {
                    let curr_signal = *strong_signal;
                    if prev_signal > curr_signal {
                        *strong_signal = 0;
                    }
                    Some((
                        [true, true, true, true],
                        [true, true, true, true],
                        curr_signal,
                        Some(SignalType::Strong(false)),
                    ))
                }
                _ => None,
            }
        }
        Some(Block { kind: BlockKind::Mechanism (Mechanism {input_ports, ref mut signal, ..}), .. }) => {
            let curr_signal = *signal;
            if prev_signal > curr_signal {
                *signal = 0;
                listeners.mechanism_state.insert((x, y), false);
            }
            Some(([false, false, false, false], input_ports, curr_signal, None))
        }

        _ => None,
    };

    let (output_ports, input_ports, curr_signal, signal_type) = match values {
        None => {
            return;
        }
        Some(val) => val,
    };

    if curr_signal >= prev_signal || curr_signal == 0 {
        ////println!("returned");
        return;
    }

    listeners.entity_map_update.insert((x, y), false);

    let next_coord = get_next(map, x, y, output_ports);
    for (next_x, next_y) in next_coord {
        set_power_to_0(map, next_x, next_y, signal_type, curr_signal, listeners);
    }

    let (prev_signal, signal_type) = get_prev_signal(map, x, y, input_ports);
    if prev_signal + 1 >= curr_signal {
        ////println!("propagation attempt");
        // debug_map(map);
        set_power(map, x, y, prev_signal, signal_type, listeners);
    }
}

pub fn set_power(
    map: &mut Map,
    x: usize,
    y: usize,
    input_signal: u8,
    signal_type: Option<SignalType>,
    listeners: &mut EventListener,
) {
    ////println!("current {x} {y}");
    let blk: &mut Option<Block> = &mut map[x][y];
    let values = match *blk {
        Some(
            Block {
                kind: BlockKind::Redstone(
                    Redstone { ref mut signal, ref mut kind, output_ports, .. },
                ),
                ref mut texture_name,
                ..
            },
        ) => {
            update_redstone_signal(
                signal_type,
                kind,
                signal,
                listeners,
                output_ports,
                input_signal,
                x,
                y,
                texture_name
            )
        }
        Some(
            Block { kind: BlockKind::Opaque { ref mut strong_signal, ref mut weak_signal }, .. },
        ) => {
            ////println!("{:?}", signal_type);
            match signal_type {
                Some(SignalType::Strong(true)) => {
                    *strong_signal = cmp::max(input_signal, *strong_signal);
                    Some((input_signal, [true, true, true, true], Some(SignalType::Strong(false))))
                }
                Some(SignalType::Weak(true)) => {
                    *weak_signal = cmp::max(input_signal, *weak_signal);
                    Some((input_signal, [true, true, true, true], Some(SignalType::Weak(false))))
                }
                Some(SignalType::Strong(false)) | Some(SignalType::Weak(false)) | None => None,
            }
        }
        Some(Block { kind: BlockKind::Mechanism (Mechanism {ref mut signal, ..}), .. }) => {
            
            if input_signal > *signal{
                listeners.mechanism_state.insert((x, y), true);
                *signal = input_signal;
                ////println!("setting true {:?} {x} {y}", listeners.mechanism_stat   e);
            };
            None
        }

        _ => None,
    };

    match values {
        None => {
            return;
        }
        Some((signal, output_ports, signal_type)) => {
            listeners.entity_map_update.insert((x, y), false);
            let next_blocks = get_next(map, x, y, output_ports);
            for (next_x, next_y) in next_blocks {
                if signal > 1 {
                    set_power(map, next_x, next_y, signal - 1, signal_type, listeners);
                }
            }
        }
    }
}

fn update_redstone_signal(
    signal_type: Option<SignalType>,
    kind: &mut RedstoneKind,
    signal: &mut u8,
    listeners: &mut EventListener,
    output_ports: Ports,
    input_signal: u8,
    x: usize,
    y: usize,
    texture_name: &mut TextureName
) -> Option<(u8, Ports, Option<SignalType>)> {
    ////println!("fuffuufufuf {input_signal}");
    match signal_type {
        Some(SignalType::Strong(_)) | Some(SignalType::Weak(true)) | None => {
            match *kind {
                RedstoneKind::Torch => {
                    if input_signal <= 0 {
                        *signal = 16;
                        *texture_name = TextureName::RedstoneTorch(true);
                        Some((16, output_ports, Some(SignalType::Strong(true))))
                    } else {
                        listeners.redstone_state.insert((x, y), (false, 20, None));
                        None
                    }
                }
                RedstoneKind::Repeater { .. } => {
                    if input_signal >= 20{
                        *signal = 16;
                        *texture_name = TextureName::Repeater(true);
                        Some((16, output_ports, Some(SignalType::Strong(true))))
                    } else if input_signal > 0{
                        listeners.repeater_state.insert((x, y), true);
                        None
                    } else {
                        None
                    }
                }
                RedstoneKind::Block => {
                    if input_signal > 0 {
                        if *signal < input_signal {
                            *signal = input_signal;
                            *texture_name = TextureName::RedstoneDust(true);
                            if input_signal > 1 {
                                Some((input_signal, output_ports, Some(SignalType::Weak(true))))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
        }
        Some(SignalType::Weak(false)) => None,
    }
}

fn update_redstone_signal_to_0(
    x: usize,
    y: usize,
    signal_type: Option<SignalType>,
    kind: &mut RedstoneKind,
    texture_name: &mut TextureName,
    signal: &mut u8,
    prev_signal: u8,
    listeners: &mut EventListener
) -> Option<(u8, Option<SignalType>)> {
    match signal_type {
        Some(SignalType::Strong(_)) | Some(SignalType::Weak(true)) | None => {
            let curr_signal = *signal;

            match *kind {
                RedstoneKind::Torch => {
                    if prev_signal < 20 {
                        listeners.redstone_state.insert((x, y), (true, 0, None));
                    } else {
                        *texture_name = TextureName::RedstoneTorch(false);
                        *signal = 0;
                    }
                }
                RedstoneKind::Repeater { .. } => {
                    if prev_signal < 20 {
                        listeners.repeater_state.insert((x, y), false);
                    } else {
                        *signal = 0;
                        *texture_name = TextureName::Repeater(false);
                    }
                }
                RedstoneKind::Block => {
                    if prev_signal > curr_signal {
                        *signal = 0;
                        *texture_name = TextureName::RedstoneDust(false);
                    }
                }
            }
            Some((curr_signal, Some(get_signal_type(*kind))))
        }
        _ => None,
    }
}

fn required_input_port(blk: &Option<Block>, ind: usize) -> bool {
    match *blk {
        Some(Block { kind: BlockKind::Redstone(Redstone { input_ports, .. }), .. }) => {
            input_ports[ind]
        }
        Some(Block { kind: BlockKind::Opaque { .. }, .. }) => true,
        Some(Block { kind: BlockKind::Mechanism (Mechanism{input_ports, ..}), .. }) => {
            input_ports[ind]
        }
        _ => false,
    }
}

pub fn get_next(map: &Map, x: usize, y: usize, output_ports: Ports) -> Vec<(usize, usize)> {
    let mut next_blk: Vec<(usize, usize)> = vec![];

    if output_ports[0] && x > 0 && required_input_port(&map[x - 1][y], 2) {
        next_blk.push((x - 1, y));
    }
    if output_ports[1] && y + 1 < MAP_SIZE.0 && required_input_port(&map[x][y + 1], 3) {
        next_blk.push((x, y + 1));
    }
    if output_ports[2] && x + 1 < MAP_SIZE.1 && required_input_port(&map[x + 1][y], 0) {
        next_blk.push((x + 1, y));
    }
    if output_ports[3] && y > 0 && required_input_port(&map[x][y - 1], 1) {
        next_blk.push((x, y - 1));
    }
    next_blk
}

fn prev_output_signal(blk: &Option<Block>, ind: usize) -> (u8, Option<SignalType>) {
    match *blk {
        Some(
            Block { kind: BlockKind::Redstone(Redstone { output_ports, signal, kind, .. }), .. },
        ) => {
            let signal_type = Some(get_signal_type(kind));
            if output_ports[ind] {
                (signal, signal_type)
            } else {
                (0, None)
            }
        }
        Some(Block { kind: BlockKind::Opaque { strong_signal, weak_signal }, .. }) => {
            if strong_signal > 0 {
                (strong_signal, Some(SignalType::Strong(false)))
            } else if weak_signal > 0 {
                (weak_signal, Some(SignalType::Weak(false)))
            } else {
                (0, None)
            }
        }
        _ => (0, None),
    }
}

pub fn get_prev_signal(
    map: &Map,
    x: usize,
    y: usize,
    input_ports: Ports
) -> (u8, Option<SignalType>) {
    let mut signal = 1;
    let mut signal_type = None;

    if input_ports[0] && x > 0 {
        let (curr_signal, curr_signal_type) = prev_output_signal(&map[x - 1][y], 2);
        if curr_signal > signal {
            signal = curr_signal;
            signal_type = curr_signal_type;
        }
    }
    if input_ports[1] && y + 1 < MAP_SIZE.0 {
        let (curr_signal, curr_signal_type) = prev_output_signal(&map[x][y + 1], 3);
        if curr_signal > signal {
            signal = curr_signal;
            signal_type = curr_signal_type;
        }
    }
    if input_ports[2] && x + 1 < MAP_SIZE.1 {
        let (curr_signal, curr_signal_type) = prev_output_signal(&map[x + 1][y], 0);
        if curr_signal > signal {
            signal = curr_signal;
            signal_type = curr_signal_type;
        }
    }
    if input_ports[3] && y > 0 {
        let (curr_signal, curr_signal_type) = prev_output_signal(&map[x][y - 1], 1);
        if curr_signal > signal {
            signal = curr_signal;
            signal_type = curr_signal_type;
        }
    }

    (signal - 1, signal_type)
}

pub fn orient_port(orientation: Orientation, ports: Ports) -> Ports {
    let shift = match orientation {
        Orientation::Up => 0,
        Orientation::Right => 1,
        Orientation::Down => 2,
        Orientation::Left => 3,
    };

    let mut oriented_ports: Ports = [true; 4];
    for i in 0..4 {
        oriented_ports[(i + shift) % 4] = ports[i];
    }

    oriented_ports
}
