use crate::{
    AppState,
    app::app_settings::AppSettings,
    camera::{camera_ext, main_camera::MainCamera2d},
    rendering::tiled_image::TiledImage,
};
use bevy::{
    asset::LoadState,
    prelude::{
        AssetServer, Assets, Camera, Color, ColorMaterial, Commands, Component, Entity,
        GlobalTransform, Handle, MeshMaterial2d, MessageWriter, On, Query, Rect, Remove, Res,
        ResMut, Resource, Result, Single, Time, Transform, Vec2, Vec3, Visibility, With, debug,
        info,
    },
    window::RequestRedraw,
};
use std::{collections::HashMap, ops::RangeInclusive};

#[derive(Resource)]
/// Invalidate this to trigger the tile update.
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
/// Invalidate this to trigger the tile pruning.
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

impl From<TileIndex> for Vec2 {
    fn from(value: TileIndex) -> Self {
        Self::new(value.x as f32, value.y as f32)
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

/// A tile of the image.
#[derive(Debug, Component)]
pub(crate) struct Tile {
    pub(crate) index: TileIndex,
    pub(crate) image_position: Rect,
    pub(crate) world_position: Rect,
    pub(crate) bevy_image: Option<Handle<bevy::image::Image>>,
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

    pub(crate) fn clear(&mut self) {
        self.cache.clear();
    }

    pub(crate) fn remove(&mut self, index: &TileIndex) {
        self.cache.remove(index);
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
pub(crate) fn update_tiles_system(
    mut commands: Commands,
    mut tile_cache: ResMut<TileCache>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    asset_server: Res<AssetServer>,
    tiles: Query<(Entity, &Tile, &mut MeshMaterial2d<ColorMaterial>), With<Tile>>,
    app_state: Res<AppState>,
    image: Single<&TiledImage>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut tile_prune_state: ResMut<TilePruneState>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) {
    let (camera, global_transform) = camera_query.into_inner();

    let Some((required_tiles, _, _)) =
        get_required_tiles(camera, global_transform, app_state.level, *image)
    else {
        // This is mainly for when the system is first up, some values seem to be not there yet.
        tile_mod_state.invalidate();
        redraw_request_writer.write(RequestRedraw);
        return;
    };

    for mut tile in required_tiles {
        let entry = tile_cache.cache.get(&tile.index);

        if entry.is_none() {
            let url = image.get_image_tile_url_at(tile.image_position);

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
            color_material.color = Color::srgba(1.0, 1.0, 1.0, 0.75);

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
    // Redraw the screen.
    redraw_request_writer.write(RequestRedraw);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prune_tiles_system(
    mut commands: Commands,
    mut tile_cache: ResMut<TileCache>,
    camera_query: Single<(&Camera, &GlobalTransform), With<MainCamera2d>>,
    tiles: Query<&Tile>,
    app_settings: Res<AppSettings>,
    image: Single<&TiledImage>,
    app_state: Res<AppState>,
    asset_server: Res<AssetServer>,
) {
    let num_cache_items = tile_cache.cache.len();

    if num_cache_items <= app_settings.max_cache_items {
        return;
    }
    debug!("Pruning tiles at current level {}", app_state.level);

    let mut num_items_to_remove = num_cache_items - app_settings.max_cache_items;
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
            match asset_server
                .get_load_state(tile.bevy_image.as_ref().expect("tile should have an image"))
            {
                Some(LoadState::Loaded) => {
                    out_of_view_tiles.push((tile.index, tile_in_cache.clone()));
                }
                _ => {
                    debug!(
                        "Remove unloaded out-of-view tile from cache {:?}",
                        tile.index
                    );
                    commands.entity(tile_in_cache.entity).despawn();
                    tile_cache.cache.remove(&tile.index);
                    num_items_to_remove = num_items_to_remove.saturating_sub(1);
                }
            }
        }
    }

    if num_items_to_remove > 0 {
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
            debug!("Remove loaded out-of-view tile from cache {:?}", tile_index);
            tile_cache.cache.remove(tile_index);
            commands.entity(cache_item.entity).despawn();
        }
    }
}

/// Triggered when the tiled image is removed to clean up and despawn related entities.
pub(crate) fn on_remove_tiled_image(
    remove: On<Remove, TiledImage>,
    mut commands: Commands,
    tiles: Query<(Entity, &Tile), With<Tile>>,
    mut tile_cache: ResMut<TileCache>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    info!("Tiled image removed (tile). {:?}", remove.entity);

    // Remove tile cache and despawn the tile entities.
    tile_cache.clear();
    for (tile_entity, _tile) in tiles {
        commands.entity(tile_entity).despawn();
    }

    // Trigger an update.
    tile_mod_state.invalidate();
    redraw_request_writer.write(RequestRedraw);

    Ok(())
}
