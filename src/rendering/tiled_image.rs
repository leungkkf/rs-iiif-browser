use crate::{
    app::app_state::AppState,
    camera::main_camera::MainCamera2d,
    iiif::{
        IiifError,
        image::{IiifFeature, IiifImageFormat, IiifImageInfo},
    },
    rendering::tile::{Tile, TileIndex, TileModState},
};
use bevy::{
    prelude::{
        Add, Camera, Component, MessageWriter, On, Projection, Rect, ResMut, Result, Single,
        Transform, Vec2, Vec3, With, info,
    },
    window::{RequestRedraw, Window},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::RangeInclusive};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct Size {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl From<Size> for Vec2 {
    fn from(value: Size) -> Self {
        Self::new(value.width as f32, value.height as f32)
    }
}

impl Size {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn on_add_tiled_image(
    add: On<Add, TiledImage>,
    tiled_image: Single<&TiledImage>,
    window: Single<&mut Window>,
    camera2d_query: Single<(&mut Camera, &mut Transform, &mut Projection), With<MainCamera2d>>,
    mut app_state: ResMut<AppState>,
    mut tile_mod_state: ResMut<TileModState>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    info!("Tiled image added (tiled_image). {:?}", add.entity);

    let (mut camera, mut transform, mut projection) = camera2d_query.into_inner();
    let Projection::Orthographic(orthogonal) = projection.as_mut() else {
        return Ok(());
    };

    camera.is_active = true;

    let world_max_rect = tiled_image.get_world_max_size_rect();

    // Fit the image to the viewport, or falling back to the window size.
    let zoom = Vec2::new(world_max_rect.width(), world_max_rect.height())
        / camera
            .logical_viewport_size()
            .unwrap_or_else(|| window.size());
    let zoom_scale = zoom.max_element();
    let initial_level = tiled_image.get_level_at(zoom_scale);

    app_state.level = initial_level;
    app_state.world_image_max_size = tiled_image.get_world_max_size_rect().size();
    orthogonal.scale = zoom_scale;

    transform.translation = Vec3::new(
        world_max_rect.width() / 2.0,
        -world_max_rect.height() / 2.0,
        0.0,
    );

    tile_mod_state.invalidate();
    redraw_request_writer.write(RequestRedraw);

    Ok(())
}

/// Image.
#[derive(Component)]
pub(crate) struct TiledImage {
    /// IFFF URL, e.g. "https://stacks.stanford.edu/image/iiif/hg676jb4964%2F0380_796-44"
    iiif_endpoint: String,
    /// The number of levels and sizes.
    levels: Vec<Size>,
    /// Tile size.
    tile_size: Size,
    /// Image format.
    image_format: IiifImageFormat,
    /// Supported features.
    supported_features: HashSet<IiifFeature>,
    /// Optional sizes when getting the full image.
    optional_sizes: Vec<Size>,
}

impl TiledImage {
    /// Create a new image.
    fn new(
        iiif_endpoint: String,
        tile_size: Size,
        levels: Vec<Size>,
        image_format: IiifImageFormat,
        supported_features: HashSet<IiifFeature>,
        optional_sizes: Vec<Size>,
    ) -> Self {
        Self {
            iiif_endpoint,
            tile_size,
            levels,
            image_format,
            supported_features,
            optional_sizes,
        }
    }

    /// Create the image from the IFFF image info JSON.
    pub(crate) fn try_from_json(
        json: &str,
        iiif_endpoint: &str,
    ) -> core::result::Result<Self, IiifError> {
        let iiif_image_info = IiifImageInfo::try_from_json(json)?;

        // Get tile size and levels.
        // We require both region by px and size by width/height for the tiling.
        // If not, we will only get the full image.
        let supported_features: HashSet<_> = iiif_image_info
            .get_profile_details()
            .flat_map(|x| (*x).get_supported_features())
            .collect();
        let tile_size: Size;
        let levels: Vec<Size>;

        if supported_features.contains(&IiifFeature::RegionByPx)
            && supported_features.contains(&IiifFeature::SizeByWh)
        {
            info!("RegionByPx and SizeByWh supported. Use tiling.");
            tile_size = iiif_image_info.get_tile_size();
            levels = iiif_image_info.get_tile_scaling_sizes();
        } else {
            info!("RegionByPx or SizeByWh not supported. Get the full image.");
            tile_size = Size::new(iiif_image_info.get_width(), iiif_image_info.get_height());
            levels = vec![tile_size];
        };

        // Get optional sizes.
        let optional_sizes = iiif_image_info.get_optional_sizes();

        // Get the image format.
        let image_format = iiif_image_info
            .get_profile_details()
            .next()
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing profile in '{}'",
                iiif_endpoint
            )))?
            .get_formats()
            .next()
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing image format in '{}'",
                iiif_endpoint
            )))?;

        Ok(TiledImage::new(
            iiif_endpoint.to_string(),
            tile_size,
            levels,
            image_format,
            supported_features,
            optional_sizes,
        ))
    }

    /// Get URl and size of the thumbnail.
    pub(crate) fn get_image_thumbnail(&self, size: u32) -> (String, Vec2) {
        let max_size = self.get_max_size();

        // If size by width/height is not supported, we will pick from the suggested sizes.
        let thumbnail_size = if self.supported_features.contains(&IiifFeature::SizeByWh) {
            let pct = size as f32 / max_size.max_element();

            Size::new((pct * max_size.x) as u32, (pct * max_size.y) as u32)
        } else {
            self.optional_sizes
                .iter()
                .find(|x| x.width * x.height > size * size)
                .map_or_else(
                    || {
                        *self
                            .optional_sizes
                            .first()
                            .expect("should have at least one size")
                    },
                    |x| Size::new(x.width, x.height),
                )
        };

        info!("Thumbnai {:?}", thumbnail_size);
        (
            self.get_image_url(0, 0, max_size.x as u32, max_size.y as u32, thumbnail_size),
            Vec2::from(thumbnail_size),
        )
    }

    /// Get URL for the image tile at the position.
    pub(crate) fn get_image_tile_url_at(&self, image_position: Rect) -> String {
        self.get_image_url(
            image_position.min.x.round() as u32,
            image_position.min.y.round() as u32,
            (image_position.max.x - image_position.min.x.round()).round() as u32,
            (image_position.max.y - image_position.min.y.round()).round() as u32,
            self.tile_size,
        )
    }

    /// Get the image max size in world space.
    pub(crate) fn get_world_max_size_rect(&self) -> Rect {
        Rect::from_corners(
            self.image_to_world(Vec2::ZERO).truncate(),
            self.image_to_world(self.get_max_size()).truncate(),
        )
    }

    /// Get the image max size in image space.
    pub(crate) fn get_image_max_size_rect(&self) -> Rect {
        Rect::from_corners(Vec2::ZERO, self.get_max_size())
    }

    // /// Get number of resolution levels.
    // pub(crate) fn get_num_levels(&self) -> usize {
    //     self.levels.len()
    // }

    /// Get the resolution level given the world zoom scale.
    pub(crate) fn get_level_at(&self, world_zoom_scale: f32) -> usize {
        let max_level = self.levels.len() - 1;
        let image_zoom_scale =
            self.world_to_image(Vec3::splat(world_zoom_scale)) - self.world_to_image(Vec3::ZERO);
        let image_size = self.get_max_size() / image_zoom_scale;

        for level in 0..=max_level {
            if image_size.x.abs() as u32 <= self.levels[level].width {
                return level;
            }
        }

        max_level
    }

    /// Get the required tile range to display between the world min and max.
    pub(crate) fn get_required_tiles(
        &self,
        level: usize,
        world_pos_min: Vec3,
        world_pos_max: Vec3,
    ) -> (Vec<Tile>, RangeInclusive<u32>, RangeInclusive<u32>) {
        // Convert from the world space to the image space, and clamp using the max image size.
        let image_max_size = self.get_max_size();

        let image_p0 = self
            .world_to_image(world_pos_min)
            .clamp(Vec2::ZERO, image_max_size - 1.0);
        let image_p1 = self
            .world_to_image(world_pos_max)
            .clamp(Vec2::ZERO, image_max_size - 1.0);

        // Get them in the correct order.
        let image_min = image_p0.min(image_p1);
        let image_max = image_p0.max(image_p1);

        // Convert from the image space to the tile space.
        let tile_min = self.image_to_tile(level, image_min);
        let tile_max = self.image_to_tile(level, image_max);

        let mut tiles = Vec::new();
        let mut tile_min_x = 0;
        let mut tile_min_y = 0;
        let mut tile_max_x = 0;
        let mut tile_max_y = 0;

        for y in tile_min.y as u32..=tile_max.y as u32 {
            for x in tile_min.x as u32..=tile_max.x as u32 {
                let tile_index = TileIndex::new(x, y, level as u32);
                let next_tile_index = TileIndex::new(x + 1, y + 1, level as u32);

                let image_top_left = self.tile_to_image(level, tile_index.into());
                let image_bot_rght = self
                    .tile_to_image(level, next_tile_index.into())
                    .min(image_max_size);

                let image_position = Rect::from_corners(image_top_left, image_bot_rght);

                if image_position.width() > 0.5 && image_position.height() > 0.5 {
                    let world_position = Rect::from_corners(
                        self.image_to_world(image_top_left).truncate(),
                        self.image_to_world(image_bot_rght).truncate(),
                    );

                    tile_min_x = tile_min_x.min(x);
                    tile_max_x = tile_max_x.max(x);
                    tile_min_y = tile_min_y.min(y);
                    tile_max_y = tile_max_y.max(y);
                    tiles.push(Tile::new(tile_index, image_position, world_position));
                }
            }
        }

        (tiles, tile_min_x..=tile_max_x, tile_min_y..=tile_max_y)
    }

    /// Convert from world to image space.
    pub(crate) fn world_to_image(&self, p: Vec3) -> Vec2 {
        p.reflect(Vec3::Y).truncate()
    }

    /// Convert from image to world space.
    pub(crate) fn image_to_world(&self, p: Vec2) -> Vec3 {
        p.extend(0.0).reflect(Vec3::Y)
    }

    /// Convert from image to tile space.
    fn image_to_tile(&self, level: usize, p: Vec2) -> Vec2 {
        let scale = self.world_to_image_scale(level);

        p / (Vec2::from(self.tile_size) * scale)
    }

    /// Convert from the tile to image space.
    fn tile_to_image(&self, level: usize, p: Vec2) -> Vec2 {
        let scale = self.world_to_image_scale(level);

        p * Vec2::from(self.tile_size) * scale
    }

    /// Get the max size of the image.
    fn get_max_size(&self) -> Vec2 {
        let last_level = self.levels.last().expect("should have at least one level");

        Vec2::from(*last_level)
    }

    /// Get the world to image scale.
    fn world_to_image_scale(&self, level: usize) -> f32 {
        let image_max_size = self.get_max_size();

        image_max_size.x / self.levels[level].width as f32
    }

    /// Get the image URL.
    fn get_image_url(&self, left: u32, top: u32, width: u32, height: u32, size: Size) -> String {
        let iiif_endpoint = &self.iiif_endpoint;
        let image_format = &self.image_format;
        let max_size = self.get_max_size();

        let region =
            if left == 0 && top == 0 && width == max_size.x as u32 && height == max_size.y as u32 {
                "full".into()
            } else {
                format!("{left},{top},{width},{height}")
            };

        let size = format!("{},{}", size.width, size.height);

        // E.g. "https://stacks.stanford.edu/image/iiif/hg676jb4964%2F0380_796-44/{},{},{},{}/pct:25/0/default.png"
        format!("{iiif_endpoint}/{region}/{size}/0/default.{image_format}")
    }

    /// Get the image info end point.
    pub(crate) fn get_image_info_url(iiif_endpoint: &str) -> String {
        format!("{iiif_endpoint}/info.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const TILE_SIZE: f32 = 1024.0;

    fn setup() -> TiledImage {
        let mut supported_features = HashSet::new();

        supported_features.insert(IiifFeature::SizeByWhListed);

        TiledImage::new(
            "https://iiif_end_point/uuid".into(),
            Size::new(TILE_SIZE as u32, TILE_SIZE as u32),
            vec![
                Size::new(678, 478),
                Size::new(1357, 955),
                Size::new(2713, 1910),
            ],
            IiifImageFormat::Png,
            supported_features,
            vec![
                Size::new(678, 478),
                Size::new(1357, 955),
                Size::new(2713, 1910),
            ],
        )
    }

    #[test]
    fn test_get_image_url() {
        let image = setup();

        assert_eq!(
            image.get_image_url(1, 2, 3, 4, Size::new(1, 2)),
            "https://iiif_end_point/uuid/1,2,3,4/1,2/0/default.png"
        );
    }

    #[test]
    fn test_get_max_size() {
        let image = setup();

        assert_eq!(image.get_max_size(), Vec2::new(2713.0, 1910.0));
    }

    #[test]
    fn test_world_to_image_scale() {
        let image = setup();

        assert_eq!(image.world_to_image_scale(0), 2713.0 / 678.0);
        assert_eq!(image.world_to_image_scale(1), 2713.0 / 1357.0);
        assert_eq!(image.world_to_image_scale(2), 2713.0 / 2713.0);
    }

    #[test]
    fn test_tile_to_image() {
        let image = setup();
        let p = Vec2::new(1.0, 2.0);

        assert_eq!(image.tile_to_image(0, p), p * TILE_SIZE * 2713.0 / 678.0);
        assert_eq!(image.tile_to_image(1, p), p * TILE_SIZE * 2713.0 / 1357.0);
        assert_eq!(image.tile_to_image(2, p), p * TILE_SIZE * 2713.0 / 2713.0);
    }

    #[test]
    fn test_image_to_tile() {
        let image = setup();
        let p = Vec2::new(10.0, 20.0);

        assert_eq!(image.image_to_tile(0, p), p / (TILE_SIZE * 2713.0 / 678.0));
        assert_eq!(image.image_to_tile(1, p), p / (TILE_SIZE * 2713.0 / 1357.0));
        assert_eq!(image.image_to_tile(2, p), p / (TILE_SIZE * 2713.0 / 2713.0));
    }

    #[test]
    fn test_get_level_at() {
        let image = setup();

        assert_eq!(image.get_level_at(1.0), 2);
        assert_eq!(image.get_level_at(2.0), 1);
        assert_eq!(image.get_level_at(4.0), 0);
    }

    #[test]
    fn test_world_to_image() {
        let image = setup();
        let p = Vec3::new(1.0, 2.0, 0.0);

        assert_eq!(image.world_to_image(p), p.reflect(Vec3::Y).truncate());
    }

    #[test]
    fn test_image_to_world() {
        let image = setup();
        let p = Vec2::new(1.0, 2.0);

        assert_eq!(image.image_to_world(p), p.extend(0.0).reflect(Vec3::Y));
    }

    #[test]
    fn test_get_image_tile_at() {
        let image = setup();

        assert_eq!(
            image.get_image_tile_url_at(Rect::from_corners(
                Vec2::new(10.3, 20.5),
                Vec2::new(200.5, 300.1)
            )),
            "https://iiif_end_point/uuid/10,21,191,279/1024,1024/0/default.png"
        );
        assert_eq!(
            image.get_image_tile_url_at(Rect::from_corners(
                Vec2::new(10.3, 20.5),
                Vec2::new(200.5, 300.1)
            )),
            "https://iiif_end_point/uuid/10,21,191,279/1024,1024/0/default.png"
        );
        assert_eq!(
            image.get_image_tile_url_at(Rect::from_corners(
                Vec2::new(10.3, 20.5),
                Vec2::new(200.5, 300.1)
            )),
            "https://iiif_end_point/uuid/10,21,191,279/1024,1024/0/default.png"
        );
    }

    #[test]
    fn test_get_required_tiles() {
        let image = setup();
        let world_pos_min = Vec3::new(-8000.0, -8000.0, 0.0);
        let world_pos_max = Vec3::new(8000.0, 8000.0, 0.0);

        let (tiles, tile_range_x, tile_range_y) =
            image.get_required_tiles(0, world_pos_min, world_pos_max);

        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].index, TileIndex::new(0, 0, 0));
        assert_eq!(
            tiles[0].image_position,
            Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(2713.0, 1910.0))
        );
        assert_eq!(
            tiles[0].world_position,
            Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(2713.0, -1910.0))
        );
        assert_eq!(tile_range_x, 0..=0);
        assert_eq!(tile_range_y, 0..=0);

        let world_pos_min = Vec3::new(-4000.0, -4000.0, 0.0);
        let world_pos_max = Vec3::new(4000.0, 4000.0, 0.0);
        let (tiles, tile_range_x, tile_range_y) =
            image.get_required_tiles(1, world_pos_min, world_pos_max);

        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].index, TileIndex::new(0, 0, 1));
        assert_eq!(tiles[1].index, TileIndex::new(1, 0, 1));
        assert_eq!(tile_range_x, 0..=1);
        assert_eq!(tile_range_y, 0..=0);
        assert_eq!(
            tiles[0].image_position,
            Rect::from_corners(
                Vec2::new(0.0, 0.0),
                Vec2::new(TILE_SIZE * 2713.0 / 1357.0, 1910.0)
            )
        );
        assert_eq!(
            tiles[1].image_position,
            Rect::from_corners(
                Vec2::new(TILE_SIZE * 2713.0 / 1357.0, 0.0),
                Vec2::new(2713.0, 1910.0)
            )
        );
        assert_eq!(
            tiles[0].world_position,
            Rect::from_corners(
                Vec2::new(0.0, 0.0),
                Vec2::new(TILE_SIZE * 2713.0 / 1357.0, -1910.0)
            )
        );
        assert_eq!(
            tiles[1].world_position,
            Rect::from_corners(
                Vec2::new(TILE_SIZE * 2713.0 / 1357.0, 0.0),
                Vec2::new(2713.0, -1910.0)
            )
        );

        let world_pos_min = Vec3::new(-2000.0, -2000.0, 0.0);
        let world_pos_max = Vec3::new(2000.0, 2000.0, 0.0);
        let (tiles, tile_range_x, tile_range_y) =
            image.get_required_tiles(2, world_pos_min, world_pos_max);

        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[0].index, TileIndex::new(0, 0, 2));
        assert_eq!(tiles[1].index, TileIndex::new(1, 0, 2));
        assert_eq!(tiles[2].index, TileIndex::new(0, 1, 2));
        assert_eq!(tiles[3].index, TileIndex::new(1, 1, 2));
        assert_eq!(tile_range_x, 0..=1);
        assert_eq!(tile_range_y, 0..=1);

        assert_eq!(
            tiles[0].image_position,
            Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(1024.0, 1024.0))
        );
        assert_eq!(
            tiles[1].image_position,
            Rect::from_corners(Vec2::new(1024.0, 0.0), Vec2::new(2048.0, 1024.0))
        );
        assert_eq!(
            tiles[2].image_position,
            Rect::from_corners(Vec2::new(0.0, 1024.0), Vec2::new(1024.0, 1910.0))
        );
        assert_eq!(
            tiles[3].image_position,
            Rect::from_corners(Vec2::new(1024.0, 1024.0), Vec2::new(2048.0, 1910.0))
        );
        assert_eq!(
            tiles[0].world_position,
            Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(1024.0, -1024.0))
        );
        assert_eq!(
            tiles[1].world_position,
            Rect::from_corners(Vec2::new(1024.0, 0.0), Vec2::new(2048.0, -1024.0))
        );
        assert_eq!(
            tiles[2].world_position,
            Rect::from_corners(Vec2::new(0.0, -1024.0), Vec2::new(1024.0, -1910.0))
        );
        assert_eq!(
            tiles[3].world_position,
            Rect::from_corners(Vec2::new(1024.0, -1024.0), Vec2::new(2048.0, -1910.0))
        );
    }

    // #[test]
    // fn test_get_num_levels() {
    //     let image = setup();

    //     assert_eq!(image.get_num_levels(), 3);
    // }

    #[test]
    fn test_get_world_max_size_rect() {
        let image = setup();

        assert_eq!(
            image.get_world_max_size_rect(),
            Rect::from_corners(Vec2::new(0.0, 0.0), Vec2::new(2713.0, -1910.0))
        );
    }

    #[test]
    fn test_get_image_thumbail() {
        let mut image = setup();

        let (url, size) = image.get_image_thumbnail(256);

        assert_eq!(
            url,
            "https://iiif_end_point/uuid/full/678,478/0/default.png"
        );
        assert_eq!(size, Vec2::new(678.0, 478.0));

        image.supported_features.insert(IiifFeature::SizeByWh);
        let (url, size) = image.get_image_thumbnail(256);

        assert_eq!(
            url,
            "https://iiif_end_point/uuid/full/256,180/0/default.png"
        );
        assert_eq!(size, Vec2::new(256.0, 180.0));
    }

    #[test]
    fn test_get_image_max_size_rect() {
        let image = setup();

        let rect = image.get_image_max_size_rect();

        assert_eq!(rect, Rect::new(0.0, 0.0, 2713.0, 1910.0));
    }

    #[test]
    fn test_get_image_info_url() {
        assert_eq!(
            TiledImage::get_image_info_url("https://example.com/uuid"),
            "https://example.com/uuid/info.json"
        );
    }
}
