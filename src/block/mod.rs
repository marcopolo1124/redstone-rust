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
    pub repeater_state: HashMap<(usize, usize), bool>,
    pub mechanism_state: HashMap<(usize, usize), bool>,
    pub entity_map_update: Listener,
}

impl EventListener {
    pub fn new() -> EventListener {
        EventListener {
            redstone_state: HashMap::new(),
            repeater_state: HashMap::new(),
            mechanism_state: HashMap::new(),
            entity_map_update: HashSet::new(),
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
    let mut traversed = HashSet::new();

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
            // println!("{prev_signal} {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
            // listeners.redstone_state.insert((x, y), (true, prev_signal, signal_type));
        }
        BlockKind::Mechanism (Mechanism{kind, input_ports}) => {
            let oriented_input_port = orient_port(facing, input_ports);
           
            map[x][y] = Some(Block {
                kind: BlockKind::Mechanism(Mechanism{kind, input_ports: oriented_input_port}),
                orientation: facing,
                ..*blk
            });
            let (prev_signal, signal_type) = get_prev_signal(map, x, y, oriented_input_port);
            // println!("{prev_signal} {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
        }
        BlockKind::Transparent => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
        }
        BlockKind::Opaque { .. } => {
            map[x][y] = Some(Block { orientation: facing, ..*blk });
            let (prev_signal, signal_type) = get_prev_signal(map, x, y, [true, true, true, true]);
            // println!("prev {prev_signal} type, {:?}", signal_type);
            set_power(map, x, y, prev_signal, signal_type, listeners, &mut traversed);
            // listeners.redstone_state.insert((x, y), (true, prev_signal, signal_type));
        }
    }
    for (x, y) in traversed {
        listeners.entity_map_update.insert((x, y));
    }
}

pub fn destroy(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) {
    let blk = &map[x][y];
    let mut traversed = HashSet::new();
    match *blk {
        Some(Block { kind, .. }) => {
            match kind {
                BlockKind::Redstone(Redstone { signal, output_ports, kind, .. }) => {
                    let next_blocks = get_next(&map, x, y, output_ports);
                    let signal_type = Some(get_signal_type(kind));
                    listeners.redstone_state.remove(&(x, y));
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
                        // listeners.redstone_state.insert(
                        //     (next_x, next_y),
                        //     (false, signal, signal_type)
                        // );
                    }
                }
                BlockKind::Opaque { strong_signal, weak_signal } => {
                    let next_blocks = get_next(&map, x, y, [true, true, true, true]);
                    listeners.redstone_state.remove(&(x, y));
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
                            // listeners.redstone_state.insert(
                            //     (*next_x, *next_y),
                            //     (false, strong_signal, Some(SignalType::Strong(false)))
                            // );
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
                            // listeners.redstone_state.insert(
                            //     (*next_x, *next_y),
                            //     (false, weak_signal, Some(SignalType::Weak(false)))
                            // );
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
    for (x, y) in traversed {
        listeners.entity_map_update.insert((x, y));
    }
}

pub fn interact(map: &mut Map, x: usize, y: usize) {
    let block = &mut map[x][y];
    match block {
        Some(
            Block {
                kind: BlockKind::Redstone(
                    Redstone { kind: RedstoneKind::Repeater { ref mut tick, .. }, .. },
                ),
                ..
            },
        ) => {
            *tick = (*tick + 1) % 4;
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
    Mechanism(Mechanism),
}

#[derive(Debug, PartialEq, Copy, Clone, Resource)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

pub type Ports = [bool; 4];
