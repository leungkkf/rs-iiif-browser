use crate::camera::{main_camera::MainCamera3d, pan_orbit_state_3d::PanOrbitState3d};
use bevy::{
    asset::AssetId,
    camera::primitives::{Aabb, Sphere},
    prelude::{
        Add, AssetServer, Camera, Commands, Component, Entity, EulerRot, GlobalTransform,
        GltfAssetLabel, Mesh3d, MessageWriter, On, Quat, Query, Remove, Res, ResMut, Result,
        SceneRoot, Single, Transform, Vec3, Vec3A, With, info, warn,
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    info!("Model image added (model_image). {:?}", add.entity);

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

/// Handler when the model loading is done.
/// Enable the camera and set up a default transform for the model.
pub(crate) fn on_remove_model_loading(
    remove: On<Remove, ModelLoading>,
    meshes: Query<(&GlobalTransform, Option<&Aabb>), With<Mesh3d>>,
    camera3d_query: Single<(&mut Camera, &mut Transform), With<MainCamera3d>>,
    mut current_state: ResMut<PanOrbitState3d>,
) {
    info!("Model loading removed (model_image). {:?}", remove.entity);

    let pan_orbit_state =
        if !meshes.is_empty() && !meshes.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
            // Find an approximate bounding box of the scene from its meshes
            // https://bevy.org/examples/tools/scene-viewer/
            let mut min = Vec3A::splat(f32::MAX);
            let mut max = Vec3A::splat(f32::MIN);
            for (transform, maybe_aabb) in &meshes {
                let aabb = maybe_aabb.unwrap();
                // If the Aabb had not been rotated, applying the non-uniform scale would produce the
                // correct bounds. However, it could very well be rotated and so we first convert to
                // a Sphere, and then back to an Aabb to find the conservative min and max points.
                let sphere = Sphere {
                    center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
                    radius: transform.radius_vec3a(aabb.half_extents),
                };
                let aabb = Aabb::from(sphere);
                min = min.min(aabb.min());
                max = max.max(aabb.max());
            }

            let size = (max - min).length();
            let aabb = Aabb::from_min_max(Vec3::from(min), Vec3::from(max));

            // Size cannot be 0 in PanOrbitState3d.
            if size != 0.0 {
                info!(
                    "found meshes in model. init PanOrbitState3d with size {} center {}",
                    size, aabb.center
                );
                PanOrbitState3d::new(Vec3::from(aabb.center), size, 0.0, 0.0, true)
            } else {
                warn!("size is 0 in the meshes. use default PanOrbitState3d");
                PanOrbitState3d::default()
            }
        } else {
            warn!("meshes not found. use default PanOrbitState3d");
            PanOrbitState3d::default()
        };

    // Set the 3D camera active.
    let (mut camera3d, mut transform) = camera3d_query.into_inner();

    camera3d.is_active = true;

    *current_state = pan_orbit_state;

    transform.rotation =
        Quat::from_euler(EulerRot::YXZ, current_state.yaw, current_state.pitch, 0.0);
    transform.translation = current_state.center + transform.back() * current_state.radius;
}
