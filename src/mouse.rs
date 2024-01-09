use super::*;

pub fn mouse_input(
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
