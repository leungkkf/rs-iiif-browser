use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::rendering::tile::{TileCache, TileModState, TilePruneState};
use bevy::asset::AssetMetaCheck;
use bevy::asset::io::web::WebAssetPlugin;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::render_resource::BlendState;
use bevy::winit::WinitSettings;
use bevy_egui::input::{egui_wants_any_keyboard_input, egui_wants_any_pointer_input};
use bevy_egui::{EguiGlobalSettings, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext};
use clap::Parser;

mod app;
mod asset_loading;
mod camera;
mod fonts;
mod iiif;
mod input;
mod minimap;
mod presentation;
mod rendering;
mod web;

/// User notification message.
#[derive(Message)]
struct UserNotification(String);

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL of the IIIF manfifest.
    #[arg(short, long)]
    manifest: Option<String>,
}

// the `bevy_main` proc_macro generates the required boilerplate for Android
#[bevy_main]
pub fn main() {
    let mut app = App::new();

    app.add_message::<UserNotification>()
        .add_plugins(
            DefaultPlugins
                // Meta data is not expected for IIIF.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                // Required for the pasting to egui (prevent_default_event_handling to false) in wasm.
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        // You may want this set to `true` if you need virtual keyboard work in mobile browsers.
                        prevent_default_event_handling: false,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
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
            (
                input::keyboard::keyboard_input_system.run_if(not(egui_wants_any_keyboard_input)),
                input::mouse::mouse_input_system::<
                    camera::main_camera::MainCamera2d,
                    camera::pan_zoom_state_2d::PanZoomState2d,
                >
                    .run_if(not(egui_wants_any_pointer_input))
                    .run_if(not(minimap::ui_has_mouse_input)),
                input::mouse::mouse_input_system::<
                    camera::main_camera::MainCamera3d,
                    camera::pan_orbit_state_3d::PanOrbitState3d,
                >
                    .run_if(not(egui_wants_any_pointer_input)),
                input::touch::touch_input_system::<
                    camera::main_camera::MainCamera2d,
                    camera::pan_zoom_state_2d::PanZoomState2d,
                >,
                input::touch::touch_input_system::<
                    camera::main_camera::MainCamera3d,
                    camera::pan_orbit_state_3d::PanOrbitState3d,
                >,
                minimap::mouse_input_system,
                web::load_presentation_system,
                web::load_canvas_system,
            ),
        )
        .add_systems(
            EguiPrimaryContextPass,
            presentation::ui::presentation_ui_system,
        )
        .add_systems(
            PostUpdate,
            (camera::main_camera::translation_bounding_system),
        )
        .add_systems(
            Last,
            (
                asset_loading::asset_event_system,
                minimap::update_view_rect_system,
                rendering::tile::prune_tiles_system.run_if(resource_changed::<TilePruneState>),
                rendering::pipeline_checker::pipeline_refresh_system
                    .run_if(resource_changed::<rendering::pipeline_checker::PipelinesModCount>),
                rendering::tile::update_tiles_system.run_if(resource_changed::<TileModState>),
            ),
        )
        .add_observer(presentation::manifest::on_remove_manifest)
        .add_observer(rendering::tile::on_remove_tiled_image)
        .add_observer(rendering::model_image::on_remove_model_image)
        .add_observer(minimap::on_remove_tiled_image)
        .add_observer(rendering::model_image::on_remove_model_loading)
        .add_observer(rendering::tiled_image::on_add_tiled_image)
        .add_observer(rendering::model_image::on_add_model_image)
        .add_observer(minimap::on_add_tiled_image);

    // In desktop mode, systems are not always run.
    // We subscribe to the ExtractSchedule to check the status of the pipeline.
    // and will refresh until all are ready.
    // https://github.com/rparrett/bevy_pipelines_ready
    app.sub_app_mut(RenderApp).add_systems(
        ExtractSchedule,
        rendering::pipeline_checker::check_pipelines_ready_system,
    );

    app.run();
}

/// Set up the camera.
fn setup(mut commands: Commands, mut egui_global_settings: ResMut<EguiGlobalSettings>) -> Result {
    // Disable the automatic creation of a primary context to set it up manually for the camera we need.
    egui_global_settings.auto_create_primary_context = false;

    // Main camera
    commands.spawn((
        camera::main_camera::MainCamera2d,
        Camera2d,
        Camera {
            is_active: false,
            ..default()
        },
    ));

    commands.spawn((
        camera::main_camera::MainCamera3d,
        Camera3d::default(),
        Camera {
            is_active: false,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    // Default ambient light.
    commands.insert_resource(AmbientLight {
        brightness: 2000.0,
        ..default()
    });

    // Tile cache resource.
    commands.insert_resource(TileCache::new());

    // App state.
    commands.insert_resource(AppState::default());

    // Pipeline mod count.
    commands.insert_resource(rendering::pipeline_checker::PipelinesModCount::default());

    // Tile mod state.
    commands.insert_resource(TileModState::new());

    // App settings.
    commands.insert_resource(AppSettings::default());

    // Tile mod state.
    commands.insert_resource(TilePruneState::new());

    // Camera 3D pan orbit state.
    commands.insert_resource(camera::pan_orbit_state_3d::PanOrbitState3d::default());

    // Camera 2D pan zoom state.
    commands.insert_resource(camera::pan_zoom_state_2d::PanZoomState2d::default());

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

fn setup_initial_presentation(mut app_state: ResMut<AppState>) -> Result {
    let args = Args::parse();

    // Try to read the manifest URL from the command line.
    if let Some(presentation_url) = args.manifest {
        web::load_presentation(&mut app_state, &presentation_url);
    } else {
        web::load_presentation(
            &mut app_state,
            "https://iiif.harvardartmuseums.org/manifests/object/21116",
        );
        // web::load_presentation(
        //     &mut app_state,
        //     "https://iiif.github.io/3d/manifests/1_basic_model_in_scene/model_origin.json",
        // );
    }

    Ok(())
}
