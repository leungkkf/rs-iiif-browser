use crate::{
    iiif::{IiifError, manifest},
    presentation::model::IsManifest,
    rendering::{model_image::ModelImage, tiled_image::TiledImage},
};
use bevy::prelude::{
    Camera, Commands, Component, Entity, On, Query, Remove, Result, With, Without, info,
};
use bevy_egui::EguiContext;

#[derive(Component)]
/// Presentation manifest.
pub(crate) struct Manifest {
    inner: Box<dyn IsManifest>,
}

impl Manifest {
    fn new(inner: Box<dyn IsManifest>) -> Self {
        Self { inner }
    }

    /// Get the reference of the inner manifest.
    pub(crate) fn model(&self) -> &dyn IsManifest {
        self.inner.as_ref()
    }

    /// Try to create the manifest from JSON.
    pub(crate) fn try_from_json(json: &str) -> core::result::Result<Self, IiifError> {
        let iiif_manifest = manifest::Manifest::try_from_json(json)?;

        Ok(Manifest::from(iiif_manifest))
    }
}

impl From<Box<dyn IsManifest>> for Manifest {
    fn from(v: Box<dyn IsManifest>) -> Self {
        Self::new(v)
    }
}

/// Handler when the manifest is removed.
pub(crate) fn on_remove_manifest(
    remove: On<Remove, Manifest>,
    camera_query: Query<&mut Camera, Without<EguiContext>>,
    tiled_image_query: Query<Entity, With<TiledImage>>,
    model_image_query: Query<Entity, With<ModelImage>>,
    mut commands: Commands,
) -> Result {
    info!("Manifest removed (manifest). {:?}", remove.entity);

    // Set all cameras to inactive.
    for mut camera in camera_query {
        camera.is_active = false;
    }

    // Despawn all the images.
    for image_entity in tiled_image_query {
        commands.entity(image_entity).despawn();
    }
    for image_entity in model_image_query {
        commands.entity(image_entity).despawn();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::iiif;

    use super::*;

    #[test]
    fn test_from_iiif_manifest() {
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

        let iiif_manifest: iiif::manifest_v2::Manifest = serde_json::from_str(json).unwrap();

        let manifest = Manifest::new(Box::new(iiif_manifest));
        let language = crate::iiif::manifest::language::EN;

        assert_eq!(
            manifest
                .model()
                .get_attribution(language)
                .collect::<Vec<_>>(),
            vec!["Provided by Example Organization"]
        );
        assert_eq!(
            manifest.model().get_license().collect::<Vec<_>>(),
            vec!["http://www.example.org/license.html"]
        );
        assert_eq!(manifest.model().get_title(language), "Book 1");
        assert_eq!(
            manifest.model().get_logo().collect::<Vec<_>>(),
            Vec::<String>::new()
        );
        assert_eq!(
            manifest
                .model()
                .get_description(language)
                .collect::<Vec<_>>(),
            vec![
                "A longer description of this example book. It should give some real information."
            ]
        );

        assert_eq!(manifest.model().get_sequences().count(), 1);

        let seq = manifest.model().get_sequence(0).unwrap();

        assert_eq!(
            seq.get_label(language).collect::<Vec<_>>(),
            vec!["Current Page Order"]
        );

        assert_eq!(seq.get_canvases().count(), 3);

        let canvas = seq.get_canvas(0).unwrap();

        assert_eq!(canvas.get_label(language).collect::<Vec<_>>(), vec!["p. 1"]);
        assert_eq!(
            canvas.get_thumbnail(),
            "http://www.example.org/images/book1-page1/full/,64/0/default.jpg"
        );
        // assert_eq!(canvas.get_images().count(), 1);

        let image = canvas.get_image(0).unwrap();

        // assert_eq!(image.get_width(), 1500);
        // assert_eq!(image.get_height(), 2000);
        assert_eq!(
            image.get_service(),
            "http://www.example.org/images/book1-page1"
        );
    }
}
