use super::*;

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
pub fn zoom_camera(
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