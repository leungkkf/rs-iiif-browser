use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::iiif::IiifError;
use crate::iiif::manifest::{Context, ViewingDirection};
use crate::iiif::one_or_many::OneTypeOrMany;
use crate::presentation::model::{IsCavas, IsImage, IsManifest, IsSequence};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum ManifestType {
    #[serde(rename = "sc:Manifest")]
    Manifest,
    #[serde(rename = "sc:Collection")]
    Collection,
    #[serde(rename = "sc:painting")]
    Painting,
    #[serde(rename = "sc:Sequence")]
    Sequence,
    #[serde(rename = "sc:Canvas")]
    Canvas,
    #[serde(rename = "sc:Range")]
    Range,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ViewingHint {
    Individuals,
    Paged,
    Continuous,
    MultiPart,
    NonPaged,
    Top,
    FacingPages,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Sequence {
    #[serde(rename = "@type")]
    type_: ManifestType,
    pub(crate) label: Option<OneTypeOrMany<String>>,
    pub(crate) viewing_direction: Option<ViewingDirection>,
    pub(crate) viewing_hint: Option<OneTypeOrMany<ViewingHint>>,
    pub(crate) canvases: Vec<Canvas>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Canvas {
    #[serde(rename = "@type")]
    type_: ManifestType,
    pub(crate) label: OneTypeOrMany<String>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) images: Vec<Image>,
    pub(crate) thumbnail: Option<OneTypeOrMany<UriLink>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
    #[serde(rename = "@id")]
    id: Option<String>,
    #[serde(rename = "@type")]
    type_: String,
    pub(crate) motivation: Option<ManifestType>,
    pub(crate) resource: ImageResource,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageResource {
    pub(crate) service: Service,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Service {
    #[serde(rename = "@context")]
    pub(crate) context: String,
    #[serde(rename = "@id")]
    pub(crate) id: String,
    pub(crate) profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum UriLink {
    StringType(String),
    IdType {
        #[serde(rename = "@id")]
        id: String,
        width: Option<u32>,
        height: Option<u32>,
    },
}

impl UriLink {
    pub(crate) fn id(&self) -> &str {
        match self {
            UriLink::StringType(v) => v,
            UriLink::IdType {
                id,
                height: _,
                width: _,
            } => id,
        }
    }

    pub(crate) fn width(&self) -> Option<u32> {
        match self {
            UriLink::StringType(_) => None,
            UriLink::IdType {
                id: _,
                height: _,
                width,
            } => width.to_owned(),
        }
    }

    pub(crate) fn height(&self) -> Option<u32> {
        match self {
            UriLink::StringType(_) => None,
            UriLink::IdType {
                id: _,
                height,
                width: _,
            } => height.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    #[serde(rename = "@context")]
    pub(crate) context: Context,
    #[serde(rename = "@type")]
    pub(crate) type_: ManifestType,
    #[serde(rename = "@id")]
    pub(crate) id: String,
    pub(crate) attribution: OneTypeOrMany<String>,
    pub(crate) label: String,
    pub(crate) license: Option<OneTypeOrMany<UriLink>>,
    pub(crate) logo: Option<OneTypeOrMany<UriLink>>,
    pub(crate) description: Option<OneTypeOrMany<String>>,
    pub(crate) service: Option<Service>,
    pub(crate) see_also: Option<OneTypeOrMany<UriLink>>,
    pub(crate) within: Option<OneTypeOrMany<String>>,
    pub(crate) sequences: Vec<Sequence>,
}

impl IsManifest for Manifest {
    fn get_title(&self) -> &str {
        &self.label
    }

    fn get_attribution(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(self.attribution.iter().map(Cow::from))
    }

    fn get_description(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.description {
            Box::new(content.iter().map(Cow::from))
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_license(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.license {
            Box::new(content.iter().map(|y| Cow::from(y.id())))
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_logo(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.logo {
            Box::new(content.iter().map(|y| Cow::from(y.id())))
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_> {
        Box::new(self.sequences.iter().map(|b| b as &dyn IsSequence))
    }

    fn get_sequence(&self, index: usize) -> Result<&dyn IsSequence, IiifError> {
        self.sequences
            .get(index)
            .map(|x| x as &dyn IsSequence)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "sequence not found at pos '{}'",
                index
            )))
    }
}

impl IsSequence for Sequence {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.label {
            Box::new(content.iter().map(Cow::from))
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCavas> + '_> {
        Box::new(self.canvases.iter().map(|b| b as &dyn IsCavas))
    }

    fn get_canvas(&self, index: usize) -> Result<&dyn IsCavas, IiifError> {
        self.canvases
            .get(index)
            .map(|x| x as &dyn IsCavas)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "canvas not found at pos '{}'",
                index
            )))
    }
}

impl IsCavas for Canvas {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(self.label.iter().map(Cow::from))
    }

    fn get_thumbnail(&self) -> Cow<'_, str> {
        // Some thumbnails are too large. Make sure that we know the size.
        // Or we will need to peek at the size of the remote image.
        if let Some(content) = &self.thumbnail
            && let Some(url_link) = content.iter().next()
            && url_link.width().is_some_and(|x| x <= 256)
            && url_link.height().is_some_and(|x| x <= 256)
        {
            Cow::from(url_link.id())
        } else if let Some(image) = self.images.first() {
            let canvas_thumbnail = format!("{}/full/,64/0/default.jpg", image.get_service());

            Cow::from(canvas_thumbnail)
        } else {
            Cow::from("")
        }
    }

    // fn get_images(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsImage> + '_> {
    //     Box::new(self.images.iter().map(|b| b as &dyn IsImage))
    // }

    fn get_image(&self, index: usize) -> Result<&dyn IsImage, IiifError> {
        self.images
            .get(index)
            .map(|x| x as &dyn IsImage)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing image at pos '{}'",
                index
            )))
    }
}

impl IsImage for Image {
    // fn get_height(&self) -> u32 {
    //     self.resource.height
    // }
    // fn get_width(&self) -> u32 {
    //     self.resource.width
    // }
    fn get_service(&self) -> Cow<'_, str> {
        Cow::from(&self.resource.service.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_2() {
    //     let url = "https://purl.stanford.edu/sr294cr5852/iiif/manifest";
    //     let info_json = ureq::get(url)
    //         .call()
    //         .unwrap()
    //         .body_mut()
    //         .read_to_string()
    //         .unwrap();

    //     println!("parse start");
    //     let now = Instant::now();
    //     let value: PresentationInfo = serde_json::de::from_str(&info_json).unwrap();
    //     let elapsed_time = now.elapsed();
    //     println!("parse done {}s", elapsed_time.as_secs_f32());

    //     // println!("{:?}", value);
    // }

    #[test]
    fn test_standard_json() {
        let json = r#"{
          "@context":"http://iiif.io/api/presentation/2/context.json",
          "@type":"sc:Manifest",
          "@id":"http://www.example.org/iiif/book1/manifest",
          "label":"Book 1",
          "metadata": [
            {"label":"Author", "value":"Anne Author"},
            {"label":"Published", "value": [
                {"@value": "Paris, circa 1400", "@language":"en"},
                {"@value": "Paris, environ 14eme siecle", "@language":"fr"}
                ]
            }
          ],
          "description":"A longer description of this example book. It should give some real information.",
          "license":"http://www.example.org/license.html",
          "attribution":"Provided by Example Organization",
          "service": {
            "@context": "http://example.org/ns/jsonld/context.json",
            "@id": "http://example.org/service/example",
            "profile": "http://example.org/docs/example-service.html"
          },
          "seeAlso":
            {
              "@id": "http://www.example.org/library/catalog/book1.marc",
              "format": "application/marc"
            },
          "within":"http://www.example.org/collections/books/",

          "sequences" : [
              {
                "@id":"http://www.example.org/iiif/book1/sequence/normal",
                "@type":"sc:Sequence",
                "label":"Current Page Order",
                "viewingDirection":"left-to-right",
                "viewingHint":"paged",
                "canvases": [
                  {
                    "@id":"http://www.example.org/iiif/book1/canvas/p1",
                    "@type":"sc:Canvas",
                    "label":"p. 1",
                    "height":1000,
                    "width":750,
                    "images": [
                      {
                        "@type":"oa:Annotation",
                        "motivation":"sc:painting",
                        "resource":{
                            "@id":"http://www.example.org/iiif/book1/res/page1.jpg",
                            "@type":"dctypes:Image",
                            "format":"image/jpeg",
                            "service": {
                                "@context": "http://iiif.io/api/image/2/context.json",
                                "@id": "http://www.example.org/images/book1-page1",
                                "profile":"http://iiif.io/api/image/2/level1.json"
                            },
                            "height":2000,
                            "width":1500
                        },
                        "on":"http://www.example.org/iiif/book1/canvas/p1"
                      }
                    ],
                    "otherContent": [
                      {
                        "@id":"http://www.example.org/iiif/book1/list/p1",
                        "@type":"sc:AnnotationList"
                      }
                    ]
                },
                  {
                    "@id":"http://www.example.org/iiif/book1/canvas/p2",
                    "@type":"sc:Canvas",
                    "label":"p. 2",
                    "height":1000,
                    "width":750,
                    "images": [
                      {
                        "@type":"oa:Annotation",
                        "motivation":"sc:painting",
                        "resource":{
                            "@id":"http://www.example.org/images/book1-page2/full/1500,2000/0/default.jpg",
                            "@type":"dctypes:Image",
                            "format":"image/jpeg",
                            "height":2000,
                            "width":1500,
                            "service": {
                                "@context": "http://iiif.io/api/image/2/context.json",
                                "@id": "http://www.example.org/images/book1-page2",
                                "profile":"http://iiif.io/api/image/2/level1.json",
                                "height":8000,
                                "width":6000,
                                "tiles" : [{"width": 512, "scaleFactors": [1,2,4,8,16]}]
                            }
                        },
                        "on":"http://www.example.org/iiif/book1/canvas/p2"
                      }
                    ],
                    "otherContent": [
                      {
                        "@id":"http://www.example.org/iiif/book1/list/p2",
                        "@type":"sc:AnnotationList"
                      }
                    ]
                  },
                  {
                    "@id":"http://www.example.org/iiif/book1/canvas/p3",
                    "@type":"sc:Canvas",
                    "label":"p. 3",
                    "height":1000,
                    "width":750,
                    "images": [
                      {
                        "@type":"oa:Annotation",
                        "motivation":"sc:painting",
                        "resource":{
                            "@id":"http://www.example.org/iiif/book1/res/page3.jpg",
                            "@type":"dctypes:Image",
                            "format":"image/jpeg",
                            "service": {
                                "@context": "http://iiif.io/api/image/2/context.json",
                                "@id": "http://www.example.org/images/book1-page3",
                                "profile":"http://iiif.io/api/image/2/level1.json"
                  },
                            "height":2000,
                            "width":1500
                        },
                        "on":"http://www.example.org/iiif/book1/canvas/p3"
                      }
                    ],
                    "otherContent": [
                      {
                        "@id":"http://www.example.org/iiif/book1/list/p3",
                        "@type":"sc:AnnotationList"
                      }
                    ]
                  }
                ]
              }
            ],
          "structures": [
            {
              "@id": "http://www.example.org/iiif/book1/range/r1",
                "@type":"sc:Range",
                "label":"Introduction",
                "canvases": [
                  "http://www.example.org/iiif/book1/canvas/p1",
                  "http://www.example.org/iiif/book1/canvas/p2",
                  "http://www.example.org/iiif/book1/canvas/p3#xywh=0,0,750,300"
                ]
            }
          ]
        }"#;

        let presentation_info: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(presentation_info.context, Context::Version2);
        assert_eq!(presentation_info.type_, ManifestType::Manifest);
        assert_eq!(
            presentation_info.id,
            "http://www.example.org/iiif/book1/manifest"
        );
        let description: Vec<_> = presentation_info.description.unwrap().into_iter().collect();
        assert_eq!(
            description,
            vec![
                "A longer description of this example book. It should give some real information."
            ]
        );
        assert_eq!(presentation_info.label, "Book 1");
        let license: Vec<_> = presentation_info
            .license
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|x| x.id())
            .collect();
        assert_eq!(license, vec!["http://www.example.org/license.html"]);
        assert!(presentation_info.logo.is_none());
        let attribution: Vec<_> = presentation_info.attribution.into_iter().collect();
        assert_eq!(attribution, vec!["Provided by Example Organization"]);

        let see_also: Vec<_> = presentation_info.see_also.unwrap().into_iter().collect();

        assert_eq!(
            see_also[0].id(),
            "http://www.example.org/library/catalog/book1.marc"
        );

        let within: Vec<_> = presentation_info.within.unwrap().into_iter().collect();
        assert_eq!(within, vec!["http://www.example.org/collections/books/"]);

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.type_, ManifestType::Sequence);

        let label: Vec<_> = seq.label.as_ref().unwrap().into_iter().collect();
        assert_eq!(label, vec!["Current Page Order"]);

        let viewing_direction = seq.viewing_direction.as_ref().unwrap();
        assert_eq!(*viewing_direction, ViewingDirection::LeftToRight);

        let viewing_hint: Vec<_> = seq.viewing_hint.as_ref().unwrap().iter().collect();
        assert_eq!(viewing_hint, vec![&ViewingHint::Paged]);

        assert_eq!(seq.canvases.len(), 3);

        for (index, canvas) in seq.canvases.iter().enumerate() {
            let num = index + 1;

            assert_eq!(canvas.type_, ManifestType::Canvas);
            assert_eq!(canvas.height, 1000);
            assert_eq!(canvas.width, 750);
            let label: Vec<_> = canvas.label.iter().collect();
            assert_eq!(label, vec![&format!("p. {num}")]);
            assert!(canvas.thumbnail.is_none());

            assert_eq!(canvas.images.len(), 1);
            let image = &canvas.images[0];

            assert!(image.id.is_none());
            assert_eq!(image.type_, "oa:Annotation");

            assert_eq!(*image.motivation.as_ref().unwrap(), ManifestType::Painting);
            let resource = &image.resource;

            let service = &resource.service;
            assert_eq!(
                service.id,
                format!("http://www.example.org/images/book1-page{num}")
            );
            assert_eq!(service.context, "http://iiif.io/api/image/2/context.json");
            assert_eq!(service.profile, "http://iiif.io/api/image/2/level1.json");
        }
    }

    #[test]
    fn test_ham_json() {
        let json = r#"
            {
                "@context": "http://iiif.io/api/presentation/2/context.json",
                "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378",
                "@type": "sc:Manifest",
                "attribution": "Provided by Harvard University",
                "label": "Harvard University, Harvard Art Museums, INV204583",
                "license": "https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright",
                "logo": "https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg",
                "sequences": [{
                    "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378/sequence/normal.json",
                    "@type": "sc:Sequence",
                    "canvases": [{
                        "@id": "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json",
                        "@type": "sc:Canvas",
                        "height": 833,
                        "images": [{
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
                        }],
                        "label": "Harvard University, Harvard Art Museums, INV204583",
                        "thumbnail": {
                            "@id": "https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg",
                            "@type": "dctypes:Image"
                        },
                        "width": 1024
                    }],
                    "label": "Harvard University, Harvard Art Museums, INV204583",
                    "startCanvas": "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json",
                    "viewingHint": "individuals"
                }]
            }"#;

        let presentation_info: Manifest = serde_json::from_str(&json).unwrap();

        assert_eq!(presentation_info.context, Context::Version2);
        assert_eq!(presentation_info.type_, ManifestType::Manifest);
        assert_eq!(
            presentation_info.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378"
        );
        assert!(presentation_info.description.is_none());
        assert_eq!(
            presentation_info.label,
            "Harvard University, Harvard Art Museums, INV204583"
        );
        let license: Vec<_> = presentation_info
            .license
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.id())
            .collect();
        assert_eq!(
            license,
            vec!["https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright"]
        );
        let logo: Vec<_> = presentation_info
            .logo
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.id())
            .collect();
        assert_eq!(
            logo,
            vec!["https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg"]
        );
        let attribution: Vec<_> = presentation_info.attribution.iter().collect();
        assert_eq!(attribution, vec!["Provided by Harvard University"]);

        assert!(presentation_info.see_also.is_none());
        assert!(presentation_info.within.is_none());

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.type_, ManifestType::Sequence);

        let label: Vec<_> = seq.label.as_ref().unwrap().iter().collect();
        assert_eq!(
            label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        assert!(seq.viewing_direction.is_none());

        let viewing_hint: Vec<_> = seq.viewing_hint.as_ref().unwrap().iter().collect();
        assert_eq!(viewing_hint, vec![&ViewingHint::Individuals]);

        assert_eq!(seq.canvases.len(), 1);

        let canvas = &seq.canvases[0];

        assert_eq!(canvas.type_, ManifestType::Canvas);
        assert_eq!(canvas.height, 833);
        assert_eq!(canvas.width, 1024);
        let label: Vec<_> = canvas.label.iter().collect();
        assert_eq!(
            label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );
        let thumbnail = canvas.thumbnail.as_ref().unwrap().iter().next().unwrap();
        assert_eq!(
            thumbnail.id(),
            "https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg"
        );
        // assert_eq!(thumbnail.type_(), Some("dctypes:Image"));

        assert_eq!(canvas.images.len(), 1);
        let image = &canvas.images[0];

        assert_eq!(
            image.id.as_ref().unwrap(),
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/annotation/anno-11927378.json"
        );
        assert_eq!(image.type_, "oa:Annotation");
        assert_eq!(*image.motivation.as_ref().unwrap(), ManifestType::Painting);
        let resource = &image.resource;

        let service = &resource.service;
        assert_eq!(service.id, "https://ids.lib.harvard.edu/ids/iiif/11927378");
        assert_eq!(service.context, "http://iiif.io/api/image/2/context.json");
        assert_eq!(service.profile, "http://iiif.io/api/image/2/level2.json");
    }
}
