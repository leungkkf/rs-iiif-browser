use sophia::{
    api::{
        dataset::{CollectibleDataset, DResult},
        prelude::{Any, Dataset},
        quad::Quad,
        term::{SimpleTerm, Term, TermKind, matcher::TermMatcher},
    },
    jsonld::{JsonLdOptions, JsonLdParser, loader::HttpLoader},
};
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;
use tokio::runtime::Runtime;

#[derive(Error, Debug)]
pub enum RdfError {
    #[error("ureq error")]
    Web(#[from] ureq::Error),

    #[error("rdf database error '{0}'")]
    DatasetError(String),

    #[error("std io error")]
    IiifStdIoError(#[from] std::io::Error),

    #[error("parse error")]
    IiifParseError(String),

    #[error("no data '{0}'")]
    NoData(String),
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
                "failed to create dataset from json '{}'. '{}'",
                json, e
            ))
        })?;

        for quad in dataset.quads() {
            println!("{:?}", quad.unwrap().to_spog());
        }

        Ok(DatasetExt(dataset))
    }

    /// Try to get the objects (ony for iri and literal) as string matching the subject and the predicate.
    pub(crate) fn get_objects_as<SM: TermMatcher, PM: TermMatcher, S: FromStr>(
        &self,
        subject: SM,
        predicate: PM,
    ) -> Result<Vec<S>, RdfError>
    where
        S::Err: Debug,
    {
        let mut output = Vec::new();

        for quad in self.0.quads_matching(subject, predicate, Any, Any) {
            let object = quad
                .map_err(|e| RdfError::DatasetError(format!("{:?}", e)))?
                .to_o();

            let content = match object.kind() {
                TermKind::Iri => object.iri().map(|x| S::from_str(x.as_str())),
                TermKind::Literal => object.lexical_form().map(|x| S::from_str(x.as_ref())),
                _ => {
                    return Err(RdfError::DatasetError(format!(
                        "unexpected term kind '{:?}' when converting",
                        object.kind()
                    )));
                }
            };

            if let Some(text) = content {
                output.push(text.map_err(|e| RdfError::IiifParseError(format!("{:?}", e)))?);
            }
        }

        Ok(output)
    }

    /// Helper iterator for the objects matching the subject and predicate.
    pub(crate) fn objects_iter<'a, SM: TermMatcher + 'a, PM: TermMatcher + 'a>(
        &'a self,
        subject: SM,
        predicate: PM,
    ) -> ObjectIterator<'a, T> {
        ObjectIterator::new(&self.0, subject, predicate)
    }

    /// Helper to get the first item. Note that it returns error if no data item is found.
    pub(crate) fn first_item<'a, SM: TermMatcher + 'a, PM: TermMatcher + 'a>(
        &'a self,
        subject: SM,
        predicate: PM,
    ) -> Result<SimpleTerm<'a>, RdfError> {
        self.objects_iter(subject, predicate)
            .next()
            .ok_or(RdfError::NoData("no data found".to_string()))
            .flatten()
    }
}

pub(crate) struct ObjectIterator<'a, T: Dataset + 'a> {
    inner: Box<dyn Iterator<Item = DResult<T, T::Quad<'a>>> + 'a>,
}

impl<'a, T: Dataset + 'a> ObjectIterator<'a, T> {
    fn new<SM: TermMatcher + 'a, PM: TermMatcher + 'a>(
        dataset: &'a T,
        subject: SM,
        predicate: PM,
    ) -> Self {
        let inner = Box::new(dataset.quads_matching(subject, predicate, Any, Any));

        Self { inner }
    }
}

