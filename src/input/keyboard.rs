use crate::{
    AppState,
    app::app_settings::AppSettings,
    camera::main_camera::MainCamera2d,
    rendering::{tile::TileModState, tiled_image::TiledImage},
};
use bevy::{
    prelude::{
        ButtonInput, KeyCode, MessageWriter, Projection, Res, ResMut, Single, Transform, Vec3, With,
    },
    window::RequestRedraw,
};

pub(crate) fn keyboard_input_system(
    camera: Single<(&mut Transform, &mut Projection), With<MainCamera2d>>,
    mut app_state: ResMut<AppState>,
    app_settings: Res<AppSettings>,
    tiled_image: Single<&TiledImage>,
    kb_input: Res<ButtonInput<KeyCode>>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) {
    let (mut transform, mut projection) = camera.into_inner();

    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return;
    };
    let mut direction = Vec3::new(0.0, 0.0, 0.0);
    let mut scale = 1.0;
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
        scale *= 0.9;
    } else if kb_input.just_pressed(KeyCode::KeyX) {
        scale *= 1.1;
    } else {
        key_pressed = false;
    }

    if key_pressed {
        transform.translation += direction;

        let max_camera_zoom_scale = tiled_image.get_world_max_size_rect().size().max_element()
            / app_settings.min_image_size;

        orthogonal.scale = (orthogonal.scale * scale)
            .max(app_settings.min_camera_zoom_scale)
            .min(max_camera_zoom_scale);

        app_state.level = tiled_image.get_level_at(orthogonal.scale);

        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
    }
}
