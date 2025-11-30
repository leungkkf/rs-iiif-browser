use crate::iiif::IiifError;
use std::borrow::Cow;

/// The manifest model.
pub(crate) trait IsManifest: Send + Sync {
    fn get_title(&self) -> Cow<'_, str>;
    fn get_attribution(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_description(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_license(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_logo(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_>;
    fn get_sequence(&self, index: usize) -> Result<&dyn IsSequence, IiifError>;
}

/// The sequence model.
pub(crate) trait IsSequence {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCavas> + '_>;
    fn get_canvas(&self, index: usize) -> Result<&dyn IsCavas, IiifError>;
}

/// The canvas model.
pub(crate) trait IsCavas {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_thumbnail(&self) -> Cow<'_, str>;
    // fn get_images(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsImage> + '_>;
    fn get_image(&self, index: usize) -> Result<&dyn IsImage, IiifError>;
}

/// The image model.
pub(crate) trait IsImage {
    fn get_service(&self) -> Cow<'_, str>;
    // fn get_width(&self) -> u32;
    // fn get_height(&self) -> u32;
}
