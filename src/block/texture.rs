use super::*;

#[derive(Resource)]
pub struct TextureMap(pub HashMap<TextureName, Handle<Image>>);

#[derive(Resource)]
pub struct EntityMap(pub [[Option<Entity>; MAP_SIZE.1]; MAP_SIZE.0]);

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
            "piston_side.png"
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

pub fn get_sprite(
    x: usize,
    y: usize,
    blk: &Option<Block>,
    textures: &Res<TextureMap>
) -> SpriteBundle {
    let texture = match *blk {
        None => { TextureName::Air }
        Some(Block { texture_name, .. }) => { texture_name }
    };

    let handle = textures.0.get(&texture).unwrap();
    SpriteBundle {
        transform: Transform::from_xyz(
            BOX_WIDTH * (y as f32),
            BOX_WIDTH * ((MAP_SIZE.1 - x - 1) as f32),
            0.0
        ).with_scale(Vec3::splat(2.2)),
        texture: handle.clone(),
        ..default()
    }
}

fn get_texture_path(texture_name: TextureName) -> String {
    format!("images/{}", get_texture_name(texture_name))
}

pub fn load_assets(asset_server: Res<AssetServer>, mut textures: ResMut<TextureMap>) {
    let assets = &mut textures.0;
    assets.insert(TextureName::Dirt, asset_server.load(get_texture_path(TextureName::Dirt)));
    assets.insert(
        TextureName::RedstoneDust(false),
        asset_server.load(get_texture_path(TextureName::RedstoneDust(false)))
    );
    assets.insert(
        TextureName::RedstoneDust(true),
        asset_server.load(get_texture_path(TextureName::RedstoneDust(true)))
    );
    assets.insert(
        TextureName::RedstoneTorch(false),
        asset_server.load(get_texture_path(TextureName::RedstoneTorch(false)))
    );
    assets.insert(
        TextureName::RedstoneTorch(true),
        asset_server.load(get_texture_path(TextureName::RedstoneTorch(true)))
    );
    assets.insert(
        TextureName::Repeater(false),
        asset_server.load(get_texture_path(TextureName::Repeater(false)))
    );
    assets.insert(
        TextureName::Repeater(true),
        asset_server.load(get_texture_path(TextureName::Repeater(true)))
    );
    assets.insert(
        TextureName::Piston { extended: false },
        asset_server.load(get_texture_path(TextureName::Piston { extended: false }))
    );
    assets.insert(
        TextureName::Piston { extended: true },
        asset_server.load(get_texture_path(TextureName::Piston { extended: true }))
    );
    assets.insert(
        TextureName::StickyPiston { extended: false },
        asset_server.load(get_texture_path(TextureName::Piston { extended: false }))
    );
    assets.insert(
        TextureName::StickyPiston { extended: true },
        asset_server.load(get_texture_path(TextureName::Piston { extended: true }))
    );

    assets.insert(
        TextureName::PistonHead,
        asset_server.load(get_texture_path(TextureName::PistonHead))
    );
    assets.insert(TextureName::Air, asset_server.load(get_texture_path(TextureName::Air)));
}


pub fn update_entity_map(
    x: usize,
    y: usize,
    map: &Map,
    entity_map: &mut [[Option<Entity>; MAP_SIZE.1]; MAP_SIZE.0],
    textures: &Res<TextureMap>,
    query: &mut Query<(&mut Transform, &mut BlockComponent, &mut Handle<Image>)>
) {
    let blk = &map[x][y];
    let entity = &mut entity_map[x][y];
    match entity {
        None => {}
        Some(blk_entity) => {
            if let Ok((mut transform, _, mut sprite)) = query.get_mut(*blk_entity) {
                let (orientation, texture_name) = match *blk {
                    None => (Orientation::Up, TextureName::Air),
                    Some(Block { texture_name, orientation, .. }) => (orientation, texture_name),
                };
                let rotate = match orientation {
                    Orientation::Up => 0.0,
                    Orientation::Right => 3.0,
                    Orientation::Down => 2.0,
                    Orientation::Left => 1.0,
                };

                let mut_ref = sprite.as_mut();
                *mut_ref = textures.0.get(&texture_name).unwrap().clone();
                transform.rotation = Quat::from_rotation_z((PI * rotate) / 2.0);
            }
        }
    }
}

pub fn entity_map_listener(
    listeners: ResMut<EventListener>,
    map: Res<WorldMap>,
    mut entity_map: ResMut<EntityMap>,
    textures: Res<TextureMap>,
    mut query: Query<(&mut Transform, &mut BlockComponent, &mut Handle<Image>)>
) {
    for (x, y) in &listeners.entity_map_update {
        update_entity_map(*x, *y, &map.0, &mut entity_map.0, &textures, &mut query);
    }
}
