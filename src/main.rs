use crate::app_state::AppState;
use crate::tile::{TileCache, TileModState};
use crate::tiled_image::{ImageBundle, Size};
use bevy::prelude::*;

mod app_state;
mod keyboard_input;
mod mouse_input;
mod tile;
mod tiled_image;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            ((
                (
                    keyboard_input::handle_keyboard_input,
                    mouse_input::handle_mouse_input,
                    tile::on_asset_event,
                ),
                (tile::update_tiles.run_if(resource_changed::<TileModState>)),
            )
                .chain(),),
        )
        .run();
}

/// Set up the camera.
fn setup(mut commands: Commands, _windows: Single<&mut Window>) {
    let initial_level = 0;

    let levels = vec![
        Size::new(678, 478),
        Size::new(1357, 955),
        Size::new(2713, 1910),
        Size::new(5426, 3820),
    ];

    let zoom = 5426.0 / 678.0;

    // Camera
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: zoom,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(
            levels[initial_level].width as f32 / 2.0 * zoom,
            -(levels[initial_level].height as f32) / 2.0 * zoom,
            0.0,
        ),
    ));

    // Image.
    commands.spawn(ImageBundle::build(
        "https://stacks.stanford.edu/image/iiif".into(),
        "hg676jb4964%2F0380_796-44".into(),
        levels,
    ));

    // Tile cache resource.
    commands.insert_resource(TileCache::new());

    // App state.
    commands.spawn(AppState::new(initial_level));

    commands.insert_resource(TileModState::new());
}
