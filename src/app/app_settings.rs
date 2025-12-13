use bevy::prelude::Resource;

pub(crate) struct PanOrbitSettings {
    /// World units per pixel of mouse motion
    pub(crate) pan_sensitivity: f32,
    /// Radians per pixel of mouse motion
    pub(crate) orbit_sensitivity: f32,
}

impl Default for PanOrbitSettings {
    fn default() -> Self {
        PanOrbitSettings {
            pan_sensitivity: 0.002,                 // 2000 pixels per world unit
            orbit_sensitivity: 0.5f32.to_radians(), // 0.5 degree per pixel
        }
    }
}

#[derive(Resource)]
pub(crate) struct AppSettings {
    /// Max number of items in the tile cache.
    pub(crate) max_cache_items: usize,
    /// Thumbnail size in the side panel.
    pub(crate) thumbnail_size: f32,
    /// Min zoom scale in the camera (which is the max zoom-in) allowed at the full image size.
    /// E.g. 1.0/4.0 means that max of 4 times magification.
    pub(crate) min_camera_zoom_scale: f32,
    /// Min image size allowed when zoom-out.
    pub(crate) min_image_size: f32,
    /// User language setting, e.g. "en", "fr".
    pub(crate) language: String,
    /// Camera 3D pan orbit settings.
    pub(crate) pan_orbit_settings: PanOrbitSettings,
}

impl AppSettings {
    fn new(
        max_cache_items: usize,
        thumbnail_size: f32,
        min_camera_zoom_scale: f32,
        min_image_size: f32,
        language: String,
        pan_orbit_settings: PanOrbitSettings,
    ) -> Self {
        Self {
            max_cache_items,
            thumbnail_size,
            min_camera_zoom_scale,
            min_image_size,
            language,
            pan_orbit_settings,
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings::new(
            4096,
            64.0,
            1.0 / 4.0,
            256.0,
            crate::iiif::manifest::language::EN.to_string(),
            PanOrbitSettings::default(),
        )
    }
}
