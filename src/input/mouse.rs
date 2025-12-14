use crate::{
    AppState,
    app::app_settings::AppSettings,
    camera::main_camera::{ApplyCameraState, CameraMode, Invalidate},
    rendering::{tile::TileModState, tiled_image::TiledImage},
};
use bevy::{
    input::mouse::MouseWheel,
    prelude::{
        ButtonInput, Camera, Component, Local, MessageReader, MessageWriter, MouseButton,
        Projection, Query, Res, ResMut, Resource, Single, Time, Transform, Vec2, Window, With,
    },
    window::{CursorMoved, PrimaryWindow, RequestRedraw},
};

/// Mouse input system for 3D.
#[allow(clippy::too_many_arguments)]
pub(crate) fn mouse_input_system<T: Component, S: Resource + Clone + Default + ApplyCameraState>(
    camera_query: Single<(&mut Transform, &Camera, &mut Projection), With<T>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut evr_motion: MessageReader<CursorMoved>,
    mut evr_scroll: MessageReader<MouseWheel>,
    app_settings: Res<AppSettings>,
    mut app_state: ResMut<AppState>,
    mut initial_state: Local<S>,
    mut current_state: ResMut<S>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
    time: Res<Time>,
    mut zoom_debounce: Local<Option<f32>>,
    tiled_image: Query<&TiledImage>,
) {
    if let Some(last_zoom) = *zoom_debounce {
        if time.elapsed_secs() - last_zoom > 1.0 / 4.0 {
            *zoom_debounce = None;
            tile_mod_state.invalidate();
        }
        // Keep redrawing while zoom debounce is on.
        redraw_request_writer.write(RequestRedraw);
    }

    let (mut transform, camera, mut projection) = camera_query.into_inner();

    let viewport_centre = camera
        .logical_viewport_rect()
        .expect("camera should have a viewport rect")
        .center();

    // First, accumulate the total amount of
    // mouse motion and scroll, from all pending events:
    let delta_motion: Vec2 = evr_motion
        .read()
        .map(|ev| ev.delta.unwrap_or_default())
        .sum();
    let delta_wheel: f32 = evr_scroll
        .read()
        .map(|ev| if ev.y != 0.0 { ev.y.signum() } else { 0.0 })
        .sum();

    let delta_zoom = 1.0 - delta_wheel * 0.1;
    let delta_move = delta_motion.extend(0.0);

    let mut invalidate = Invalidate::empty();

    let current_centre = window.cursor_position().unwrap_or_default();

    *initial_state = current_state.get_initial_state(&transform, &projection);

    let mode = if mouse.pressed(MouseButton::Left) && !mouse.pressed(MouseButton::Right) {
        CameraMode::Pan
    } else if mouse.pressed(MouseButton::Right) && !mouse.pressed(MouseButton::Left) {
        CameraMode::Orbit
    } else {
        CameraMode::Zoom
    };

    current_state.apply(
        mode,
        &initial_state,
        current_centre,
        viewport_centre,
        delta_zoom,
        delta_move,
        &app_settings,
        &mut app_state,
        tiled_image,
        &mut transform,
        &mut projection,
        &mut invalidate,
    );

    if invalidate.intersects(Invalidate::Translate) {
        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
    } else if invalidate.intersects(Invalidate::Zoom) {
        *zoom_debounce = Some(time.elapsed_secs());
        redraw_request_writer.write(RequestRedraw);
    }
}
