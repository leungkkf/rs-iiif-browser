use crate::{AppState, tile::TileModState, tiled_image::TiledImage};
use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

#[derive(Component)]
pub(crate) struct MousePosition(Vec2);

pub(crate) fn handle_mouse_input(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<Camera2d>>,
    app_state_query: Single<(Entity, &mut AppState), With<AppState>>,
    stored_mouse_pos: Query<&MousePosition>,
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

    let (app_state_entity, mut app_state) = app_state_query.into_inner();
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

    if stored_mouse_pos.is_empty() {
        if mouse.just_pressed(MouseButton::Left) {
            commands
                .entity(app_state_entity)
                .insert(MousePosition(current_mouse_pos));
        }
    } else {
        if is_pressed {
            let mouse_pos = stored_mouse_pos.single().expect("should have one item");

            transform.translation += Vec3::new(
                mouse_pos.0.x - current_mouse_pos.x,
                mouse_pos.0.y - current_mouse_pos.y,
                0.0,
            );
        }

        if mouse.just_released(MouseButton::Left) {
            commands.entity(app_state_entity).remove::<MousePosition>();
            tile_mod_state.invalidate();
        }
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
