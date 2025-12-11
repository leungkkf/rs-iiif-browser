use crate::{
    AppState,
    app::app_settings::AppSettings,
    camera::main_camera::{MainCamera2d, PanOrbitState},
    rendering::{tile::TileModState, tiled_image::TiledImage},
};
use bevy::{
    input::mouse::{AccumulatedMouseScroll, MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::{
        ButtonInput, Camera, Camera3d, EulerRot, GlobalTransform, Local, MessageReader,
        MessageWriter, MouseButton, Projection, Quat, Res, ResMut, Single, Time, Transform, Vec2,
        Vec3, Window, With,
    },
    window::{PrimaryWindow, RequestRedraw},
};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

#[allow(clippy::too_many_arguments)]
pub(crate) fn mouse_input_system(
    camera_query: Single<
        (&Camera, &GlobalTransform, &mut Transform, &mut Projection),
        With<MainCamera2d>,
    >,
    mut app_state: ResMut<AppState>,
    app_settings: Res<AppSettings>,
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

    let max_zoom_scale =
        tiled_image.get_world_max_size_rect().size().max_element() / app_settings.min_image_size;
    let max_delta_zoom = max_zoom_scale / orthogonal.scale;
    let min_delta_zoom = app_settings.min_camera_zoom_scale / orthogonal.scale;

    // Keep it consistent across different platforms.
    let wheel_delta = if mouse_wheel_input.delta.y != 0.0 {
        mouse_wheel_input.delta.y.signum()
    } else {
        0.0
    };

    let delta_zoom = (1.0 - wheel_delta * 0.1)
        .max(min_delta_zoom)
        .min(max_delta_zoom);

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

/// Mouse input system for 3D.
/// Taken from https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
pub(crate) fn mouse_input_system_3d(
    mouse: Res<ButtonInput<MouseButton>>,
    mut evr_motion: MessageReader<MouseMotion>,
    mut evr_scroll: MessageReader<MouseWheel>,
    app_settings: Res<AppSettings>,
    mut state: ResMut<PanOrbitState>,
    mut transform: Single<&mut Transform, With<Camera3d>>,
) {
    let settings = &app_settings.pan_orbit_settings;
    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let mut total_motion: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();

    // Reverse Y (Bevy's Worldspace coordinate system is Y-Up,
    // but events are in window/ui coordinates, which are Y-Down)
    total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    // Check how much of each thing we need to apply.
    // Accumulate values from motion and scroll,
    // based on our configuration settings.

    let mut total_pan = Vec2::ZERO;

    if mouse.pressed(MouseButton::Right) && !mouse.pressed(MouseButton::Left) {
        total_pan -= total_motion * settings.pan_sensitivity;
    }

    let mut total_orbit = Vec2::ZERO;

    if mouse.pressed(MouseButton::Left) && !mouse.pressed(MouseButton::Right) {
        total_orbit -= total_motion * settings.orbit_sensitivity;
    }

    let mut total_zoom = Vec2::ZERO;

    total_zoom -= total_scroll_lines * settings.scroll_line_sensitivity * settings.zoom_sensitivity;
    total_zoom -=
        total_scroll_pixels * settings.scroll_pixel_sensitivity * settings.zoom_sensitivity;

    // Upon starting a new orbit maneuver (key is just pressed),
    // check if we are starting it upside-down
    if mouse.just_pressed(MouseButton::Left) && !mouse.pressed(MouseButton::Right) {
        state.upside_down = state.pitch < -FRAC_PI_2 || state.pitch > FRAC_PI_2;
    }

    // If we are upside down, reverse the X orbiting
    if state.upside_down {
        total_orbit.x = -total_orbit.x;
    }

    // Now we can actually do the things!

    let mut any = false;

    // To ZOOM, we need to multiply our radius.
    if total_zoom != Vec2::ZERO {
        any = true;
        // in order for zoom to feel intuitive,
        // everything needs to be exponential
        // (done via multiplication)
        // not linear
        // (done via addition)

        // so we compute the exponential of our
        // accumulated value and multiply by that
        state.radius *= (-total_zoom.y).exp();
    }

    // To ORBIT, we change our pitch and yaw values
    if total_orbit != Vec2::ZERO {
        any = true;
        state.yaw += total_orbit.x;
        state.pitch += total_orbit.y;
        // wrap around, to stay between +- 180 degrees
        if state.yaw > PI {
            state.yaw -= TAU; // 2 * PI
        }
        if state.yaw < -PI {
            state.yaw += TAU; // 2 * PI
        }
        if state.pitch > PI {
            state.pitch -= TAU; // 2 * PI
        }
        if state.pitch < -PI {
            state.pitch += TAU; // 2 * PI
        }
    }

    // To PAN, we can get the UP and RIGHT direction
    // vectors from the camera's transform, and use
    // them to move the center point. Multiply by the
    // radius to make the pan adapt to the current zoom.
    if total_pan != Vec2::ZERO {
        any = true;
        let radius = state.radius;
        state.center += transform.right() * total_pan.x * radius;
        state.center += transform.up() * total_pan.y * radius;
    }

    // Finally, compute the new camera transform.
    // (if we changed anything, or if the pan-orbit
    // controller was just added and thus we are running
    // for the first time and need to initialize)
    if any || state.is_added {
        // YXZ Euler Rotation performs yaw/pitch/roll.
        transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        // To position the camera, get the backward direction vector
        // and place the camera at the desired radius from the center.
        transform.translation = state.center + transform.back() * state.radius;
        state.is_added = false;
    }
}
