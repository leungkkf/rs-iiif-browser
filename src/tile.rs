use crate::{AppState, image::Image};
use bevy::{asset::LoadState, platform::collections::HashMap, prelude::*};

pub(crate) const TILE_SIZE: f32 = 1024.0;

#[derive(Resource)]
pub(crate) struct TileModState(u32);

impl TileModState {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn invalidate(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

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

    fn key(&self) -> String {
        format!("{}-{}-{}", self.index.x, self.index.y, self.level)
    }
}

#[derive(Component)]
pub(crate) struct TileLoading;

pub(crate) struct TileCacheItem {
    entity: Entity,
    bevy_image: Option<Handle<bevy::image::Image>>,
}

#[derive(Resource)]
pub(crate) struct TileCache {
    cache: HashMap<String, TileCacheItem>,
}

impl TileCache {
    pub(crate) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

pub(crate) fn update_tiles(
    mut commands: Commands,
    mut tile_cache: ResMut<TileCache>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    asset_server: Res<AssetServer>,
    tiles: Query<(Entity, &Tile, Option<&TileLoading>), With<Tile>>,
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

    for mut tile in required_tiles {
        let tile_key = tile.key();
        let entry = tile_cache.cache.get(&tile_key);

        if entry.is_none() {
            let url = image.get_image_tile_at(app_state.level, tile.image_position);

            let handle = asset_server.load(url);
            tile.bevy_image = Some(handle.clone());

            let id = commands.spawn((tile, TileLoading)).id();

            tile_cache.cache.insert(
                tile_key,
                TileCacheItem {
                    entity: id,
                    bevy_image: Some(handle),
                },
            );
        }
    }

    for (entity, tile, _) in tiles.iter() {
        if tile.level != app_state.level {
            commands.entity(entity).insert(Visibility::Hidden);
        } else {
            commands.entity(entity).insert(Visibility::Visible);
        }
    }
}

pub(crate) fn on_asset_event(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles: Query<(Entity, &Tile), With<TileLoading>>,
    mut tile_cache: ResMut<TileCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tile_mod_state: ResMut<TileModState>,
) {
    for (entity, tile) in tiles.iter() {
        match asset_server
            .get_load_state(tile.bevy_image.as_ref().expect("tile should have an image"))
        {
            Some(LoadState::NotLoaded) => {}
            Some(LoadState::Loading) => {}
            Some(LoadState::Loaded) => {
                commands.entity(entity).remove::<TileLoading>();
                commands.entity(entity).insert((
                    Transform::from_translation(tile.world_position.center().extend(0.0)),
                    Mesh2d(meshes.add(Rectangle::new(
                        tile.world_position.width(),
                        tile.world_position.height(),
                    ))),
                    MeshMaterial2d(materials.add(ColorMaterial {
                        texture: tile.bevy_image.clone(),
                        ..default()
                    })),
                    Visibility::Hidden,
                ));
                tile_mod_state.invalidate();
            }
            Some(LoadState::Failed(_)) => {
                warn!("failed to load tile at {:?}. retry...", tile.index);
                commands.entity(entity).despawn();
                tile_cache.cache.remove(&tile.key());
            }
            None => {}
        }
    }
}
