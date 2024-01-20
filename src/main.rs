use bevy::{ prelude::*, utils::HashMap, window::PrimaryWindow };
use bevy::input::mouse::MouseWheel;

use bevy_asset_loader::prelude::*;

pub mod block;
use bevy_asset_loader::loading_state::{ LoadingStateAppExt, LoadingState };
pub use block::*;

mod camera;
pub use camera::*;

mod mouse;
pub use mouse::*;

mod debug;
pub use debug::*;

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
        kind: MechanismKind::Piston { extended: false },
        input_ports: [false, true, true, true],
        signal: 0,
    }),
};

const STICKY_PISTON: Block = Block {
    movable: true,
    texture_name: TextureName::StickyPiston { extended: false },
    orientation: Orientation::Up,
    kind: BlockKind::Mechanism(Mechanism {
        kind: MechanismKind::StickyPiston { extended: false },
        input_ports: [false, true, true, true],
        signal: 0,
    }),
};

const PISTON_HEAD: Block = Block {
    movable: false,
    texture_name: TextureName::PistonHead,
    orientation: Orientation::Up,
    kind: BlockKind::Transparent,
};

const STICKY_PISTON_HEAD: Block = Block {
    movable: false,
    texture_name: TextureName::StickyPistonHead,
    orientation: Orientation::Up,
    kind: BlockKind::Transparent,
};

const TICK: f64 = 0.2;

fn main() {
    let mut world_map = [[None; MAP_SIZE.1]; MAP_SIZE.0];
    world_map[0][0] = Some(REDSTONE_TORCH);
    let entity_map = [[None; MAP_SIZE.1]; MAP_SIZE.0];
    App::new()
        .add_state::<MyStates>()
        .add_plugins(DefaultPlugins)
        .insert_resource(WorldMap(world_map))
        .insert_resource(EntityMap(entity_map))
        .insert_resource(TextureMap(HashMap::new()))
        .insert_resource(Orientation::Up)
        .insert_resource(EventListener::new())
        .insert_resource(SelectedBlock(None))
        .insert_resource(Time::<Fixed>::from_seconds(TICK))
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<ImageAssets>()
        )
        .add_systems(OnEnter(MyStates::Next), init)
        .add_systems(Update, move_camera.run_if(in_state(MyStates::Next)))
        .add_systems(Update, mouse_input.run_if(in_state(MyStates::Next)))
        .add_systems(Update, zoom_camera.run_if(in_state(MyStates::Next)))
        .add_systems(Update, update_selected_block.run_if(in_state(MyStates::Next)))
        .add_systems(FixedUpdate, (run_listeners).run_if(in_state(MyStates::Next)))
        .add_systems(Update, entity_map_listener.run_if(in_state(MyStates::Next)))
        .add_systems(Update, update_orientation.run_if(in_state(MyStates::Next)))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}

#[derive(Resource)]
pub struct WorldMap([[Option<Block>; MAP_SIZE.1]; MAP_SIZE.0]);

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
    image_assets: Res<ImageAssets>
) {
    commands.spawn(Camera2dBundle {
        // transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });

    for (x, row) in map.as_ref().0.iter().enumerate() {
        for (y, blk) in row.iter().enumerate() {
            let sprite_bundle = get_sprite(x, y, blk, image_assets.as_ref());
            commands.spawn(get_sprite(x, y, &None, image_assets.as_ref()));
            let entity = commands.spawn((sprite_bundle, BlockComponent)).id();

            entity_map.as_mut().0[x][y] = Some(entity);
        }
    }
    //println!("{:?}", map.0);
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

fn run_listeners(mut listeners: ResMut<EventListener>, mut world_map: ResMut<WorldMap>) {
    redstone_torch_delayed_listener(listeners.as_mut(), world_map.as_mut());
    repeater_listener(listeners.as_mut(), world_map.as_mut());
    mechanism_listener(listeners.as_mut(), world_map.as_mut());
}
