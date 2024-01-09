pub use super::*;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MechanismKind {
    Piston,
    ExtendedPiston,
    StickyPiston,
    StickyExtendedPiston,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Mechanism{
    pub kind: MechanismKind,
    pub input_ports: Ports
}

pub fn move_blocks(map: &mut Map, x: i16, y: i16, orientation: Orientation, strength: usize, listeners: &mut EventListener) -> bool{
    if strength <= 0 {
        return false
    };

    if (x < 0 || x >= (MAP_SIZE.1 as i16 - 1)) || (y < 0 || y >= (MAP_SIZE.0 as i16 - 1)){
        //println!("return");
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
        listeners.entity_map_update.insert((next_x as usize, next_y as usize));
        listeners.entity_map_update.insert((x as usize, y as usize));
    };

    moved

}

pub fn execute(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) -> bool {
    let block = map[x][y];
    match block{
        Some(Block{orientation, kind: BlockKind::Mechanism (Mechanism{kind, ..}), ..}) => {
            let (next_x, next_y) = match orientation{
                Orientation::Up => (x - 1, y),
                Orientation::Right => (x, y + 1),
                Orientation::Down => (x + 1, y),
                Orientation::Left => (x, y - 1)
            };
            match kind {

                MechanismKind::Piston => {
                    let moved = move_blocks(map, next_x as i16, next_y as i16, orientation, 20, listeners);
                    if moved {
                        destroy(map, x, y, listeners);
                        place(&EXTENDED_PISTON, x, y, orientation, map, listeners );
                        place(&PISTON_HEAD, next_x, next_y, orientation, map, listeners);
                        listeners.entity_map_update.insert((next_x as usize, next_y as usize));
                        listeners.entity_map_update.insert((x as usize, y as usize));
                    }
                    moved
                },
                MechanismKind::StickyPiston => {
                    let moved = move_blocks(map, next_x as i16, next_y as i16, orientation, 20, listeners);
                    if moved {
                        destroy(map, x, y, listeners);
                        place(&STICKY_EXTENDED_PISTON, x, y, orientation, map, listeners );
                        place(&PISTON_HEAD, next_x, next_y, orientation, map, listeners);
                        listeners.entity_map_update.insert((next_x as usize, next_y as usize));
                        listeners.entity_map_update.insert((x as usize, y as usize));
                    }
                    moved
                },
                _ => true
            }
        },
        _ => true
    }
}


pub fn execute_off(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) -> bool {
    let block = map[x][y];
    match block{
        Some(Block{orientation, kind: BlockKind::Mechanism (Mechanism{kind, ..}), ..}) => {
            let (next_x, next_y) = match orientation{
                Orientation::Up => (x - 1, y),
                Orientation::Right => (x, y + 1),
                Orientation::Down => (x + 1, y),
                Orientation::Left => (x, y - 1)
            };
            match kind {

                MechanismKind::ExtendedPiston => {
                    //println!("retract");
                    destroy(map, x, y, listeners);
                    place(&PISTON, x, y, orientation, map, listeners );
                    destroy(map, next_x, next_y, listeners);
                    listeners.entity_map_update.insert((next_x as usize, next_y as usize));
                    listeners.entity_map_update.insert((x as usize, y as usize));
                },
                MechanismKind::StickyExtendedPiston => {
                    let pull_orientation = match orientation {
                        Orientation::Up => Orientation::Down,
                        Orientation::Left => Orientation::Right,
                        Orientation::Right => Orientation::Left,
                        Orientation::Down => Orientation::Up
                    };
                    let (next_next_x, next_next_y) = match orientation{
                        Orientation::Up => (next_x - 1, next_y),
                        Orientation::Right => (next_x, next_y + 1),
                        Orientation::Down => (next_x + 1, next_y),
                        Orientation::Left => (next_x, next_y - 1)
                    };
                    destroy(map, x, y, listeners);
                    place(&STICKY_PISTON, x, y, orientation, map, listeners );
                    destroy(map, next_x, next_y, listeners);
                    listeners.entity_map_update.insert((next_x as usize, next_y as usize));
                    listeners.entity_map_update.insert((x as usize, y as usize));
                    move_blocks(map, next_next_x as i16, next_next_y as i16, pull_orientation, 20, listeners);
                },
                _ => {}

            }
                
        }, 
        _ => {}
    }
    true
}