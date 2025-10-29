use crate::app_state::AppState;
use crate::image::{ImageBundle, Size};
use bevy::prelude::*;

mod app_state;
mod image;
mod keyboard_input;
mod mouse_input;
mod tile;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                (
                    keyboard_input::handle_keyboard_input,
                    mouse_input::handle_mouse_input,
                ),
                tile::update_tiles,
            )
                .chain(),
        )
        .run();
}

/// Set up the camera.
fn setup(mut commands: Commands, _windows: Single<&mut Window>) {
    let initial_level = 0;

    let levels = vec![
        Size::new(640, 384),
        Size::new(1280, 768),
        Size::new(2560, 1536),
        Size::new(5120, 3072),
    ];

    // Camera
    commands.spawn((
        Camera2d::default(),
        Transform::from_xyz(
            levels[initial_level].width as f32 / 2.0,
            -(levels[initial_level].height as f32) / 2.0,
            0.0,
        ),
    ));

    // Image.
    commands.spawn(ImageBundle::build(
        "https://stacks.stanford.edu/image/iiif".into(),
        "hg676jb4964%2F0380_796-44".into(),
        levels,
    ));

    // App state.
    commands.spawn(AppState::new(initial_level));
}
