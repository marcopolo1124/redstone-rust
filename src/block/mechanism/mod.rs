pub use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MechanismKind {
    Piston,
    ExtendedPiston,
    StickyPiston,
    StickyExtendedPiston,
}

pub fn move_blocks(map: &mut Map, x: i16, y: i16, orientation: Orientation, strength: usize, listeners: &mut EventListener) -> bool{
    if strength <= 0 {
        return false
    };

    if (x < 0 || x >= MAP_SIZE.1 as i16) && (y < 0 || x >= MAP_SIZE.0 as i16){
        return false
    };

    let blk = map[x as usize][y as usize].clone();
    let blk = match blk {
        Some(block) => {
            if block.movable {
                block
            } else {
                return false
            }
        }
        None => return true
    };


    let (next_x, next_y) = match orientation{
        Orientation::Up => (x - 1, y),
        Orientation::Right => (x, y + 1),
        Orientation::Down => (x + 1, y),
        Orientation::Left => (x, y - 1)
    };

    let moved = move_blocks(map, next_x, next_y, orientation, strength - 1, listeners);
    if moved {
        place(&blk, next_x as usize, next_y as usize, blk.orientation, map, listeners);
        destroy(map, x as usize, y as usize, listeners);
    };

    moved

}