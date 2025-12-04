use bevy::prelude::Resource;

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
}

impl AppSettings {
    pub(crate) fn new(
        max_cache_items: usize,
        thumbnail_size: f32,
        min_camera_zoom_scale: f32,
        min_image_size: f32,
        language: String,
    ) -> Self {
        Self {
            max_cache_items,
            thumbnail_size,
            min_camera_zoom_scale,
            min_image_size,
            language,
        }
    }
}
