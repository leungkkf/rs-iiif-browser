use crate::rendering::{
    model_image::ModelLoading,
    tile::{Tile, TileCache, TileLoading, TileModState},
};
use bevy::{
    asset::LoadState,
    prelude::{
        AssetServer, Assets, ColorMaterial, Commands, Entity, Local, Mesh, Mesh2d, MeshMaterial2d,
        MessageWriter, Query, Rectangle, Res, ResMut, Time, Transform, Visibility, With, default,
        warn,
    },
    window::RequestRedraw,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn asset_event_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles: Query<(Entity, &Tile), With<TileLoading>>,
    models: Query<(Entity, &ModelLoading)>,
    mut tile_cache: ResMut<TileCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
    mut refresh_time: Local<Option<f32>>,
    time: Res<Time>,
) {
    // Keep polling if tiles or models are being loaded.
    if !tiles.is_empty() || !models.is_empty() {
        redraw_request_writer.write(RequestRedraw);
    }

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
                tile_cache.remove(&tile.index);
                tile_mod_state.invalidate();
            }
            None => {}
        }
    }

    for (entity, ModelLoading(id)) in models {
        match asset_server.get_load_state(*id) {
            Some(LoadState::NotLoaded) => {}
            Some(LoadState::Loading) => {}
            Some(LoadState::Loaded) => {
                // Hack to keep the system going for a while the first time
                // to get the rendering system ready to show the model.
                if refresh_time.is_some_and(|x| x < time.delta_secs()) {
                    commands.entity(entity).despawn();
                } else {
                    *refresh_time = Some(time.delta_secs() + 3.0);
                }
            }
            Some(LoadState::Failed(_)) => {
                warn!("failed to load model ID {:?}.", id);
            }
            None => {}
        }
    }
}
