use crate::{
    AppState, app_settings::AppSettings, camera_ext, main_camera::MainCamera,
    tiled_image::TiledImage,
};
use bevy::{asset::LoadState, prelude::*};
use std::{collections::HashMap, ops::RangeInclusive};

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

#[derive(Resource)]
pub(crate) struct TilePruneState(u32);

impl TilePruneState {
    pub(crate) fn new() -> Self {
        Self(0)
    }

    pub(crate) fn invalidate(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct TileIndex {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) z: u32,
}

impl TileIndex {
    pub(crate) fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub(crate) fn level(&self) -> usize {
        self.z as usize
    }
}

impl From<TileIndex> for Vec3 {
    fn from(value: TileIndex) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}

impl From<Vec3> for TileIndex {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
            z: value.z as u32,
        }
    }
}

/// A tile of the imge.
#[derive(Debug, Component)]
pub(crate) struct Tile {
    pub(crate) index: TileIndex,
    pub(crate) image_position: Rect,
    pub(crate) world_position: Rect,
    bevy_image: Option<Handle<bevy::image::Image>>,
}

impl Tile {
    pub(crate) fn new(index: TileIndex, image_position: Rect, world_position: Rect) -> Self {
        Self {
            index,
            image_position,
            world_position,
            bevy_image: None,
        }
    }
}

#[derive(Component)]
pub(crate) struct TileLoading;

#[derive(Debug, Clone)]
struct TileCacheItem {
    entity: Entity,
    last_visible_secs: f64,
}

#[derive(Resource)]
pub(crate) struct TileCache {
    cache: HashMap<TileIndex, TileCacheItem>,
}

impl TileCache {
    pub(crate) fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

fn get_required_tiles(
    camera: &Camera,
    global_transform: &GlobalTransform,
    level: usize,
    image: &TiledImage,
) -> Option<(Vec<Tile>, RangeInclusive<u32>, RangeInclusive<u32>)> {
    let (world_pos_min, world_pos_max) =
        camera_ext::get_world_viewport_rect(camera, global_transform)?;

    Some(image.get_required_tiles(level, world_pos_min, world_pos_max))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_tiles(
    mut commands: Commands,
    mut tile_cache: ResMut<TileCache>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    asset_server: Res<AssetServer>,
    tiles: Query<(Entity, &Tile, &mut MeshMaterial2d<ColorMaterial>), With<Tile>>,
    app_state: Single<&mut AppState>,
    image: Single<&TiledImage>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut tile_prune_state: ResMut<TilePruneState>,
) {
    let (camera, global_transform) = camera_query.into_inner();

    let Some((required_tiles, _, _)) =
        get_required_tiles(camera, global_transform, app_state.level, *image)
    else {
        return;
    };

    for mut tile in required_tiles {
        let entry = tile_cache.cache.get(&tile.index);

        if entry.is_none() {
            let url = image.get_image_tile_url_at(app_state.level, tile.image_position);

            debug!("Load {:?} for {:?}", url, tile.index);

            let handle = asset_server.load(url);
            let tile_index = tile.index;

            tile.bevy_image = Some(handle.clone());

            let id = commands.spawn((tile, TileLoading)).id();

            tile_cache.cache.insert(
                tile_index,
                TileCacheItem {
                    entity: id,
                    last_visible_secs: 0.0,
                },
            );
        }
    }

    for (entity, tile, material) in tiles.iter() {
        let color_material = materials
            .get_mut(material.id())
            .expect("tile should have a color material");

        if tile.index.level() != app_state.level {
            color_material.alpha_mode = bevy::sprite_render::AlphaMode2d::Blend;
            color_material.color = Color::srgba(1.0, 1.0, 1.0, 0.25);

            commands.entity(entity).insert(Transform::from_translation(
                tile.world_position
                    .center()
                    .extend(-100.0 + tile.index.z as f32),
            ));

            tile_prune_state.invalidate();
        } else {
            color_material.alpha_mode = bevy::sprite_render::AlphaMode2d::default();
            color_material.color = Color::default();
            tile_cache
                .cache
                .entry(tile.index)
                .and_modify(|t| t.last_visible_secs = time.elapsed_secs_f64());

            commands.entity(entity).insert((
                Visibility::Visible,
                Transform::from_translation(tile.world_position.center().extend(0.0)),
            ));
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
                tile_cache.cache.remove(&tile.index);
                tile_mod_state.invalidate();
            }
            None => {}
        }
    }
}

pub(crate) fn prune_tiles(
    mut commands: Commands,
    mut tile_cache: ResMut<TileCache>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    tiles: Query<&Tile>,
    app_settings: Res<AppSettings>,
    image: Single<&TiledImage>,
    app_state: Single<&mut AppState>,
) {
    let num_cache_items = tile_cache.cache.len();

    if num_cache_items <= app_settings.max_cache_items {
        return;
    }
    debug!("Pruning tiles at current level {}", app_state.level);

    let num_items_to_remove = num_cache_items - app_settings.max_cache_items;
    let (camera, global_transform) = camera_query.into_inner();
    // Only keep the tiles in view for this level and the lower-res levels.
    let all_required_tiles: Vec<_> = (0..=app_state.level)
        .map(|level| get_required_tiles(camera, global_transform, level, *image))
        .collect();
    let mut out_of_view_tiles = Vec::new();

    for tile in tiles {
        // Out of view if the tile has a higher res or outside the range.
        let is_out_of_view =
            all_required_tiles
                .get(tile.index.level())
                .is_none_or(|required_tiles| {
                    required_tiles
                        .as_ref()
                        .is_some_and(|(_, tile_range_x, tile_range_y)| {
                            !tile_range_x.contains(&tile.index.x)
                                || !tile_range_y.contains(&tile.index.y)
                        })
                });

        if is_out_of_view && let Some(tile_in_cache) = tile_cache.cache.get(&tile.index) {
            out_of_view_tiles.push((tile.index, tile_in_cache.clone()));
        }
    }

    out_of_view_tiles.sort_by(|(_, a), (_, b)| {
        if a.last_visible_secs < b.last_visible_secs {
            std::cmp::Ordering::Less
        } else if a.last_visible_secs > b.last_visible_secs {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    });

    for (tile_index, cache_item) in out_of_view_tiles.iter().take(num_items_to_remove) {
        debug!("Remove tile from cache {:?}", tile_index);
        tile_cache.cache.remove(tile_index);
        commands.entity(cache_item.entity).despawn();
    }
}
