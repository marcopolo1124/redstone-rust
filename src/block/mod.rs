mod redstone;
mod mechanism;

pub use redstone::*;
pub use mechanism::*;
pub use super::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub movable: bool,
    pub texture_name: TextureName,
    pub orientation: Orientation,
    pub kind: BlockKind,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextureName{
    Dirt,
    RedstoneTorch(bool),
    RedstoneDust(bool),
}

pub fn place(
    blk: &Block,
    x: usize,
    y: usize,
    facing: Orientation,
    map: &mut Map,
    redstone_listener: &mut Listener,
    redstone_source_listener: &mut Listener,
    mechanism_listener: &mut Listener,
    redstone_was_off: &mut bool
) {
    if map[x][y] != None {
        return;
    }
    match blk.kind {
        BlockKind::Redstone(Redstone{ signal, input_ports, output_ports, kind }) => {
            let redstone = place_redstone(
                redstone_listener,
                redstone_source_listener,
                x,
                y,
                signal,
                facing,
                kind,
                input_ports,
                output_ports
            );
            map[x][y] = Some(Block { kind: BlockKind::Redstone(redstone), orientation: facing, ..*blk });
            let prev_signal = get_prev_signal(map, x, y, redstone.input_ports);
            set_power(map, x, y, prev_signal, redstone_was_off);
        }
        BlockKind::Mechanism { kind } => {
            let mechanism = place_mechanism(map, mechanism_listener, x, y, facing, kind);
            map[x][y] = Some(Block { kind: mechanism, orientation: facing, ..*blk });
        }
        BlockKind::Transparent => map[x][y] = Some(Block { orientation: facing, ..*blk }),
        BlockKind::Opaque { .. } => map[x][y] = Some(Block { orientation: facing, ..*blk }),
    };
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum BlockKind {
    Transparent,
    Opaque {
        strong_signal: u8,
        weak_signal: u8,
    },
    Redstone (Redstone),
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
