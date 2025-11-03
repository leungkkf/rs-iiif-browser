use bevy::prelude::*;

use crate::tiled_image::TiledImage;

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

/// Keep the image within the viewport.
pub(crate) fn handle_tranlation_bounding(
    camera: Single<(&Camera, &GlobalTransform, &mut Transform), With<Camera2d>>,
    tiled_image: Single<&TiledImage>,
) {
    let (camera, global_transform, mut transform) = camera.into_inner();

    let Some((world_viewport_min, world_viewport_max)) =
        get_world_viewport_rect(camera, global_transform)
    else {
        return;
    };

    let world_viewport_rect =
        Rect::from_corners(world_viewport_min.truncate(), world_viewport_max.truncate());

    let world_margin = camera
        .viewport_to_world(global_transform, Vec2::splat(8.0))
        .expect("should transform to world space")
        .origin
        - camera
            .viewport_to_world(global_transform, Vec2::ZERO)
            .expect("should transform to world space")
            .origin;

    let abs_world_margin = world_margin.abs().x;

    let world_image_rect = tiled_image.get_world_max_size_rect();

    // The camera (viewport) should see at least some of the image (given by the margin).
    // The center of the camera should be bounded by the world image rect and a margin.
    let world_bounding_rect = Rect::from_corners(
        world_image_rect.min + abs_world_margin - world_viewport_rect.half_size(),
        world_image_rect.max - abs_world_margin + world_viewport_rect.half_size(),
    );

    let bounded_translation = transform
        .translation
        .truncate()
        .max(world_bounding_rect.min)
        .min(world_bounding_rect.max);

    transform.translation = bounded_translation.extend(transform.translation.z);
}
