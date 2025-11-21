use crate::iiif::{IiifError, manifest};
use bevy::prelude::Component;

#[derive(Component)]
/// Presentation manifest.
pub(crate) struct Manifest {
    inner: Box<dyn HasManifest>,
}

impl Manifest {
    fn new(inner: Box<dyn HasManifest>) -> Self {
        Self { inner }
    }

    /// Try to create the manifest from the URL.
    pub(crate) fn try_from_url(url: &str) -> core::result::Result<Self, IiifError> {
        let iiif_manifest = manifest::try_from_url(url)?;

        Ok(Manifest::from(iiif_manifest))
    }

    pub(crate) fn get_title(&self) -> &str {
        self.inner.get_title()
    }

    pub(crate) fn get_attribution(&self) -> Vec<&str> {
        self.inner.get_attribution().collect()
    }

    pub(crate) fn get_description(&self) -> Vec<&str> {
        self.inner.get_description().collect()
    }

    pub(crate) fn get_license(&self) -> Vec<&str> {
        self.inner.get_license().collect()
    }

    pub(crate) fn get_logo(&self) -> Vec<&str> {
        self.inner.get_logo().collect()
    }

    pub(crate) fn get_sequences(&self) -> Box<dyn Iterator<Item = &dyn HasSequence> + '_> {
        self.inner.get_sequences()
    }

    pub(crate) fn get_sequence(&self, index: usize) -> &dyn HasSequence {
        self.inner.get_sequence(index)
    }
}

impl From<Box<dyn HasManifest>> for Manifest {
    fn from(v: Box<dyn HasManifest>) -> Self {
        Self::new(v)
    }
}

pub(crate) trait HasManifest: Send + Sync {
    fn get_title(&self) -> &str;
    fn get_attribution(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_description(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_license(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_logo(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_sequences(&self) -> Box<dyn Iterator<Item = &dyn HasSequence> + '_>;
    fn get_sequence(&self, index: usize) -> &dyn HasSequence;
}

pub(crate) trait HasSequence: Send + Sync {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_canvases(&self) -> Box<dyn Iterator<Item = &dyn HasCavas> + '_>;
    fn get_canvase(&self, index: usize) -> &dyn HasCavas;
}

pub(crate) trait HasCavas: Send + Sync {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_thumbnail(&self) -> Box<dyn Iterator<Item = &str> + '_>;
    fn get_images(&self) -> Box<dyn Iterator<Item = &dyn HasImage> + '_>;
    fn get_image(&self, index: usize) -> &dyn HasImage;
}

pub(crate) trait HasImage: Send + Sync {
    fn get_service(&self) -> &str;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_from_iiif_manifest() {
    //     let iiif_resource = iiif::manifest::Resource::new(
    //         "format".to_string(),
    //         iiif::manifest::Service::new("service_id".to_string()),
    //         100,
    //         200,
    //     );

    //     let iiif_image = iiif::manifest::Image::new(iiif_resource);

    //     let iiif_thumbnail = iiif::manifest::Thumbnail::new("thumbnail_id".to_string());

    //     let iiif_canvas = iiif::manifest::Canvas::new(
    //         vec!["canvas_label".to_string()],
    //         1,
    //         2,
    //         vec![iiif_image],
    //         Some(iiif_thumbnail),
    //     );

    //     let iiif_seq =
    //         iiif::manifest::Sequence::new(vec!["seq_label".to_string()], vec![iiif_canvas]);

    //     let iiif_manifest = iiif::manifest::Manifest::new(
    //         "title".to_string(),
    //         vec!["attribution".to_string()],
    //         vec!["description".to_string()],
    //         vec!["license".to_string()],
    //         vec!["logo".to_string()],
    //         vec![iiif_seq],
    //     );

    //     let manifest = Manifest::from(iiif_manifest);

    //     assert_eq!(manifest.attribution, vec!["attribution"]);
    //     assert_eq!(manifest.license, vec!["license"]);
    //     assert_eq!(manifest.title, "title");
    //     assert_eq!(manifest.logo, vec!["logo"]);
    //     assert_eq!(manifest.description, vec!["description"]);

    //     assert_eq!(manifest.sequences.len(), 1);

    //     let seq = &manifest.sequences[0];

    //     assert_eq!(seq.label, vec!["seq_label"]);

    //     assert_eq!(seq.canvases.len(), 1);

    //     let canvas = &seq.canvases[0];

    //     assert_eq!(canvas.width, 1);
    //     assert_eq!(canvas.height, 2);
    //     assert_eq!(canvas.label, vec!["canvas_label"]);
    //     assert_eq!(canvas.thumbnail.as_ref().unwrap().id, "thumbnail_id");
    //     assert_eq!(canvas.images.len(), 1);

    //     let image = &canvas.images[0];

    //     assert_eq!(image.resource.width, 100);
    //     assert_eq!(image.resource.height, 200);
    //     assert_eq!(image.resource.service.id, "service_id");
    // }
}
