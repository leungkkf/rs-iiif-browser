use crate::iiif::IiifError;
use std::borrow::Cow;

/// Trait that represents an IIIF manifest needed by the UI.
pub(crate) trait IsManifest: Send + Sync {
    fn get_title(&self, language: &str) -> Cow<'_, str>;
    fn get_attribution(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_required_statements(
        &self,
        language: &str,
    ) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_description(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_license(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_logo(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_>;
    fn get_sequence(&self, index: usize) -> Result<&dyn IsSequence, IiifError>;
}

/// Trait that represents a sequence in IIIF manifest needed by the UI.
pub(crate) trait IsSequence {
    fn get_label(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCanvas> + '_>;
    fn get_canvas(&self, index: usize) -> Result<&dyn IsCanvas, IiifError>;
}

/// Trait that represents a canvas in a sequence in IIIF manifest needed by the UI.
pub(crate) trait IsCanvas {
    fn get_label(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
    fn get_thumbnail(&self) -> Cow<'_, str>;
    // fn get_images(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsImage> + '_>;
    fn get_image(&self, index: usize) -> Result<&dyn IsImage, IiifError>;
}

/// Trait that represents a image in a canvas in a sequence in IIIF manifest needed by the UI.
pub(crate) trait IsImage {
    fn get_service(&self) -> Cow<'_, str>;
    fn get_id(&self) -> Cow<'_, str>;
    fn get_type(&self) -> Cow<'_, str>;
    // fn get_width(&self) -> u32;
    // fn get_height(&self) -> u32;
}
