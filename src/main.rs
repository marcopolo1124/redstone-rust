mod chunks;
use std::{ f32::consts::PI, path::Path };
use std::time::Duration;

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;

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

mod hud;
pub use hud::*;

#[derive(Resource)]
pub struct EventListeners {
    pub entity_map_update: HashSet<(i128, i128)>,
    pub mechanism_listener: HashMap<(i128, i128), bool>,
    pub repropagation_listener: HashSet<(i128, i128)>,
    pub redstone_component_listener: HashMap<(i128, i128), bool>,
}

impl EventListeners {
    pub fn new() -> EventListeners {
        EventListeners {
            entity_map_update: HashSet::new(),
            mechanism_listener: HashMap::new(),
            repropagation_listener: HashSet::new(),
            redstone_component_listener: HashMap::new(),
        }
    }

    pub fn update_entity(&mut self, x: i128, y: i128) {
        self.entity_map_update.insert((x, y));
    }

    pub fn turn_mechanism_on(&mut self, x: i128, y: i128, is_redstone: bool) {
        if is_redstone {
            self.redstone_component_listener.insert((x, y), true);
        } else {
            self.mechanism_listener.insert((x, y), true);
        }
    }

    pub fn turn_mechanism_off(&mut self, x: i128, y: i128, is_redstone: bool) {
        if is_redstone {
            self.redstone_component_listener.insert((x, y), false);
        } else {
            self.mechanism_listener.insert((x, y), false);
        }
    }
    pub fn remove_mechanism(&mut self, x: i128, y: i128) {
        self.mechanism_listener.remove(&(x, y));
    }

    pub fn repropagate(&mut self, x: i128, y: i128) {
        self.repropagation_listener.insert((x, y));
    }

    pub fn change_state(&mut self, x: i128, y: i128, from_port: Orientation, chunks: &Chunks) {
        let blk = chunks.get_block_ref(x, y);
        let orientation = if
            let Some(Block { mechanism: Some(MechanismKind::Observer), orientation, .. }) = blk
        {
            orientation
        } else {
            return;
        };

        if from_port == orientation.get_opposing() {
            self.turn_mechanism_on(x, y, false)
        }
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
struct SaveData(Vec<((i128, i128), Block)>);

#[derive(Resource)]
struct AutosaveTimer {
    timer: Timer,
}

#[derive(Resource)]
pub struct UpdatesPerSecondTimer {
    pub number_of_updates: u16,
    pub timer: Timer,
}

#[derive(Clone, Debug)]
struct PropagationArgs {
    x: i128,
    y: i128,
    input_signal: u8,
    from_port: Option<Orientation>,
    previous_signal: u8,
    prev_signal_type: Option<SignalType>,
}

#[derive(Resource, Debug)]
pub struct PropagationQueue(Vec<PropagationArgs>);

impl PropagationQueue {
    pub fn append(
        &mut self,
        x: i128,
        y: i128,
        input_signal: u8,
        from_port: Option<Orientation>,
        previous_signal: u8,
        prev_signal_type: Option<SignalType>
    ) {
        self.0.push(PropagationArgs {
            x,
            y,
            input_signal,
            from_port,
            previous_signal,
            prev_signal_type,
        })
    }

    fn execute_queue(
        &mut self,
        chunks: &mut Chunks,
        listeners: &mut EventListeners,
        calculations: &mut u32
    ) {
        let queue = self.0.clone();
        self.0.clear();
        *calculations = 0;
        for job in queue.iter() {
            propagate_signal_at(
                chunks,
                job.x,
                job.y,
                job.from_port,
                job.input_signal,
                job.previous_signal,
                job.prev_signal_type,
                listeners,
                self,
                calculations
            );
        }
    }

    fn is_empty(&self) -> bool {
        self.0.len() <= 0
    }
}

const SLOW_TICK: f64 = 0.5;
const TICK: f64 = 0.02;
const FAST_TICK: f64 = 0.001;

#[derive(Resource)]
pub struct TextureToBlockMap(HashMap<TextureName, Block>);
const DIRT: Block = Block {
    movable: true,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Dirt,
    symmetric: true,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        kind: None,
        signal_type_port_mapping: [None, None, None, None],
        input_ports: [true, true, true, true],
        output_ports: [true, true, true, true],
    }),
    mechanism: None,
};

