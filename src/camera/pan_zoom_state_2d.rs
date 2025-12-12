use crate::{
    app::{app_settings::AppSettings, app_state::AppState},
    camera::main_camera::{ApplyCameraState, CameraMode},
};
use bevy::prelude::{Projection, Resource, Transform, Vec2, Vec3};

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
        app_state: &AppState,
        transform: &mut Transform,
        projection: &mut Projection,
        invalidate: &mut bool,
    ) {
        if !mode.intersects(CameraMode::Pan | CameraMode::Zoom) {
            return;
        }

        let Projection::Orthographic(orthogonal) = projection else {
            return;
        };

        let max_camera_zoom_scale =
            app_state.world_image_max_size.max_element() / app_settings.min_image_size;

        // Clamp the scale.
        orthogonal.scale = (initial_state.scale * delta_zoom)
            .max(app_settings.min_camera_zoom_scale)
            .min(max_camera_zoom_scale);

        let zoom_changed = initial_state.scale - orthogonal.scale;

        let zoom_adjustment = (current_pos - viewport_centre) * zoom_changed;

        transform.translation = initial_state.translation
            - orthogonal.scale * delta_move.reflect(Vec3::Y)
            + zoom_adjustment.extend(0.0);

        if delta_move != Vec3::ZERO || zoom_changed != 0.0 {
            *invalidate = true;
        }
    }
}
