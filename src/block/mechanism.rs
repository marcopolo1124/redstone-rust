pub use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MechanismKind {
    Piston,
    ExtendedPiston,
    StickyPiston,
    StickyExtendedPiston,
}

pub fn place_mechanism(
    map: &mut Map,
    mechanism_listener: &mut Listener,
    x: usize,
    y: usize,
    orientation: Orientation,
    kind: MechanismKind
) -> BlockKind {
    mechanism_listener.push((x, y));
    BlockKind::Mechanism { kind }
}
