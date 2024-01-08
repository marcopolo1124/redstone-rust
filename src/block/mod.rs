mod redstone;
mod mechanism;
mod texture;

pub use redstone::*;
pub use mechanism::*;
pub use super::*;
pub use texture::*;

#[derive(Debug, PartialEq, Clone, Copy, Component)]
pub struct Block {
    pub movable: bool,
    pub texture_name: TextureName,
    pub orientation: Orientation,
    pub kind: BlockKind,
}

pub type Listener = HashSet<(usize, usize)>;

#[derive(Resource, Debug, Clone)]
pub struct EventListener {
    pub redstone_torch_off: Listener,
    pub redstone_torch_on: Listener,
    pub repeater_off: Listener,
    pub repeater_on: Listener,
    pub mechanism_on: Listener,
    pub mechanism_off: Listener,
}

impl EventListener {
    pub fn new() -> EventListener {
        EventListener {
            redstone_torch_off: HashSet::new(),
            redstone_torch_on: HashSet::new(),
            repeater_off: HashSet::new(),
            repeater_on: HashSet::new(),
            mechanism_off: HashSet::new(),
            mechanism_on: HashSet::new(),
        }
    }
}

pub fn place(
    blk: &Block,
    x: usize,
    y: usize,
    facing: Orientation,
    map: &mut Map,
    listeners: &mut EventListener
) -> HashSet<(usize, usize)> {
    let mut traversed: HashSet<(usize, usize)> = HashSet::new();

    if map[x][y] != None {
        return traversed;
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
            // println!("{prev_signal} {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
        }
        BlockKind::Mechanism { kind } => {
            map[x][y] = Some(Block {
                kind: BlockKind::Mechanism { kind },
                orientation: facing,
                ..*blk
            });
        }
        BlockKind::Transparent => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
        }
        BlockKind::Opaque { .. } => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
            let (prev_signal, signal_type) = get_prev_signal(map, x, y, [true, true, true, true]);
            // // println!("prev {prev_signal} type, {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
        }
    }
    traversed
}

pub fn destroy(
    map: &mut Map,
    x: usize,
    y: usize,
    listeners: &mut EventListener
) -> HashSet<(usize, usize)> {
    let blk = &map[x][y];
    let mut traversed: HashSet<(usize, usize)> = HashSet::new();
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
                            listeners,
                            &mut traversed
                        );
                    }
                }
                BlockKind::Opaque { strong_signal, weak_signal } => {
                    let next_blocks = get_next(&map, x, y, [true, true, true, true]);
                    map[x][y] = None;
                    if strong_signal > 0 {
                        for (next_x, next_y) in &next_blocks {
                            set_power_to_0(
                                map,
                                *next_x,
                                *next_y,
                                Some(SignalType::Strong(false)),
                                strong_signal,
                                listeners,
                                &mut traversed
                            );
                        }
                    }
                    if weak_signal > 0 {
                        for (next_x, next_y) in &next_blocks {
                            set_power_to_0(
                                map,
                                *next_x,
                                *next_y,
                                Some(SignalType::Weak(false)),
                                weak_signal,
                                listeners,
                                &mut traversed
                            );
                        }
                    }
                }
                _ => {
                    map[x][y] = None;
                }
            }
        }

        _ => {}
    }
    traversed
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
