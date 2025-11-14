use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct AppSettings {
    pub(crate) max_cache_items: usize,
}

impl AppSettings {
    pub(crate) fn new(max_cache_items: usize) -> Self {
        Self { max_cache_items }
    }
}
