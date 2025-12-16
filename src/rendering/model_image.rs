use crate::camera::{main_camera::MainCamera3d, pan_orbit_state_3d::PanOrbitState3d};
use bevy::{
    asset::AssetId,
    prelude::{
        Add, AssetServer, Camera, Commands, Component, Entity, EulerRot, GltfAssetLabel,
        MessageWriter, On, Quat, Query, Remove, Res, ResMut, Result, SceneRoot, Single, Transform,
        With, info,
    },
    scene::Scene,
    window::RequestRedraw,
};

#[derive(Component)]
pub(crate) struct ModelLoading(pub(crate) AssetId<Scene>);

#[derive(Component)]
pub(crate) struct ModelImage {
    url: String,
}

impl ModelImage {
    pub(crate) fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

/// Handler when adding the model image.
pub(crate) fn on_add_model_image(
    add: On<Add, ModelImage>,
    model_image: Single<&ModelImage>,
    camera3d_query: Single<(&mut Camera, &mut Transform), With<MainCamera3d>>,
    mut commands: Commands,
    mut current_state: ResMut<PanOrbitState3d>,
    asset_server: Res<AssetServer>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    info!("Model image added (model_image). {:?}", add.entity);

    // Set the 3D camera active.
    let (mut camera3d, mut transform) = camera3d_query.into_inner();

    camera3d.is_active = true;
    // Reset the initial state for the new model.
    *current_state = PanOrbitState3d::default();
    transform.rotation =
        Quat::from_euler(EulerRot::YXZ, current_state.yaw, current_state.pitch, 0.0);
    transform.translation = current_state.center + transform.back() * current_state.radius;

    // Load the 3D model.
    let asset_3d =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_image.url.to_string()));

    commands.spawn(ModelLoading(asset_3d.id()));

    commands.spawn(SceneRoot(asset_3d));

    redraw_request_writer.write(RequestRedraw);

    Ok(())
}

/// Handler when removing the model image.
pub(crate) fn on_remove_model_image(
    remove: On<Remove, ModelImage>,
    mut commands: Commands,
    scene: Single<Entity, With<SceneRoot>>,
    model_loading: Query<Entity, With<ModelLoading>>,
) -> Result {
    info!("Model image removed (model_image). {:?}", remove.entity);

    // Despawn the scene.
    commands.entity(*scene).despawn();
    // Despawn the loading.
    for loading in model_loading {
        commands.entity(loading).despawn();
    }

    Ok(())
}
