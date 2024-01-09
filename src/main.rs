use bevy::{ prelude::*, utils::HashMap, window::PrimaryWindow };
use bevy::input::mouse::MouseWheel;
use std::f32::consts::PI;

pub mod block;
pub use block::*;

pub const MAP_SIZE: (usize, usize) = (30, 30);
pub type Map = [[Option<Block>; MAP_SIZE.0]; MAP_SIZE.1];

pub use std::collections::HashSet;

const DIRT: Block = Block {
    texture_name: TextureName::Dirt,
    movable: true,
    orientation: Orientation::Up,
    kind: BlockKind::Opaque { strong_signal: 0, weak_signal: 0 },
};

const REDSTONE_DUST: Block = Block {
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

const REDSTONE_TORCH: Block = Block {
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

const REPEATER: Block = Block {
    movable: false,
    texture_name: TextureName::Repeater(false),
    orientation: Orientation::Up,
    kind: BlockKind::Redstone(Redstone {
        signal: 0,
        input_ports: [false, false, true, false],
        output_ports: [true, false, false, false],
        kind: RedstoneKind::Repeater { tick: 0, countdown: -1 },
    }),
};

const PISTON: Block = Block {
    movable: true,
    texture_name: TextureName::Piston { extended: false },
    orientation: Orientation::Up,
    kind: BlockKind::Mechanism(Mechanism {
        kind: MechanismKind::Piston,
        input_ports: [false, true, true, true],
    }),
};

const EXTENDED_PISTON: Block = Block {
    movable: false,
    texture_name: TextureName::Piston { extended: true },
    orientation: Orientation::Up,
    kind: BlockKind::Mechanism(Mechanism {
        kind: MechanismKind::ExtendedPiston,
        input_ports: [false, true, true, true],
    }),
};

const STICKY_PISTON: Block = Block {
    movable: true,
    texture_name: TextureName::Piston { extended: false },
    orientation: Orientation::Up,
    kind: BlockKind::Mechanism(Mechanism {
        kind: MechanismKind::StickyPiston,
        input_ports: [false, true, true, true],
    }),
};

const STICKY_EXTENDED_PISTON: Block = Block {
    movable: false,
    texture_name: TextureName::Piston { extended: true },
    orientation: Orientation::Up,
    kind: BlockKind::Mechanism(Mechanism {
        kind: MechanismKind::StickyExtendedPiston,
        input_ports: [false, true, true, true],
    }),
};

const PISTON_HEAD: Block = Block {
    movable: false,
    texture_name: TextureName::PistonHead,
    orientation: Orientation::Up,
    kind: BlockKind::Transparent,
};

const TICK: f64 = 0.5;

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
        // println!("{:?}", new_row);
    }
}

fn main() {
    let world_map = [[None; MAP_SIZE.1]; MAP_SIZE.0];
    let entity_map = [[None; MAP_SIZE.1]; MAP_SIZE.0];
    App::new()
        .insert_resource(WorldMap(world_map))
        .insert_resource(EntityMap(entity_map))
        .insert_resource(TextureMap(HashMap::new()))
        .insert_resource(Orientation::Up)
        .insert_resource(EventListener::new())
        .insert_resource(SelectedBlock(None))
        .insert_resource(Time::<Fixed>::from_seconds(TICK))
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, load_assets)
        .add_systems(Startup, init)
        .add_systems(Update, move_camera)
        .add_systems(Update, mouse_input)
        .add_systems(Update, zoom_camera)
        .add_systems(Update, update_selected_block)
        .add_systems(FixedUpdate, (run_listeners, entity_map_listener))
        .add_systems(Update, update_orientation)
        .run();
}

#[derive(Resource)]
pub struct WorldMap([[Option<Block>; MAP_SIZE.1]; MAP_SIZE.0]);

#[derive(Resource)]
pub struct EntityMap([[Option<Entity>; MAP_SIZE.1]; MAP_SIZE.0]);

#[derive(Resource, PartialEq)]
pub struct SelectedBlock(Option<Block>);

#[derive(Component)]
pub struct BlockComponent;

