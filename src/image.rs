use crate::tile::{TILE_SIZE, TileIndex};
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

    /// Get display info for the image tile at the TileIndex, including the URL, mesh position and the mesh size.
    pub(crate) fn get_image_tile_display_info(
        &self,
        level: usize,
        tile_index: TileIndex,
    ) -> (String, Vec3, Vec3) {
        // Convert the tile index in the tile space to image space.
        let image_pos = self.tile_to_image(level, tile_index.into());

        // Clamp the size of the tile by the max image size.
        let image_size = self.tile_to_image(level, Vec3::ONE);
        let max_image_size = self.get_max_size();
        let clamped_size = (image_pos + image_size).min(max_image_size) - image_pos;

        let pct = 100.0 * self.levels[level].width as f32 / max_image_size.x;

        let url = self.get_image_url(
            image_pos.x.round() as u32,
            image_pos.y.round() as u32,
            clamped_size.x.round() as u32,
            clamped_size.y.round() as u32,
            pct.round() as u32,
        );

        let world_mesh_pos = self.image_to_world(
            level,
            self.tile_to_image(level, tile_index.into()) + clamped_size / 2.0,
        );

        let world_mesh_size = self.image_to_world(level, clamped_size).abs() - 1.0;

        (url, world_mesh_pos, world_mesh_size)
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
    ) -> (TileIndex, TileIndex) {
        // Convert from the world space to the image space, and clamp using the max image size.
        let max_size = self.get_max_size();

        let image_min = self
            .world_to_image(level, world_pos_min)
            .clamp(Vec3::ZERO, max_size);
        let image_max = self
            .world_to_image(level, world_pos_max)
            .clamp(Vec3::ZERO, max_size);

        // Convert from the image space to the tile space.
        let tile_min = self.image_to_tile(level, image_min);
        let tile_max = self.image_to_tile(level, image_max);

        let tile_min = TileIndex::from(tile_min);
        let mut tile_max = TileIndex::from(tile_max);

        // No need to get the last one if the remaining is less than half a pixel.
        let image_tile_max = self.tile_to_image(level, tile_max.into());

        if f32::abs(image_tile_max.x - image_max.x) <= 0.5 {
            tile_max.x -= 1;
        }
        if f32::abs(image_tile_max.y - image_max.y) <= 0.5 {
            tile_max.y -= 1;
        }

        (tile_min, tile_max)
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
