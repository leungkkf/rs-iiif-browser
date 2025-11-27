use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct AppState {
    pub(crate) level: usize,
    pub(crate) presentation_url: String,
}

impl AppState {
    pub(crate) fn new(level: usize, presentation_url: String) -> Self {
        Self {
            level,
            presentation_url,
        }
    }
}