#[derive(Component)]
pub struct GridBox;

pub const BOX_WIDTH: f32 = 40.0;
fn init(
    mut commands: Commands,
    map: Res<WorldMap>,
    mut entity_map: ResMut<EntityMap>,
    textures: Res<TextureMap>
) {
    commands.spawn(Camera2dBundle {
        // transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    for (x, row) in map.as_ref().0.iter().enumerate() {
        for (y, blk) in row.iter().enumerate() {
            let sprite_bundle = get_sprite(x, y, blk, &textures);
            commands.spawn(get_sprite(x, y, &None, &textures));
            let entity = commands.spawn((sprite_bundle, BlockComponent)).id();

            entity_map.as_mut().0[x][y] = Some(entity);
        }
    }
}

pub fn update_selected_block(
    mut selected: ResMut<SelectedBlock>,
    keyboard_input: Res<Input<KeyCode>>
) {
    if keyboard_input.pressed(KeyCode::Key1) {
        selected.0 = Some(DIRT);
    } else if keyboard_input.pressed(KeyCode::Key2) {
        selected.0 = Some(REDSTONE_DUST);
    } else if keyboard_input.pressed(KeyCode::Key3) {
        selected.0 = Some(REDSTONE_TORCH);
    } else if keyboard_input.pressed(KeyCode::Key4) {
        selected.0 = Some(REPEATER);
    } else if keyboard_input.pressed(KeyCode::Key5) {
        selected.0 = Some(PISTON);
    } else if keyboard_input.pressed(KeyCode::Key6) {
        selected.0 = Some(STICKY_PISTON);
    }
}

pub fn update_orientation(
    keyboard_input: Res<Input<KeyCode>>,
    mut orientation: ResMut<Orientation>
) {
    if keyboard_input.pressed(KeyCode::Left) {
        *orientation = Orientation::Left;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        *orientation = Orientation::Right;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        *orientation = Orientation::Up;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        *orientation = Orientation::Down;
    }
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

fn mouse_input(
    buttons: Res<Input<MouseButton>>,
    mut world_map: ResMut<WorldMap>,
    mut entity_map: ResMut<EntityMap>,
    mut listeners: ResMut<EventListener>,
    orientation: Res<Orientation>,
    textures: Res<TextureMap>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<(&mut Transform, &mut BlockComponent, &mut Handle<Image>)>,
    selected: Res<SelectedBlock>
) {
    let (camera, camera_transform) = q_camera.single();
    let (x, y) = if
        let Some(position) = q_windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
    {
        let map_coord = get_mouse_coord(position.x, position.y);
        match map_coord {
            Some(coord) => coord,
            _ => {
                return;
            }
        }
    } else {
        return;
    };
    let map = &mut world_map.0;
    let ent_map = &mut entity_map.0;
    if buttons.just_pressed(MouseButton::Left) {
        // println!("{} {}", x, y);
        destroy(map, x, y, &mut listeners);
        update_entity_map(x, y, map, ent_map, &textures, &mut query);
    }
    if buttons.just_pressed(MouseButton::Right) && selected.0 != None {
        if map[x][y] != None {
            interact(map, x, y);
        } else {
            place(&selected.0.unwrap(), x, y, *orientation, map, &mut listeners);
        }
        // println!("{:?}", map[x][y]);
        update_entity_map(x, y, map, ent_map, &textures, &mut query);
        // println!("{:?}", map);
    }
}

#[derive(Resource)]
pub struct TextureMap(HashMap<TextureName, Handle<Image>>);

fn load_assets(asset_server: Res<AssetServer>, mut textures: ResMut<TextureMap>) {
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

const SPEED: f32 = 500.0;
fn move_camera(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>
) {
    if let Ok(mut transform) = query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::A) {
            direction += Vec3::new(-1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::W) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }
        if keyboard_input.pressed(KeyCode::S) {
            direction += Vec3::new(0.0, -1.0, 0.0);
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * SPEED * time.delta_seconds();
    }
}
fn zoom_camera(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_evr: EventReader<MouseWheel>
) {
    use bevy::input::mouse::MouseScrollUnit;
    if let Ok(mut transform) = query.get_single_mut() {
        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    let new_scale = transform.scale + 0.1 * ev.y;
                    if new_scale > 0.0 {
                        transform.scale = new_scale;
                    } else {
                    }
                }
                MouseScrollUnit::Pixel => {
                    let new_scale = transform.scale + 0.1 * ev.y;
                    if new_scale > 0.0 {
                        transform.scale = new_scale;
                    } else {
                    }
                }
            }
        }
    }
}

