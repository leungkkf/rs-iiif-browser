use crate::{AppState, tile::TileModState, tiled_image::TiledImage};
use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

pub(crate) fn handle_mouse_input(
    camera: Single<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<Camera2d>>,
    mut app_state: Single<&mut AppState>,
    mut stored_mouse_pos: Local<Option<Vec2>>,
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    tiled_image: Single<&TiledImage>,
    mut tile_mod_state: ResMut<TileModState>,
) {
    let (camera, global_transform, mut transform, mut projection) = camera.into_inner();

    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return;
    };

    let mut current_mouse_pos = Vec2::ZERO;
    let is_pressed = mouse.pressed(MouseButton::Left);

    if is_pressed {
        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(world_position) = window
            .cursor_position()
            .map(|cursor| camera.viewport_to_world(global_transform, cursor))
            .map(|ray| ray.unwrap().origin.truncate())
        {
            current_mouse_pos = world_position;
        }
    }

    if let Some(mouse_pos) = *stored_mouse_pos {
        if is_pressed {
            transform.translation += Vec3::new(
                mouse_pos.x - current_mouse_pos.x,
                mouse_pos.y - current_mouse_pos.y,
                0.0,
            );
        }

        if mouse.just_released(MouseButton::Left) {
            *stored_mouse_pos = None;
            tile_mod_state.invalidate();
        }
    } else if mouse.just_pressed(MouseButton::Left) {
        *stored_mouse_pos = Some(current_mouse_pos);
    }

    let delta_zoom = 1.0 - mouse_wheel_input.delta.y * 0.1;

    if delta_zoom != 1.0 {
        // Zoom at the mouse position.
        if let Some(mouse_pos) = window.cursor_position() {
            let zoom_changed = orthogonal.scale * (1.0 - delta_zoom);
            let viewport = camera
                .logical_viewport_rect()
                .expect("camera should have a viewport rect");
            let delta_x = (mouse_pos.x - viewport.center().x) * zoom_changed;
            let delta_y = -(mouse_pos.y - viewport.center().y) * zoom_changed;

            transform.translation += Vec3::new(delta_x, delta_y, 0.0);
        }

        orthogonal.scale *= delta_zoom;

        app_state.level = tiled_image.get_level_at(orthogonal.scale);

        tile_mod_state.invalidate();
    }
}
