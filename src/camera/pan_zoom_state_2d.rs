use crate::{
    app::{app_settings::AppSettings, app_state::AppState},
    camera::main_camera::{ApplyCameraState, CameraMode, Invalidate},
    rendering::tiled_image::TiledImage,
};
use bevy::prelude::{Projection, Query, Resource, Transform, Vec2, Vec3};

#[derive(Resource, Clone, Default)]
pub(crate) struct PanZoomState2d {
    pub(crate) translation: Vec3,
    pub(crate) scale: f32,
}

impl ApplyCameraState for PanZoomState2d {
    fn get_initial_state(&self, transform: &Transform, projection: &Projection) -> Self {
        let scale = match projection {
            Projection::Orthographic(orthographic) => orthographic.scale,
            _ => 1.0,
        };

        Self {
            scale,
            translation: transform.translation,
        }
    }

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
    ) {
        // Doing pan or zoom?
        if !mode.intersects(CameraMode::Pan | CameraMode::Zoom) {
            return;
        }
        // Are we in orthogonal project?
        let Projection::Orthographic(orthogonal) = projection else {
            return;
        };

        // Set the delta zoom if we are in the zoom mode, 1.0 if not.
        let delta_zoom_with_mode = if mode.intersects(CameraMode::Zoom) {
            delta_zoom
        } else {
            1.0
        };

        // Get the max zoom scale for clamping.
        let max_camera_zoom_scale =
            app_state.world_image_max_size.max_element() / app_settings.min_image_size;
        // Clamp the scale.
        let scale = (initial_state.scale * delta_zoom_with_mode)
            .max(app_settings.min_camera_zoom_scale)
            .min(max_camera_zoom_scale);

        // Get the change in the scale.
        let delta_scale = initial_state.scale - scale;

        // Adjust translation to zoom at the current position.
        let move_due_to_zoom = (current_pos - viewport_centre).reflect(Vec2::Y) * delta_scale;

        // Set the delta move if we are in the pan mode, 0 if not.
        let delta_move_with_mode = if mode.intersects(CameraMode::Pan) {
            delta_move.reflect(Vec3::Y)
        } else {
            Vec3::ZERO
        };

        // Apply the changes to the projection and transform.
        if delta_move != Vec3::ZERO || delta_scale != 0.0 {
            orthogonal.scale = scale;

            if let Ok(tiled_image) = tiled_image.single() {
                app_state.level = tiled_image.get_level_at(orthogonal.scale);
            }

            transform.translation = initial_state.translation
                - orthogonal.scale * delta_move_with_mode
                + move_due_to_zoom.extend(0.0);

            if delta_move != Vec3::ZERO {
                *invalidate |= Invalidate::Translate;
            }
            if delta_scale != 0.0 {
                *invalidate |= Invalidate::Zoom;
            }
        }
    }
}
