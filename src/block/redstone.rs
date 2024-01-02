pub use super::*;
use std::cmp;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum RedstoneKind {
    Torch,
    Repeater,
    Block,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Redstone {
    pub signal: u8,
    pub input_ports: Ports,
    pub output_ports: Ports,
    pub kind: RedstoneKind,
}

pub fn place_redstone(
    listener: &mut Listener,
    source_listener: &mut Listener,
    x: usize,
    y: usize,
    signal: u8,
    orientation: Orientation,
    kind: RedstoneKind,
    input_ports: Ports,
    output_ports: Ports
) -> Redstone {

    match kind {
        RedstoneKind::Block => listener.push((x, y)),
        _ => source_listener.push((x, y)),
    }

    Redstone {
        signal,
        input_ports: orient_port(orientation, input_ports),
        output_ports: orient_port(orientation, output_ports),
        kind,
    }
}

fn get_power_update_value(kind: RedstoneKind, current_signal: u8, signal: u8, redstone_was_off: &mut bool) -> u8 {
    match kind {
        RedstoneKind::Block => cmp::max(current_signal, signal),
        RedstoneKind::Torch => {
            if signal > 0 {
                *redstone_was_off = true;
                0
            } else { 16 }
        }
        RedstoneKind::Repeater => 0,
    }
}

pub fn set_power(map: &mut Map, x: usize, y: usize, input_signal: u8, redstone_was_off: &mut bool) {
    if input_signal <= 1 {
        return;
    }

    let blk: &mut Option<Block> = &mut map[x][y];
    let (updated, signal, output_ports) = match *blk {
        Some(
            Block { kind: BlockKind::Redstone(Redstone{ref mut signal, kind, output_ports, .. }), .. },
        ) => {
            let update_value = get_power_update_value(kind, *signal, input_signal, redstone_was_off);
            let updated = *signal < update_value;
            *signal = update_value;
            (updated, *signal, output_ports)
        }

        _ => (false, 0, [false, false, false, false]),
    };

    if updated {
        let next_blocks = get_next(map, x, y, output_ports);
        for (next_x, next_y) in next_blocks {
            set_power(map, next_x, next_y, signal - 1, redstone_was_off);
        }
    }
}

fn required_input_port(blk: &Option<Block>, ind: usize) -> bool {
    match *blk {
        Some(Block { kind: BlockKind::Redstone (Redstone{ input_ports, .. }), .. }) => { input_ports[ind] }
        Some(Block { kind: BlockKind::Opaque { .. }, .. }) => true,
        _ => false,
    }
}

fn get_next(map: &Map, x: usize, y: usize, output_ports: Ports) -> Vec<(usize, usize)> {
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

fn prev_output_signal(blk: &Option<Block>, ind: usize) -> u8 {
    match *blk {
        Some(Block { kind: BlockKind::Redstone (Redstone{ output_ports, signal, .. }), .. }) => {
            if output_ports[ind] { signal } else { 0 }
        }
        _ => 0,
    }
}

pub fn get_prev_signal(map: &Map, x: usize, y: usize, input_ports: Ports) -> u8 {
    let mut signal = 1;

    if input_ports[0] && x > 0 {
        signal = cmp::max(prev_output_signal(&map[x - 1][y], 2), signal);
    }
    if input_ports[1] && y + 1 < MAP_SIZE.0 {
        signal = cmp::max(prev_output_signal(&map[x][y + 1], 3), signal);
    }
    if input_ports[2] && x + 1 < MAP_SIZE.1 {
        signal = cmp::max(prev_output_signal(&map[x + 1][y], 0), signal);
    }
    if input_ports[3] && y > 0 {
        signal = cmp::max(prev_output_signal(&map[x][y - 1], 0), signal);
    }

    signal - 1
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
