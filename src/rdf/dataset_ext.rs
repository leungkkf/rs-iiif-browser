use sophia::{
    api::{
        dataset::CollectibleDataset,
        prelude::Any,
        quad::Quad,
        term::{Term, TermKind, matcher::TermMatcher},
    },
    jsonld::{JsonLdOptions, JsonLdParser, loader::HttpLoader},
};
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum RdfError {
    #[error("ureq error")]
    Web(#[from] ureq::Error),

    #[error("rdf database error {0}")]
    DatasetError(String),

    #[error("std io error")]
    IiifStdIoError(#[from] std::io::Error),
}

/// Wrapper around the RDF dataset to extend it with some functions.
pub(crate) struct DatasetExt<T>(T)
where
    T: CollectibleDataset;

impl<T> DatasetExt<T>
where
    T: CollectibleDataset,
{
    /// try to construct DatasetExt from a URL.
    pub(crate) fn try_from_url(url: &str) -> core::result::Result<DatasetExt<T>, RdfError> {
        let info_json = ureq::get(url).call()?.body_mut().read_to_string()?;

        Self::try_from_json(&info_json)
    }

    /// Try to construct DatasetExt from a Json.
    pub(crate) fn try_from_json(json: &str) -> core::result::Result<DatasetExt<T>, RdfError> {
        let options = JsonLdOptions::new().with_default_document_loader::<HttpLoader>();
        let parser = JsonLdParser::new_with_options(options);

        let quads = Runtime::new()?.block_on(async { parser.async_parse_str(json).await });

        let dataset = T::from_quad_source(quads).map_err(|e| {
            RdfError::DatasetError(format!(
                "failed to create dataset from json {}. {}",
                json, e
            ))
        })?;

        // for quad in dataset.quads() {
        //     println!("{:?}", quad.unwrap().to_spog());
        // }

        Ok(DatasetExt(dataset))
    }

    /// Try to get the objects (ony for iri and literal) as string matching the subject and the predicate.
    pub(crate) fn get_objects_as_string<SM: TermMatcher, PM: TermMatcher>(
        &self,
        subject: SM,
        predicate: PM,
    ) -> Result<Vec<String>, RdfError> {
        let mut output = Vec::new();

        for quad in self.0.quads_matching(subject, predicate, Any, Any) {
            let object = quad
                .map_err(|e| RdfError::DatasetError(format!("{}", e)))?
                .to_o();

            let content = match object.kind() {
                TermKind::Iri => object.iri().map(|x| x.to_string()),
                TermKind::Literal => object.lexical_form().map(|x| x.to_string()),
                _ => {
                    return Err(RdfError::DatasetError(format!(
                        "unexpected term kind {:?} when converting to string",
                        object.kind()
                    )));
                }
            };

            if let Some(text) = content {
                output.push(text);
            }
        }

        Ok(output)
    }

    /// Try to get the children objects (ony for iri and literal) as string matching the subject and the predicate.
    pub(crate) fn get_children_as_string<SM: TermMatcher, PM: TermMatcher>(
        &self,
        subject: SM,
        predicate: PM,
    ) -> Result<Vec<String>, RdfError> {
        let mut output = Vec::new();

        for result in self.0.quads_matching(subject, predicate, Any, Any) {
            if let Ok(quad) = result {
                let object = quad.to_o();

                for child in self.get_objects_as_string([object], Any)? {
                    if child != sophia::api::ns::rdf::nil.to_string() {
                        output.push(child);
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::rdf;
    use sophia::{
        api::ns::{Namespace, NsTerm},
        inmem::dataset::FastDataset,
        iri::IriRef,
    };

    #[test]
    fn test_from_json1() {
        let json = r#"{
          "@context": "http://iiif.io/api/presentation/2/context.json",
          "@id": "https://iiif.harvardartmuseums.org/manifests/object/323250",
          "@type": "sc:Manifest",
          "attribution": "Harvard Art Museums",
          "description": "",
          "label": "Newcomb Vase",
          "logo": {
            "@id": "https://ids.lib.harvard.edu/ids/iiif/437958013/full/!800,800/0/default.jpg",
            "service": {
              "@context": "http://iiif.io/api/image/2/context.json",
              "@id": "https://ids.lib.harvard.edu/ids/iiif/437958013",
              "profile": "http://iiif.io/api/image/2/level2.json"
            }
          },
          "metadata": [
            {
              "label": "Date",
              "value": "1901"
            },
            {
              "label": "Classification",
              "value": "Vessels"
            },
            {
              "label": "Credit Line",
              "value": "Harvard Art Museums/Fogg Museum, Gift of Dr. Ernest G. Stillman, Class of 1907, by exchange"
            },
            {
              "label": "Object Number",
              "value": "2008.4"
            },
            {
              "label": "People",
              "value": [
                "Artist: Marie De Hoa LeBlanc, American, 1874 - 1954"
              ]
            },
            {
              "label": "Medium",
              "value": "Ceramic"
            },
            {
              "label": "Dimensions",
              "value": "32.4 x 12.1 cm (12 3/4 x 4 3/4 in.)"
            }
          ],
          "rendering": {
            "@id": "https://www.harvardartmuseums.org/collections/object/323250",
            "format": "text/html",
            "label": "Full record view"
          },
          "sequences": [
            {
              "@id": "https://iiif.harvardartmuseums.org/manifests/object/323250/sequence/normal",
              "@type": "sc:Sequence",
              "canvases": [
                {
                  "@id": "https://iiif.harvardartmuseums.org/manifests/object/323250/canvas/canvas-20430287",
                  "@type": "sc:Canvas",
                  "height": 2550,
                  "images": [
                    {
                      "@id": "https://iiif.harvardartmuseums.org/manifests/object/323250/annotation/anno-20430287",
                      "@type": "oa:Annotation",
                      "motivation": "sc:painting",
                      "on": "https://iiif.harvardartmuseums.org/manifests/object/323250/canvas/canvas-20430287",
                      "resource": {
                        "@id": "https://ids.lib.harvard.edu/ids/iiif/20430287/full/full/0/default.jpg",
                        "@type": "dctypes:Image",
                        "format": "image/jpeg",
                        "height": 2550,
                        "service": {
                          "@context": "http://iiif.io/api/image/2/context.json",
                          "@id": "https://ids.lib.harvard.edu/ids/iiif/20430287",
                          "profile": "http://iiif.io/api/image/2/level2.json"
                        },
                        "width": 1074
                      }
                    }
                  ],
                  "label": "1",
                  "otherContent": [
                    {
                      "@id": "https://iiif.harvardartmuseums.org/manifests/object/323250/list/20430287",
                      "@type": "sc:AnnotationList"
                    }
                  ],
                  "width": 1074
                }
              ],
              "viewingHint": "individuals"
            }
          ],
          "thumbnail": {
            "@id": "https://ids.lib.harvard.edu/ids/iiif/20430287/full/!170,170/0/default.jpg",
            "service": {
              "@context": "http://iiif.io/api/image/2/context.json",
              "@id": "https://ids.lib.harvard.edu/ids/iiif/20430287",
              "profile": "http://iiif.io/api/image/2/level2.json"
            }
          },
          "within": "https://www.harvardartmuseums.org/collections"
        }"#;
        let ns =
            Namespace::new_unchecked("https://iiif.harvardartmuseums.org/manifests/object/323250");
        let subject = ns.get_unchecked("");

        let dataset = DatasetExt::<FastDataset>::try_from_json(json).unwrap();
        let attribution = dataset
            .get_objects_as_string([subject], [rdf::iiif_present2::attributionLabel])
            .unwrap();
        let title = dataset
            .get_objects_as_string([subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let logo = dataset
            .get_objects_as_string([subject], [rdf::foaf::logo])
            .unwrap();
        let license = dataset
            .get_objects_as_string([subject], [rdf::dcterms::rights])
            .unwrap();

        assert_eq!(attribution, vec!["Harvard Art Museums"]);
        assert_eq!(license, Vec::<String>::new());
        assert_eq!(title, vec!["Newcomb Vase"]);
        assert_eq!(
            logo,
            vec!["https://ids.lib.harvard.edu/ids/iiif/437958013/full/!800,800/0/default.jpg"]
        );
    }

    #[test]
    fn test_from_json2() {
        let url = "https://iiif.lib.harvard.edu/manifests/ids:11927378";
        let subject = NsTerm::new_unchecked(IriRef::new_unchecked(url), "");
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

        let dataset = DatasetExt::<FastDataset>::try_from_json(json).unwrap();
        let attribution = dataset
            .get_objects_as_string([subject], [rdf::iiif_present2::attributionLabel])
            .unwrap();
        let title = dataset
            .get_objects_as_string([subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let logo = dataset
            .get_objects_as_string([subject], [rdf::foaf::logo])
            .unwrap();
        let license = dataset
            .get_objects_as_string([subject], [rdf::dcterms::rights])
            .unwrap();

        assert_eq!(attribution, vec!["Provided by Harvard University"]);
        assert_eq!(
            license,
            vec!["https://nrs.harvard.edu/urn-3:HUL.eother:idscopyright"]
        );
        assert_eq!(
            title,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );
        assert_eq!(
            logo,
            vec!["https://iiif.lib.harvard.edu/static/manifests/harvard_logo.jpg"]
        );

        let sequences = dataset
            .get_children_as_string([subject], [rdf::iiif_present2::hasSequences])
            .unwrap();

        let sequence_subject = NsTerm::new_unchecked(IriRef::new_unchecked(&sequences[0]), "");

        let sequence_label = dataset
            .get_objects_as_string([sequence_subject], [sophia::api::ns::rdfs::label])
            .unwrap();

        assert_eq!(
            sequences,
            vec!["https://iiif.lib.harvard.edu/manifests/ids:11927378/sequence/normal.json"]
        );
        assert_eq!(
            sequence_label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        let canvases = dataset
            .get_children_as_string([sequence_subject], [rdf::iiif_present2::hasCanvases])
            .unwrap();

        let canvs_subject = NsTerm::new_unchecked(IriRef::new_unchecked(&canvases[0]), "");

        let canvas_width = dataset
            .get_objects_as_string([canvs_subject], [rdf::exif::width])
            .unwrap();
        let canvas_height = dataset
            .get_objects_as_string([canvs_subject], [rdf::exif::height])
            .unwrap();
        let canvas_label = dataset
            .get_objects_as_string([canvs_subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let canvas_thumbnail = dataset
            .get_objects_as_string([canvs_subject], [rdf::foaf::thumbnail])
            .unwrap();

        assert_eq!(canvas_width, vec!["1024"]);
        assert_eq!(canvas_height, vec!["833"]);
        assert_eq!(
            canvas_label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );
        assert_eq!(
            canvas_thumbnail,
            vec!["https://ids.lib.harvard.edu/ids/iiif/11927378/full/,150/0/default.jpg"]
        );

        let image_annotations = dataset
            .get_children_as_string([canvs_subject], [rdf::iiif_present2::hasImageAnnotations])
            .unwrap();

        println!("image_annotations {:?}", image_annotations);

        let image_annotation_subject =
            NsTerm::new_unchecked(IriRef::new_unchecked(&image_annotations[0]), "");

        let image_bodies = dataset
            .get_objects_as_string([image_annotation_subject], [rdf::oa::hasBody])
            .unwrap();

        println!("image_bodies {:?}", image_bodies);

        let image_bodies_subject =
            NsTerm::new_unchecked(IriRef::new_unchecked(&image_bodies[0]), "");

        let image_service = dataset
            .get_objects_as_string([image_bodies_subject], [rdf::svcs::has_service])
            .unwrap();

        assert_eq!(
            image_service,
            vec!["https://ids.lib.harvard.edu/ids/iiif/11927378"]
        );
    }
}
