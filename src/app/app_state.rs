use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct AppState {
    pub(crate) level: usize,
    pub(crate) presentation_url: String,
    pub(crate) canvas_index: usize,
}

impl AppState {
    pub(crate) fn new(level: usize, presentation_url: String, canvas_index: usize) -> Self {
        Self {
            level,
            presentation_url,
            canvas_index,
        }
    }
}
