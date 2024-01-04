mod redstone;
mod mechanism;

pub use redstone::*;
pub use mechanism::*;
pub use super::*;

#[derive(Debug, PartialEq, Clone, Component)]
pub struct Block {
    pub movable: bool,
    pub texture_name: TextureName,
    pub orientation: Orientation,
    pub kind: BlockKind,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextureName {
    Dirt,
    RedstoneTorch(bool),
    RedstoneDust(bool),
    Piston{extended: bool},
    StickyPiston{extended: bool},
    PistonHead,
    StickyPistonHead,
    Repeater{tick: u8, on: bool},
}

pub fn place(
    blk: &Block,
    x: usize,
    y: usize,
    facing: Orientation,
    map: &mut Map,
    redstone_block_off_delay: &mut HashSet<(usize, usize)>,
    mechanism_on: &mut HashSet<(usize, usize)>,
    repeater_on_listener:&mut HashSet<(usize, usize)>
) {
    if map[x][y] != None {
        return;
    }
    match blk.kind {
        BlockKind::Redstone(Redstone { signal, input_ports, output_ports, kind }) => {
            let redstone = place_redstone(signal, facing, kind, input_ports, output_ports);
            map[x][y] = Some(Block {
                kind: BlockKind::Redstone(redstone),
                orientation: facing,
                ..*blk
            });
            let (prev_signal, signal_type) = get_prev_signal(map, x, y, redstone.input_ports);
            set_power(map, x, y, prev_signal, signal_type, redstone_block_off_delay, mechanism_on, repeater_on_listener);
        }
        BlockKind::Mechanism { kind } => {
            map[x][y] = Some(Block { kind: BlockKind::Mechanism { kind }, orientation: facing, ..*blk });
        }
        BlockKind::Transparent => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
        }
        BlockKind::Opaque { .. } => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
            let (prev_signal, signal_type) = get_prev_signal(map, x, y, [true, true, true, true]);
            // println!("prev {prev_signal} type, {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, redstone_block_off_delay, mechanism_on, repeater_on_listener)
        }
    };
}

pub fn destroy(
    map: &mut Map,
    x: usize,
    y: usize,
    redstone_block_off_delay: &mut HashSet<(usize, usize)>,
    redstone_block_on_delay: &mut HashSet<(usize, usize)>,
    mechanism_on: &mut HashSet<(usize, usize)>,
    mechanism_off: &mut HashSet<(usize, usize)>,
    repeater_on_listener:&mut HashSet<(usize, usize)>,
    repeater_off_listner: &mut HashSet<(usize, usize)>
) {
    let blk = &map[x][y];
    match *blk {
        Some(Block { kind, .. }) => {
            match kind {
                BlockKind::Redstone(Redstone { signal, output_ports, kind, .. }) => {
                    let next_blocks = get_next(&map, x, y, output_ports);
                    let signal_type = Some(get_signal_type(kind));
                    map[x][y] = None;
                    for (next_x, next_y) in next_blocks {
                        set_power_to_0(
                            map,
                            next_x,
                            next_y,
                            signal_type,
                            signal,
                            redstone_block_on_delay,
                            redstone_block_off_delay,
                            mechanism_on,
                            mechanism_off,
                            repeater_on_listener,
                            repeater_off_listner
                        );
                    }
                }
                _ => {
                    map[x][y] = None;
                }
            }
        }

        _ => {
            return;
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BlockKind {
    Transparent,
    Opaque {
        strong_signal: u8,
        weak_signal: u8,
    },
    Redstone(Redstone),
    Mechanism {
        kind: MechanismKind,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

pub type Ports = [bool; 4];
pub type Listener = Vec<(usize, usize)>;
