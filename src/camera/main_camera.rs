use crate::{
    app::{app_settings::AppSettings, app_state::AppState},
    camera::camera_ext::get_world_viewport_rect,
    rendering::tiled_image::TiledImage,
};
use bevy::prelude::{
    Camera, Component, GlobalTransform, Projection, Query, Rect, Single, Transform, Vec2, Vec3,
    With,
};
use bitflags::bitflags;

#[derive(Component)]
pub(crate) struct MainCamera2d;

#[derive(Component)]
pub(crate) struct MainCamera3d;

bitflags! {
    pub(crate) struct CameraMode: u32 {
        const Pan = 0b00000001;
        const Zoom = 0b00000010;
        const Orbit = 0b00000100;
    }
}

bitflags! {
    pub(crate) struct Invalidate: u8 {
        const Translate = 0b00000001;
        const Zoom = 0b00000010;
    }
}

pub(crate) trait ApplyCameraState {
    /// Get the initial state of the camera and
    /// will be used as the starting point of the running state.
    fn get_initial_state(&self, transform: &Transform, projection: &Projection) -> Self;

    /// Apply the state to the transform and the projection,
    /// given the initial state and the deltas and other related information.
    ///
    /// * `mode` - Camera mode.
    /// * `initial_state` - Initial state.
    /// * `current_pos` - Current position of the operation.
    /// * `viewport_centre` - Centre of the camera viewport.
    /// * `delta_zoom` - Amount of zoom changed of the operation.
    /// * `delta_move` - Amount of move changed of the operation.
    /// * `app_settings` - Application settings.
    /// * `app_state` - Application state.
    /// * `transform` - Camera transform to be updated.  
    /// * `projection` - Camera projection to be updated.
    /// * `invalidate` - Should be set if tile update is required.
    #[allow(clippy::too_many_arguments)]
    fn apply(
        &mut self,
        mode: CameraMode,
        initial_state: &Self,
        current_pos: Vec2,
        viewport_centre: Vec2,
        delta_zoom: f32,
        delta_move: Vec3,
        app_settings: &AppSettings,
        app_state: &mut AppState,
        tiled_image: Query<&TiledImage>,
        transform: &mut Transform,
        projection: &mut Projection,
        invalidate: &mut Invalidate,
    );
}

/// Keep the image within the viewport.
pub(crate) fn translation_bounding_system(
    camera: Single<(&Camera, &GlobalTransform, &mut Transform), With<MainCamera2d>>,
    tiled_image: Single<&TiledImage>,
) {
    let (camera, global_transform, mut transform) = camera.into_inner();

    let Some((world_viewport_min, world_viewport_max)) =
        get_world_viewport_rect(camera, global_transform)
    else {
        return;
    };

    let world_viewport_rect =
        Rect::from_corners(world_viewport_min.truncate(), world_viewport_max.truncate());

    let world_margin = camera
        .viewport_to_world(global_transform, Vec2::splat(8.0))
        .expect("should transform to world space")
        .origin
        - camera
            .viewport_to_world(global_transform, Vec2::ZERO)
            .expect("should transform to world space")
            .origin;

    let abs_world_margin = world_margin.abs().x;

    let world_image_rect = tiled_image.get_world_max_size_rect();

    // The camera (viewport) should see at least some of the image (given by the margin).
    // The center of the camera should be bounded by the world image rect and a margin.
    let world_bounding_rect = Rect::from_corners(
        world_image_rect.min + abs_world_margin - world_viewport_rect.half_size(),
        world_image_rect.max - abs_world_margin + world_viewport_rect.half_size(),
    );

    let bounded_translation = transform
        .translation
        .truncate()
        .max(world_bounding_rect.min)
        .min(world_bounding_rect.max);

    transform.translation = bounded_translation.extend(transform.translation.z);
}
