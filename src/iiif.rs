use crate::rdf;
use thiserror::Error;

pub(crate) mod image;

#[derive(Error, Debug)]
pub enum IiifError {
    #[error("ureq error")]
    Web(#[from] ureq::Error),

    #[error("serde_json deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("IIIF missing info {0}")]
    IiifMissingInfo(String),

    #[error("IIIF format error {0}")]
    IiifFormatError(String),

    // #[error("IIIF unsupported error {0}")]
    // IiifUnsupportedError(String),
    #[error("std io error")]
    IiifStdIoError(#[from] std::io::Error),

    #[error("sophia rdf dataset error")]
    IiifRdfDatasetError(#[from] rdf::dataset_ext::RdfError),

    #[error("sophia invalid iri error")]
    IiifInvalidIri(#[from] sophia::iri::InvalidIri),

    #[error("IIIF parse int error {0}")]
    IiifParseIntError(#[from] std::num::ParseIntError),
}
