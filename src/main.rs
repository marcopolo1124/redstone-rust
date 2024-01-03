pub mod block;
pub use block::*;

pub const MAP_SIZE: (usize, usize) = (3, 10);
pub type Map = [[Option<Block>; MAP_SIZE.0]; MAP_SIZE.1];

fn translate_map(map: &Map) -> [[u8; MAP_SIZE.0]; MAP_SIZE.1] {
    let mut signal_array = [[0; MAP_SIZE.0]; MAP_SIZE.1];
    for x in 0..map.len() {
        for y in 0..map[x].len() {
            let blk = &map[x][y];
            match *blk {
                Some(Block { kind: BlockKind::Redstone(Redstone{ signal, .. }), .. }) => {
                    signal_array[x][y] = signal;
                }
                _ => (),
            }
        }
    }
    signal_array
}

fn main() {
    let mut map: Map = std::array::from_fn(|_| { std::array::from_fn(|_| { None }) });
    let mut redstone_listener: Listener = vec![];
    let mut redstone_source_listener: Listener = vec![];
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
        kind: BlockKind::Redstone (Redstone{
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
        kind: BlockKind::Redstone( Redstone{
            signal: 16,
            input_ports: [false, false, true, false],
            output_ports: [true, true, false, true],
            kind: RedstoneKind::Torch,
        }),
    };

    let mut place = |blk: &Block, x: usize, y: usize, facing: Orientation| {
        place(
            blk,
            x,
            y,
            facing,
            &mut map,
            &mut redstone_listener,
            &mut redstone_source_listener,
            &mut mechanism_listener
        )
    };

    place(&redstone_torch, 1, 1, Orientation::Up);
    // place(&dirt, 1, 2, Orientation::Up);

    for x in 0..MAP_SIZE.1 {
        for y in 0..MAP_SIZE.0 {
            place(&redstone_dust, x, y, Orientation::Up);
        }
    }

    let signal_array = translate_map(&map);
    for row in signal_array {
        println!("{:?}", row);
    }

    destroy(&mut map, 1, 1);
    let signal_array = translate_map(&map);
    println!("\n");
    for row in signal_array {
        println!("{:?}", row);
    }
}
