use serde::{Deserialize, Serialize};

use crate::iiif::manifest::{Context, Language, ViewingDirection};
use crate::iiif::one_or_many::{OneOrMany, OneTypeOrMany};
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
/// Presentation language map/pair.
pub(crate) struct LanguageValuePair {
    #[serde(rename = "@language")]
    language: Language,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum LabelValue {
    StringValue(String),
    LanguageTable(LanguageValuePair),
}

#[derive(Debug, Serialize, Deserialize)]
/// Label value map/pair, either one string or a map.
pub(crate) struct LabelValuePair {
    label: String,
    value: OneOrMany<String, LabelValue>,
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
    #[serde(rename = "@id")]
    id: String,
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
    #[serde(rename = "@id")]
    id: String,
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
    pub(crate) on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageResource {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    type_: String,
    pub(crate) format: String,
    pub(crate) service: Service,
    pub(crate) height: u32,
    pub(crate) width: u32,
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
#[serde(rename_all = "camelCase")]
pub(crate) struct Structure {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    type_: ManifestType,
    label: OneTypeOrMany<String>,
    canvases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum UriLink {
    StringType(String),
    IdType {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@type")]
        type_: Option<String>,
        format: Option<String>,
    },
}

impl UriLink {
    pub(crate) fn id(&self) -> &str {
        match self {
            UriLink::StringType(v) => v,
            UriLink::IdType {
                id,
                type_: _,
                format: _,
            } => id,
        }
    }

    pub(crate) fn format(&self) -> Option<&str> {
        match self {
            UriLink::StringType(_) => None,
            UriLink::IdType {
                id: _,
                type_: _,
                format,
            } => format.as_ref().map(|x| x.as_str()),
        }
    }

    pub(crate) fn type_(&self) -> Option<&str> {
        match self {
            UriLink::StringType(_) => None,
            UriLink::IdType {
                id: _,
                type_,
                format: _,
            } => type_.as_ref().map(|x| x.as_str()),
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
    pub(crate) metadata: Option<Vec<LabelValuePair>>,
    pub(crate) license: Option<OneTypeOrMany<UriLink>>,
    pub(crate) logo: Option<OneTypeOrMany<UriLink>>,
    pub(crate) description: Option<OneTypeOrMany<String>>,
    pub(crate) service: Option<Service>,
    pub(crate) see_also: Option<OneTypeOrMany<UriLink>>,
    pub(crate) within: Option<OneTypeOrMany<String>>,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) structures: Option<Vec<Structure>>,
}

impl IsManifest for Manifest {
    fn get_title(&self) -> &str {
        &self.label
    }

    fn get_attribution(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.attribution.iter().map(|x| x.as_str()))
    }

    fn get_description(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        if let Some(content) = &self.description {
            Box::new(content.iter().map(|y| y.as_str()))
        } else {
            Box::new(std::iter::empty::<&str>())
        }
    }

    fn get_license(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        if let Some(content) = &self.license {
            Box::new(content.iter().map(|y| y.id()))
        } else {
            Box::new(std::iter::empty::<&str>())
        }
    }

    fn get_logo(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        if let Some(content) = &self.logo {
            Box::new(content.iter().map(|y| y.id()))
        } else {
            Box::new(std::iter::empty::<&str>())
        }
    }

    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_> {
        Box::new(self.sequences.iter().map(|b| b as &dyn IsSequence))
    }

    fn get_sequence(&self, index: usize) -> &dyn IsSequence {
        &self.sequences[index]
    }
}

impl IsSequence for Sequence {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        if let Some(content) = &self.label {
            Box::new(content.iter().map(|y| y.as_str()))
        } else {
            Box::new(std::iter::empty::<&str>())
        }
    }

    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCavas> + '_> {
        Box::new(self.canvases.iter().map(|b| b as &dyn IsCavas))
    }

    fn get_canvas(&self, index: usize) -> &dyn IsCavas {
        &self.canvases[index]
    }
}

impl IsCavas for Canvas {
    fn get_label(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        Box::new(self.label.iter().map(|y| y.as_str()))
    }
    fn get_thumbnail(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        if let Some(content) = &self.thumbnail {
            Box::new(content.iter().map(|y| y.id()))
        } else {
            Box::new(std::iter::empty::<&str>())
        }
    }
    fn get_images(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsImage> + '_> {
        Box::new(self.images.iter().map(|b| b as &dyn IsImage))
    }
    fn get_image(&self, index: usize) -> &dyn IsImage {
        &self.images[index]
    }
}

impl IsImage for Image {
    fn get_height(&self) -> u32 {
        self.resource.height
    }
    fn get_width(&self) -> u32 {
        self.resource.width
    }
    fn get_service(&self) -> &str {
        &self.resource.service.id
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

        // let metadata = presentation_info.metadata.as_ref().unwrap();

        // assert_eq!(metadata[0].label, "Author");
        // assert_eq!(metadata[0].value, OneOrMany::One("Anne Author".into()));
        // assert_eq!(metadata[1].label, "Published");
        // assert_eq!(
        //     metadata[1].value,
        //     OneOrMany::Many(vec![
        //         LanguageValuePair {
        //             language: Language::En,
        //             value: "Paris, circa 1400".into()
        //         },
        //         LanguageValuePair {
        //             language: Language::Fr,
        //             value: "Paris, environ 14eme siecle".into()
        //         }
        //     ])
        // );

        let see_also: Vec<_> = presentation_info.see_also.unwrap().into_iter().collect();

        assert_eq!(
            see_also[0].id(),
            "http://www.example.org/library/catalog/book1.marc"
        );
        assert_eq!(see_also[0].format().unwrap(), "application/marc");

        let structures = presentation_info.structures.as_ref().unwrap();
        assert_eq!(structures.len(), 1);
        assert_eq!(
            structures[0].id,
            "http://www.example.org/iiif/book1/range/r1"
        );
        assert_eq!(structures[0].type_, ManifestType::Range);
        let label: Vec<_> = (&structures[0].label).into_iter().collect();
        assert_eq!(label, vec!["Introduction"]);
        assert_eq!(
            structures[0].canvases,
            vec![
                "http://www.example.org/iiif/book1/canvas/p1",
                "http://www.example.org/iiif/book1/canvas/p2",
                "http://www.example.org/iiif/book1/canvas/p3#xywh=0,0,750,300"
            ]
        );

        let within: Vec<_> = presentation_info.within.unwrap().into_iter().collect();
        assert_eq!(within, vec!["http://www.example.org/collections/books/"]);

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.id, "http://www.example.org/iiif/book1/sequence/normal");
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

