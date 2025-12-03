use bevy::prelude::Resource;

#[derive(Resource)]
pub(crate) struct AppState {
    /// Current image scale level.
    pub(crate) level: usize,
    /// Current presentation manifest URL.
    pub(crate) presentation_url: String,
    /// Current canvas index.
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
