use bevy::prelude::{
    Add, AssetServer, Camera, Commands, Component, GltfAssetLabel, On, Projection, Res, Result,
    SceneRoot, Single, Transform, With, info,
};

use crate::camera::main_camera::MainCamera3d;

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
    asset_server: Res<AssetServer>,
) -> Result {
    info!("Model image added (model_image). {:?}", add.entity);

    // Set the 3D camera active.
    let (mut camera3d, _transform, _projection) = camera3d_query.into_inner();

    camera3d.is_active = true;

    // Load the 3D model.
    let asset_3d =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_image.url.to_string()));

    commands.spawn(SceneRoot(asset_3d));

    Ok(())
}