const SLIME: Block = Block {
    movable: true,
    sticky: true,
    orientation: Orientation::Up,
    texture_name: TextureName::SlimeBlock,
    symmetric: true,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        kind: None,
        signal_type_port_mapping: [None, None, None, None],
        input_ports: [true, true, true, true],
        output_ports: [true, true, true, true],
    }),
    mechanism: None,
};

const REDSTONE_TORCH: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::RedstoneTorch,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 16,
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        signal_type_port_mapping: [
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(false)),
            None,
            Some(SignalType::Strong(false)),
        ],
        input_ports: [false, false, true, false],
        output_ports: [true, true, false, true],
    }),
    mechanism: Some(MechanismKind::RedstoneTorch),
};

const BUTTON: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Button,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        signal_type_port_mapping: [
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(false)),
        ],
        input_ports: [false, false, false, false],
        output_ports: [true, true, true, true],
    }),
    mechanism: Some(MechanismKind::Button),
};

const LEVER: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Lever,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        signal_type_port_mapping: [
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(true)),
            Some(SignalType::Strong(false)),
        ],
        input_ports: [false, false, false, false],
        output_ports: [true, true, true, true],
    }),
    mechanism: Some(MechanismKind::Lever),
};

const REPEATER: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Repeater,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type_port_mapping: [Some(SignalType::Strong(true)), None, None, None],
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, false, true, false],
        output_ports: [true, false, false, false],
    }),
    mechanism: Some(MechanismKind::Repeater { countdown: -1, tick: 0 }),
};

const OBSERVER: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Observer,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type_port_mapping: [Some(SignalType::Strong(true)), None, None, None],
        signal_type: Some(SignalType::Strong(true)),
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, false, false, false],
        output_ports: [true, false, false, false],
    }),
    mechanism: Some(MechanismKind::Observer),
};

const REDSTONE_DUST: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::RedstoneDust,
    symmetric: true,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: Some(SignalType::Weak(true)),
        signal_type_port_mapping: [
            Some(SignalType::Weak(true)),
            Some(SignalType::Weak(true)),
            Some(SignalType::Weak(true)),
            Some(SignalType::Weak(true)),
        ],
        kind: Some(RedstoneKind::Dust),
        input_ports: [true, true, true, true],
        output_ports: [true, true, true, true],
    }),
    mechanism: None,
};

const PISTON: Block = Block {
    movable: true,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::Piston,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        signal_type_port_mapping: [None, None, None, None],
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, true, true, true],
        output_ports: [false, false, false, false],
    }),
    mechanism: Some(MechanismKind::Piston { extended: false, sticky: false }),
};

const PISTON_HEAD: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::PistonHead,
    symmetric: false,
    redstone: None,
    mechanism: None,
};

const STICKY_PISTON: Block = Block {
    movable: true,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::StickyPiston,
    symmetric: false,
    redstone: Some(Redstone {
        signal: 0,
        signal_type: None,
        signal_type_port_mapping: [None, None, None, None],
        kind: Some(RedstoneKind::Mechanism),
        input_ports: [false, true, true, true],
        output_ports: [false, false, false, false],
    }),
    mechanism: Some(MechanismKind::Piston { extended: false, sticky: true }),
};

const STICKY_PISTON_HEAD: Block = Block {
    movable: false,
    sticky: false,
    orientation: Orientation::Up,
    texture_name: TextureName::StickyPistonHead,
    symmetric: false,
    redstone: None,
    mechanism: None,
};



