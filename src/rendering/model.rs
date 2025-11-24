use crate::{
    iiif::image::{IiifFeature, IiifImageFormat},
    rendering::tiled_image::Size,
};

pub(crate) trait IsImage {
    fn get_tile_size(&self) -> Size;
    fn get_tile_scaling_sizes(&self) -> Vec<Size>;
    fn get_profile_details(&self) -> impl ExactSizeIterator<Item = &impl IsProfileDetails>;
    fn get_optional_sizes(&self) -> Vec<Size>;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

pub(crate) trait IsProfileDetails {
    fn get_supported_features(&self) -> impl ExactSizeIterator<Item = IiifFeature>;
    fn get_formats(&self) -> impl ExactSizeIterator<Item = IiifImageFormat>;
}
