use crate::{AppState, image::Image, tile::Tile};
use bevy::prelude::*;

pub(crate) fn handle_keyboard_input(
    mut commands: Commands,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut app_state: Single<&mut AppState, With<AppState>>,
    image_details: Single<&Image>,
    kb_input: Res<ButtonInput<KeyCode>>,
    tiles: Query<(Entity, &Tile), With<Tile>>,
) {
    let (mut transform, mut projection) = camera.into_inner();

    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return;
    };
    let mut direction = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 0.0;
    let mut key_pressed = true;

    if kb_input.pressed(KeyCode::ArrowUp) {
        direction.y += 5.0;
    } else if kb_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 5.0;
    } else if kb_input.pressed(KeyCode::ArrowLeft) {
        direction.x += 5.0;
    } else if kb_input.pressed(KeyCode::ArrowRight) {
        direction.x -= 5.0;
    } else if kb_input.just_pressed(KeyCode::KeyZ) {
        scale = 0.1;
    } else if kb_input.just_pressed(KeyCode::KeyX) {
        scale = -0.1;
    } else {
        key_pressed = false;
    }

    if key_pressed {
        transform.translation += direction;
        orthogonal.scale += scale;

        if orthogonal.scale <= 1.0 / 2.0 && app_state.level < image_details.levels().len() - 1 {
            app_state.level += 1;
            orthogonal.scale *= 2.0;
            transform.translation *= 2.0;

            for (entity, tile) in tiles.iter() {
                if tile.level != app_state.level {
                    commands.entity(entity).despawn();
                }
            }
        } else if orthogonal.scale > 2.0 && app_state.level > 0 {
            app_state.level -= 1;
            orthogonal.scale /= 2.0;
            transform.translation /= 2.0;

            for (entity, tile) in tiles.iter() {
                if tile.level != app_state.level {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