fn main() {
    let chunks = Chunks::new();
    let event_listeners = EventListeners::new();
    let state_dir = dirs
        ::config_dir()
        .map(|dir| dir.join("redstone_rust"))
        .unwrap_or(Path::new("local").join("save"));

    let mut all_blocks = HashMap::from([
        (TextureName::Dirt, DIRT),
        (TextureName::RedstoneDust, REDSTONE_DUST),
        (TextureName::RedstoneTorch, REDSTONE_TORCH),
        (TextureName::Observer, OBSERVER),
        (TextureName::Piston, PISTON),
        (TextureName::PistonHead, PISTON_HEAD),
        (TextureName::StickyPiston, STICKY_PISTON),
        (TextureName::StickyPistonHead, STICKY_PISTON_HEAD),
        (TextureName::Repeater, REPEATER),
        (TextureName::SlimeBlock, SLIME),
        (TextureName::Button, BUTTON),
        (TextureName::Lever, LEVER),
    ]);

    let mut placeable: Vec<Block> = vec![
        DIRT,
        REDSTONE_TORCH,
        REDSTONE_DUST,
        OBSERVER,
        LEVER,
        BUTTON,
        SLIME,
        PISTON,
        STICKY_PISTON,
        REPEATER
    ];

    let wool_textures = [
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
    ];

    for wool in wool_textures{
        let mut wool_blk = DIRT.clone();
        wool_blk.texture_name = wool;
        placeable.push(wool_blk);
        all_blocks.insert(wool, wool_blk);
    }

    let mut default_save = Vec::new();
    let start_x = 0;
    let start_y = 0;

    for (idx, blk) in placeable.iter().enumerate() {
        default_save.push(((start_x as i128, start_y + idx as i128), *blk));
    }

    App::new()
        .add_state::<MyStates>()
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(Time::<Fixed>::from_seconds(TICK))
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::Off)
        .insert_resource(chunks)
        .insert_resource(event_listeners)
        .insert_resource(PropagationQueue(Vec::new()))
        .insert_resource(SelectedBlock(Some(DIRT)))
        .insert_resource(Orientation::Up)
        .insert_resource(Fast(1))
        .insert_resource(TextureToBlockMap(all_blocks))
        .insert_resource(
            Persistent::<SaveData>
                ::builder()
                .name("save data")
                .format(StorageFormat::Json)
                .path(state_dir.join("save_data.json"))
                .default(SaveData(default_save))
                .build()
                .expect("failed to initialize game state")
        )
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::InGame)
                .load_collection::<ImageAssets>()
        )
        .add_systems(Startup, setup_fps_counter)
        .add_systems(Update, (fps_text_update_system, fps_counter_showhide))
        .add_systems(OnEnter(MyStates::InGame), init)
        .add_systems(Update, mouse_pos_update_system.run_if(in_state(MyStates::InGame)))
        .add_systems(FixedUpdate, execute_listeners.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, delayed_redstone_listeners.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, mouse_input.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_selected_block.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, move_camera.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_orientation.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, autosave.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, zoom_camera.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_tick)
        .add_systems(Update, update_cursor_position.run_if(in_state(MyStates::InGame)))
        .add_systems(Update, update_tps_text.run_if(in_state(MyStates::InGame)))
        .run()
}

const AUTOSAVE_INTERVAL_SECONDS: f32 = 3.0;
const UPDATES_TIMER_INTERVAL_SECONDS: f32 = 5.0;

fn init(
    mut commands: Commands,
    save_data: Res<Persistent<SaveData>>,
    mut chunks: ResMut<Chunks>,
    mut listeners: ResMut<EventListeners>,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    mut propagation_queue: ResMut<PropagationQueue>,
    texture_to_block_map: Res<TextureToBlockMap>
) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(BOX_WIDTH * 5.0, BOX_WIDTH * 20.0, 0.),
        ..default()
    });

    commands.insert_resource(AutosaveTimer {
        timer: Timer::new(Duration::from_secs_f32(AUTOSAVE_INTERVAL_SECONDS), TimerMode::Repeating),
    });

    commands.insert_resource(UpdatesPerSecondTimer {
        timer: Timer::new(
            Duration::from_secs_f32(UPDATES_TIMER_INTERVAL_SECONDS),
            TimerMode::Repeating
        ),
        number_of_updates: 0,
    });

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.1),
                custom_size: Some(Vec2::new(BOX_WIDTH, BOX_WIDTH)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..default()
        },
        Cursor,
    ));

    let mut calculations = 0;
    for ((x, y), blk) in save_data.0.iter() {
        place(
            &mut chunks,
            *blk,
            blk.orientation,
            *x,
            *y,
            &mut listeners,
            &mut commands,
            &image_assets,
            &mut query,
            &mut propagation_queue,
            &mut calculations,
            &texture_to_block_map.0
        );
    }
}