            assert_eq!(
                canvas.id,
                format!("http://www.example.org/iiif/book1/canvas/p{num}")
            );
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
            assert_eq!(
                *image.on.as_ref().unwrap(),
                format!("http://www.example.org/iiif/book1/canvas/p{num}")
            );
            let resource = &image.resource;
            if num == 2 {
                assert_eq!(
                    resource.id,
                    "http://www.example.org/images/book1-page2/full/1500,2000/0/default.jpg"
                );
            } else {
                assert_eq!(
                    resource.id,
                    format!("http://www.example.org/iiif/book1/res/page{num}.jpg")
                );
            }

            assert_eq!(resource.type_, "dctypes:Image");
            assert_eq!(resource.format, "image/jpeg");
            assert_eq!(resource.width, 1500);
            assert_eq!(resource.height, 2000);

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

        // assert!(presentation_info.metadata.is_none());
        assert!(presentation_info.see_also.is_none());
        assert!(presentation_info.structures.is_none());
        assert!(presentation_info.within.is_none());

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(
            seq.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/sequence/normal.json"
        );
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

        assert_eq!(
            canvas.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json"
        );
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
        assert_eq!(thumbnail.type_(), Some("dctypes:Image"));

        assert_eq!(canvas.images.len(), 1);
        let image = &canvas.images[0];

        assert_eq!(
            image.id.as_ref().unwrap(),
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/annotation/anno-11927378.json"
        );
        assert_eq!(image.type_, "oa:Annotation");
        assert_eq!(*image.motivation.as_ref().unwrap(), ManifestType::Painting);
        assert_eq!(
            image.on.as_ref().unwrap(),
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json"
        );
        let resource = &image.resource;
        assert_eq!(
            resource.id,
            "https://ids.lib.harvard.edu/ids/iiif/11927378/full/full/0/default.jpg"
        );
        assert_eq!(resource.type_, "dctypes:Image");
        assert_eq!(resource.format, "image/jpeg");
        assert_eq!(resource.width, 1024);
        assert_eq!(resource.height, 833);

        let service = &resource.service;
        assert_eq!(service.id, "https://ids.lib.harvard.edu/ids/iiif/11927378");
        assert_eq!(service.context, "http://iiif.io/api/image/2/context.json");
        assert_eq!(service.profile, "http://iiif.io/api/image/2/level2.json");
    }
}
