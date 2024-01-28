mod chunks;
use std::{ f32::consts::PI, path::Path };
use std::time::Duration;

use bevy_asset_loader::loading_state::{
    config::ConfigureLoadingState,
    LoadingState,
    LoadingStateAppExt,
};
pub use chunks::*;

pub use serde::{ Serialize, Deserialize };

pub use bevy_persistent::prelude::*;

mod texture;
pub use texture::*;

use bevy::{ prelude::*, utils::{ HashMap, HashSet }, window::PrimaryWindow };

const BOX_WIDTH: f32 = 48.0;

#[derive(Resource)]
pub struct EventListeners {
    pub entity_map_update: HashSet<(i128, i128)>,
    pub mechanism_listener: HashMap<(i128, i128), bool>,
}

impl EventListeners {
    pub fn new() -> EventListeners {
        EventListeners {
            entity_map_update: HashSet::new(),
            mechanism_listener: HashMap::new(),
        }
    }

    pub fn update_entity(&mut self, x: i128, y: i128) {
        self.entity_map_update.insert((x, y));
    }

    pub fn turn_mechanism_on(&mut self, x: i128, y: i128) {
        self.mechanism_listener.insert((x, y), true);
    }

    pub fn turn_mechanism_off(&mut self, x: i128, y: i128) {
        self.mechanism_listener.insert((x, y), false);
    }
    pub fn remove_mechanism(&mut self, x: i128, y: i128) {
        self.mechanism_listener.remove(&(x, y));
    }
}

#[derive(Resource, PartialEq)]
pub struct SelectedBlock(Option<Block>);

impl SelectedBlock {
    pub fn get_block(&self) -> Option<Block> {
        return self.0;
    }
}

#[derive(Resource, Serialize, Deserialize)]
struct SaveData(
    Vec<((i128, i128), [[Option<Block>; CHUNK_SIZE.0 as usize]; CHUNK_SIZE.1 as usize])>,
);

#[derive(Resource)]
struct AutosaveTimer {
    timer: Timer,
}

const TICK: f64 = 0.2;

fn main() {
    let chunks = Chunks::new();
    let event_listeners = EventListeners::new();
    let state_dir = dirs
        ::config_dir()
        .map(|dir| dir.join("redstone_rust"))
        .unwrap_or(Path::new("local").join("save"));

    App::new()
        .add_state::<MyStates>()
        .insert_resource(Time::<Fixed>::from_seconds(TICK))
        .add_plugins(DefaultPlugins)
        .insert_resource(chunks)
        .insert_resource(event_listeners)
        .insert_resource(SelectedBlock(Some(DIRT)))
        .insert_resource(Orientation::Up)
        .insert_resource(
            Persistent::<SaveData>
                ::builder()
                .name("save data")
                .format(StorageFormat::Json)
                .path(state_dir.join("save_data.json"))
                .default(SaveData(Vec::new()))
                .build()
                .expect("failed to initialize game state")
        )
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::InGame)
                .load_collection::<ImageAssets>()
        )
        .add_systems(OnEnter(MyStates::InGame), init)
        .add_systems(FixedUpdate, mechanism_listener.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, mouse_input.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_selected_block.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_entity_listener.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, move_camera.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_orientation.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, autosave.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, zoom_camera.run_if(in_state(MyStates::InGame)))
        .run()
}

const DIRT: Block = Block {
    movable: true,
    orientation: Orientation::Up,
    texture_name: TextureName::Dirt,
    symmetric: true,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        kind: None,
        input_ports: [true, true, true, true],
        output_ports: [true, true, true, true],
    }),
    mechanism: None,
};

const REDSTONE_TORCH: Block = Block {
    movable: false,
    orientation: Orientation::Up,
    texture_name: TextureName::RedstoneTorch,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 16,
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, false, true, false],
        output_ports: [true, true, false, true],
    }),
    mechanism: Some(MechanismKind::RedstoneTorch),
};

const REPEATER: Block = Block {
    movable: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Repeater,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, false, true, false],
        output_ports: [true, false, false, false],
    }),
    mechanism: Some(MechanismKind::Repeater { countdown: -1, tick: 0 }),
};

