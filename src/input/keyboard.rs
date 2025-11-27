use crate::{
    AppState, camera::main_camera::MainCamera, rendering::tile::TileModState,
    rendering::tiled_image::TiledImage,
};
use bevy::{
    prelude::{
        ButtonInput, KeyCode, MessageWriter, Projection, Res, ResMut, Single, Transform, Vec3, With,
    },
    window::RequestRedraw,
};

pub(crate) fn keyboard_input_system(
    camera: Single<(&mut Transform, &mut Projection), With<MainCamera>>,
    mut app_state: ResMut<AppState>,
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
        scale += 0.1;
    } else if kb_input.just_pressed(KeyCode::KeyX) {
        scale += -0.1;
    } else {
        key_pressed = false;
    }

    if key_pressed {
        transform.translation += direction;
        orthogonal.scale *= scale;

        app_state.level = tiled_image.get_level_at(orthogonal.scale);

        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
    }
}
