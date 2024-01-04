pub use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MechanismKind {
    Piston,
    ExtendedPiston,
    StickyPiston,
    StickyExtendedPiston,
}


// pub fn move