use crate::iiif::presentation::{Context, Language, OneOrMany, OneTypeOrMany, ViewingDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
/// Presentation type.
pub(crate) enum PresentationType {
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
/// Label value map/pair, either one string or a map.
pub(crate) struct LabelValuePair {
    label: String,
    value: OneOrMany<String, LanguageValuePair>,
}

#[derive(Debug, Serialize, Deserialize)]
/// Presentation service.
pub(crate) struct Service {
    #[serde(rename = "@context")]
    context: String,
    #[serde(rename = "@id")]
    id: String,
    profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// Presentation thumbnail.
pub(crate) struct Thumbnail {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    thumbnail_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
/// Presentation "see also".
pub(crate) struct SeeAlso {
    #[serde(rename = "@id")]
    id: String,
    format: Option<String>,
    profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
/// Presenation viewing hint.
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
/// Presentation sequence.
pub(crate) struct Sequence {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    sequence_type: PresentationType,
    label: OneTypeOrMany<String>,
    viewing_direction: Option<ViewingDirection>,
    viewing_hint: Option<OneTypeOrMany<ViewingHint>>,
    canvases: Vec<Canvas>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Presentation canvas.
pub(crate) struct Canvas {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    canvas_type: PresentationType,
    label: OneTypeOrMany<String>,
    width: u32,
    height: u32,
    images: Vec<Image>,
    thumbnail: Option<Thumbnail>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Presentation image.
pub(crate) struct Image {
    #[serde(rename = "@id")]
    id: Option<String>,
    #[serde(rename = "@type")]
    image_type: String,
    motivation: Option<PresentationType>,
    resource: Resource,
    on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Presentation resource.
pub(crate) struct Resource {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    resource_type: String,
    format: String,
    service: Service,
    height: u32,
    width: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Presentation structure.
pub(crate) struct Structure {
    #[serde(rename = "@id")]
    id: String,
    #[serde(rename = "@type")]
    structure_type: PresentationType,
    label: OneTypeOrMany<String>,
    canvases: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Presentation.
pub(crate) struct PresentationInfo {
    #[serde(rename = "@context")]
    context: Context,
    #[serde(rename = "@type")]
    presentation_type: PresentationType,
    #[serde(rename = "@id")]
    id: String,
    attribution: OneTypeOrMany<String>,
    label: String,
    metadata: Option<Vec<LabelValuePair>>,
    license: OneTypeOrMany<String>,
    logo: Option<OneTypeOrMany<String>>,
    description: Option<OneTypeOrMany<String>>,
    service: Option<Service>,
    see_also: Option<OneTypeOrMany<SeeAlso>>,
    within: Option<OneTypeOrMany<String>>,
    sequences: Vec<Sequence>,
    structures: Option<Vec<Structure>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iiif::presentation::Language;

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

        let presentation_info: PresentationInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(presentation_info.context, Context::Version2);
        assert_eq!(
            presentation_info.presentation_type,
            PresentationType::Manifest
        );
        assert_eq!(
            presentation_info.id,
            "http://www.example.org/iiif/book1/manifest"
        );
        let description: Vec<_> = presentation_info.description.as_ref().unwrap().into();
        assert_eq!(
            description,
            vec![
                "A longer description of this example book. It should give some real information."
            ]
        );
        assert_eq!(presentation_info.label, "Book 1");
        let license: Vec<_> = presentation_info.license.into();
        assert_eq!(license, vec!["http://www.example.org/license.html"]);
        assert!(presentation_info.logo.is_none());
        let attribution: Vec<_> = presentation_info.attribution.into();
        assert_eq!(attribution, vec!["Provided by Example Organization"]);

        let metadata = presentation_info.metadata.as_ref().unwrap();

        assert_eq!(metadata[0].label, "Author");
        assert_eq!(metadata[0].value, OneOrMany::One("Anne Author".into()));
        assert_eq!(metadata[1].label, "Published");
        assert_eq!(
            metadata[1].value,
            OneOrMany::Many(vec![
                LanguageValuePair {
                    language: Language::En,
                    value: "Paris, circa 1400".into()
                },
                LanguageValuePair {
                    language: Language::Fr,
                    value: "Paris, environ 14eme siecle".into()
                }
            ])
        );

        let see_also: Vec<&SeeAlso> = presentation_info.see_also.as_ref().unwrap().into();

        assert_eq!(
            see_also[0].id,
            "http://www.example.org/library/catalog/book1.marc"
        );
        assert_eq!(see_also[0].format.as_ref().unwrap(), "application/marc");

        let structures = presentation_info.structures.as_ref().unwrap();
        assert_eq!(structures.len(), 1);
        assert_eq!(
            structures[0].id,
            "http://www.example.org/iiif/book1/range/r1"
        );
        assert_eq!(structures[0].structure_type, PresentationType::Range);
        let label: Vec<_> = (&structures[0].label).into();
        assert_eq!(label, vec!["Introduction"]);
        assert_eq!(
            structures[0].canvases,
            vec![
                "http://www.example.org/iiif/book1/canvas/p1",
                "http://www.example.org/iiif/book1/canvas/p2",
                "http://www.example.org/iiif/book1/canvas/p3#xywh=0,0,750,300"
            ]
        );

        let within: Vec<_> = presentation_info.within.as_ref().unwrap().into();
        assert_eq!(within, vec!["http://www.example.org/collections/books/"]);

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.id, "http://www.example.org/iiif/book1/sequence/normal");
        assert_eq!(seq.sequence_type, PresentationType::Sequence);

        let label: Vec<_> = (&seq.label).into();
        assert_eq!(label, vec!["Current Page Order"]);

        let viewing_direction = seq.viewing_direction.as_ref().unwrap();
        assert_eq!(*viewing_direction, ViewingDirection::LeftToRight);

        let viewing_hint: Vec<_> = seq.viewing_hint.clone().unwrap().into();
        assert_eq!(viewing_hint, vec![ViewingHint::Paged]);

        assert_eq!(seq.canvases.len(), 3);

        for (index, canvas) in seq.canvases.iter().enumerate() {
            let num = index + 1;

            assert_eq!(
                canvas.id,
                format!("http://www.example.org/iiif/book1/canvas/p{num}")
            );
            assert_eq!(canvas.canvas_type, PresentationType::Canvas);
            assert_eq!(canvas.height, 1000);
            assert_eq!(canvas.width, 750);
            let label: Vec<_> = canvas.label.clone().into();
            assert_eq!(label, vec![format!("p. {num}")]);
            assert!(canvas.thumbnail.is_none());

            assert_eq!(canvas.images.len(), 1);
            let image = &canvas.images[0];

            assert!(image.id.is_none());
            assert_eq!(image.image_type, "oa:Annotation");

            assert_eq!(
                *image.motivation.as_ref().unwrap(),
                PresentationType::Painting
            );
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

            assert_eq!(resource.resource_type, "dctypes:Image");
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

        let presentation_info: PresentationInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(presentation_info.context, Context::Version2);
        assert_eq!(
            presentation_info.presentation_type,
            PresentationType::Manifest
        );
        assert_eq!(
            presentation_info.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378"
        );
        assert!(presentation_info.description.is_none());
        assert_eq!(
            presentation_info.label,
            "Harvard University, Harvard Art Museums, INV204583"
        );
        let license: Vec<_> = presentation_info.license.clone().into();
        assert_eq!(
            license,
            vec!["https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright"]
        );
        let logo: Vec<_> = presentation_info.logo.clone().unwrap().into();
        assert_eq!(
            logo,
            vec!["https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg"]
        );
        let attribution: Vec<_> = presentation_info.attribution.clone().into();
        assert_eq!(attribution, vec!["Provided by Harvard University"]);

        assert!(presentation_info.metadata.is_none());
        assert!(presentation_info.see_also.is_none());
        assert!(presentation_info.structures.is_none());
        assert!(presentation_info.within.is_none());

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(
            seq.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/sequence/normal.json"
        );
        assert_eq!(seq.sequence_type, PresentationType::Sequence);

        let label: Vec<_> = seq.label.clone().into();
        assert_eq!(
            label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        assert!(seq.viewing_direction.is_none());

        let viewing_hint: Vec<_> = seq.viewing_hint.clone().unwrap().into();
        assert_eq!(viewing_hint, vec![ViewingHint::Individuals]);

        assert_eq!(seq.canvases.len(), 1);

        let canvas = &seq.canvases[0];

        assert_eq!(
            canvas.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json"
        );
        assert_eq!(canvas.canvas_type, PresentationType::Canvas);
        assert_eq!(canvas.height, 833);
        assert_eq!(canvas.width, 1024);
        let label: Vec<_> = canvas.label.clone().into();
        assert_eq!(
            label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );
        let thumbnail = canvas.thumbnail.as_ref().unwrap();
        assert_eq!(
            thumbnail.id,
            "https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg"
        );
        assert_eq!(thumbnail.thumbnail_type, "dctypes:Image");

        assert_eq!(canvas.images.len(), 1);
        let image = &canvas.images[0];

        assert_eq!(
            image.id.as_ref().unwrap(),
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/annotation/anno-11927378.json"
        );
        assert_eq!(image.image_type, "oa:Annotation");
        assert_eq!(
            *image.motivation.as_ref().unwrap(),
            PresentationType::Painting
        );
        assert_eq!(
            image.on.as_ref().unwrap(),
            "https://iiif.lib.harvard.edu/manifests/ids:11927378/canvas/canvas-11927378.json"
        );
        let resource = &image.resource;
        assert_eq!(
            resource.id,
            "https://ids.lib.harvard.edu/ids/iiif/11927378/full/full/0/default.jpg"
        );
        assert_eq!(resource.resource_type, "dctypes:Image");
        assert_eq!(resource.format, "image/jpeg");
        assert_eq!(resource.width, 1024);
        assert_eq!(resource.height, 833);

        let service = &resource.service;
        assert_eq!(service.id, "https://ids.lib.harvard.edu/ids/iiif/11927378");
        assert_eq!(service.context, "http://iiif.io/api/image/2/context.json");
        assert_eq!(service.profile, "http://iiif.io/api/image/2/level2.json");
    }
}