#[derive(Component)]
struct Cursor;

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
    } else if keyboard_input.pressed(KeyCode::Key7) {
        selected.0 = Some(OBSERVER);
    } else if keyboard_input.pressed(KeyCode::Key8) {
        selected.0 = Some(SLIME);
    } else if keyboard_input.pressed(KeyCode::Key9) {
        selected.0 = Some(BUTTON);
    } else if keyboard_input.pressed(KeyCode::Key0) {
        selected.0 = Some(LEVER);
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

#[derive(Resource)]
struct Fast(u8);

fn update_tick(
    keyboard_input: Res<Input<KeyCode>>,
    mut time: ResMut<Time<Fixed>>,
    mut fast: ResMut<Fast>
) {
    let tick_rates = [SLOW_TICK, TICK, FAST_TICK];
    if keyboard_input.just_pressed(KeyCode::E) {
        fast.0 = (fast.0 + 1) % 3;
        let current_rate = tick_rates[fast.0 as usize];
        println!("{current_rate}");
        let mutable = time.as_mut();
        *mutable = Time::from_seconds(current_rate);
    }
}

fn update_cursor_position(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut Transform, With<Cursor>>
) {
    let (camera, camera_transform) = q_camera.single();
    let (x, y, _, _) = if
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

    for mut cursor in &mut query {
        cursor.translation = Vec3::new(
            (y as f32) * BOX_WIDTH,
            ((CHUNK_SIZE.0 - x - 1) as f32) * BOX_WIDTH,
            0.0
        );
    }
}

pub fn mouse_input(
    mut commands: Commands,
    mut listeners: ResMut<EventListeners>,
    buttons: Res<Input<MouseButton>>,
    mut selected_block: ResMut<SelectedBlock>,
    orientation: Res<Orientation>,
    mut chunks: ResMut<Chunks>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    mut propagation_queue: ResMut<PropagationQueue>,
    keyboard_input: Res<Input<KeyCode>>,
    texture_to_block_map: Res<TextureToBlockMap>
) {
    let (camera, camera_transform) = q_camera.single();
    let (x, y, x_dist, y_dist) = if
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

    let mut calculations = 0;

    if buttons.just_pressed(MouseButton::Right) {
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            let blk = chunks.get_block_ref(x, y);
            if let Some(blk) = blk {
                selected_block.0 = Some(*texture_to_block_map.0.get(&blk.texture_name).unwrap());
            }
        } else {
            if let Some(blk) = selected_block.get_block() {
                let mut curr_orientation = *orientation;
                if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                    let horiz = if y_dist > 0.5 { Orientation::Right } else { Orientation::Left };
                    let vertical = if x_dist > 0.5 { Orientation::Down } else { Orientation::Up };
                    curr_orientation = if (x_dist - 0.5).abs() > (y_dist - 0.5).abs() {
                        vertical
                    } else {
                        horiz
                    };
                }

                if
                    !place(
                        chunks.as_mut(),
                        blk,
                        curr_orientation,
                        x,
                        y,
                        &mut listeners,
                        &mut commands,
                        &image_assets,
                        &mut query,
                        &mut propagation_queue,
                        &mut calculations,
                        &texture_to_block_map.0
                    )
                {
                    interact(
                        chunks.as_mut(),
                        x,
                        y,
                        &mut commands,
                        &image_assets,
                        &mut query,
                        &mut listeners
                    );
                }
            }
        }
    } else if buttons.just_pressed(MouseButton::Left) {
        destroy(
            chunks.as_mut(),
            x,
            y,
            &mut listeners,
            &mut commands,
            &image_assets,
            &mut query,
            &mut propagation_queue,
            &mut calculations
        );
    }
}

