use crate::{
    app::app_settings::AppSettings,
    camera::main_camera::MainCamera,
    rendering::{tile::TileModState, tiled_image::TiledImage},
};
use bevy::{
    input::touch::Touch,
    prelude::{
        Camera, Local, MessageWriter, Projection, Res, ResMut, Single, Touches, Transform, Vec2,
        Vec3, With,
    },
    window::RequestRedraw,
};

#[derive(Default)]
pub(crate) struct TouchHistory {
    start_translation: Vec3,
    start_scale: f32,
    touches: [Option<Touch>; 2],
}

fn compute_centre_and_distance_squared(touches: [&Touch; 2]) -> (Vec2, f32) {
    let centre = (touches[0].position() + touches[1].position()) / 2.0;
    let distance = touches[1]
        .position()
        .distance_squared(touches[0].position());

    (centre, distance)
}

pub(crate) fn touch_input_system(
    camera_query: Single<(&Camera, &mut Transform, &mut Projection), With<MainCamera>>,
    touches: Res<Touches>,
    app_settings: Res<AppSettings>,
    mut touch_history: Local<TouchHistory>,
    tiled_image: Single<&TiledImage>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) {
    let (camera, mut transform, mut projection) = camera_query.into_inner();

    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return;
    };

    // Reset it when any event is cancelled.
    if touches.any_just_canceled() {
        touch_history.touches = [None; 2];
    }

    // Remove the touch from the history.
    for just_released in touches.iter_just_released() {
        for start_touch in touch_history.touches.iter_mut() {
            if start_touch.is_some_and(|x| x.id() == just_released.id()) {
                start_touch.take();
            }
        }
    }

    let all_pressed_events: Vec<_> = touches.iter().collect();

    // Nothing to do if nothing is pressed.
    if all_pressed_events.is_empty() {
        return;
    }

    // Fill up the history if there are empy slot.
    for just_pressed in touches.iter_just_pressed() {
        let filled_items = touch_history.touches.iter().filter(|x| x.is_some()).count();

        // Just fill in the first one.
        if filled_items == 0 {
            touch_history.touches[0] = Some(*just_pressed);
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

                touch_history.start_scale = orthogonal.scale;
                touch_history.start_translation = transform.translation;
            }
        }
    }

    // If two pressed events and the history records are filled,
    // zoom and translate according to the events.
    if all_pressed_events.len() == 2 && touch_history.touches.iter().all(|x| x.is_some()) {
        let (current_centre, current_distance_squared) = compute_centre_and_distance_squared(
            all_pressed_events[0..2]
                .try_into()
                .expect("should have two items in the pressed events"),
        );

        let start_1 = touch_history.touches[0].expect("should have two items in the history");
        let start_2 = touch_history.touches[1].expect("should have two items in the history");
        let (start_centre, start_distance_squared) =
            compute_centre_and_distance_squared([&start_1, &start_2]);

        let delta_zoom = start_distance_squared / current_distance_squared.max(0.01);
        let max_camera_zoom_scale = tiled_image.get_world_max_size_rect().size().max_element()
            / app_settings.min_image_size;

        // Clamp the scale.
        orthogonal.scale = (touch_history.start_scale * delta_zoom)
            .max(app_settings.min_camera_zoom_scale)
            .min(max_camera_zoom_scale);

        let zoom_changed = touch_history.start_scale - orthogonal.scale;

        let viewport = camera
            .logical_viewport_rect()
            .expect("camera should have a viewport rect");
        let delta_x = (start_centre.x - viewport.center().x) * zoom_changed;
        let delta_y = -(start_centre.y - viewport.center().y) * zoom_changed;

        let touch_moved =
            (orthogonal.scale * (current_centre - start_centre).reflect(Vec2::Y)).extend(0.0);

        transform.translation =
            touch_history.start_translation - touch_moved + Vec3::new(delta_x, delta_y, 0.0);

        if touch_moved.length_squared() != 0.0 && zoom_changed != 0.0 {
            tile_mod_state.invalidate();
            redraw_request_writer.write(RequestRedraw);
        }
    }
}
