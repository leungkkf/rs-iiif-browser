use crate::tile::{TILE_SIZE, Tile, TileIndex};
use bevy::prelude::*;

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
    image: Image,
}

impl ImageBundle {
    pub(crate) fn build(url: String, uuid: String, levels: Vec<Size>) -> Self {
        Self {
            image: Image::new(url, uuid, levels),
        }
    }
}

/// Image.
#[derive(Component)]
pub(crate) struct Image {
    /// IFFF URL, e.g. "https://stacks.stanford.edu/image/iiif"
    iif_endpoint: String,
    /// IFFF UUID, e.g. "hg676jb4964%2F0380_796-44"
    uuid: String,
    /// The number of levels and sizes.
    levels: Vec<Size>,
}

impl Image {
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
            image_position.width().round() as u32,
            image_position.height().round() as u32,
            pct.round() as u32,
        )
    }

    pub(crate) fn levels(&self) -> &[Size] {
        &self.levels
    }

    /// Get the required tile range to display between the world min and max.
    pub(crate) fn get_required_tiles(
        &self,
        level: usize,
        world_pos_min: Vec3,
        world_pos_max: Vec3,
    ) -> Vec<Tile> {
        // Convert from the world space to the image space, and clamp using the max image size.
        let image_max_size = self.get_max_size();

        let image_min = self
            .world_to_image(level, world_pos_min)
            .clamp(Vec3::ZERO, image_max_size - 1.0);
        let image_max = self
            .world_to_image(level, world_pos_max)
            .clamp(Vec3::ZERO, image_max_size - 1.0);

        // Convert from the image space to the tile space.
        let tile_min = self.image_to_tile(level, image_min);
        let tile_max = self.image_to_tile(level, image_max);

        // Tile size in image space.
        let image_tile_size = self.tile_to_image(level, Vec3::ONE).round();

        let mut tiles = Vec::new();

        for y in tile_min.y as u32..=tile_max.y as u32 {
            for x in tile_min.x as u32..=tile_max.x as u32 {
                let tile_index = TileIndex::new(x, y);

                let image_top_left = self.tile_to_image(level, tile_index.into());
                let image_bot_rght =
                    (image_top_left + image_tile_size - 1.0).min(image_max_size - 1.0);

                let image_position =
                    Rect::from_corners(image_top_left.truncate(), image_bot_rght.truncate());

                if image_position.width() > 0.5 && image_position.height() > 0.5 {
                    // Add 1.0 to have no gaps between tiles in world space.
                    let world_position = Rect::from_corners(
                        self.image_to_world(level, image_top_left).truncate(),
                        self.image_to_world(level, image_bot_rght + 1.0).truncate(),
                    );

                    tiles.push(Tile::new(tile_index, level, image_position, world_position));
                }
            }
        }

        tiles
    }

    /// Convert from world to image space.
    fn world_to_image(&self, level: usize, p: Vec3) -> Vec3 {
        let scale = self.world_to_image_scale(level);

        (p * scale).reflect(Vec3::Y)
    }

    /// Convert from image to world space.
    fn image_to_world(&self, level: usize, p: Vec3) -> Vec3 {
        let scale = self.world_to_image_scale(level);

        (p / scale).reflect(Vec3::Y)
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
        self.levels[self.levels.len() - 1].width as f32 / self.levels[level].width as f32
    }

    /// Get the image URL.
    fn get_image_url(&self, left: u32, top: u32, width: u32, height: u32, pct: u32) -> String {
        let iif_endpoint = &self.iif_endpoint;
        let uuid = &self.uuid;

        // E.g. "https://stacks.stanford.edu/image/iiif/hg676jb4964%2F0380_796-44/{},{},{},{}/pct:25/0/default.png"
        format!("{iif_endpoint}/{uuid}/{left},{top},{width},{height}/pct:{pct}/0/default.png")
    }
}