fn get_mouse_coord(x: f32, y: f32) -> (i128, i128, f32, f32) {
    let x_coord_raw = (CHUNK_SIZE.1 as f32) - (y + BOX_WIDTH / 2.0) / BOX_WIDTH;
    let y_coord_raw = (x + BOX_WIDTH / 2.0) / BOX_WIDTH;

    let x_dist = x_coord_raw - x_coord_raw.floor();
    let y_dist = y_coord_raw - y_coord_raw.floor();

    (x_coord_raw.floor() as i128, y_coord_raw.floor() as i128, x_dist, y_dist)
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

            conn_ind * 16 + (signal as usize)
        }
        Block {
            redstone: Some(Redstone { signal, kind: Some(RedstoneKind::Mechanism), .. }),
            mechanism: Some(MechanismKind::RedstoneTorch)
            | Some(MechanismKind::Button)
            | Some(MechanismKind::Lever),
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
        Block {
            redstone: Some(Redstone { signal, .. }),
            mechanism: Some(MechanismKind::Observer),
            ..
        } => {
            if signal > 0 { 1 } else { 0 }
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
        if let Some(entity_handle) = curr_entity {
            commands.entity(*entity_handle).despawn();
        }

        *curr_entity = None;
        chunks.delete_chunk(x, y);
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

fn delayed_redstone_listeners(
    mut listeners: ResMut<EventListeners>,
    mut chunks: ResMut<Chunks>,
    mut propagation_queue: ResMut<PropagationQueue>
) {
    let mut calculations = 0;

    propagation_queue.execute_queue(&mut chunks, &mut listeners, &mut calculations);

    if !propagation_queue.is_empty() {
        return;
    }

    let queue = listeners.repropagation_listener.clone();
    listeners.repropagation_listener.clear();

    for (x, y) in queue.iter() {
        let prev_redstone = get_max_prev(&mut chunks, *x, *y);
        let (from_port, previous_signal, prev_signal_type) = prev_redstone;
        let transmitted_signal = if previous_signal > 0 { previous_signal - 1 } else { 0 };
        propagate_signal_at(
            &mut chunks,
            *x,
            *y,
            from_port,
            transmitted_signal,
            previous_signal,
            prev_signal_type,
            &mut listeners,
            &mut propagation_queue,
            &mut calculations
        );
    }
}

fn execute_listeners(
    mut listeners: ResMut<EventListeners>,
    mut chunks: ResMut<Chunks>,
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut query: Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    mut propagation_queue: ResMut<PropagationQueue>,
    mut updates_timer: ResMut<UpdatesPerSecondTimer>,
    texture_to_block_map: Res<TextureToBlockMap>
) {
    if !propagation_queue.is_empty() {
        return;
    }

    if listeners.repropagation_listener.len() > 0 {
        return;
    }

    let mechanism_listener = listeners.mechanism_listener.clone();
    if mechanism_listener.len() > 0 {
        // println!("mechanism listener {:?}", mechanism_listener);
    }
    listeners.mechanism_listener.clear();
    let mut calculations = 0;
    for ((x, y), on) in mechanism_listener {
        execute_mechanism(
            &mut chunks,
            x,
            y,
            on,
            &mut listeners,
            &mut commands,
            &image_assets,
            &mut query,
            &mut propagation_queue,
            &mut calculations,
            &texture_to_block_map.0
        );
    }

    let redstone_component_listener = listeners.redstone_component_listener.clone();
    listeners.redstone_component_listener.clear();
    if redstone_component_listener.len() > 0 {
        // println!("redstone comp {:?}", redstone_component_listener);
    }

    for ((x, y), on) in redstone_component_listener {
        execute_mechanism(
            &mut chunks,
            x,
            y,
            on,
            &mut listeners,
            &mut commands,
            &image_assets,
            &mut query,
            &mut propagation_queue,
            &mut calculations,
            &texture_to_block_map.0
        );
    }

    let entity_map_update = listeners.entity_map_update.clone();
    for (x, y) in entity_map_update {
        update_entity(&mut commands, &mut chunks, x, y, &image_assets, &mut query);
        alert_neighbours(x, y, &chunks, &mut listeners);
    }
    listeners.entity_map_update.clear();
    updates_timer.number_of_updates += 1;
}

fn interact(
    chunks: &mut Chunks,
    x: i128,
    y: i128,
    commands: &mut Commands,
    image_assets: &ImageAssets,
    query: &mut Query<&mut TextureAtlasSprite, With<BlockComponent>>,
    listeners: &mut EventListeners
) {
    let blk = chunks.get_block(x, y);
    match blk {
        Some(Block { mechanism: Some(MechanismKind::Repeater { tick, .. }), .. }) => {
            *tick = (*tick + 1) % 4;
        }
        Some(Block { mechanism: Some(MechanismKind::Button) | Some(MechanismKind::Lever), .. }) => {
            listeners.turn_mechanism_on(x, y, true);
        }
        _ => {}
    }
    update_entity(commands, chunks, x, y, image_assets, query);
    alert_neighbours(x, y, &chunks, listeners);
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
        for ((chunk_x, chunk_y), chunk) in chunks.0.iter() {
            for (u, row) in chunk.map.iter().enumerate() {
                for (v, blk) in row.iter().enumerate() {
                    let x = chunk_x * CHUNK_SIZE.0 + (u as i128);
                    let y = chunk_y * CHUNK_SIZE.1 + (v as i128);
                    if let Some(blk) = *blk {
                        let tuple = ((x, y), blk);
                        current_state.push(tuple);
                    }
                }
            }
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
                }
            }
        }

        if transform.scale + scale_delta > 0.0 {
            transform.scale += scale_delta;
        } else {
            transform.scale = 0.0;
        }
    }
}

pub fn alert_neighbours(x: i128, y: i128, chunks: &Chunks, listeners: &mut EventListeners) {
    for orientation in Orientation::iter() {
        let (next_x, next_y) = orientation.get_next_coord(x, y);
        listeners.change_state(next_x, next_y, orientation.get_opposing(), &chunks);
    }
}
