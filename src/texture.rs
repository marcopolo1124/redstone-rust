pub use super::*;

use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/redstone_torch.png")]
    #[asset(image(sampler = nearest))]
    redstone_torch: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/redstone_block.png")]
    #[asset(image(sampler = nearest))]
    redstone_block: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/button.png")]
    #[asset(image(sampler = nearest))]
    button: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/lever.png")]
    #[asset(image(sampler = nearest))]
    lever: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/observer.png")]
    #[asset(image(sampler = nearest))]
    observer: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/dirt.png")]
    #[asset(image(sampler = nearest))]
    dirt: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/piston.png")]
    #[asset(image(sampler = nearest))]
    piston: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/sticky_piston.png")]
    #[asset(image(sampler = nearest))]
    sticky_piston: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/piston_extension.png")]
    #[asset(image(sampler = nearest))]
    piston_head: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/sticky_piston_extension.png")]
    #[asset(image(sampler = nearest))]
    sticky_piston_head: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 8, rows = 1))]
    #[asset(path = "images/redstone_repeater.png")]
    #[asset(image(sampler = nearest))]
    repeater: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 4, rows = 1))]
    #[asset(path = "images/comparator.png")]
    #[asset(image(sampler = nearest))]
    comparator: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/target_block.png")]
    #[asset(image(sampler = nearest))]
    target_block: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 176, rows = 1))]
    #[asset(path = "images/redstone_dust_var.png")]
    #[asset(image(sampler = nearest))]
    redstone_dust_var: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/slime_block.png")]
    #[asset(image(sampler = nearest))]
    slime_block: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/black_wool.png")]
    #[asset(image(sampler = nearest))]
    black_wool: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/blue_wool.png")]
    #[asset(image(sampler = nearest))]
    blue_wool: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/brown_wool.png")]
    #[asset(image(sampler = nearest))]
    brown_wool: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/cyan_wool.png")]
    #[asset(image(sampler = nearest))]
    cyan_wool: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/gray_wool.png")]
    #[asset(image(sampler = nearest))]
    gray_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/green_wool.png")]
    #[asset(image(sampler = nearest))]
    green_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/light_blue_wool.png")]
    #[asset(image(sampler = nearest))]
    light_blue_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/light_gray_wool.png")]
    #[asset(image(sampler = nearest))]
    light_gray_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/lime_wool.png")]
    #[asset(image(sampler = nearest))]
    lime_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/magenta_wool.png")]
    #[asset(image(sampler = nearest))]
    magenta_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/orange_wool.png")]
    #[asset(image(sampler = nearest))]
    orange_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/pink_wool.png")]
    #[asset(image(sampler = nearest))]
    pink_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/purple_wool.png")]
    #[asset(image(sampler = nearest))]
    purple_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/red_wool.png")]
    #[asset(image(sampler = nearest))]
    red_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/white_wool.png")]
    #[asset(image(sampler = nearest))]
    white_wool: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/wool/yellow_wool.png")]
    #[asset(image(sampler = nearest))]
    yellow_wool: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 12, rows = 1))]
    #[asset(path = "images/redstone_lamp.png")]
    #[asset(image(sampler = nearest))]
    redstone_lamp: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/glass.png")]
    #[asset(image(sampler = nearest))]
    glass: Handle<TextureAtlas>,
}

