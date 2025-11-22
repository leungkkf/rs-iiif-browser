/// The manifest model.
pub(crate) trait IsManifest: Send + Sync {
    fn get_title(&self) -> &str;
    fn get_attribution(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_description(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_license(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_logo(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_>;
    fn get_sequence(&self, index: usize) -> &dyn IsSequence;
}

/// The sequence model.
pub(crate) trait IsSequence: Send + Sync {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCavas> + '_>;
    fn get_canvas(&self, index: usize) -> &dyn IsCavas;
}

/// The canvas model.
pub(crate) trait IsCavas: Send + Sync {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_thumbnail(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_images(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsImage> + '_>;
    fn get_image(&self, index: usize) -> &dyn IsImage;
}

/// The image model.
pub(crate) trait IsImage: Send + Sync {
    fn get_service(&self) -> &str;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}
