use crate::iiif::presentation::{Context, Language, ViewingDirection};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum PresentationType {
    Manifest,
    Collection,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Label(HashMap<Language, Vec<String>>);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LabelValue {
    label: Label,
    value: Label,
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
pub(crate) struct Service {
    id: String,
    #[serde(rename = "type")]
    service_type: String,
    profile: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Thumbnail {
    id: String,
    #[serde(rename = "type")]
    thumbnail_type: DataType,
    format: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    duration: Option<u32>,
    service: Option<Vec<Service>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Behaviour {
    AutoAdvance,
    NoAutoAdvance,
    Repeat,
    NoRepeat,
    Unordered,
    Individuals,
    Continuous,
    Paged,
    FacingPages,
    NonPaged,
    MultiPart,
    Ttogether,
    Sequence,
    ThumbnailNav,
    NoNav,
    Hidden,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HomePage {
    id: String,
    #[serde(rename = "type")]
    homepage_type: String,
    label: Label,
    format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SeeAlso {
    id: String,
    #[serde(rename = "type")]
    seealso_type: String,
    label: Option<Label>,
    format: Option<String>,
    profile: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Provider {
    id: String,
    #[serde(rename = "type")]
    provider_type: String,
    label: Label,
    homepage: Vec<HomePage>,
    logo: Vec<Thumbnail>,
    see_also: Vec<SeeAlso>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Rendering {
    id: String,
    #[serde(rename = "type")]
    rendering_type: String,
    label: Label,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Item {
    id: String,
    #[serde(rename = "type")]
    item_type: String,
    label: Option<Label>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PresentationInfo {
    #[serde(rename = "@context")]
    context: Context,
    #[serde(rename = "type")]
    presentation_type: PresentationType,
    label: Label,
    metadata: Vec<LabelValue>,
    summary: Label,
    thumbnail: Vec<Thumbnail>,
    viewing_direction: ViewingDirection,
    behavior: Vec<Behaviour>,
    nav_date: DateTime<Utc>,
    rights: String,
    required_statement: LabelValue,
    provider: Vec<Provider>,
    homepage: Vec<HomePage>,
    service: Vec<Service>,
    see_also: Vec<SeeAlso>,
    rendering: Vec<Rendering>,
    part_of: Vec<Item>,
    start: Item,
    items: Vec<Item>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
          "summary": { "en": [ "Book 1, written by Anne Author, published in Paris around 1400." ] },

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

          "rights": "http://creativecommons.org/licenses/by/4.0/",
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
              "label": { "none": [ "p. 1" ] }
            }
          ],

          "structures": [
            {
              "id": "https://example.org/iiif/book1/range/top",
              "type": "Range"
            }
          ],

          "annotations": [
            {
              "id": "https://example.org/iiif/book1/annotations/p1",
              "type": "AnnotationPage",
              "items": [
              ]
            }
          ]
        }
        "#;

        let presentation_info: PresentationInfo = serde_json::from_str(&json).unwrap();

        println!("{:?}", presentation_info);
    }
}
