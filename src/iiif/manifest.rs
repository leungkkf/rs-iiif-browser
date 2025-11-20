use crate::{
    iiif::IiifError,
    rdf::{self, dataset_ext::DatasetExt},
    rendering::tiled_image::TiledImage,
};
use bevy::prelude::Component;
use sophia::{
    api::{dataset::CollectibleDataset, ns::NsTerm, prelude::Any, term::SimpleTerm},
    inmem::dataset::FastDataset,
    iri::IriRef,
};

pub(crate) struct Service {
    pub(crate) id: String,
}

impl Service {
    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }
}

pub(crate) struct Resource {
    pub(crate) format: String,
    pub(crate) service: Service,
    pub(crate) height: u32,
    pub(crate) width: u32,
}

impl Resource {
    pub(crate) fn new(format: String, service: Service, width: u32, height: u32) -> Self {
        Self {
            format,
            service,
            height,
            width,
        }
    }
}

pub(crate) struct Image {
    pub(crate) resource: Resource,
}

impl Image {
    pub(crate) fn new(resource: Resource) -> Self {
        Self { resource }
    }

    fn try_from_dataset<T: CollectibleDataset>(
        image_body: &str,
        dataset: &DatasetExt<T>,
    ) -> Result<Self, IiifError> {
        let image_bodies_subject = NsTerm::new_unchecked(IriRef::new_unchecked(image_body), "");

        let image_service = dataset
            .get_first_object_cloned_as([image_bodies_subject], [rdf::svcs::has_service])?
            .ok_or(IiifError::IiifMissingInfo(
                "Missing service in image".into(),
            ))?;
        let image_width = dataset
            .get_first_object_cloned_as([image_bodies_subject], [rdf::exif::width])?
            .ok_or(IiifError::IiifMissingInfo("Missing width in image".into()))?;
        let image_height = dataset
            .get_first_object_cloned_as([image_bodies_subject], [rdf::exif::height])?
            .ok_or(IiifError::IiifMissingInfo("Missing height in image".into()))?;
        let image_format = dataset
            .get_first_object_cloned_as([image_bodies_subject], [rdf::dc::format])?
            .unwrap_or_default();

        let service = Service::new(image_service);

        let resource = Resource::new(image_format, service, image_width, image_height);

        Ok(Self::new(resource))
    }
}

pub(crate) struct Thumbnail {
    pub(crate) id: String,
}

impl Thumbnail {
    pub(crate) fn new(id: String) -> Self {
        Self { id }
    }
}

pub(crate) struct Canvas {
    pub(crate) label: Vec<String>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) images: Vec<Image>,
    pub(crate) thumbnail: Option<Thumbnail>,
}

impl Canvas {
    pub(crate) fn new(
        label: Vec<String>,
        width: u32,
        height: u32,
        images: Vec<Image>,
        thumbnail: Option<Thumbnail>,
    ) -> Self {
        Self {
            label,
            width,
            height,
            images,
            thumbnail,
        }
    }

    fn try_from_dataset<T: CollectibleDataset>(
        canvs_subject: &SimpleTerm,
        dataset: &DatasetExt<T>,
    ) -> Result<Self, IiifError> {
        let canvas_width = dataset
            .get_first_object_cloned_as([canvs_subject], [rdf::exif::width])?
            .ok_or(IiifError::IiifMissingInfo(format!(
                "Missing width in canvas in subject '{:?}'",
                canvs_subject
            )))?;
        let canvas_height = dataset
            .get_first_object_cloned_as([canvs_subject], [rdf::exif::height])?
            .ok_or(IiifError::IiifMissingInfo(format!(
                "Missing height in canvas in subject '{:?}'",
                canvs_subject
            )))?;
        let canvas_label =
            dataset.get_objects_as([canvs_subject], [sophia::api::ns::rdfs::label])?;
        let canvas_thumbnail = dataset
            .get_first_object_cloned_as([canvs_subject], [rdf::foaf::thumbnail])?
            .map(Thumbnail::new);

        let mut images = Vec::new();

        for image_annotation_node in
            dataset.objects_iter([canvs_subject], [rdf::iiif_present2::hasImageAnnotations])
        {
            for image_annotation_subject in dataset.objects_iter([image_annotation_node?], Any) {
                for image_body in dataset.get_objects_as::<_, _, String>(
                    [image_annotation_subject?],
                    [rdf::oa::hasBody],
                )? {
                    images.push(Image::try_from_dataset(&image_body, dataset)?);
                }
            }
        }

        Ok(Self::new(
            canvas_label,
            canvas_width,
            canvas_height,
            images,
            canvas_thumbnail,
        ))
    }
}

pub(crate) struct Sequence {
    pub(crate) label: Vec<String>,
    pub(crate) canvases: Vec<Canvas>,
}

impl Sequence {
    pub(crate) fn new(label: Vec<String>, canvases: Vec<Canvas>) -> Self {
        Self { label, canvases }
    }

    fn try_from_dataset<T: CollectibleDataset>(
        sequence_subject: &SimpleTerm,
        dataset: &DatasetExt<T>,
    ) -> Result<Self, IiifError> {
        let sequence_label =
            dataset.get_objects_as([sequence_subject], [sophia::api::ns::rdfs::label])?;

        let mut canvases = Vec::new();

        for canvas_node in
            dataset.objects_iter([sequence_subject], [rdf::iiif_present2::hasCanvases])
        {
            for canvas in dataset.objects_iter([canvas_node?], Any) {
                canvases.push(Canvas::try_from_dataset(&canvas?, dataset)?);
            }
        }

        Ok(Sequence::new(sequence_label, canvases))
    }
}

