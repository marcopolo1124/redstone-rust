#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TextureName {
    Dirt,
    RedstoneTorch(bool),
    RedstoneDust(bool),
    Piston {
        extended: bool,
    },
    StickyPiston {
        extended: bool,
    },
    PistonHead,
    StickyPistonHead,
    Repeater(bool),
    Air,
}

pub fn get_texture_name(texture: TextureName) -> String {
    let name = match texture {
        TextureName::Dirt => "dirt.png",
        TextureName::RedstoneTorch(on) => if on {
            "redstone_torch.png"
        } else {
            "redstone_torch_off.png"
        }
        TextureName::RedstoneDust(on) => if on {
            "redstone_dust_on.png"
        } else {
            "redstone_dust_off.png"
        }
        TextureName::Piston { extended } | TextureName::StickyPiston { extended } => if extended {
            "piston_extended_base.png"
        } else {
            "piston_top.opng"
        }
        TextureName::PistonHead | TextureName::StickyPistonHead => "piston_extension.png",
        TextureName::Repeater(on) => if on {
            "redstone_repeater_on.png"
        } else {
            "redstone_repeater_off.png"
        }
        _ => { "white_wool.png" }
    };

    name.to_string()
}