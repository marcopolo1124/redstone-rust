use super::*;
use bevy::prelude::*;
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


    #[asset(texture_atlas(tile_size_x = 16.0, tile_size_y = 16.0, columns = 1, rows = 1))]
    #[asset(path = "images/black_wool.png")]
    #[asset(image(sampler = nearest))]
    air: Handle<TextureAtlas>,
}

pub fn get_atlas(texture_name: TextureName, image_assets: &ImageAssets) -> Handle<TextureAtlas> {
    match texture_name {
        TextureName::Dirt => image_assets.dirt.clone(),
        TextureName::RedstoneTorch(_) => image_assets.redstone_torch.clone(),
        TextureName::RedstoneCross( .. ) => image_assets.redstone_dust.clone(),
        TextureName::RedstoneDust => image_assets.redstone_dust.clone(),
        TextureName::Piston { .. } => image_assets.piston.clone(),
        TextureName::StickyPiston { .. } => image_assets.sticky_piston.clone(),
        TextureName::PistonHead => image_assets.piston_head.clone(),
        TextureName::StickyPistonHead => image_assets.sticky_piston_head.clone(),
        TextureName::Repeater(..) => image_assets.repeater.clone(),
        TextureName::Air => image_assets.air.clone(),
    }
    // image_assets.dirt.clone()
}

use std::f32::consts::PI;
#[derive(Resource)]
pub struct TextureMap(pub HashMap<TextureName, Handle<Image>>);

#[derive(Resource)]
pub struct EntityMap(pub [[Option<Entity>; MAP_SIZE.1]; MAP_SIZE.0]);

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TextureName {
    Dirt,
    RedstoneTorch(bool),
    RedstoneCross(bool),
    RedstoneDust,
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

pub fn get_sprite(
    x: usize,
    y: usize,
    blk: &Option<Block>,
    image_assets: &ImageAssets
) -> SpriteSheetBundle {
    let (texture_name, state) = match *blk {
        None => { (TextureName::Air, 0) }
        Some(block) => { (block.texture_name, get_state(block)) }
    };

    let handle = get_atlas(texture_name, image_assets);
    SpriteSheetBundle {
        transform: Transform::from_xyz(
            BOX_WIDTH * (y as f32),
            BOX_WIDTH * ((MAP_SIZE.1 - x - 1) as f32),
            0.0
        ).with_scale(Vec3::splat(2.2)),
        sprite: TextureAtlasSprite::new(state),
        texture_atlas: handle,
        ..default()
    }
}

pub fn get_state(blk: Block) -> usize {
    match blk {
        Block { kind: BlockKind::Redstone(Redstone { signal, kind, .. }), .. } => {
            match kind {
                RedstoneKind::Torch  => {
                    if signal > 0 {
                        //println!("1 {:?}", blk);
                        1
                    } else {
                        0
                    }
                }
                RedstoneKind::Block | RedstoneKind::Dust => signal as usize,
                RedstoneKind::Repeater {tick, ..} => {
                    let col_ind = if signal > 0 {1} else {0};
                    let row_ind = tick * 2;
                    (row_ind + col_ind) as usize
                }
            }
        }
        Block {kind: BlockKind::Mechanism(Mechanism{ kind, ..}), ..} => {
            match kind {
                MechanismKind::Piston { extended } | MechanismKind::StickyPiston { extended } => {
                    if extended {1} else {0}
                }
                _ => 0
            }
        }
        _ => 0,
    }
}

pub fn update_entity_map(
    x: usize,
    y: usize,
    image_assets: &ImageAssets,
    entity_map: &mut EntityMap,
    map: &Map,
    query: &mut Query<
        (&mut Transform, &mut BlockComponent, &mut Handle<TextureAtlas>, &mut TextureAtlasSprite)
    >
) {
    let entity = &mut entity_map.0[x][y];
    let blk = &map[x][y];
    match entity {
        None => {}
        Some(blk_entity) => {
            if let Ok((mut transform, _, mut atlas, mut sprite)) = query.get_mut(*blk_entity) {
                let (orientation, texture_name, state) = match *blk {
                    None => (Orientation::Up, TextureName::Air, 0),
                    Some(blk) => (blk.orientation, blk.texture_name, get_state(blk)),
                };
                let rotate = match orientation {
                    Orientation::Up => 0.0,
                    Orientation::Right => 3.0,
                    Orientation::Down => 2.0,
                    Orientation::Left => 1.0,
                };

                let mut_ref = atlas.as_mut();
                sprite.index = state;
                *mut_ref = get_atlas(texture_name, image_assets).clone();
                transform.rotation = Quat::from_rotation_z((PI * rotate) / 2.0);
            }
        }
    }
}

pub fn update_entity_state(
    x: usize,
    y: usize,
    entity_map: &mut EntityMap,
    map: &Map,
    query: &mut Query<
        (&mut Transform, &mut BlockComponent, &mut Handle<TextureAtlas>, &mut TextureAtlasSprite)
    >
) {
    let entity = &mut entity_map.0[x][y];
    let blk = &map[x][y];
    match entity {
        None => {}
        Some(blk_entity) => {
            if let Ok((_, _, _, mut sprite)) = query.get_mut(*blk_entity) {
                let state = match *blk {
                    None => 0,
                    Some(blk) => get_state(blk),
                };
                sprite.index = state;
            }
        }
    }
}

pub fn entity_map_listener(
    mut listeners: ResMut<EventListener>,
    map: Res<WorldMap>,
    mut entity_map: ResMut<EntityMap>,
    mut query: Query<
        (&mut Transform, &mut BlockComponent, &mut Handle<TextureAtlas>, &mut TextureAtlasSprite)
    >,
    image_assets: Res<ImageAssets>
) {
    for ((x, y), update_atlas) in &listeners.entity_map_update {
        if *update_atlas {
            update_entity_map(*x, *y, image_assets.as_ref(), &mut entity_map, &map.0, &mut query);
        } else{
            update_entity_state(*x, *y, &mut entity_map, &map.0, &mut query)
        }
       
    }
    listeners.entity_map_update.clear()
}
