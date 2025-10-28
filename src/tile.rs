use crate::{AppState, image::Image};
use bevy::prelude::*;

pub(crate) const TILE_SIZE: f32 = 256.0;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TileIndex {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

impl TileIndex {
    fn new(x: u32, y: u32) -> Self {
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
    bevy_image: Option<Handle<bevy::image::Image>>,
}

#[derive(Bundle)]
pub(crate) struct TileBundle {
    tile: Tile,
    transform: Transform,
}

impl TileBundle {
    pub(crate) fn build(index: TileIndex, level: usize, transform: Transform) -> Self {
        Self {
            tile: Tile {
                index,
                level,
                bevy_image: None,
            },
            transform,
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

    let (clamped_tile_min, clamped_tile_max) =
        image.get_required_tiles(app_state.level, world_pos_min.origin, world_pos_max.origin);

    for y in clamped_tile_min.y as u32..=clamped_tile_max.y {
        for x in clamped_tile_min.x as u32..=clamped_tile_max.x as u32 {
            if tiles
                .iter()
                .find(|t| t.index.x == x && t.index.y == y && t.level == app_state.level)
                .is_none()
            {
                let (url, mesh_pos, mesh_size) =
                    image.get_image_tile_display_info(app_state.level, TileIndex::new(x, y));

                commands.spawn((
                    TileBundle::build(
                        TileIndex::new(x, y),
                        app_state.level,
                        Transform::from_translation(mesh_pos),
                    ),
                    Mesh2d(meshes.add(Rectangle::new(mesh_size.x, mesh_size.y))),
                    MeshMaterial2d(materials.add(ColorMaterial {
                        texture: Some(asset_server.load(url)),
                        ..default()
                    })),
                ));
            }
        }
    }
}
