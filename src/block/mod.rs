pub use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Block {
    pub movable: bool,
    pub orientation: Orientation,
    pub texture_name: TextureName,
    pub symmetric: bool,
    pub redstone: Option<Redstone>,
    pub mechanism: Option<MechanismKind>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Redstone {
    pub signal: u8,
    pub signal_type: Option<SignalType>,
    pub kind: Option<RedstoneKind>,
    pub input_ports: [bool; 4],
    pub output_ports: [bool; 4],
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MechanismKind {
    RedstoneTorch,
    Repeater,
    Piston{extended: bool},
    StickyPiston{extended: bool},
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RedstoneKind {
    Mechanism,
    Dust,
    Block,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SignalType {
    Strong(bool),
    Weak(bool),
}

#[derive(Debug, PartialEq, Copy, Clone, Resource)]
pub enum Orientation {
    Up,
    Right,
    Down,
    Left,
}

impl Orientation {
    pub fn to_port_idx(&self) -> usize {
        match self {
            Orientation::Up => 0,
            Orientation::Right => 1,
            Orientation::Down => 2,
            Orientation::Left => 3,
        }
    }

    pub fn get_next_coord(&self, x: i128, y: i128) -> (i128, i128) {
        match self {
            Orientation::Up => (x - 1, y),
            Orientation::Right => (x, y + 1),
            Orientation::Down => (x + 1, y),
            Orientation::Left => (x, y - 1),
        }
    }

    pub fn get_opposing(&self) -> Orientation {
        match self {
            Orientation::Up => Orientation::Down,
            Orientation::Right => Orientation::Left,
            Orientation::Down => Orientation::Up,
            Orientation::Left => Orientation::Right,
        }
    }

    pub fn port_idx_to_orientation(idx: usize) -> Orientation {
        if idx == 0 {
            Orientation::Up
        } else if idx == 1 {
            Orientation::Right
        } else if idx == 2 {
            Orientation::Down
        } else if idx == 3 {
            Orientation::Left
        } else {
            panic!("port_idx must be < 4 and >= 0")
        }
    }
}

#[derive(Component)]
pub struct BlockComponent;
