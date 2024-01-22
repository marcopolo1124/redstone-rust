// pub use super::*;

// #[derive(Debug, PartialEq, Copy, Clone)]
// pub enum MechanismKind {
//     Piston {
//         extended: bool,
//     },
//     StickyPiston {
//         extended: bool,
//     },
//     Debug,
// }

// #[derive(Debug, PartialEq, Copy, Clone)]
// pub struct Mechanism {
//     pub kind: MechanismKind,
//     pub input_ports: Ports,
//     pub signal: u8,
// }

// pub fn move_blocks(
//     map: &mut Map,
//     x: i16,
//     y: i16,
//     orientation: Orientation,
//     strength: usize,
//     listeners: &mut EventListener
// ) -> bool {
//     if strength <= 0 {
//         return false;
//     }

//     if x < 0 || x >= (CHUNK_SIZE.1 as i16) - 1 || y < 0 || y >= (CHUNK_SIZE.0 as i16) - 1 {
//         return false;
//     }

//     let blk = map[x as usize][y as usize].clone();
//     let blk = match blk {
//         Some(block) => {
//             if block.movable {
//                 block
//             } else {
//                 return false;
//             }
//         }
//         None => {
//             return true;
//         }
//     };

//     let (next_x, next_y) = match orientation {
//         Orientation::Up => (x - 1, y),
//         Orientation::Right => (x, y + 1),
//         Orientation::Down => (x + 1, y),
//         Orientation::Left => (x, y - 1),
//     };

//     let moved = move_blocks(map, next_x, next_y, orientation, strength - 1, listeners);
//     if moved {
//         place(&blk, next_x as usize, next_y as usize, blk.orientation, map, listeners);
//         destroy(map, x as usize, y as usize, listeners);
//     }

//     moved
// }

// pub fn execute(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) -> bool {
//     let block = map[x][y];

//     if let Some(Block { orientation, kind, .. }) = block {
//         let (next_x, next_y) = match orientation {
//             Orientation::Up => (x - 1, y),
//             Orientation::Right => (x, y + 1),
//             Orientation::Down => (x + 1, y),
//             Orientation::Left => (x, y - 1),
//         };
//         if let BlockKind::Mechanism(Mechanism { kind, .. }) = kind {
//             if
//                 let MechanismKind::Piston { extended } | MechanismKind::StickyPiston { extended } =
//                     kind
//             {
//                 if extended {
//                     return true;
//                 }

//                 let moved = move_blocks(
//                     map,
//                     next_x as i16,
//                     next_y as i16,
//                     orientation,
//                     20,
//                     listeners
//                 );
//                 if moved {
//                     *get_extended(map, x, y).unwrap() = true;
//                     if let Some(Block{ref mut movable, ..}) = map[x][y]{
//                         *movable = false
//                     }
//                     if let MechanismKind::StickyPiston { .. } = kind {
//                         place(&STICKY_PISTON_HEAD, next_x, next_y, orientation, map, listeners);
//                     } else {
//                         place(&PISTON_HEAD, next_x, next_y, orientation, map, listeners);
//                     }
                    
//                     listeners.entity_map_update.insert((x, y), false);
//                     //println!("{:?}", listeners.entity_map_update);
//                 }
//                 return moved;
//             }
//         }
//     }
//     return true;
// }

// pub fn execute_off(map: &mut Map, x: usize, y: usize, listeners: &mut EventListener) -> bool {
//     let block = &mut map[x][y];
    
//     if let Some(Block { orientation, ref mut kind, .. }) = *block {
//         let (next_x, next_y) = match orientation {
//             Orientation::Up => (x - 1, y),
//             Orientation::Right => (x, y + 1),
//             Orientation::Down => (x + 1, y),
//             Orientation::Left => (x, y - 1),
//         };
//         if let BlockKind::Mechanism(Mechanism { ref mut kind, .. }) = *kind {
//             match *kind {
//                 MechanismKind::Piston { ref mut extended } => {
//                     if !*extended {
//                         return true;
//                     }
//                     *extended = false;
//                     if let Some(Block{ref mut movable, ..}) = map[x][y]{
//                         *movable = true
//                     }
//                     destroy(map, next_x, next_y, listeners);
//                 }
//                 MechanismKind::StickyPiston { ref mut extended } => {
//                     if !*extended {
//                         return true;
//                     }
//                     let pull_orientation = match orientation {
//                         Orientation::Up => Orientation::Down,
//                         Orientation::Left => Orientation::Right,
//                         Orientation::Right => Orientation::Left,
//                         Orientation::Down => Orientation::Up,
//                     };
//                     let (next_next_x, next_next_y) = match orientation {
//                         Orientation::Up => (next_x - 1, next_y),
//                         Orientation::Right => (next_x, next_y + 1),
//                         Orientation::Down => (next_x + 1, next_y),
//                         Orientation::Left => (next_x, next_y - 1),
//                     };
//                     *extended = false;
//                     if let Some(Block{ref mut movable, ..}) = map[x][y]{
//                         *movable = true
//                     }
//                     destroy(map, next_x, next_y, listeners);
//                     move_blocks(
//                         map,
//                         next_next_x as i16,
//                         next_next_y as i16,
//                         pull_orientation,
//                         20,
//                         listeners
//                     );
//                 }
//                 _ => (),
//             }
//         };
//     }
//     return true;
// }

// fn get_extended(map: &mut Map, x: usize, y: usize) -> Option<&mut bool> {
//     let block = &mut map[x][y];
//     if
//         let
//         | Some(
//               Block {
//                   kind: BlockKind::Mechanism(
//                       Mechanism { kind: MechanismKind::Piston { ref mut extended }, .. },
//                   ),
//                   ..
//               },
//               ..,
//           )
//         | Some(
//               Block {
//                   kind: BlockKind::Mechanism(
//                       Mechanism { kind: MechanismKind::StickyPiston { ref mut extended }, .. },
//                   ),
//                   ..
//               },
//               ..,
//           ) = block
//     {
//         return Some(extended);
//     }
//     return None;
// }

// pub fn mechanism_listener(mut listeners: &mut EventListener, world_map: &mut WorldMap) {
//     let mechanism_state = listeners.mechanism_state.clone();

//     for ((x, y), on) in mechanism_state {
//         let success = if on {
//             execute(&mut world_map.0, x, y, &mut listeners)
//         } else {
//             execute_off(&mut world_map.0, x, y, &mut listeners)
//         };
        

//         if success {
//             listeners.entity_map_update.insert((x, y), false);
//             listeners.mechanism_state.remove(&(x, y));
//         }
//     }
// }
