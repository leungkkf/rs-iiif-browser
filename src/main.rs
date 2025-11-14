use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::presentation::presentation_info::PresentationInfo;
use crate::rendering::tile::{TileCache, TileModState, TilePruneState};
use crate::rendering::tiled_image::TiledImage;
use bevy::asset::AssetMetaCheck;
use bevy::asset::io::web::WebAssetPlugin;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::winit::WinitSettings;
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};

mod app;
mod camera;
mod iiif;
mod input;
mod minimap;
mod presentation;
mod rendering;

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
        .add_systems(
            Startup,
            (
                setup,
                minimap::setup,
                presentation::ui::setup,
                setup_initial_presentation,
            )
                .chain(),
        )
        .add_systems(
            Update,
            ((
                (
                    input::keyboard::keyboard_input_system
                        .run_if(not(egui_wants_any_keyboard_input)),
                    input::mouse::mouse_input_system.run_if(not(egui_wants_any_pointer_input)),
                    minimap::mouse_input_system,
                    rendering::tile::asset_event_system,
                ),
                (rendering::tile::update_tiles_system.run_if(resource_changed::<TileModState>)),
            )
                .chain(),),
        )
        .add_systems(
            EguiPrimaryContextPass,
            presentation::ui::presentation_ui_system,
        )
        .add_systems(
            PostUpdate,
            (
                camera::main_camera::translation_bounding_system,
                minimap::update_view_rect_system,
            ),
        )
        .add_systems(
            Last,
            rendering::tile::prune_tiles_system.run_if(resource_changed::<TilePruneState>),
        )
        .add_observer(rendering::tiled_image::on_remove_image)
        .add_observer(minimap::on_remove_image)
        .add_observer(rendering::tiled_image::on_add_image)
        .add_observer(minimap::on_add_image)
        .run();
}

/// Set up the camera.
fn setup(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) -> Result {
    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    // Main camera
    commands.spawn((camera::main_camera::MainCamera, Camera2d));

    // Image.
    // commands.spawn(image);

    // Tile cache resource.
    commands.insert_resource(TileCache::new());

    // App state.
    commands.spawn(AppState::new(0));

    // Tile mod state.
    commands.insert_resource(TileModState::new());

    // App settings.
    commands.insert_resource(AppSettings::new(4096));

    // Tile mod state.
    commands.insert_resource(TilePruneState::new());

    // Egui camera.
    commands.spawn((
        // The `PrimaryEguiContext` component requires everything needed to render a primary context.
        PrimaryEguiContext,
        Camera2d,
        // Setting RenderLayers to none makes sure we won't render anything apart from the UI.
        RenderLayers::none(),
        Camera {
            order: 1,
            output_mode: bevy::camera::CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    ));

    Ok(())
}

fn setup_initial_presentation(mut commands: Commands) -> Result {
    // let image = TiledImage::build(
    //     "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee",
    // )?;

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

    let presentation =
        PresentationInfo::build("https://iiif.lib.harvard.edu/manifests/ids:11927378")?;
    // let presentation =
    //     PresentationInfo::build("https://iiif.harvardartmuseums.org/manifests/object/323250")?;

    let image = TiledImage::build(
        &presentation.sequences[0].canvases[0].images[0]
            .resource
            .service
            .id,
    )?;

    commands.spawn(presentation);

    commands.spawn(image);

    Ok(())
}