const REDSTONE_DUST: Block = Block {
    movable: false,
    orientation: Orientation::Up,
    texture_name: TextureName::RedstoneDust,
    symmetric: true,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: Some(SignalType::Weak(true)),
        kind: Some(RedstoneKind::Dust),
        input_ports: [true, true, true, true],
        output_ports: [true, true, true, true],
    }),
    mechanism: None,
};

const PISTON: Block = Block {
    movable: true,
    orientation: Orientation::Up,
    texture_name: TextureName::Piston,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, true, true, true],
        output_ports: [false, false, false, false],
    }),
    mechanism: Some(MechanismKind::Piston { extended: false, sticky: false }),
};

const PISTON_HEAD: Block = Block {
    movable: false,
    orientation: Orientation::Up,
    texture_name: TextureName::PistonHead,
    symmetric: false,
    redstone: None,
    mechanism: None,
};

const STICKY_PISTON: Block = Block {
    movable: true,
    orientation: Orientation::Up,
    texture_name: TextureName::StickyPiston,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, true, true, true],
        output_ports: [false, false, false, false],
    }),
    mechanism: Some(MechanismKind::Piston { extended: false, sticky: true }),
};

const STICKY_PISTON_HEAD: Block = Block {
    movable: false,
    orientation: Orientation::Up,
    texture_name: TextureName::StickyPistonHead,
    symmetric: false,
    redstone: None,
    mechanism: None,
};

const AUTOSAVE_INTERVAL_SECONDS: f32 = 3.0;

