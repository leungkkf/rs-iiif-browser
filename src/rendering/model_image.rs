use crate::camera::{main_camera::MainCamera3d, pan_orbit_state_3d::PanOrbitState3d};
use bevy::{
    asset::AssetId,
    prelude::{
        Add, AssetServer, Camera, Commands, Component, GltfAssetLabel, MessageWriter, On,
        Projection, Res, ResMut, Result, SceneRoot, Single, Transform, With, info,
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
    camera3d_query: Single<(&mut Camera, &mut Transform, &mut Projection), With<MainCamera3d>>,
    mut commands: Commands,
    mut current_state: ResMut<PanOrbitState3d>,
    asset_server: Res<AssetServer>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    info!("Model image added (model_image). {:?}", add.entity);

    // Set the 3D camera active.
    let (mut camera3d, _transform, _projection) = camera3d_query.into_inner();

    camera3d.is_active = true;
    // Reset the initial state for the new model.
    *current_state = PanOrbitState3d::default();

    // Load the 3D model.
    let asset_3d =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_image.url.to_string()));

    commands.spawn(ModelLoading(asset_3d.id()));

    commands.spawn(SceneRoot(asset_3d));

    redraw_request_writer.write(RequestRedraw);

    Ok(())
}
