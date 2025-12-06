use thiserror::Error;

pub(crate) mod image;
pub(crate) mod image_v2;
pub(crate) mod image_v3;
pub(crate) mod manifest;
pub(crate) mod manifest_v2;
pub(crate) mod manifest_v3;
pub(crate) mod one_or_many;

#[derive(Error, Debug)]
pub enum IiifError {
    #[error("channel error")]
    Channel(#[from] std::sync::mpsc::RecvError),

    #[error("channel error")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("serde_json deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("IIIF missing info '{0}'")]
    IiifMissingInfo(String),

    #[error("IIIF format error '{0}'")]
    IiifFormatError(String),

    // #[error("IIIF unsupported error {0}")]
    // IiifUnsupportedError(String),
    #[error("std io error")]
    IiifStdIoError(#[from] std::io::Error),

    #[error("IIIF parse int error {0}")]
    IiifParseIntError(#[from] std::num::ParseIntError),
}
