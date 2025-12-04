use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::presentation::manifest::Manifest;
use crate::presentation::ui::EguiUiState;
use crate::rendering::tile::{TileCache, TileModState, TilePruneState};
use crate::rendering::tiled_image::TiledImage;
use bevy::asset::AssetMetaCheck;
use bevy::asset::io::web::WebAssetPlugin;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::BlendState;
use bevy::window::RequestRedraw;
use bevy::winit::WinitSettings;
use bevy_egui::egui::TextBuffer;
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use clap::Parser;

mod app;
mod camera;
mod fonts;
mod iiif;
mod input;
mod minimap;
mod presentation;
mod rendering;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the IIIF manfifest.
    #[arg(short, long)]
    manifest: Option<String>,
}

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
        .add_observer(rendering::tile::on_remove_image)
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
    commands.insert_resource(AppState::new(0, "".to_string(), 0));

    // Tile mod state.
    commands.insert_resource(TileModState::new());

    // App settings.
    commands.insert_resource(AppSettings::new(
        4096,
        64.0,
        1.0 / 4.0,
        256.0,
        crate::iiif::manifest::language::EN.to_string(),
    ));

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

fn setup_initial_presentation(
    mut commands: Commands,
    mut app_state: ResMut<AppState>,
    mut egui_ui_state: ResMut<EguiUiState>,
    presentation_query: Query<(Entity, &Manifest)>,
    tiled_image_query: Query<(Entity, &TiledImage)>,
) -> Result {
    let args = Args::parse();

    // Try to read the manifest URL from the command line.
    if let Some(presentation_url) = args.manifest {
        load_presentation(
            &mut commands,
            &mut app_state,
            &mut egui_ui_state,
            &presentation_url,
            &presentation_query,
            &tiled_image_query,
        )?;
    }

    Ok(())
}

fn load_presentation(
    commands: &mut Commands,
    app_state: &mut ResMut<AppState>,
    egui_ui_state: &mut ResMut<EguiUiState>,
    presentation_url: &str,
    presentation_query: &Query<(Entity, &Manifest)>,
    tiled_image_query: &Query<(Entity, &TiledImage)>,
) -> Result {
    let presentation = presentation::manifest::Manifest::try_from_url(presentation_url)?;

    let image = TiledImage::try_from_url(
        presentation
            .model()
            .get_sequence(0)?
            .get_canvas(0)?
            .get_image(0)?
            .get_service()
            .as_str(),
    )?;

    for (presentation_entity, _) in presentation_query {
        commands.entity(presentation_entity).despawn();
    }

    for (image_entity, _) in tiled_image_query {
        commands.entity(image_entity).despawn();
    }

    app_state.presentation_url = presentation_url.to_string();
    app_state.canvas_index = 0;
    egui_ui_state.presentation_url = app_state.presentation_url.to_string();
    egui_ui_state.canvas_index = (app_state.canvas_index.saturating_add(1)).to_string();

    commands.spawn(presentation);

    commands.spawn(image);

    egui_ui_state.open_left_panel = true;

    Ok(())
}

fn load_canvas(
    presentation: &Manifest,
    commands: &mut Commands,
    tiled_image_query: &Query<(Entity, &TiledImage)>,
    app_state: &mut ResMut<AppState>,
    egui_ui_state: &mut ResMut<EguiUiState>,
    canvas_index: usize,
    redraw_request_writer: &mut MessageWriter<'_, RequestRedraw>,
) -> Result {
    let canvas = presentation
        .model()
        .get_sequence(0)?
        .get_canvas(canvas_index)?;

    let image_url = canvas.get_image(0)?.get_service().to_string();

    let image = TiledImage::try_from_url(&image_url)?;

    for (image_entity, _) in tiled_image_query {
        commands.entity(image_entity).despawn();
    }
    commands.spawn(image);

    app_state.canvas_index = canvas_index;
    egui_ui_state.canvas_index = (app_state.canvas_index.saturating_add(1)).to_string();

    redraw_request_writer.write(RequestRedraw);

    Ok(())
}
