use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct AppState {
    pub(crate) level: usize,
}

impl AppState {
    pub(crate) fn new(level: usize) -> Self {
        Self { level }
    }
}
