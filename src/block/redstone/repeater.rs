// use super::*;

// pub fn repeater_listener(listeners: &mut EventListener, world_map: &mut WorldMap) {
//     let repeater_listeners = listeners.repeater_state.clone();

//     for ((x, y), on) in repeater_listeners {
//         let blk = &mut world_map.0[x][y];
//         match *blk {
//             Some(
//                 Block {
//                     kind: BlockKind::Redstone(
//                         Redstone {
//                             kind: RedstoneKind::Repeater { tick, ref mut countdown },
//                             signal,
//                             ..
//                         },
//                     ),
//                     ..
//                 },
//             ) => {

//                 if *countdown < 0 {
//                     if (on && signal > 0) || (!on && signal <= 0){
//                         listeners.repeater_state.remove(&(x, y));
//                     } else {
//                         *countdown = tick;
//                     }
//                 }

//                 if *countdown > 0 {
//                     *countdown -= 1;
//                 }
//                 else if *countdown == 0 {
//                     *countdown -= 1;
//                     if signal <= 0 {
//                         listeners.redstone_state.insert((x, y), (true, 20, None));
//                         listeners.entity_map_update.insert((x, y), false);
//                     } else {
//                         listeners.redstone_state.insert((x, y), (false, 30, None));
//                         listeners.entity_map_update.insert((x, y), false);
//                     }
                    
//                 } 
//             }
//             _ => {}
//         }
//     }
// }