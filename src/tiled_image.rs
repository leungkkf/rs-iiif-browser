use crate::tile::{TILE_SIZE, Tile, TileIndex};
use bevy::prelude::*;
use std::ops::RangeInclusive;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Size {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Size {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[derive(Bundle)]
pub(crate) struct ImageBundle {
    image: TiledImage,
}

impl ImageBundle {
    pub(crate) fn build(url: String, uuid: String, levels: Vec<Size>) -> Self {
        Self {
            image: TiledImage::new(url, uuid, levels),
        }
    }
}

/// Image.
#[derive(Component)]
pub(crate) struct TiledImage {
    /// IFFF URL, e.g. "https://stacks.stanford.edu/image/iiif"
    iif_endpoint: String,
    /// IFFF UUID, e.g. "hg676jb4964%2F0380_796-44"
    uuid: String,
    /// The number of levels and sizes.
    levels: Vec<Size>,
}

impl TiledImage {
    /// Create a new image.
    pub(crate) fn new(iif_endpoint: String, uuid: String, levels: Vec<Size>) -> Self {
        Self {
            iif_endpoint,
            uuid,
            levels,
        }
    }

    /// Get URL for the image tile at the position.
    pub(crate) fn get_image_tile_at(&self, level: usize, image_position: Rect) -> String {
        let image_max_size = self.get_max_size();
        let pct = 100.0 * self.levels[level].width as f32 / image_max_size.x;

        self.get_image_url(
            image_position.min.x.round() as u32,
            image_position.min.y.round() as u32,
            (image_position.max.x - image_position.min.x.round()).round() as u32,
            (image_position.max.y - image_position.min.y.round()).round() as u32,
            pct,
        )
    }

    /// Get number of resolution levels.
    pub(crate) fn get_num_levels(&self) -> usize {
        self.levels.len()
    }

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
            .clamp(Vec3::ZERO, image_max_size - 1.0);
        let image_p1 = self
            .world_to_image(world_pos_max)
            .clamp(Vec3::ZERO, image_max_size - 1.0);

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

                let image_position =
                    Rect::from_corners(image_top_left.truncate(), image_bot_rght.truncate());

                if image_position.width() > 0.5 && image_position.height() > 0.5 {
                    let world_position = Rect::from_corners(
                        self.image_to_world(image_top_left).truncate(),
                        self.image_to_world(image_bot_rght).truncate(),
                    );

                    tile_min_x = tile_min_x.min(x);
                    tile_max_x = tile_max_x.max(x);
                    tile_min_y = tile_min_y.min(x);
                    tile_max_y = tile_max_y.max(x);
                    tiles.push(Tile::new(tile_index, image_position, world_position));
                }
            }
        }

        (tiles, tile_min_x..=tile_max_x, tile_min_y..=tile_max_y)
    }

    /// Convert from world to image space.
    fn world_to_image(&self, p: Vec3) -> Vec3 {
        p.reflect(Vec3::Y)
    }

    /// Convert from image to world space.
    fn image_to_world(&self, p: Vec3) -> Vec3 {
        p.reflect(Vec3::Y)
    }

    /// Convert from image to tile space.
    fn image_to_tile(&self, level: usize, p: Vec3) -> Vec3 {
        let scale = self.world_to_image_scale(level);

        p / (TILE_SIZE * scale)
    }

    /// Convert from the tile to image space.
    fn tile_to_image(&self, level: usize, p: Vec3) -> Vec3 {
        let scale = self.world_to_image_scale(level);

        p * TILE_SIZE * scale
    }

    /// Get the max size of the image.
    fn get_max_size(&self) -> Vec3 {
        let last_level = self.levels.last().expect("should have at least one level");

        Vec3::new(last_level.width as f32, last_level.height as f32, 0.0)
    }

    /// Get the world to image scale.
    fn world_to_image_scale(&self, level: usize) -> f32 {
        let image_max_size = self.get_max_size();

        image_max_size.x / self.levels[level].width as f32
    }

    /// Get the image URL.
    fn get_image_url(&self, left: u32, top: u32, width: u32, height: u32, pct: f32) -> String {
        let iif_endpoint = &self.iif_endpoint;
        let uuid = &self.uuid;

        // E.g. "https://stacks.stanford.edu/image/iiif/hg676jb4964%2F0380_796-44/{},{},{},{}/pct:25/0/default.png"
        format!("{iif_endpoint}/{uuid}/{left},{top},{width},{height}/pct:{pct}/0/default.png")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> TiledImage {
        TiledImage::new(
            "https://iif_end_point".into(),
            "uuid".into(),
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
            image.get_image_url(1, 2, 3, 4, 30.1),
            "https://iif_end_point/uuid/1,2,3,4/pct:30.1/0/default.png"
        );
    }

    #[test]
    fn test_get_max_size() {
        let image = setup();

        assert_eq!(image.get_max_size(), Vec3::new(2713.0, 1910.0, 0.0));
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
        let p = Vec3::new(1.0, 2.0, 0.0);

        assert_eq!(image.tile_to_image(0, p), p * TILE_SIZE * 2713.0 / 678.0);
        assert_eq!(image.tile_to_image(1, p), p * TILE_SIZE * 2713.0 / 1357.0);
        assert_eq!(image.tile_to_image(2, p), p * TILE_SIZE * 2713.0 / 2713.0);
    }

    #[test]
    fn test_image_to_tile() {
        let image = setup();
        let p = Vec3::new(10.0, 20.0, 0.0);

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

        assert_eq!(image.world_to_image(p), p.reflect(Vec3::Y));
    }

    #[test]
    fn test_image_to_world() {
        let image = setup();
        let p = Vec3::new(1.0, 2.0, 0.0);

        assert_eq!(image.image_to_world(p), p.reflect(Vec3::Y));
    }

    #[test]
    fn test_get_image_tile_at() {
        let image = setup();

        assert_eq!(
            image.get_image_tile_at(
                0,
                Rect::from_corners(Vec2::new(10.3, 20.5), Vec2::new(200.5, 300.1))
            ),
            "https://iif_end_point/uuid/10,21,191,279/pct:24.990786/0/default.png"
        );
        assert_eq!(
            image.get_image_tile_at(
                1,
                Rect::from_corners(Vec2::new(10.3, 20.5), Vec2::new(200.5, 300.1))
            ),
            "https://iif_end_point/uuid/10,21,191,279/pct:50.01843/0/default.png"
        );
        assert_eq!(
            image.get_image_tile_at(
                2,
                Rect::from_corners(Vec2::new(10.3, 20.5), Vec2::new(200.5, 300.1))
            ),
            "https://iif_end_point/uuid/10,21,191,279/pct:100/0/default.png"
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
        assert_eq!(tile_range_y, 0..=1);
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

    #[test]
    fn test_get_num_levels() {
        let image = setup();

        assert_eq!(image.get_num_levels(), 3);
    }
}
