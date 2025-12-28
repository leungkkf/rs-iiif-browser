use crate::{
    app::{app_settings::AppSettings, app_state::AppState},
    camera::main_camera::{ApplyCameraState, CameraMode, Invalidate},
    rendering::{tile::TileModState, tiled_image::TiledImage},
};
use bevy::{
    ecs::{component::Component, resource::Resource},
    input::touch::Touch,
    prelude::{
        Camera, Local, MessageWriter, Projection, Query, Res, ResMut, Single, Touches, Transform,
        Vec2, With,
    },
    window::RequestRedraw,
};

#[derive(Default)]
pub(crate) struct TouchHistory {
    touches: [Option<Touch>; 2],
}

fn compute_centre_and_distance_squared(touches: [&Touch; 2]) -> (Vec2, f32) {
    let centre = (touches[0].position() + touches[1].position()) / 2.0;
    let distance = touches[1]
        .position()
        .distance_squared(touches[0].position());

    (centre, distance)
}

/// Touch input system for 2D and 3D to collect and process the touch events.
///
/// The function handles the logic to collect and process the touch events,
/// in order to call the following functions when the pan/zoom/orbit operation start.
/// ApplyCameraState::get_initial_state will be called to remember the initial state
/// of pan/zoom/orbit operations when the operations are about to start,
/// ApplyCameraState::apply will be called with the delta changes
/// and other information to apply the current state to the transform and projection.
#[allow(clippy::too_many_arguments)]
pub(crate) fn touch_input_system<T: Component, S: Resource + Clone + Default + ApplyCameraState>(
    camera_query: Single<(&mut Transform, &Camera, &mut Projection), With<T>>,
    touches: Res<Touches>,
    app_settings: Res<AppSettings>,
    mut app_state: ResMut<AppState>,
    mut touch_history: Local<TouchHistory>,
    mut initial_state: Local<S>,
    mut current_state: ResMut<S>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
    tiled_image: Query<&TiledImage>,
) {
    let (mut transform, camera, mut projection) = camera_query.into_inner();

    let viewport_centre = camera
        .logical_viewport_rect()
        .expect("camera should have a viewport rect")
        .center();

    // Reset it when any event is cancelled or is released.
    if touches.any_just_canceled() || touches.any_just_released() {
        touch_history.touches = [None; 2];
    }

    let all_pressed_events: Vec<_> = touches.iter().collect();

    // Nothing to do if nothing is pressed.
    if all_pressed_events.is_empty() {
        return;
    }

    // Fill up the history if there are empty slot.
    for just_pressed in touches.iter_just_pressed() {
        let filled_items = touch_history.touches.iter().filter(|x| x.is_some()).count();

        // Just fill in the first one.
        if filled_items == 0 {
            touch_history.touches[0] = Some(*just_pressed);
            *initial_state = current_state.get_initial_state(&transform, &projection);
        }
        // If the new press is not the same as the existing one,
        // fill it in and mark the start scale and translation.
        else if filled_items == 1 {
            let index = touch_history
                .touches
                .iter()
                .enumerate()
                .filter(|(_, touch)| {
                    touch.is_some_and(|x| {
                        x.id() != just_pressed.id()
                            && x.position().distance(just_pressed.position()) > 0.01
                    })
                })
                .map(|(index, _)| 1 - index)
                .next();

            if let Some(index) = index {
                touch_history.touches[index] = Some(*just_pressed);
                *initial_state = current_state.get_initial_state(&transform, &projection);
            }
        }
    }

    let mut invalidate = Invalidate::empty();

    // If two pressed events and the history records are filled,
    // zoom and translate according to the events.
    if all_pressed_events.len() == 2 && touch_history.touches.iter().all(|x| x.is_some()) {
        let (current_centre, current_distance_squared) = compute_centre_and_distance_squared(
            all_pressed_events[0..2]
                .try_into()
                .expect("should have two items in the pressed events"),
        );

        let initial_touch_1 =
            touch_history.touches[0].expect("should have two items in the history");
        let initial_touch_2 =
            touch_history.touches[1].expect("should have two items in the history");
        let (initial_centre, start_distance_squared) =
            compute_centre_and_distance_squared([&initial_touch_1, &initial_touch_2]);

        let delta_zoom = start_distance_squared / current_distance_squared.max(0.01);
        let delta_move = (current_centre - initial_centre).extend(0.0);

        current_state.apply(
            CameraMode::Pan | CameraMode::Zoom,
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
    } else if all_pressed_events.len() == 1 && touch_history.touches.iter().any(|x| x.is_some()) {
        let start_pos = touch_history
            .touches
            .iter()
            .filter_map(|x| x.map(|y| y.position()))
            .next()
            .expect("should have one item in the history");
        let current_pos = all_pressed_events[0].position();
        let delta_move = (current_pos - start_pos).extend(0.0);

        current_state.apply(
            CameraMode::Orbit,
            &initial_state,
            current_pos,
            viewport_centre,
            1.0,
            delta_move,
            &app_settings,
            &mut app_state,
            tiled_image,
            &mut transform,
            &mut projection,
            &mut invalidate,
        );
    }

    if !invalidate.is_empty() {
        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
    }
}
