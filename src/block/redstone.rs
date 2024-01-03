pub use super::*;
use std::cmp;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RedstoneKind {
    Torch,
    Repeater,
    Block,
}

pub fn get_signal_type(kind: RedstoneKind) -> SignalType {
    match kind {
        RedstoneKind::Torch | RedstoneKind::Repeater => SignalType::Strong,
        RedstoneKind::Block => SignalType::Weak,
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Redstone {
    pub signal: u8,
    pub input_ports: Ports,
    pub output_ports: Ports,
    pub kind: RedstoneKind,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum SignalType {
    Weak,
    Strong,
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

fn get_power_update_value(kind: RedstoneKind, current_signal: u8, signal: u8) -> u8 {
    match kind {
        RedstoneKind::Block => cmp::max(current_signal, signal),
        RedstoneKind::Torch => {
            // println!("torch found, input signal is {signal}");
            if signal == 0 {
                16
            } else {
                current_signal
            }
        }
        RedstoneKind::Repeater => current_signal,
    }
}

pub fn set_power_to_0(
    map: &mut Map,
    x: usize,
    y: usize,
    signal_type: Option<SignalType>,
    prev_signal: u8,
    redstone_block_on_delay: &mut HashSet<(usize, usize)>,
    redstone_block_off_delay: &mut HashSet<(usize, usize)>
) {
    let blk: &mut Option<Block> = &mut map[x][y];
    let (output_ports, input_ports, curr_signal, signal_type) = match *blk {
        Some(
            Block {
                kind: BlockKind::Redstone(
                    Redstone { ref mut signal, kind, output_ports, input_ports, .. },
                ),
                ..
            },
        ) => {
            let curr_signal = *signal;
            if prev_signal > curr_signal {
                *signal = 0;
            }
            match kind {
                RedstoneKind::Torch => {
                    redstone_block_on_delay.insert((x, y));
                }
                _ => (),
            }
            (output_ports, input_ports, curr_signal, Some(get_signal_type(kind)))
        }
        Some(
            Block { kind: BlockKind::Opaque { ref mut strong_signal, ref mut weak_signal }, .. },
        ) => {
            match signal_type {
                Some(SignalType::Weak) => {
                    *weak_signal = 0;
                    ([false, false, false, false], [false, false, false, false], 0, None)
                }
                Some(SignalType::Strong) => {
                    let curr_signal = *strong_signal;
                    *strong_signal = 0;
                    ([true, true, true, true], [true, true, true, true], curr_signal, None)
                }
                _ => ([false, false, false, false], [false, false, false, false], 0, None),
            }
        }

        _ => ([false, false, false, false], [false, false, false, false], 0, None),
    };

    if curr_signal >= prev_signal || curr_signal == 0 {
        return;
    }

    let next_coord = get_next(map, x, y, output_ports);
    for (next_x, next_y) in next_coord {
        set_power_to_0(
            map,
            next_x,
            next_y,
            signal_type,
            curr_signal,
            redstone_block_on_delay,
            redstone_block_off_delay
        );
    }

    let (prev_signal, signal_type) = get_prev_signal(map, x, y, input_ports);
    if prev_signal + 1 >= curr_signal {
        set_power(map, x, y, prev_signal, signal_type, redstone_block_off_delay);
    }
}

pub fn set_power(
    map: &mut Map,
    x: usize,
    y: usize,
    input_signal: u8,
    signal_type: Option<SignalType>,
    redstone_block_off_delay: &mut HashSet<(usize, usize)>
) {
    let blk: &mut Option<Block> = &mut map[x][y];
    let (updated, signal, output_ports, signal_type) = match *blk {
        Some(
            Block {
                kind: BlockKind::Redstone(Redstone { ref mut signal, kind, output_ports, .. }),
                ..
            },
        ) => {
            let update_value = get_power_update_value(kind, *signal, input_signal);
            let updated = *signal < update_value;
            *signal = update_value;
            match kind {
                RedstoneKind::Torch => {
                    if input_signal > 0 {
                        redstone_block_off_delay.insert((x, y));
                    }
                }
                _ => (),
            }
            (updated, *signal, output_ports, Some(get_signal_type(kind)))
        }
        Some(
            Block { kind: BlockKind::Opaque { ref mut strong_signal, ref mut weak_signal }, .. },
        ) => {
            match signal_type {
                Some(SignalType::Strong) => {
                    *strong_signal = input_signal;
                    (true, input_signal, [true, true, true, true], None)
                }
                Some(SignalType::Weak) => {
                    *weak_signal = input_signal;
                    (false, 0, [false, false, false, false], None)
                }
                None => (false, 0, [false, false, false, false], None),
            }
        }

        _ => (false, 0, [false, false, false, false], None),
    };

    if signal <= 1 {
        return;
    }

    if updated {
        let next_blocks = get_next(map, x, y, output_ports);
        for (next_x, next_y) in next_blocks {
            set_power(map, next_x, next_y, signal - 1, signal_type, redstone_block_off_delay);
        }
    }
}

fn required_input_port(blk: &Option<Block>, ind: usize) -> bool {
    match *blk {
        Some(Block { kind: BlockKind::Redstone(Redstone { input_ports, .. }), .. }) => {
            input_ports[ind]
        }
        Some(Block { kind: BlockKind::Opaque { .. }, .. }) => true,
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
            if output_ports[ind] { (signal, Some(get_signal_type(kind))) } else { (0, None) }
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
        let (curr_signal, curr_signal_type) = prev_output_signal(&map[x][y - 1], 0);
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