fn init(
    mut commands: Commands,
    save_data: Res<Persistent<SaveData>>,
    mut chunks: ResMut<Chunks>,
    mut listeners: ResMut<EventListeners>,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    commands.spawn(Camera2dBundle {
        ..default()
    });

    commands.insert_resource(AutosaveTimer {
        timer: Timer::new(Duration::from_secs_f32(AUTOSAVE_INTERVAL_SECONDS), TimerMode::Repeating),
    });

    for ((chunk_x, chunk_y), map) in save_data.0.iter() {
        for (u, row) in map.iter().enumerate() {
            for (v, blk) in row.iter().enumerate() {
                let x = chunk_x * CHUNK_SIZE.0 + (u as i128);
                let y = chunk_y * CHUNK_SIZE.1 + (v as i128);

                listeners.update_entity(x, y);
                if let Some(blk_data) = blk {
                    let mut blk_clone = blk_data.clone();
                    if
                        let Block {
                            redstone: Some(Redstone { signal, .. }),

                            mechanism: Some(MechanismKind::RedstoneTorch),
                            ..
                        } = &mut blk_clone
                    {
                        *signal = 16;
                    }

                    if
                        let Block {
                            symmetric: false,
                            redstone: Some(Redstone { input_ports, output_ports, .. }),
                            orientation,
                            ..
                        } = &mut blk_clone
                    {
                        let orientation_reversion = Orientation::port_idx_to_orientation(
                            (4 - orientation.to_port_idx()).rem_euclid(4)
                        );
                        *input_ports = orientation_reversion.rotate_ports(*input_ports);
                        *output_ports = orientation_reversion.rotate_ports(*output_ports);
                    }

                    place(
                        &mut chunks,
                        blk_clone,
                        blk_clone.orientation,
                        x,
                        y,
                        &mut listeners,
                        &mut commands,
                        &image_assets,
                        &mut query
                    );
                }
            }
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
        selected.0 = Some(REDSTONE_TORCH);
    } else if keyboard_input.pressed(KeyCode::Key3) {
        selected.0 = Some(REDSTONE_DUST);
    } else if keyboard_input.pressed(KeyCode::Key4) {
        selected.0 = Some(PISTON);
    } else if keyboard_input.pressed(KeyCode::Key5) {
        selected.0 = Some(STICKY_PISTON);
    } else if keyboard_input.pressed(KeyCode::Key6) {
        selected.0 = Some(REPEATER);
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

pub fn mouse_input(
    mut commands: Commands,
    mut listeners: ResMut<EventListeners>,
    buttons: Res<Input<MouseButton>>,
    selected_block: Res<SelectedBlock>,
    orientation: Res<Orientation>,
    mut chunks: ResMut<Chunks>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    let (camera, camera_transform) = q_camera.single();
    let (x, y) = if
        let Some(position) = q_windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
    {
        get_mouse_coord(position.x, position.y)
    } else {
        return;
    };

    if buttons.just_pressed(MouseButton::Right) {
        //  println!("click at {x} {y}");
        if let Some(blk) = selected_block.get_block() {
            if
                !place(
                    chunks.as_mut(),
                    blk,
                    *orientation,
                    x,
                    y,
                    &mut listeners,
                    &mut commands,
                    &image_assets,
                    &mut query
                )
            {
                interact(chunks.as_mut(), x, y, &mut commands, &image_assets, &mut query);
            }
        }
    } else if buttons.just_pressed(MouseButton::Left) {
        destroy(chunks.as_mut(), x, y, &mut listeners, &mut commands, &image_assets, &mut query);
    }
}

fn get_mouse_coord(x: f32, y: f32) -> (i128, i128) {
    let x_coord = (CHUNK_SIZE.1 as f32) - ((y + BOX_WIDTH / 2.0) / BOX_WIDTH).floor() - 1.0;
    let y_coord = ((x + BOX_WIDTH / 2.0) / BOX_WIDTH).floor();
    (x_coord as i128, y_coord as i128)
}

fn update_entity_listener(
    mut commands: Commands,
    mut listeners: ResMut<EventListeners>,
    image_assets: Res<ImageAssets>,
    mut chunks: ResMut<Chunks>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    for (x, y) in &listeners.entity_map_update {
        update_entity(&mut commands, &mut chunks, *x, *y, &image_assets, &mut query);
    }
    listeners.entity_map_update.clear();
}

fn get_connection(ports: &[bool; 4]) -> usize {
    match *ports {
        [true, true, true, true] => 10,
        [false, true, true, true] => 9,
        [true, false, true, true] => 8,
        [true, true, false, true] => 7,
        [true, true, true, false] => 6,
        [false, false, true, true] => 5,
        [false, true, false, true] => 4,
        [false, true, true, false] => 3,
        [true, false, false, true] => 2,
        [true, false, true, false] => 1,
        [true, true, false, false] => 0,
        _ => 10,
    }
}

fn get_state(blk: Block) -> usize {
    match blk {
        Block {
            redstone: Some(Redstone { signal, kind: Some(RedstoneKind::Dust), output_ports, .. }),
            ..
        } => {
            let conn_ind = get_connection(&output_ports);
            //  println!("{conn_ind}");
            conn_ind * 16 + (signal as usize)
        }
        Block {
            redstone: Some(Redstone { signal, kind: Some(RedstoneKind::Mechanism), .. }),
            mechanism: Some(MechanismKind::RedstoneTorch),
            ..
        } => {
            if signal > 0 { 1 } else { 0 }
        }
        Block { mechanism: Some(MechanismKind::Piston { extended, .. }), .. } => {
            if !extended { 0 } else { 1 }
        }
        Block {
            redstone: Some(Redstone { signal, .. }),
            mechanism: Some(MechanismKind::Repeater { tick, .. }),
            ..
        } => {
            let col_ind = if signal > 0 { 1 } else { 0 };
            let row_ind = tick * 2;
            (row_ind + col_ind) as usize
        }
        _ => 0,
    }
}

fn update_entity(
    commands: &mut Commands,
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    //  println!("{:?}", chunks);
    //  println!("updating entity {x} {y}");
    let curr_blk = chunks.get_block(x, y).clone();
    let curr_entity = chunks.get_entity(x, y);

    if let Some(blk) = curr_blk {
        let Block { texture_name, orientation, .. } = blk;
        let rotate = match orientation {
            Orientation::Up => 0.0,
            Orientation::Right => 3.0,
            Orientation::Down => 2.0,
            Orientation::Left => 1.0,
        };
        let state = get_state(blk);

        if let Some(entity_handle) = *curr_entity {
            if let Ok(mut sprite) = query.get_mut(entity_handle) {
                sprite.index = state;
            }
        } else {
            let handle = commands
                .spawn((
                    BlockComponent,
                    SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(state),
                        texture_atlas: get_atlas(texture_name, image_assets),
                        transform: Transform::from_xyz(
                            (y as f32) * BOX_WIDTH,
                            ((CHUNK_SIZE.0 - 1 - x) as f32) * BOX_WIDTH,
                            0.0
                        )
                            .with_scale(Vec3 { x: 3.0, y: 3.0, z: 1.0 })
                            .with_rotation(Quat::from_rotation_z((PI * rotate) / 2.0)),
                        ..default()
                    },
                ))
                .id();
            *curr_entity = Some(handle);
        }
    } else {
        //  println!("entity deleting {x} {y} {:?}", curr_entity);
        if let Some(entity_handle) = curr_entity {
            commands.entity(*entity_handle).despawn();
        }

        *curr_entity = None;
        chunks.delete_chunk(x, y);
        //  println!("{:?}", chunks);
    }
}

const SPEED: f32 = 500.0;
pub fn move_camera(
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

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    InGame,
}

fn mechanism_listener(
    mut listeners: ResMut<EventListeners>,
    mut chunks: ResMut<Chunks>,
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    let mechanism_listener = listeners.mechanism_listener.clone();
    listeners.mechanism_listener.clear();

    for ((x, y), on) in mechanism_listener {
        execute_mechanism(
            &mut chunks,
            x,
            y,
            on,
            &mut listeners,
            &mut commands,
            &image_assets,
            &mut query
        );
    }
}

fn interact(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>
) {
    let blk = chunks.get_block(x, y);
    match blk {
        Some(Block { mechanism: Some(MechanismKind::Repeater { tick, .. }), .. }) => {
            *tick = (*tick + 1) % 4;
        }
        _ => {}
    }
    update_entity(commands, chunks, x, y, image_assets, query)
}

fn autosave(
    time: Res<Time>,
    mut autosave: ResMut<AutosaveTimer>,
    mut save_data: ResMut<Persistent<SaveData>>,
    chunks: Res<Chunks>
) {
    autosave.timer.tick(time.delta());
    if autosave.timer.finished() {
        let mut current_state = Vec::new();
        for ((x, y), chunk) in chunks.0.iter() {
            let tuple = ((*x, *y), chunk.map.clone());
            current_state.push(tuple);
        }
        save_data.0 = current_state;

        save_data.persist().ok();
    }
}

use bevy::input::mouse::MouseWheel;
pub fn zoom_camera(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_evr: EventReader<MouseWheel>
) {
    use bevy::input::mouse::MouseScrollUnit;
    if let Ok(mut transform) = query.get_single_mut() {
        let mut scale_delta = 0.0;
        for ev in scroll_evr.read() {
            //  println!("scrolled");
            match ev.unit {
                MouseScrollUnit::Line => {
                    let new_scale_delta = scale_delta + 0.1 * ev.y;
                    if new_scale_delta.abs() < 0.2 {
                        scale_delta = new_scale_delta;
                    } else {
                        if new_scale_delta > 0.0 {
                            scale_delta = 0.2;
                        } else {
                            scale_delta = -0.2;
                        }
                    }
                    // let new_scale = transform.scale + 0.1 * ev.y;
                    // if new_scale > 0.0 {
                    //     transform.scale = new_scale;
                    // } else {
                    // }
                }
                MouseScrollUnit::Pixel => {
                    let new_scale_delta = scale_delta + 0.1 * ev.y;
                    if new_scale_delta.abs() < 0.2 {
                        scale_delta = new_scale_delta;
                    } else {
                        if new_scale_delta > 0.0 {
                            scale_delta = 0.2;
                        } else {
                            scale_delta = -0.2;
                        }
                    }
                    // let new_scale = transform.scale + 0.1 * ev.y;
                    // if new_scale > 0.0 {
                    //     transform.scale = new_scale;
                    // } else {
                    // }
                }
            }
        }

        if transform.scale + scale_delta > 0.0 {
            transform.scale += scale_delta;
        } else {
            transform.scale = 0.0;
        }
        // transform.scale = std::cmp::max(0.0, transform.scale + scale_delta);
    }
}
