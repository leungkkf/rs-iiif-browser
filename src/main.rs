use crate::app_settings::AppSettings;
use crate::app_state::AppState;
use crate::tile::{TileCache, TileModState, TilePruneState};
use crate::tiled_image::TiledImage;
use bevy::asset::AssetMetaCheck;
use bevy::asset::io::web::WebAssetPlugin;
use bevy::prelude::*;

mod app_settings;
mod app_state;
mod camera_ext;
mod iiif;
mod keyboard_input;
mod main_camera;
mod minimap;
mod mouse_input;
mod tile;
mod tiled_image;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                // Meta data is not expected for IIIF.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                // Understood the warning.
                .set(WebAssetPlugin {
                    silence_startup_warning: true,
                })
                // Use nearest for the tiling for now. Will probably need to use virtual texture for the linear interpolation.
                .set(ImagePlugin::default_nearest()),
        )
        .add_systems(Startup, (setup, minimap::setup).chain())
        .add_systems(
            Update,
            ((
                (
                    keyboard_input::handle_keyboard_input,
                    mouse_input::handle_mouse_input,
                    minimap::on_mouse_click,
                    tile::on_asset_event,
                ),
                (tile::update_tiles.run_if(resource_changed::<TileModState>)),
            )
                .chain(),),
        )
        .add_systems(
            PostUpdate,
            (
                main_camera::handle_translation_bounding,
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

    let image = TiledImage::build(
        "https://nationalmuseumse.iiifhosting.com/iiif".into(),
        "6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee".into(),
    )
    .unwrap();
    // let image = TiledImage::build(
    //     "https://iiif.wellcomecollection.org/thumbs".into(),
    //     "b20432033_B0008608.JP2".into(),
    // )
    // .unwrap();

    let sizes = image.get_image_sizes();

    let zoom = sizes.last().unwrap().width as f32 / sizes[initial_level].width as f32;

    // Main camera
    commands.spawn((
        main_camera::MainCamera,
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: zoom,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(
            sizes[initial_level].width as f32 / 2.0 * zoom,
            -(sizes[initial_level].height as f32) / 2.0 * zoom,
            0.0,
        ),
    ));

    // Image.
    commands.spawn(image);

    // Tile cache resource.
    commands.insert_resource(TileCache::new());

    // App state.
    commands.spawn(AppState::new(initial_level));

    // Tile mod state.
    commands.insert_resource(TileModState::new());

    // App settings.
    commands.insert_resource(AppSettings::new(512));

    // Tile mod state.
    commands.insert_resource(TilePruneState::new());
}
