use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct AppSettings {
    pub(crate) max_cache_items: usize,
    pub(crate) thumbnail_size: f32,
}

impl AppSettings {
    pub(crate) fn new(max_cache_items: usize, thumbnail_size: f32) -> Self {
        Self {
            max_cache_items,
            thumbnail_size,
        }
    }
}
