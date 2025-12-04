use crate::{
    iiif::{
        IiifError,
        manifest::language::{self},
        one_or_many::OneTypeOrMany,
    },
    presentation::model::{IsCanvas, IsImage, IsManifest, IsSequence},
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, vec};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) enum ManifestType {
    Manifest,
    Collection,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub(crate) enum LabelText {
    Map(HashMap<String, Vec<String>>),
    Text(OneTypeOrMany<String>),
}

impl LabelText {
    fn get(&self, lang: &str) -> Vec<&str> {
        match self {
            Self::Text(v) =>
            //If all of the values are associated with the none key, the client must display all of those values.
            {
                v.iter().map(|x| x.as_str()).collect::<Vec<_>>()
            }
            Self::Map(map) => {
                if map.is_empty() {
                    Vec::new()
                }
                //If all of the values are associated with the none key, the client must display all of those values.
                else if let Some(v) = map.get(language::NONE)
                    && map.len() == 1
                {
                    v.iter().map(|x| x.as_str()).collect()
                }
                // If any of the values have a language associated with them,
                // the client must display all of the values associated with the language that best matches the language preference.
                else if let Some(v) = map.get(lang) {
                    v.iter().map(|x| x.as_str()).collect()
                }
                // If all of the values have a language associated with them, and none match the language preference,
                // the client must select a language and display all of the values associated with that language
                else if map.get(language::NONE).is_none() {
                    let mut keys = map.keys().collect::<Vec<_>>();

                    // Hash map doesn't garuantee the order.
                    // Try to be a bit consistent at least and pick the first language in alphabetically order.
                    keys.sort();

                    let first_key = keys
                        .first()
                        .expect("should have at least one item with a language at this point");

                    map[*first_key].iter().map(|x| x.as_str()).collect()
                }
                // If some of the values have a language associated with them, but none match the language preference,
                // the client must display all of the values that do not have a language associated with them.
                else {
                    map[language::NONE].iter().map(|x| x.as_str()).collect()
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LabelValue {
    label: LabelText,
    value: LabelText,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DataType {
    Image,
    Dataset,
    Model,
    Sound,
    Text,
    Video,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Service3 {
    id: String,
    profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Service {
    Service2(crate::iiif::manifest_v2::Service),
    Service3(Service3),
}

impl Service {
    fn get_id(&self) -> &str {
        match self {
            Self::Service2(v) => &v.id,
            Self::Service3(v) => &v.id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Thumbnail {
    id: String,
    #[serde(rename = "type")]
    type_: DataType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Provider {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    label: LabelText,
    logo: OneTypeOrMany<Thumbnail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanvasItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    label: Option<LabelText>,
    thumbnail: Option<OneTypeOrMany<Thumbnail>>,
    items: Vec<AnnotationPageItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationPageItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    items: Vec<AnnotationItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationItem {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    body: AnnotationItemBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct AnnotationItemBody {
    id: String,
    #[serde(rename = "type")]
    type_: String,
    width: u32,
    height: u32,
    service: Vec<Service>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    #[serde(rename = "@context")]
    context: OneTypeOrMany<String>,
    #[serde(rename = "type")]
    manifest_type: ManifestType,
    label: LabelText,
    summary: OneTypeOrMany<LabelText>,
    rights: String,
    required_statement: Option<LabelValue>,
    provider: Option<Vec<Provider>>,
    items: Vec<CanvasItem>,
}

impl IsManifest for Manifest {
    fn get_title(&self) -> Cow<'_, str> {
        Cow::from(self.label.get(language::EN).join("\n"))
    }

    fn get_attribution(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        match &self.provider {
            None => Box::new(Vec::new().into_iter()),
            Some(v) => Box::new(
                v.iter()
                    .flat_map(|x| x.label.get(language::EN))
                    .map(Cow::from)
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }

    fn get_required_statements(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        match &self.required_statement {
            None => Box::new(Vec::new().into_iter()),
            Some(required_statement) => Box::new(
                required_statement
                    .label
                    .get(language::EN)
                    .iter()
                    .zip(required_statement.value.get(language::EN))
                    .map(|(label, value)| Cow::from(format!("{}: {}", label, value)))
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }

    fn get_description(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(
            self.summary
                .iter()
                .flat_map(|x| x.get(language::EN))
                .map(Cow::from)
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }

    fn get_license(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(vec![Cow::from(&self.rights)].into_iter())
    }

    fn get_logo(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        match &self.provider {
            None => Box::new(Vec::new().into_iter()),
            Some(v) => Box::new(
                v.iter()
                    .flat_map(|x| &x.logo)
                    .map(|x| Cow::from(&x.id))
                    .collect::<Vec<_>>()
                    .into_iter(),
            ),
        }
    }

    fn get_sequences(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsSequence> + '_> {
        Box::new(vec![self as &dyn IsSequence].into_iter())
    }

    fn get_sequence(&self, _: usize) -> Result<&dyn IsSequence, IiifError> {
        Ok(self as &dyn IsSequence)
    }
}

impl IsSequence for Manifest {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        Box::new(std::iter::empty::<Cow<str>>())
    }

    fn get_canvases(&self) -> Box<dyn ExactSizeIterator<Item = &dyn IsCanvas> + '_> {
        Box::new(self.items.iter().map(|b| b as &dyn IsCanvas))
    }

    fn get_canvas(&self, index: usize) -> Result<&dyn IsCanvas, IiifError> {
        self.items
            .get(index)
            .map(|x| x as &dyn IsCanvas)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "canvas not found at pos '{}'",
                index
            )))
    }
}

impl IsCanvas for CanvasItem {
    fn get_label(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_> {
        if let Some(label) = &self.label {
            Box::new(
                label
                    .get(language::EN)
                    .iter()
                    .map(|y| Cow::from(*y))
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        } else {
            Box::new(Vec::new().into_iter())
        }
    }

    fn get_thumbnail(&self) -> Cow<'_, str> {
        if let Some(thumbnail) = &self.thumbnail
            && let Some(thumbnail) = thumbnail.iter().next()
        {
            Cow::from(&thumbnail.id)
        } else if let Some(annotation_page) = self.items.first()
            && let Some(image) = annotation_page.items.first()
        {
            let canvas_thumbnail = format!("{}/full/,64/0/default.jpg", image.get_service());

            Cow::from(canvas_thumbnail)
        } else {
            Cow::from("")
        }
    }

    fn get_image(&self, index: usize) -> Result<&dyn IsImage, IiifError> {
        self.items
            .get(index)
            .map(|x| x.items.first())
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing annotation page at pos '{}'",
                index
            )))?
            .map(|x| x as &dyn IsImage)
            .ok_or(IiifError::IiifMissingInfo(format!(
                "missing annotation item at pos '{}'",
                index
            )))
    }
}

impl IsImage for AnnotationItem {
    fn get_service(&self) -> Cow<'_, str> {
        if let Some(service) = self.body.service.first() {
            Cow::from(service.get_id())
        } else {
            Cow::from("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iiif::manifest::language::{DE, EN, FR, NONE};

    // #[test]
    // fn test_url_json() {
    //     // let url = "https://bl.digirati.io/iiif/ark:/81055/vdc_100110232122.0x000001";
    //     let url = "https://iiif.rbge.org.uk/herb/iiif/E00008781/manifest";

    //     let json = ureq::get(url)
    //         .call()
    //         .unwrap()
    //         .body_mut()
    //         .read_to_string()
    //         .unwrap();

    //     let presentation_info: Manifest = serde_json::from_str(&json).unwrap();

    //     println!("{:?}", presentation_info);
    // }

    #[test]
    fn test_json() {
        let json = r#"{
          "@context": "http://iiif.io/api/presentation/3/context.json",
          "id": "https://example.org/iiif/book1/manifest",
          "type": "Manifest",
          "label": { "en": [ "Book 1" ] },
          "metadata": [
            {
              "label": { "en": [ "Author" ] },
              "value": { "none": [ "Anne Author" ] }
            },
            {
              "label": { "en": [ "Published" ] },
              "value": {
                "en": [ "Paris, circa 1400" ],
                "fr": [ "Paris, environ 1400" ]
              }
            },
            {
              "label": { "en": [ "Notes" ] },
              "value": {
                "en": [
                  "Text of note 1",
                  "Text of note 2"
                ]
              }
            },
            {
              "label": { "en": [ "Source" ] },
              "value": { "none": [ "<span>From: <a href=\"https://example.org/db/1.html\">Some Collection</a></span>" ] }
            }
          ],
          "summary": { "en": [ "Book 1, written be Anne Author, published in Paris around 1400." ] },

          "thumbnail": [
            {
              "id": "https://example.org/iiif/book1/page1/full/80,100/0/default.jpg",
              "type": "Image",
              "format": "image/jpeg",
              "service": [
                {
                  "id": "https://example.org/iiif/book1/page1",
                  "type": "ImageService3",
                  "profile": "level1"
                }
              ]
            }
          ],

          "viewingDirection": "right-to-left",
          "behavior": [ "paged" ],
          "navDate": "1856-01-01T00:00:00Z",

          "rights": "https://creativecommons.org/licenses/by/4.0/",
          "requiredStatement": {
            "label": { "en": [ "Attribution" ] },
            "value": { "en": [ "Provided by Example Organization" ] }
          },

          "provider": [
              {
                "id": "https://example.org/about",
                "type": "Agent",
                "label": { "en": [ "Example Organization" ] },
                "homepage": [
                  {
                    "id": "https://example.org/",
                    "type": "Text",
                    "label": { "en": [ "Example Organization Homepage" ] },
                    "format": "text/html"
                  }
                ],
                "logo": [
                  {
                    "id": "https://example.org/service/inst1/full/max/0/default.png",
                    "type": "Image",
                    "format": "image/png",
                    "service": [
                      {
                        "id": "https://example.org/service/inst1",
                        "type": "ImageService3",
                        "profile": "level2"
                      }
                    ]
                  }
                ],
                "seeAlso": [
                  {
                    "id": "https://data.example.org/about/us.jsonld",
                    "type": "Dataset",
                    "format": "application/ld+json",
                    "profile": "https://schema.org/"
                  }
                ]
              }
            ],
          "homepage": [
            {
              "id": "https://example.org/info/book1/",
              "type": "Text",
              "label": { "en": [ "Home page for Book 1" ] },
              "format": "text/html"
            }
          ],
          "service": [
            {
              "id": "https://example.org/service/example",
              "type": "ExampleExtensionService",
              "profile": "https://example.org/docs/example-service.html"
            }
          ],
          "seeAlso": [
            {
              "id": "https://example.org/library/catalog/book1.xml",
              "type": "Dataset",
              "format": "text/xml",
              "profile": "https://example.org/profiles/bibliographic"
            }
          ],
          "rendering": [
            {
              "id": "https://example.org/iiif/book1.pdf",
              "type": "Text",
              "label": { "en": [ "Download as PDF" ] },
              "format": "application/pdf"
            }
          ],
          "partOf": [
            {
              "id": "https://example.org/collections/books/",
              "type": "Collection"
            }
          ],
          "start": {
            "id": "https://example.org/iiif/book1/canvas/p2",
            "type": "Canvas"
          },

          "services": [
            {
              "@id": "https://example.org/iiif/auth/login",
              "@type": "AuthCookieService1",
              "profile": "http://iiif.io/api/auth/1/login",
              "label": "Login to Example Institution",
              "service": [
                {
                  "@id": "https://example.org/iiif/auth/token",
                  "@type": "AuthTokenService1",
                  "profile": "http://iiif.io/api/auth/1/token"
                }
              ]
            }
          ],

          "items": [
            {
              "id": "https://example.org/iiif/book1/canvas/p1",
              "type": "Canvas",
              "label": { "none": [ "p. 1" ] },
              "height": 1000,
              "width": 750,
              "items": [
                {
                  "id": "https://example.org/iiif/book1/page/p1/1",
                  "type": "AnnotationPage",
                  "items": [
                    {
                      "id": "https://example.org/iiif/book1/annotation/p0001-image",
                      "type": "Annotation",
                      "motivation": "painting",
                      "body": {
                        "id": "https://example.org/iiif/book1/page1/full/max/0/default.jpg",
                        "type": "Image",
                        "format": "image/jpeg",
                        "service": [
                          {
                            "id": "https://example.org/iiif/book1/page1",
                            "type": "ImageService3",
                            "profile": "level2",
                            "service": [
                              {
                                "@id": "https://example.org/iiif/auth/login",
                                "@type": "AuthCookieService1"
                              }
                            ]
                          }
                        ],
                        "height": 2000,
                        "width": 1500
                      },
                      "target": "https://example.org/iiif/book1/canvas/p1"
                    }
                  ]
                }
              ],
              "annotations": [
                {
                  "id": "https://example.org/iiif/book1/comments/p1/1",
                  "type": "AnnotationPage"
                }
              ]
            },
            {
              "id": "https://example.org/iiif/book1/canvas/p2",
              "type": "Canvas",
              "label": { "none": [ "p. 2" ] },
              "height": 1000,
              "width": 750,
              "items": [
                {
                  "id": "https://example.org/iiif/book1/page/p2/1",
                  "type": "AnnotationPage",
                  "items": [
                    {
                      "id": "https://example.org/iiif/book1/annotation/p0002-image",
                      "type": "Annotation",
                      "motivation": "painting",
                      "body": {
                        "id": "https://example.org/iiif/book1/page2/full/max/0/default.jpg",
                        "type": "Image",
                        "format": "image/jpeg",
                        "service": [
                          {
                            "id": "https://example.org/iiif/book1/page2",
                            "type": "ImageService3",
                            "profile": "level2"
                          }
                        ],
                        "height": 2000,
                        "width": 1500
                      },
                      "target": "https://example.org/iiif/book1/canvas/p2"
                    }
                  ]
                }
              ]
            }
          ],

          "structures": [
            {
              "id": "https://example.org/iiif/book1/range/r0",
              "type": "Range",
              "label": { "en": [ "Table of Contents" ] },
              "items": [
                {
                  "id": "https://example.org/iiif/book1/range/r1",
                  "type": "Range",
                  "label": { "en": [ "Introduction" ] },
                  "supplementary": {
                    "id": "https://example.org/iiif/book1/annocoll/introTexts",
                    "type": "AnnotationCollection"
                  },
                  "items": [
                    {
                      "id": "https://example.org/iiif/book1/canvas/p1",
                      "type": "Canvas"
                    },
                    {
                      "type": "SpecificResource",
                      "source": "https://example.org/iiif/book1/canvas/p2",
                      "selector": {
                        "type": "FragmentSelector",
                        "value": "xywh=0,0,750,300"
                      }
                    }
                  ]
                }
              ]
            }
          ],

          "annotations": [
            {
              "id": "https://example.org/iiif/book1/page/manifest/1",
              "type": "AnnotationPage",
              "items": [
                {
                  "id": "https://example.org/iiif/book1/page/manifest/a1",
                  "type": "Annotation",
                  "motivation": "commenting",
                  "body": {
                    "type": "TextualBody",
                    "language": "en",
                    "value": "I love this manifest!"
                  },
                  "target": "https://example.org/iiif/book1/manifest"
                }
              ]
            }
          ]
        }"#;

        let presentation_info: Manifest = serde_json::from_str(&json).unwrap();
        // println!("{:?}", presentation_info);

        assert_eq!(presentation_info.label.get(language::EN).join(""), "Book 1");

        assert_eq!(presentation_info.manifest_type, ManifestType::Manifest);

        assert_eq!(
            presentation_info.context.iter().collect::<Vec<_>>(),
            vec!["http://iiif.io/api/presentation/3/context.json"]
        );

        assert_eq!(
            presentation_info.rights,
            "https://creativecommons.org/licenses/by/4.0/"
        );

        assert_eq!(
            presentation_info
                .required_statement
                .as_ref()
                .unwrap()
                .label
                .get(language::EN)
                .join(""),
            "Attribution"
        );
        assert_eq!(
            presentation_info
                .required_statement
                .as_ref()
                .unwrap()
                .value
                .get(language::EN)
                .join(""),
            "Provided by Example Organization"
        );

        assert_eq!(
            presentation_info
                .summary
                .iter()
                .next()
                .unwrap()
                .get(language::EN),
            vec!["Book 1, written be Anne Author, published in Paris around 1400."]
        );

        let provider = presentation_info
            .provider
            .as_ref()
            .unwrap()
            .iter()
            .next()
            .unwrap();

        assert_eq!(provider.id, "https://example.org/about");
        assert_eq!(
            provider.label.get(language::EN),
            vec!["Example Organization"]
        );
        let canvas = &presentation_info.items[0];

        assert_eq!(canvas.id, "https://example.org/iiif/book1/canvas/p1");
        assert!(canvas.thumbnail.is_none());
        assert_eq!(
            canvas.label.as_ref().unwrap().get(language::EN),
            vec!["p. 1"]
        );
        let image = &(&canvas.items[0]).items[0];

        assert_eq!(
            image.id,
            "https://example.org/iiif/book1/annotation/p0001-image"
        );

        assert_eq!(
            image.body.id,
            "https://example.org/iiif/book1/page1/full/max/0/default.jpg"
        );

        assert_eq!(image.body.width, 1500);
        assert_eq!(image.body.height, 2000);

        assert_eq!(
            image.body.service[0].get_id(),
            "https://example.org/iiif/book1/page1"
        );
    }

    #[test]
    fn test_label_text_plain_text() {
        let label = LabelText::Text(OneTypeOrMany::<String>::One("text".to_string()));

        assert_eq!(label.get(EN), vec!["text"]);
        assert_eq!(label.get(DE), vec!["text"]);
    }

    #[test]
    fn test_label_text_plain_texts() {
        let label = LabelText::Text(OneTypeOrMany::<String>::Many(vec![
            "text".to_string(),
            "test".to_string(),
        ]));

        assert_eq!(label.get(EN), vec!["text", "test"]);
        assert_eq!(label.get(DE), vec!["text", "test"]);
    }

    #[test]
    fn test_label_text_map_with_none() {
        let label = LabelText::Map(HashMap::from([(
            NONE.to_string(),
            vec!["text".to_string()],
        )]));

        assert_eq!(label.get(DE), vec!["text"]);
        assert_eq!(label.get(EN), vec!["text"]);
    }

    #[test]
    fn test_label_text_map_with_en_de() {
        let label = LabelText::Map(HashMap::from([
            (EN.to_string(), vec!["text".to_string()]),
            (DE.to_string(), vec!["de".to_string()]),
        ]));

        assert_eq!(label.get(EN), vec!["text"]);
        assert_eq!(label.get(DE), vec!["de"]);
        assert_eq!(label.get(FR), vec!["de"]);
    }

    #[test]
    fn test_label_text_map_with_en_none() {
        let label = LabelText::Map(HashMap::from([
            (EN.to_string(), vec!["text".to_string()]),
            (DE.to_string(), vec!["de".to_string()]),
            (NONE.to_string(), vec!["none".to_string()]),
        ]));

        assert_eq!(label.get(EN), vec!["text"]);
        assert_eq!(label.get(DE), vec!["de"]);
        assert_eq!(label.get(FR), vec!["none"]);
        assert_eq!(label.get(NONE), vec!["none"]);
    }
}
