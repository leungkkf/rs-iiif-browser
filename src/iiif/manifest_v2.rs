use crate::iiif::IiifError;
use crate::iiif::manifest::language;
use crate::iiif::one_or_many::OneTypeOrMany;
use crate::presentation::model::{IsCanvas, IsImage, IsManifest, IsSequence};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub(crate) struct LanguageValuePair {
    #[serde(rename = "@language")]
    language: String,
    #[serde(rename = "@value")]
    value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum LabelTextValue {
    SimpleText(String),
    LanguageValuePair(LanguageValuePair),
}

impl LabelTextValue {
    fn get(&self) -> Cow<'_, LanguageValuePair> {
        match self {
            LabelTextValue::SimpleText(v) => Cow::Owned(LanguageValuePair {
                language: language::NONE.to_string(),
                value: v.clone(),
            }),
            LabelTextValue::LanguageValuePair(v) => Cow::Borrowed(v),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LabelText(OneTypeOrMany<LabelTextValue>);

impl LabelText {
    fn get(&self, lang: &str) -> Vec<Cow<'_, str>> {
        let lvp: Vec<_> = self.0.iter().collect();

        if lvp.is_empty() {
            return Vec::new();
        }

        let output: Vec<_>;

        // If none of the values have a language associated with them, the client must display all of the values.
        if lvp.iter().all(|x| x.get().language == language::NONE) {
            output = lvp;
        }
        // If any of the values have a language associated with them,
        // the client must display all of the values associated with the language that best matches the language preference.
        else if lvp.iter().any(|x| x.get().language == lang) {
            output = lvp
                .into_iter()
                .filter(|x| x.get().language == lang)
                .collect();
        }
        // If all of the values have a language associated with them,
        // and none match the language preference,
        // the client must select a language and display all of the values associated with that language.
        else if !lvp.iter().any(|x| x.get().language == language::NONE) {
            let first_language = lvp
                .first()
                .expect("should have at least one item with a language at this point")
                .get()
                .language
                .clone();

            output = lvp
                .into_iter()
                .filter(|x| x.get().language == first_language)
                .collect();
        }
        // If some of the values have a language associated with them, but none match the language preference,
        // the client must display all of the values that do not have a language associated with them.
        else {
            output = lvp
                .into_iter()
                .filter(|x| x.get().language == language::NONE)
                .collect();
        }

        output
            .into_iter()
            .map(|x| match x.get() {
                Cow::Borrowed(v) => Cow::from(&v.value),
                Cow::Owned(v) => Cow::from(v.value),
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Sequence {
    #[serde(rename = "@type")]
    type_: ManifestType,
    pub(crate) label: Option<LabelText>,
    pub(crate) canvases: Vec<Canvas>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Canvas {
    #[serde(rename = "@type")]
    type_: ManifestType,
    pub(crate) label: LabelText,
    pub(crate) images: Vec<Image>,
    pub(crate) thumbnail: Option<OneTypeOrMany<UriLink>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Image {
    pub(crate) resource: ImageResource,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageResource {
    #[serde(rename = "@id")]
    pub(crate) id: String,
    #[serde(rename = "@type")]
    pub(crate) type_: String,
    pub(crate) service: Service,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Service {
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
    },
}

impl UriLink {
    pub(crate) fn id(&self) -> &str {
        match self {
            UriLink::StringType(v) => v,
            UriLink::IdType { id } => id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    #[serde(rename = "@type")]
    pub(crate) type_: ManifestType,
    #[serde(rename = "@id")]
    pub(crate) id: String,
    pub(crate) attribution: Option<LabelText>,
    pub(crate) label: LabelText,
    pub(crate) license: Option<OneTypeOrMany<UriLink>>,
    pub(crate) logo: Option<OneTypeOrMany<UriLink>>,
    pub(crate) description: Option<LabelText>,
    pub(crate) sequences: Vec<Sequence>,
}

impl IsManifest for Manifest {
    fn get_title(&self, language: &str) -> Cow<'_, str> {
        Cow::from(self.label.get(language).join("\n"))
    }

    fn get_attribution(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.attribution {
            Box::new(content.get(language).into_iter())
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_required_statements(&self, _: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(Vec::new().into_iter())
    }

    fn get_description(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.description {
            Box::new(content.get(language).into_iter())
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
    fn get_label(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(content) = &self.label {
            Box::new(content.get(language).into_iter())
        } else {
            Box::new(std::iter::empty::<Cow<str>>())
        }
    }

    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCanvas> + '_> {
        Box::new(self.canvases.iter().map(|b| b as &dyn IsCanvas))
    }

    fn get_canvas(&self, index: usize) -> Result<&dyn IsCanvas, IiifError> {
        self.canvases
            .get(index)
            .map(|x| x as &dyn IsCanvas)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "canvas not found at pos '{}'",
                index
            )))
    }
}

impl IsCanvas for Canvas {
    fn get_label(&self, language: &str) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(self.label.get(language).into_iter())
    }

    fn get_thumbnail(&self) -> Cow<'_, str> {
        // Some thumbnails are too large. Make sure that we know the size.
        // Or we will need to peek at the size of the remote image.
        if let Some(content) = &self.thumbnail
            && let Some(url_link) = content.iter().next()
            && !url_link.id().is_empty()
        {
            Cow::from(url_link.id())
        } else if let Some(image) = self.images.first()
            && let service = image.get_service()
            && !service.is_empty()
        {
            let canvas_thumbnail = format!("{}/full/,64/0/default.jpg", service);

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

    fn get_id(&self) -> Cow<'_, str> {
        Cow::from(&self.resource.id)
    }

    fn get_type(&self) -> Cow<'_, str> {
        Cow::from(&self.resource.type_)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_2() {
    //     let url = "https://digitalarchive.npm.gov.tw/Integrate/GetJson?cid=6742&dept=P";
    //     let info_json = ureq::get(url)
    //         .call()
    //         .unwrap()
    //         .body_mut()
    //         .read_to_string()
    //         .unwrap();

    //     let presentation_info: Manifest = serde_json::from_str(&info_json).unwrap();

    //     println!("{:?}", presentation_info);
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

        let presentation_info: Manifest = serde_json::from_str(json).unwrap();

        assert_eq!(presentation_info.type_, ManifestType::Manifest);
        assert_eq!(
            presentation_info.id,
            "http://www.example.org/iiif/book1/manifest"
        );
        let description: Vec<_> = presentation_info
            .description
            .as_ref()
            .unwrap()
            .get(language::EN)
            .into_iter()
            .collect();
        assert_eq!(
            description,
            vec![
                "A longer description of this example book. It should give some real information."
            ]
        );
        assert_eq!(
            presentation_info.label.get(language::EN).join("\n"),
            "Book 1"
        );
        let license: Vec<_> = presentation_info
            .license
            .as_ref()
            .unwrap()
            .into_iter()
            .map(|x| x.id())
            .collect();
        assert_eq!(license, vec!["http://www.example.org/license.html"]);
        assert!(presentation_info.logo.is_none());
        let attribution: Vec<_> = presentation_info
            .attribution
            .as_ref()
            .unwrap()
            .get(language::EN);
        assert_eq!(attribution, vec!["Provided by Example Organization"]);

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.type_, ManifestType::Sequence);

        let label: Vec<_> = seq
            .label
            .as_ref()
            .unwrap()
            .get(language::EN)
            .into_iter()
            .collect();
        assert_eq!(label, vec!["Current Page Order"]);

        assert_eq!(seq.canvases.len(), 3);

        for (index, canvas) in seq.canvases.iter().enumerate() {
            let num = index + 1;

            assert_eq!(canvas.type_, ManifestType::Canvas);
            let label: Vec<_> = canvas.label.get(language::EN).into_iter().collect();
            assert_eq!(label, vec![format!("p. {num}")]);
            assert!(canvas.thumbnail.is_none());

            assert_eq!(canvas.images.len(), 1);
            let image = &canvas.images[0];

            let resource = &image.resource;

            let service = &resource.service;
            assert_eq!(
                service.id,
                format!("http://www.example.org/images/book1-page{num}")
            );
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

        let presentation_info: Manifest = serde_json::from_str(json).unwrap();

        assert_eq!(presentation_info.type_, ManifestType::Manifest);
        assert_eq!(
            presentation_info.id,
            "https://iiif.lib.harvard.edu/manifests/ids:11927378"
        );
        assert!(presentation_info.description.is_none());
        assert_eq!(
            presentation_info.label.get(language::EN).join("\n"),
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
        let attribution: Vec<_> = presentation_info
            .attribution
            .as_ref()
            .unwrap()
            .get(language::EN);
        assert_eq!(attribution, vec!["Provided by Harvard University"]);

        assert_eq!(presentation_info.sequences.len(), 1);

        let seq = &presentation_info.sequences[0];
        assert_eq!(seq.type_, ManifestType::Sequence);

        let label: Vec<_> = seq
            .label
            .as_ref()
            .unwrap()
            .get(language::EN)
            .into_iter()
            .collect();
        assert_eq!(
            label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        assert_eq!(seq.canvases.len(), 1);

        let canvas = &seq.canvases[0];

        assert_eq!(canvas.type_, ManifestType::Canvas);
        let label: Vec<_> = canvas.label.get(language::EN).into_iter().collect();
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

        let resource = &image.resource;

        let service = &resource.service;
        assert_eq!(service.id, "https://ids.lib.harvard.edu/ids/iiif/11927378");
        assert_eq!(service.profile, "http://iiif.io/api/image/2/level2.json");
    }

    #[test]
    fn test_text_simple_one() {
        let text = LabelText(OneTypeOrMany::<LabelTextValue>::One(
            LabelTextValue::SimpleText("Simple".to_string()),
        ));

        assert_eq!(text.get(language::EN).join(" "), "Simple");
    }

    #[test]
    fn test_text_simple_many() {
        let text = LabelText(OneTypeOrMany::<LabelTextValue>::Many(vec![
            LabelTextValue::SimpleText("Simple".to_string()),
            LabelTextValue::SimpleText("Text".to_string()),
        ]));

        assert_eq!(text.get(language::EN).join(" "), "Simple Text");
        assert_eq!(text.get(language::DE).join(" "), "Simple Text");
    }

    #[test]
    fn test_text_language_value_one() {
        let text = LabelText(OneTypeOrMany::<LabelTextValue>::One(
            LabelTextValue::LanguageValuePair(LanguageValuePair {
                language: language::EN.to_string(),
                value: "Simple".to_string(),
            }),
        ));

        assert_eq!(text.get(language::EN).join(" "), "Simple");
        assert_eq!(text.get(language::DE).join(" "), "Simple");
    }

    #[test]
    fn test_text_language_value_many() {
        let text = LabelText(OneTypeOrMany::<LabelTextValue>::Many(vec![
            LabelTextValue::LanguageValuePair(LanguageValuePair {
                language: language::EN.to_string(),
                value: "Simple".to_string(),
            }),
            LabelTextValue::LanguageValuePair(LanguageValuePair {
                language: language::DE.to_string(),
                value: "De".to_string(),
            }),
        ]));

        assert_eq!(text.get(language::EN).join(" "), "Simple");
        assert_eq!(text.get(language::DE).join(" "), "De");
        assert_eq!(text.get(language::ZH).join(" "), "Simple");
    }

    #[test]
    fn test_text_language_value_many_with_no_language() {
        let text = LabelText(OneTypeOrMany::<LabelTextValue>::Many(vec![
            LabelTextValue::SimpleText("Default".to_string()),
            LabelTextValue::LanguageValuePair(LanguageValuePair {
                language: language::EN.to_string(),
                value: "Simple".to_string(),
            }),
            LabelTextValue::LanguageValuePair(LanguageValuePair {
                language: language::DE.to_string(),
                value: "De".to_string(),
            }),
        ]));

        assert_eq!(text.get(language::EN).join(" "), "Simple");
        assert_eq!(text.get(language::DE).join(" "), "De");
        assert_eq!(text.get(language::ZH).join(" "), "Default");
    }
}