pub fn get_atlas(texture_name: TextureName, image_assets: &ImageAssets) -> Handle<TextureAtlas> {
    match texture_name {
        TextureName::Dirt => image_assets.dirt.clone(),
        TextureName::RedstoneBlock => image_assets.redstone_block.clone(),
        TextureName::RedstoneTorch => image_assets.redstone_torch.clone(),
        TextureName::TargetBlock => image_assets.target_block.clone(),
        TextureName::RedstoneDust => image_assets.redstone_dust_var.clone(),
        TextureName::Piston => image_assets.piston.clone(),
        TextureName::StickyPiston => image_assets.sticky_piston.clone(),
        TextureName::PistonHead => image_assets.piston_head.clone(),
        TextureName::StickyPistonHead => image_assets.sticky_piston_head.clone(),
        TextureName::Repeater => image_assets.repeater.clone(),
        TextureName::Comparator => image_assets.comparator.clone(),
        TextureName::Observer => image_assets.observer.clone(),
        TextureName::SlimeBlock => image_assets.slime_block.clone(),
        TextureName::Button => image_assets.button.clone(),
        TextureName::Lever => image_assets.lever.clone(),
        TextureName::BlackWool => image_assets.black_wool.clone(),
        TextureName::BlueWool => image_assets.blue_wool.clone(),
        TextureName::BrownWool => image_assets.brown_wool.clone(),
        TextureName::CyanWool => image_assets.cyan_wool.clone(),
        TextureName::GrayWool => image_assets.gray_wool.clone(),
        TextureName::GreenWool => image_assets.green_wool.clone(),
        TextureName::LightBlueWool => image_assets.light_blue_wool.clone(),
        TextureName::LightGrayWool => image_assets.light_gray_wool.clone(),
        TextureName::LimeWool => image_assets.lime_wool.clone(),
        TextureName::MagentaWool => image_assets.magenta_wool.clone(),
        TextureName::OrangeWool => image_assets.orange_wool.clone(),
        TextureName::PinkWool => image_assets.pink_wool.clone(),
        TextureName::PurpleWool => image_assets.purple_wool.clone(),
        TextureName::RedWool => image_assets.red_wool.clone(),
        TextureName::WhiteWool => image_assets.white_wool.clone(),
        TextureName::YellowWool => image_assets.yellow_wool.clone(),
        TextureName::RedstoneLamp => image_assets.redstone_lamp.clone(),
        TextureName::Glass => image_assets.glass.clone()

    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum TextureName {
    Dirt,
    RedstoneTorch,
    RedstoneBlock,
    TargetBlock,
    RedstoneDust,
    Piston,
    StickyPiston,
    PistonHead,
    StickyPistonHead,
    Repeater,
    Comparator,
    Observer,
    SlimeBlock,
    Button,
    Lever,
    BlackWool,
    BlueWool,
    BrownWool,
    CyanWool,
    GrayWool,
    GreenWool,
    LightBlueWool,
    LightGrayWool,
    LimeWool,
    MagentaWool,
    OrangeWool,
    PinkWool,
    PurpleWool,
    RedWool,
    WhiteWool,
    YellowWool,
    RedstoneLamp,
    Glass
}

impl TextureName {
    pub fn get_string_value(&self) -> String {
        let str = serde_json::to_string(&self).unwrap();
        let string_val: String = serde_json::from_str(&str).unwrap();
        string_val
    }

    pub fn iter() -> Vec<TextureName> {
        vec![
            TextureName::Dirt,
            TextureName::RedstoneTorch,
            TextureName::RedstoneBlock,
            TextureName::TargetBlock,
            TextureName::RedstoneDust,
            TextureName::Piston,
            TextureName::StickyPiston,
            TextureName::PistonHead,
            TextureName::StickyPistonHead,
            TextureName::Repeater,
            TextureName::Comparator,
            TextureName::Observer,
            TextureName::SlimeBlock,
            TextureName::Button,
            TextureName::Lever,
            TextureName::BlackWool,
            TextureName::BlueWool,
            TextureName::BrownWool,
            TextureName::CyanWool,
            TextureName::GrayWool,
            TextureName::GreenWool,
            TextureName::LightBlueWool,
            TextureName::LightGrayWool,
            TextureName::LimeWool,
            TextureName::MagentaWool,
            TextureName::OrangeWool,
            TextureName::PinkWool,
            TextureName::PurpleWool,
            TextureName::RedWool,
            TextureName::WhiteWool,
            TextureName::YellowWool,
            TextureName::RedstoneLamp,
            TextureName::Glass
        ]
    }

    pub fn texture_map() -> HashMap<String, TextureName>{
        let all_textures = TextureName::iter();
        
        let mut hashmap = HashMap::new(); 
        for texture in all_textures {
            hashmap.insert(texture.get_string_value(), texture);
        }
        hashmap
    }
}
