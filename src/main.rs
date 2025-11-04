use crate::app_settings::AppSettings;
use crate::app_state::AppState;
use crate::tile::{TileCache, TileModState, TilePruneState};
use crate::tiled_image::{Size, TiledImage};
use bevy::prelude::*;

mod app_settings;
mod app_state;
mod camera_ext;
mod keyboard_input;
mod main_camera;
mod minimap;
mod mouse_input;
mod tile;
mod tiled_image;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, (setup, minimap::setup).chain())
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
        .add_systems(
            PostUpdate,
            (
                main_camera::handle_tranlation_bounding,
                minimap::update_view_rect,
            ),
        )
        .add_systems(
            Last,
            tile::prune_tiles.run_if(resource_changed::<TilePruneState>),
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

    // Main camera
    commands.spawn((
        main_camera::MainCamera,
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

    let image = TiledImage::new(
        "https://stacks.stanford.edu/image/iiif".into(),
        "hg676jb4964%2F0380_796-44".into(),
        levels,
    );

    // Image.
    commands.spawn(image);

    // Tile cache resource.
    commands.insert_resource(TileCache::new());

    // App state.
    commands.spawn(AppState::new(initial_level));

    // Tile mod state.
    commands.insert_resource(TileModState::new());

    // App settings.
    commands.insert_resource(AppSettings::new(8));

    // Tile mod state.
    commands.insert_resource(TilePruneState::new());
}
