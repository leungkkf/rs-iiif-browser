use crate::{
    iiif::image::{IiifFeature, IiifImageFormat},
    rendering::tiled_image::Size,
};

/// Trait that represents an IIIF image needed by the TiledImage.
pub(crate) trait IsImage {
    fn get_tile_size(&self) -> Size;
    fn get_tile_scaling_sizes(&self) -> Vec<Size>;
    fn get_profile_details(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsProfileDetails> + '_>;
    fn get_optional_sizes(&self) -> Vec<Size>;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

/// Trait that represents the profile details in an IIIF image needed by the TiledImage.
pub(crate) trait IsProfileDetails {
    fn get_supported_features(&self) -> Box<dyn ExactSizeIterator<Item = IiifFeature> + '_>;
    fn get_formats(&self) -> Box<dyn ExactSizeIterator<Item = IiifImageFormat> + '_>;
}
