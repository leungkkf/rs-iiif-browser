use crate::{
    AppState, camera::main_camera::MainCamera, rendering::tile::TileModState,
    rendering::tiled_image::TiledImage,
};
use bevy::{
    input::mouse::AccumulatedMouseScroll,
    prelude::{
        ButtonInput, Camera, GlobalTransform, Local, MessageWriter, MouseButton, Projection, Res,
        ResMut, Single, Time, Transform, Vec2, Vec3, Window, With,
    },
    window::{PrimaryWindow, RequestRedraw},
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn mouse_input_system(
    camera_query: Single<
        (&Camera, &GlobalTransform, &mut Transform, &mut Projection),
        With<MainCamera>,
    >,
    mut app_state: ResMut<AppState>,
    local_params: (Local<Option<Vec2>>, Local<Option<f32>>),
    mouse_wheel_input: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    tiled_image: Single<&TiledImage>,
    mut tile_mod_state: ResMut<TileModState>,
    time: Res<Time>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) {
    let (camera, global_transform, mut transform, mut projection) = camera_query.into_inner();

    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return;
    };

    let (mut stored_mouse_pos, mut zoom_debounce) = local_params;

    if let Some(last_zoom) = *zoom_debounce {
        if time.elapsed_secs() - last_zoom > 1.0 / 3.0 {
            *zoom_debounce = None;
            tile_mod_state.invalidate();
        }
        // Keep redrawing while zoom debounce is on.
        redraw_request_writer.write(RequestRedraw);
    }

    if mouse.pressed(MouseButton::Left) {
        // check if the cursor is inside the window and get its position
        // then, ask bevy to convert into world coordinates, and truncate to discard Z
        if let Some(current_mouse_pos) = window
            .cursor_position()
            .map(|cursor| camera.viewport_to_world(global_transform, cursor))
            .map(|ray| ray.unwrap().origin.truncate())
        {
            if mouse.just_pressed(MouseButton::Left) {
                *stored_mouse_pos = Some(current_mouse_pos);
            } else if let Some(mouse_pos) = *stored_mouse_pos {
                transform.translation += Vec3::new(
                    mouse_pos.x - current_mouse_pos.x,
                    mouse_pos.y - current_mouse_pos.y,
                    0.0,
                );
            }
        }
    } else if mouse.just_released(MouseButton::Left) {
        *stored_mouse_pos = None;
        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
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

        *zoom_debounce = Some(time.elapsed_secs());
        redraw_request_writer.write(RequestRedraw);
    }
}
