use crate::iiif::{self, IiifError, manifest::Sequence};
use bevy::prelude::Component;

#[derive(Component)]
/// Presentation manifest.
pub(crate) struct Manifest {
    title: String,
    attribution: Vec<String>,
    description: Vec<String>,
    license: Vec<String>,
    logo: Vec<String>,
    sequences: Vec<Sequence>,
}

impl Manifest {
    fn new(
        title: String,
        attribution: Vec<String>,
        description: Vec<String>,
        license: Vec<String>,
        logo: Vec<String>,
        sequences: Vec<Sequence>,
    ) -> Self {
        Self {
            title,
            attribution,
            description,
            license,
            logo,
            sequences,
        }
    }

    /// Try to create the manifest from the URL.
    pub(crate) fn try_from_url(url: &str) -> core::result::Result<Self, IiifError> {
        let iiif_manifest = iiif::manifest::Manifest::try_from_url(url)?;

        Ok(Manifest::from(iiif_manifest))
    }

    pub(crate) fn get_title(&self) -> &str {
        &self.title
    }

    pub(crate) fn get_attribution(&self) -> &Vec<String> {
        &self.attribution
    }

    pub(crate) fn get_description(&self) -> &Vec<String> {
        &self.description
    }

    pub(crate) fn get_license(&self) -> &Vec<String> {
        &self.license
    }

    pub(crate) fn get_logo(&self) -> &Vec<String> {
        &self.logo
    }

    pub(crate) fn get_sequences(&self) -> &Vec<Sequence> {
        &self.sequences
    }
}

impl From<iiif::manifest::Manifest> for Manifest {
    fn from(iiif_manifest: iiif::manifest::Manifest) -> Self {
        Manifest::new(
            iiif_manifest.title,
            iiif_manifest.attribution,
            iiif_manifest.description,
            iiif_manifest.license,
            iiif_manifest.logo,
            iiif_manifest.sequences,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iiif_manifest() {
        let iiif_resource = iiif::manifest::Resource::new(
            "format".to_string(),
            iiif::manifest::Service::new("service_id".to_string()),
            100,
            200,
        );

        let iiif_image = iiif::manifest::Image::new(iiif_resource);

        let iiif_thumbnail = iiif::manifest::Thumbnail::new("thumbnail_id".to_string());

        let iiif_canvas = iiif::manifest::Canvas::new(
            vec!["canvas_label".to_string()],
            1,
            2,
            vec![iiif_image],
            Some(iiif_thumbnail),
        );

        let iiif_seq =
            iiif::manifest::Sequence::new(vec!["seq_label".to_string()], vec![iiif_canvas]);

        let iiif_manifest = iiif::manifest::Manifest::new(
            "title".to_string(),
            vec!["attribution".to_string()],
            vec!["description".to_string()],
            vec!["license".to_string()],
            vec!["logo".to_string()],
            vec![iiif_seq],
        );

        let manifest = Manifest::from(iiif_manifest);

        assert_eq!(manifest.attribution, vec!["attribution"]);
        assert_eq!(manifest.license, vec!["license"]);
        assert_eq!(manifest.title, "title");
        assert_eq!(manifest.logo, vec!["logo"]);
        assert_eq!(manifest.description, vec!["description"]);

        assert_eq!(manifest.sequences.len(), 1);

        let seq = &manifest.sequences[0];

        assert_eq!(seq.label, vec!["seq_label"]);

        assert_eq!(seq.canvases.len(), 1);

        let canvas = &seq.canvases[0];

        assert_eq!(canvas.width, 1);
        assert_eq!(canvas.height, 2);
        assert_eq!(canvas.label, vec!["canvas_label"]);
        assert_eq!(canvas.thumbnail.as_ref().unwrap().id, "thumbnail_id");
        assert_eq!(canvas.images.len(), 1);

        let image = &canvas.images[0];

        assert_eq!(image.resource.width, 100);
        assert_eq!(image.resource.height, 200);
        assert_eq!(image.resource.service.id, "service_id");
    }
}
