use crate::iiif::{self, IiifError};
use bevy::prelude::Component;

pub(crate) struct Service {
    pub(crate) id: String,
}

impl Service {
    fn new(id: String) -> Self {
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
    fn new(format: String, service: Service, height: u32, width: u32) -> Self {
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
    fn new(resource: Resource) -> Self {
        Self { resource }
    }
}

pub(crate) struct Thumbnail {
    pub(crate) id: String,
    pub(crate) thumbnail_type: String,
}

impl Thumbnail {
    fn new(id: String, thumbnail_type: String) -> Self {
        Self { id, thumbnail_type }
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
    fn new(
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
}

pub(crate) struct Sequence {
    pub(crate) label: Vec<String>,
    pub(crate) canvases: Vec<Canvas>,
}

impl Sequence {
    fn new(label: Vec<String>, canvases: Vec<Canvas>) -> Self {
        Self { label, canvases }
    }
}

#[derive(Component)]
pub(crate) struct PresentationInfo {
    pub(crate) title: String,
    pub(crate) attribution: Vec<String>,
    pub(crate) description: Vec<String>,
    pub(crate) license: Vec<String>,
    pub(crate) logo: Vec<String>,
    pub(crate) sequences: Vec<Sequence>,
}

impl PresentationInfo {
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

    pub(crate) fn build(url: &str) -> core::result::Result<Self, IiifError> {
        let iiif_presentation_info = iiif::presentation::PresentationInfo::from_url(url)?;

        match iiif_presentation_info {
            iiif::presentation::PresentationInfo::Version2(iiif_presentation_info) => {
                let title = iiif_presentation_info.label;
                let attribution = iiif_presentation_info.attribution.into_iter().collect();
                let description: Vec<_> = iiif_presentation_info
                    .description
                    .map_or_else(Vec::new, |x| x.into_iter().collect());
                let license: Vec<_> = iiif_presentation_info
                    .license
                    .map_or_else(Vec::new, |x| x.iter().map(|y| y.to_string()).collect());
                let logo = iiif_presentation_info
                    .logo
                    .map_or_else(Vec::new, |x| x.into_iter().collect());

                let mut sequences = Vec::new();

                for seq in iiif_presentation_info.sequences {
                    let mut canvases = Vec::new();

                    for canvas in seq.canvases {
                        let mut images = Vec::new();
                        for image in canvas.images {
                            let service = Service::new(image.resource.service.id);
                            let resource = Resource::new(
                                image.resource.format,
                                service,
                                image.resource.width,
                                image.resource.height,
                            );

                            images.push(Image::new(resource));
                        }
                        let thumbnail = canvas
                            .thumbnail
                            .map(|x| Thumbnail::new(x.id, x.thumbnail_type));

                        canvases.push(Canvas::new(
                            canvas.label.into_iter().collect(),
                            canvas.width,
                            canvas.height,
                            images,
                            thumbnail,
                        ));
                    }

                    let seq_label = seq
                        .label
                        .map_or_else(Vec::new, |x| x.iter().map(|y| y.to_string()).collect());

                    sequences.push(Sequence::new(seq_label, canvases));
                }

                Ok(PresentationInfo::new(
                    title,
                    attribution,
                    description,
                    license,
                    logo,
                    sequences,
                ))
            }
            iiif::presentation::PresentationInfo::Version3(_iiif_presentation_info) => Err(
                IiifError::IiifUnsupportedError("Version 3 not supported".into()),
            ),
        }
    }
}
