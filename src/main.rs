pub mod block;
pub use block::*;

pub const MAP_SIZE: (usize, usize) = (3, 4);
pub type Map = [[Option<Block>; MAP_SIZE.0]; MAP_SIZE.1];

pub use std::collections::HashSet;

fn translate_map(map: &Map) {
    let mut signal_array = [[(0, 0); MAP_SIZE.0]; MAP_SIZE.1];
    for x in 0..map.len() {
        for y in 0..map[x].len() {
            let blk = &map[x][y];
            match *blk {
                Some(Block { kind: BlockKind::Redstone(Redstone { signal, .. }), .. }) => {
                    signal_array[x][y] = (signal, 0);
                }
                Some(Block{ kind: BlockKind::Opaque { strong_signal, weak_signal }, ..}) => {
                    signal_array[x][y] = (strong_signal, weak_signal);
                }
                _ => (),
            }
        }
    }
    for row in signal_array {
        println!("{:?}", row);
    }
}

fn main() {
    let mut map: Map = std::array::from_fn(|_| { std::array::from_fn(|_| { None }) });
    let mut redstone_block_on_delay: HashSet<(usize, usize)> = HashSet::new();
    let mut redstone_block_off_delay: HashSet<(usize, usize)> = HashSet::new();
    let mut mechanism_listener: Listener = vec![];

    let dirt = Block {
        texture_name: TextureName::Dirt,
        movable: true,
        orientation: Orientation::Up,
        kind: BlockKind::Opaque { strong_signal: 0, weak_signal: 0 },
    };

    let redstone_dust = Block {
        texture_name: TextureName::RedstoneDust(false),
        movable: false,
        orientation: Orientation::Up,
        kind: BlockKind::Redstone(Redstone {
            signal: 0,
            input_ports: [true, true, true, true],
            output_ports: [true, true, true, true],
            kind: RedstoneKind::Block,
        }),
    };

    let redstone_torch = Block {
        movable: false,
        texture_name: TextureName::RedstoneTorch(true),
        orientation: Orientation::Up,
        kind: BlockKind::Redstone(Redstone {
            signal: 16,
            input_ports: [false, false, true, false],
            output_ports: [true, true, false, true],
            kind: RedstoneKind::Torch,
        }),
    };

    let mut place = |blk: &Block, x: usize, y: usize, facing: Orientation| {
        place(blk, x, y, facing, &mut map, &mut redstone_block_off_delay, &mut mechanism_listener)
    };

    place(&redstone_torch, 1, 1, Orientation::Up);
    place(&dirt, 1, 2, Orientation::Up);

    for x in 0..MAP_SIZE.1 {
        for y in 0..MAP_SIZE.0 {
            place(&redstone_dust, x, y, Orientation::Up);
        }
    }
    let mut i = 0;
    while i < 3 {
        for (curr_x, curr_y) in redstone_block_on_delay.clone() {
            println!("setting power at {curr_x} {curr_y}");
            set_power(&mut map, curr_x, curr_y, 0, None, &mut redstone_block_off_delay);
        }
        redstone_block_on_delay.clear();
        println!("");
        translate_map(&map);

        for (curr_x, curr_y) in redstone_block_off_delay.clone() {
            set_power_to_0(
                &mut map,
                curr_x,
                curr_y,
                None,
                20,
                &mut redstone_block_on_delay,
                &mut redstone_block_off_delay
            );
        }
        redstone_block_off_delay.clear();
        println!("");
        println!("{:?}", redstone_block_on_delay);
        translate_map(&map);
        i += 1;
    }
}
