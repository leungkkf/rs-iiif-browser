use bevy::log::warn;
use std::{sync::mpsc::channel, time::Duration};
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
    #[error("web error '{0}'")]
    Web(String),

    #[error("channel error")]
    Channel(#[from] std::sync::mpsc::RecvTimeoutError),

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

/// Fetch json from URL.
fn fetch_json(url: &str) -> Result<String, IiifError> {
    let request = ehttp::Request::get(url);
    let (sx, rx) = channel();

    ehttp::fetch(request, move |x| {
        if let Err(e) = sx.send(x) {
            warn!("failed to send back web result '{e:?}'");
        }
    });

    let response = rx
        .recv_timeout(Duration::from_secs(30))?
        .map_err(|e| IiifError::Web(e))?;

    Ok(String::from_utf8(response.bytes)?)
}