#[derive(Component)]
/// Presentation manifest.
pub(crate) struct Manifest {
    pub(crate) title: String,
    pub(crate) attribution: Vec<String>,
    pub(crate) description: Vec<String>,
    pub(crate) license: Vec<String>,
    pub(crate) logo: Vec<String>,
    pub(crate) sequences: Vec<Sequence>,
}

impl Manifest {
    pub(crate) fn new(
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
        let dataset = DatasetExt::<FastDataset>::try_from_url(url, None)?;

        Self::try_from_dataset(&dataset)
    }

    /// Try to create the manifest from the RDF database.
    pub(crate) fn try_from_dataset<T: CollectibleDataset>(
        dataset: &DatasetExt<T>,
    ) -> core::result::Result<Self, IiifError> {
        let id_subject = dataset.id();

        let title = dataset
            .get_first_object_cloned_as([id_subject], [sophia::api::ns::rdfs::label])?
            .unwrap_or_default();
        let attribution =
            dataset.get_objects_as([id_subject], [rdf::iiif_present2::attributionLabel])?;
        let license = dataset.get_objects_as([id_subject], [rdf::dcterms::rights])?;
        let description = dataset.get_objects_as([id_subject], [rdf::dc::description])?;
        let logo = dataset.get_objects_as([id_subject], [rdf::foaf::logo])?;

        let mut sequences = Vec::new();

        for seq_node in dataset.objects_iter([id_subject], [rdf::iiif_present2::hasSequences]) {
            for seq_subject in dataset.objects_iter([seq_node?], Any) {
                sequences.push(Sequence::try_from_dataset(&seq_subject?, dataset)?);
            }
        }

        for seq in sequences.iter_mut() {
            for canvas in seq.canvases.iter_mut() {
                if canvas.thumbnail.is_none() {
                    let image = TiledImage::try_from_url(&canvas.images[0].resource.service.id)?;

                    canvas.thumbnail = Some(Thumbnail::new(image.get_image_thumbnail(64).0));
                }
            }
        }

        Ok(Manifest::new(
            title,
            attribution,
            description,
            license,
            logo,
            sequences,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let json = r#"{
          "@context": "http://iiif.io/api/presentation/2/context.json",
          "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378",
          "@type": "sc:Manifest",
          "attribution": "Provided by Harvard University",
          "label": "Harvard University, Harvard Art Museums, INV204583",
          "license": "https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright",
          "logo": "https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg",
          "sequences": [
            {
              "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378/sequence/normal.json",
              "@type": "sc:Sequence",
              "canvases": [
                {
                  "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json",
                  "@type": "sc:Canvas",
                  "height": 833,
                  "images": [
                    {
                      "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378/annotation/anno-11927378.json",
                      "@type": "oa:Annotation",
                      "motivation": "sc:painting",
                      "on": "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json",
                      "resource": {
                        "@id": "https://ids.lib.harvard.edu/ids/iiif/11927378/full/full/0/default.jpg",
                        "@type": "dctypes:Image",
                        "format": "image/jpeg",
                        "height": 833,
                        "service": {
                          "@context": "http://iiif.io/api/image/2/context.json",
                          "@id": "https://ids.lib.harvard.edu/ids/iiif/11927378",
                          "profile": "http://iiif.io/api/image/2/level2.json"
                        },
                        "width": 1024
                      }
                    }
                  ],
                  "label": "Harvard University, Harvard Art Museums, INV204583",
                  "thumbnail": {
                    "@id": "https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg",
                    "@type": "dctypes:Image"
                  },
                  "width": 1024
                }
              ],
              "label": "Harvard University, Harvard Art Museums, INV204583",
              "startCanvas": "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json",
              "viewingHint": "individuals"
            }
          ]
        }"#;
        let id = "https://iiif.lib.harvard.edu/manifests/ids:11927378";
        let dataset = DatasetExt::<FastDataset>::try_from_json(id, json).unwrap();
        let manifest = Manifest::try_from_dataset(&dataset).unwrap();

        assert_eq!(manifest.attribution, vec!["Provided by Harvard University"]);
        assert_eq!(
            manifest.license,
            vec!["https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright"]
        );
        assert_eq!(
            manifest.title,
            "Harvard University, Harvard Art Museums, INV204583"
        );
        assert_eq!(
            manifest.logo,
            vec!["https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg"]
        );
        assert_eq!(manifest.description, Vec::<String>::new());

        assert_eq!(manifest.sequences.len(), 1);

        let seq = &manifest.sequences[0];

        assert_eq!(
            seq.label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        assert_eq!(seq.canvases.len(), 1);

        let canvas = &seq.canvases[0];

        assert_eq!(canvas.width, 1024);
        assert_eq!(canvas.height, 833);
        assert_eq!(
            canvas.label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );
        assert_eq!(
            canvas.thumbnail.as_ref().unwrap().id,
            "https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg"
        );
        assert_eq!(canvas.images.len(), 1);

        let image = &canvas.images[0];

        assert_eq!(image.resource.width, 1024);
        assert_eq!(image.resource.height, 833);
        assert_eq!(
            image.resource.service.id,
            "https://ids.lib.harvard.edu/ids/iiif/11927378"
        );
        assert_eq!(image.resource.format, "image/jpeg");
    }
}
