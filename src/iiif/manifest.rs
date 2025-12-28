use crate::{
    iiif::{IiifError, manifest_v2, manifest_v3},
    presentation::model::IsManifest,
};
use bevy::prelude::debug;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub(crate) mod language {
    pub(crate) const NONE: &str = "none";
    pub(crate) const EN: &str = "en";
    pub(crate) const FR: &str = "fr";
    pub(crate) const DE: &str = "de";
    pub(crate) const ZH: &str = "zh";
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum Manifest {
    Version2(manifest_v2::Manifest),
    Version3(manifest_v3::Manifest),
}

impl Manifest {
    /// Build from a Json string.
    pub(crate) fn try_from_json(
        info_json: &str,
    ) -> core::result::Result<Box<dyn IsManifest>, IiifError> {
        let iiif_presentation_info: Manifest = serde_json::from_str(info_json)?;
        debug!("iiif_image_info {:?}", iiif_presentation_info);

        let output = match iiif_presentation_info {
            Manifest::Version2(v) => Box::new(v) as Box<dyn IsManifest>,
            Manifest::Version3(v) => Box::new(v) as Box<dyn IsManifest>,
        };

        // Check if we can get at least one sequence, one canvas and one image.
        output.get_sequence(0)?.get_canvas(0)?.get_image(0)?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json() {
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

        assert!(Manifest::try_from_json(json).is_ok());

        let _json = r#"{
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

        assert!(Manifest::try_from_json(json).is_ok());
    }
}
