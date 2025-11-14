use thiserror::Error;

pub(crate) mod image;
pub(crate) mod one_or_many;
pub(crate) mod presentation;
pub(crate) mod presentation_v2;
pub(crate) mod presentation_v3;

#[derive(Error, Debug)]
pub enum IiifError {
    #[error("ureq error")]
    Web(#[from] ureq::Error),

    #[error("serde_json deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("IIIF missing info")]
    IiifMissingInfo(String),

    #[error("IIIF format error")]
    IiifFormatError(String),

    #[error("IIIF unsupported error")]
    IiifUnsupportedError(String),
}
