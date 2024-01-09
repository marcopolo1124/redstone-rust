use super::*;

pub fn debug_map(map: &Map) {
    for row in map {
        let mut new_row = Vec::new();
        for blk in row {
            match *blk {
                Some(Block { kind: BlockKind::Redstone(Redstone { signal, .. }), .. }) => {
                    new_row.push((signal, 0));
                }
                Some(Block { kind: BlockKind::Opaque { strong_signal, weak_signal }, .. }) => {
                    new_row.push((strong_signal, weak_signal));
                }
                _ => {
                    new_row.push((0, 0));
                }
            }
        }
    }
}