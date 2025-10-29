use crate::{AppState, image::Image};
use bevy::prelude::*;

pub(crate) const TILE_SIZE: f32 = 256.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TileIndex {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl TileIndex {
    pub(crate) fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Into<Vec3> for TileIndex {
    fn into(self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, 0.0)
    }
}

impl From<Vec3> for TileIndex {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
        }
    }
}

/// A tile of the imge.
#[derive(Component)]
pub(crate) struct Tile {
    pub(crate) index: TileIndex,
    pub(crate) level: usize,
    pub(crate) image_position: Rect,
    pub(crate) world_position: Rect,
    bevy_image: Option<Handle<bevy::image::Image>>,
}

impl Tile {
    pub(crate) fn new(
        index: TileIndex,
        level: usize,
        image_position: Rect,
        world_position: Rect,
    ) -> Self {
        Self {
            index,
            level,
            image_position,
            world_position,
            bevy_image: None,
        }
    }
}

pub(crate) fn update_tiles(
    mut commands: Commands,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    tiles: Query<&Tile>,
    app_state: Single<&mut AppState>,
    image: Single<&Image>,
) {
    let (camera, global_transform) = camera_query.into_inner();
    let viewport = camera.logical_viewport_rect().unwrap();

    let world_pos_min = camera
        .viewport_to_world(global_transform, viewport.min)
        .unwrap();
    let world_pos_max = camera
        .viewport_to_world(global_transform, viewport.max)
        .unwrap();

    let required_tiles =
        image.get_required_tiles(app_state.level, world_pos_min.origin, world_pos_max.origin);

    for tile in required_tiles {
        if tiles
            .iter()
            .find(|t| t.index == tile.index && t.level == app_state.level)
            .is_none()
        {
            let url = image.get_image_tile_at(app_state.level, tile.image_position);

            commands.spawn((
                Transform::from_translation(tile.world_position.center().extend(0.0)),
                Mesh2d(meshes.add(Rectangle::new(
                    tile.world_position.width(),
                    tile.world_position.height(),
                ))),
                MeshMaterial2d(materials.add(ColorMaterial {
                    texture: Some(asset_server.load(url)),
                    ..default()
                })),
                tile,
            ));
        }
    }
}