fn get_mouse_coord(x: f32, y: f32) -> Option<(usize, usize)> {
    let x_coord = (MAP_SIZE.1 as f32) - ((y + BOX_WIDTH / 2.0) / BOX_WIDTH).floor() - 1.0;
    let y_coord = ((x + BOX_WIDTH / 2.0) / BOX_WIDTH).floor();
    if
        0.0 <= x_coord &&
        x_coord < (MAP_SIZE.0 as f32) &&
        0.0 <= y_coord &&
        y_coord < (MAP_SIZE.1 as f32)
    {
        Some((x_coord as usize, y_coord as usize))
    } else {
        None
    }
}

fn redstone_torch_delayed_listener(
    mut listeners: &mut EventListener,
    world_map: &mut WorldMap
) {
    // println!("start of listening");
    let mut traversed: HashSet<(usize, usize)> = HashSet::new();
    let torch_listeners = listeners.redstone_state.clone();
    listeners.redstone_state.clear();

    for ((x, y), (on, signal, signal_type)) in torch_listeners {
        if on {
            set_power(&mut world_map.0, x, y, signal, signal_type, &mut listeners, &mut traversed);
        } else {
            set_power_to_0(
                &mut world_map.0,
                x,
                y,
                signal_type,
                signal,
                &mut listeners,
                &mut traversed
            );
        }
    }
    for (x, y) in traversed {
        listeners.entity_map_update.insert((x, y));
    }
}

pub fn repeater_listener(listeners: &mut EventListener, world_map: &mut WorldMap) {
    let traversed: HashSet<(usize, usize)> = HashSet::new();
    let repeater_listeners = listeners.repeater_state.clone();

    // println!("{:?}", repeater_listeners);

    for ((x, y), on) in repeater_listeners {
        let blk = &mut world_map.0[x][y];
        match *blk {
            Some(
                Block {
                    kind: BlockKind::Redstone(
                        Redstone {
                            kind: RedstoneKind::Repeater { tick, ref mut countdown },
                            signal,
                            ..
                        },
                    ),
                    ..
                },
            ) => {

                if *countdown < 0 {
                    if (on && signal > 0) || (!on && signal <= 0){
                        listeners.repeater_state.remove(&(x, y));
                    } else {
                        *countdown = tick;
                    }
                }

                if *countdown > 0 {
                    *countdown -= 1;
                }
                else if *countdown == 0 {
                    *countdown -= 1;
                    if signal <= 0 {
                        listeners.redstone_state.insert((x, y), (true, 20, None));
                    } else {
                        listeners.redstone_state.insert((x, y), (false, 30, None));
                    }
                    
                } 
            }
            _ => {}
        }
    }
    for (x, y) in traversed {
        listeners.entity_map_update.insert((x, y));
    }
}

fn mechanism_listener(mut listeners: &mut EventListener, world_map: &mut WorldMap) {
    let mechanism_state = listeners.mechanism_state.clone();
    for ((x, y), on) in mechanism_state {
        let success = if on {
            execute(&mut world_map.0, x, y, &mut listeners)
        } else {
            execute_off(&mut world_map.0, x, y, &mut listeners)
        };

        if success {
            listeners.mechanism_state.remove(&(x, y));
        }
    }
}

fn run_listeners(mut listeners: ResMut<EventListener>, mut world_map: ResMut<WorldMap>) {
    redstone_torch_delayed_listener(listeners.as_mut(), world_map.as_mut());
    repeater_listener(listeners.as_mut(), world_map.as_mut());
    mechanism_listener(listeners.as_mut(), world_map.as_mut());
}

fn entity_map_listener(
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