impl<'a, T: Dataset + 'a> Iterator for ObjectIterator<'a, T> {
    type Item = Result<SimpleTerm<'a>, RdfError>;

    /// Get next object as Result<SimpleTerm, RdfError>.
    fn next(&mut self) -> Option<Self::Item> {
        let next_item = self.inner.next();

        // Check for sophia::api::ns::rdf::nil.
        if let Some(Ok(content)) = &next_item
            && content.matched_by(Any, Any, [sophia::api::ns::rdf::nil], Any)
        {
            return None;
        }

        next_item.map(|x| {
            x.map(|y| y.to_o().into_term())
                .map_err(|e| RdfError::DatasetError(format!("{}", e)))
        })
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
    fn test_objects_iter() {
        let url = "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee/info.json";
        let ns = Namespace::new_unchecked(
            "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee",
        );
        let subject = ns.get_unchecked("");
        let dataset = DatasetExt::<FastDataset>::try_from_url(url).unwrap();
        let mut width_u32 = Vec::new();

        for size_object in dataset.objects_iter([subject], [rdf::iiif_image2::hasSize]) {
            let width: Vec<u32> = dataset
                .get_objects_as([&size_object.unwrap()], [rdf::exif::width])
                .unwrap();
            width_u32.push(width[0]);
        }
        assert_eq!(width_u32, vec![220, 440, 880]);
    }

    #[test]
    fn test_get_objects_as() {
        let url = "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee/info.json";
        let ns = Namespace::new_unchecked(
            "https://nationalmuseumse.iiifhosting.com/iiif/6b67e82d21f66308380c15509e97bafa5e696618cff1137988ff80c1aa05e4ee",
        );
        let subject = ns.get_unchecked("");

        let dataset = DatasetExt::<FastDataset>::try_from_url(url).unwrap();
        let mut width_string = Vec::new();
        let mut width_u32 = Vec::new();
        let mut height_string = Vec::new();
        let mut height_u32 = Vec::new();

        for size_object in dataset.objects_iter([subject], [rdf::iiif_image2::hasSize]) {
            let size_object = size_object.unwrap();

            let width: Vec<String> = dataset
                .get_objects_as([&size_object], [rdf::exif::width])
                .unwrap();
            width_string.push(width[0].to_string());

            let width: Vec<u32> = dataset
                .get_objects_as([&size_object], [rdf::exif::width])
                .unwrap();
            width_u32.push(width[0]);

            let height: Vec<String> = dataset
                .get_objects_as([&size_object], [rdf::exif::height])
                .unwrap();
            height_string.push(height[0].to_string());

            let height: Vec<u32> = dataset
                .get_objects_as([&size_object], [rdf::exif::height])
                .unwrap();
            height_u32.push(height[0]);
        }

        assert_eq!(width_string, vec!["220", "440", "880"]);
        assert_eq!(width_u32, vec![220, 440, 880]);
        assert_eq!(height_string, vec!["180", "361", "723"]);
        assert_eq!(height_u32, vec![180, 361, 723]);
    }

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
        let attribution: Vec<String> = dataset
            .get_objects_as([subject], [rdf::iiif_present2::attributionLabel])
            .unwrap();
        let title: Vec<String> = dataset
            .get_objects_as([subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let logo: Vec<String> = dataset
            .get_objects_as([subject], [rdf::foaf::logo])
            .unwrap();
        let license: Vec<String> = dataset
            .get_objects_as([subject], [rdf::dcterms::rights])
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
        let attribution: Vec<String> = dataset
            .get_objects_as([subject], [rdf::iiif_present2::attributionLabel])
            .unwrap();
        let title: Vec<String> = dataset
            .get_objects_as([subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let logo: Vec<String> = dataset
            .get_objects_as([subject], [rdf::foaf::logo])
            .unwrap();
        let license: Vec<String> = dataset
            .get_objects_as([subject], [rdf::dcterms::rights])
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

        let sequence_node = dataset
            .first_item([subject], [rdf::iiif_present2::hasSequences])
            .unwrap();

        let sequence_subject = dataset.first_item([sequence_node], Any).unwrap();

        println!("{:?}", sequence_subject);

        let sequence_label: Vec<String> = dataset
            .get_objects_as([&sequence_subject], [sophia::api::ns::rdfs::label])
            .unwrap();

        assert_eq!(
            sequence_label,
            vec!["Harvard University, Harvard Art Museums, INV204583"]
        );

        let canvas_node = dataset
            .first_item([&sequence_subject], [rdf::iiif_present2::hasCanvases])
            .unwrap();

        let canvs_subject = dataset.first_item([&canvas_node], Any).unwrap();

        let canvas_width: Vec<String> = dataset
            .get_objects_as([&canvs_subject], [rdf::exif::width])
            .unwrap();
        let canvas_height: Vec<String> = dataset
            .get_objects_as([&canvs_subject], [rdf::exif::height])
            .unwrap();
        let canvas_label: Vec<String> = dataset
            .get_objects_as([&canvs_subject], [sophia::api::ns::rdfs::label])
            .unwrap();
        let canvas_thumbnail: Vec<String> = dataset
            .get_objects_as([&canvs_subject], [rdf::foaf::thumbnail])
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

        let image_annotation_node = dataset
            .first_item([canvs_subject], [rdf::iiif_present2::hasImageAnnotations])
            .unwrap();

        println!("image_annotations {:?}", image_annotation_node);

        let image_annotation_subject = dataset.first_item([&image_annotation_node], Any).unwrap();

        let image_bodies: Vec<String> = dataset
            .get_objects_as([&image_annotation_subject], [rdf::oa::hasBody])
            .unwrap();

        println!("image_bodies {:?}", image_bodies);

        let image_bodies_subject =
            NsTerm::new_unchecked(IriRef::new_unchecked(&image_bodies[0]), "");

        let image_service: Vec<String> = dataset
            .get_objects_as([image_bodies_subject], [rdf::svcs::has_service])
            .unwrap();

        assert_eq!(
            image_service,
            vec!["https://ids.lib.harvard.edu/ids/iiif/11927378"]
        );
    }
}
