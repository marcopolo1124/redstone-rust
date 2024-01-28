// use bevy::prelude::*;
pub use super::*;
// use bevy_asset_loader::asset_collection::AssetCollection;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 2, rows = 1))]
    #[asset(path = "images/redstone_torch.png")]
    #[asset(image(sampler = nearest))]
    redstone_torch: Handle<TextureAtlas>,

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

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 16, rows = 1))]
    #[asset(path = "images/redstone_dust.png")]
    #[asset(image(sampler = nearest))]
    redstone_dust: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 176, rows = 1))]
    #[asset(path = "images/redstone_dust_var.png")]
    #[asset(image(sampler = nearest))] 
    redstone_dust_var: Handle<TextureAtlas>,
}

pub fn get_atlas(texture_name: TextureName, image_assets: &ImageAssets) -> Handle<TextureAtlas> {
    match texture_name {
        TextureName::Dirt => image_assets.dirt.clone(),
        TextureName::RedstoneTorch => image_assets.redstone_torch.clone(),
        TextureName::RedstoneCross => image_assets.redstone_dust.clone(),
        TextureName::RedstoneDust => image_assets.redstone_dust_var.clone(),
        TextureName::Piston  => image_assets.piston.clone(),
        TextureName::StickyPiston  => image_assets.sticky_piston.clone(),
        TextureName::PistonHead => image_assets.piston_head.clone(),
        TextureName::StickyPistonHead => image_assets.sticky_piston_head.clone(),
        TextureName::Repeater => image_assets.repeater.clone(),
    }
    // image_assets.dirt.clone()
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum TextureName {
    Dirt,
    RedstoneTorch,
    RedstoneCross,
    RedstoneDust,
    Piston,
    StickyPiston,
    PistonHead,
    StickyPistonHead,
    Repeater
}