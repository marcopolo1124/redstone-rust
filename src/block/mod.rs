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
    pub redstone_state: HashMap<(usize, usize), (bool, u8, Option<SignalType>)>,
    pub repeater_off: Listener,
    pub repeater_on: Listener,
    pub mechanism_on: Listener,
    pub mechanism_off: Listener,
}

impl EventListener {
    pub fn new() -> EventListener {
        EventListener {
            redstone_state: HashMap::new(),
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
            // println!("{prev_signal} {:?}", signal_type);
            // set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
            listeners.redstone_state.insert((x, y), (true, prev_signal, signal_type));
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
            // println!("prev {prev_signal} type, {:?}", signal_type);
            listeners.redstone_state.insert((x, y), (true, prev_signal, signal_type));
        }
    }
}

pub fn destroy(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) {
    let blk = &map[x][y];
    match *blk {
        Some(Block { kind, .. }) => {
            match kind {
                BlockKind::Redstone(Redstone { signal, output_ports, kind, .. }) => {
                    let next_blocks = get_next(&map, x, y, output_ports);
                    let signal_type = Some(get_signal_type(kind));
                    listeners.redstone_state.remove(&(x, y));
                    map[x][y] = None;
                    for (next_x, next_y) in next_blocks {
                        listeners.redstone_state.insert(
                            (next_x, next_y),
                            (false, signal, signal_type)
                        );
                    }
                }
                BlockKind::Opaque { strong_signal, weak_signal } => {
                    let next_blocks = get_next(&map, x, y, [true, true, true, true]);
                    listeners.redstone_state.remove(&(x, y));
                    map[x][y] = None;
                    if strong_signal > 0 {
                        for (next_x, next_y) in &next_blocks {
                            listeners.redstone_state.insert(
                                (*next_x, *next_y),
                                (false, strong_signal, Some(SignalType::Strong(false)))
                            );
                        }
                    }
                    if weak_signal > 0 {
                        for (next_x, next_y) in &next_blocks {
                            listeners.redstone_state.insert(
                                (*next_x, *next_y),
                                (false, weak_signal, Some(SignalType::Weak(false)))
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
