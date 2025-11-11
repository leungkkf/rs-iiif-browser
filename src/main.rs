use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::tile_rendering::tile::{TileCache, TileModState, TilePruneState};
use crate::tile_rendering::tiled_image::TiledImage;
use bevy::asset::AssetMetaCheck;
use bevy::asset::io::web::WebAssetPlugin;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitSettings;
use bevy_egui::egui::{Id, Popup};
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

mod app;
mod camera;
mod iiif;
mod input;
mod minimap;
mod tile_rendering;

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
        .add_plugins(EguiPlugin::default())
        // Desktop mode to reduce CPU usage.
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, (setup, minimap::setup).chain())
        .add_systems(
            Update,
            ((
                (
                    input::keyboard::keyboard_input_system
                        .run_if(not(egui_wants_any_keyboard_input)),
                    input::mouse::mouse_input_system.run_if(not(egui_wants_any_pointer_input)),
                    minimap::mouse_input_system,
                    tile_rendering::tile::asset_event_system,
                ),
                (tile_rendering::tile::update_tiles_system
                    .run_if(resource_changed::<TileModState>)),
            )
                .chain(),),
        )
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .add_systems(
            PostUpdate,
            (
                camera::main_camera::translation_bounding_system,
                minimap::update_view_rect_system,
            ),
        )
        .add_systems(
            Last,
            tile_rendering::tile::prune_tiles_system.run_if(resource_changed::<TilePruneState>),
        )
        .run();
}

/// Set up the camera.
fn setup(mut commands: Commands, window: Single<&mut Window>) {
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

    // let image = TiledImage::build(
    //     "https://api.nga.gov/iiif".into(),
    //     "99758d9d-c10b-4d02-a198-7e49afb1f3a6".into(),
    // )
    // .unwrap();

    // let image = TiledImage::build(
    //     "https://mps.lib.harvard.edu/assets/images".into(),
    //     "VPAL.HARVARDONLINE:cellxeukcell".into(),
    // )
    // .unwrap();

    // This is version 3.
    // let image = TiledImage::build(
    //     "https://research.ng-london.org.uk/iiif-int/pics/pyrByDate/2010/03/04".into(),
    //     "N-0728-00-000035-PYR.tif".into(),
    // )
    // .unwrap();

    let world_max_rect = image.get_world_max_size_rect();
    let zoom = Vec2::new(world_max_rect.width(), world_max_rect.height()) / window.size();
    let zoom_scale = zoom.max_element();
    let initial_level = image.get_level_at(zoom_scale);

    // Main camera
    commands.spawn((
        camera::main_camera::MainCamera,
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: zoom_scale,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(
            world_max_rect.width() / 2.0,
            -world_max_rect.height() / 2.0,
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
    commands.insert_resource(AppSettings::new(4096));

    // Tile mod state.
    commands.insert_resource(TilePruneState::new());
}

fn ui_example_system(
    mut contexts: EguiContexts,
    _window: Single<&mut Window, With<PrimaryWindow>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // egui::Window::new("Hello").show(ctx, |ui| {
    //     ui.label("world");
    // });

    egui::TopBottomPanel::top("top_panel")
        .frame(egui::Frame::NONE)
        .show_separator_line(false)
        .show(ctx, |ui| {
            let response = ui.label("Top panel");

            Popup::menu(&response).id(Id::new("menu")).show(|ui| {
                ui.set_max_width(200.0); // To make sure we
                ui.label("Popup text");
            });

            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        });

    Ok(())
}
