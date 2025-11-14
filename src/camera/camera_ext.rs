use bevy::prelude::{Camera, GlobalTransform, Vec3};

/// Get the viewport rect in world space.
pub(crate) fn get_world_viewport_rect(
    camera: &Camera,
    global_transform: &GlobalTransform,
) -> Option<(Vec3, Vec3)> {
    let viewport = camera.logical_viewport_rect()?;

    let world_pos_min = camera
        .viewport_to_world(global_transform, viewport.min)
        .ok()?;
    let world_pos_max = camera
        .viewport_to_world(global_transform, viewport.max)
        .ok()?;

    Some((world_pos_min.origin, world_pos_max.origin))
}
