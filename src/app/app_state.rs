use bevy::prelude::Resource;
use std::sync::{Arc, Mutex};

pub(crate) struct ManifestDownloadInfo {
    pub(crate) url: String,
}

pub(crate) struct ImageDownloadInfo {
    pub(crate) iiif_endpoint: String,
    pub(crate) canvas_index: usize,
}

pub(crate) enum DownloadState<T> {
    None,
    InProgress { url: String },
    Done { json: String, info: T },
    Error { url: String, msg: String },
}

#[derive(Resource)]
pub(crate) struct AppState {
    /// Current image scale level.
    pub(crate) level: usize,
    /// Current presentation manifest URL.
    pub(crate) presentation_url: String,
    /// Current canvas index.
    pub(crate) canvas_index: usize,
    pub(crate) manifest_json_download_state: Arc<Mutex<DownloadState<ManifestDownloadInfo>>>,
    pub(crate) image_json_download_state: Arc<Mutex<DownloadState<ImageDownloadInfo>>>,
}

impl AppState {
    fn new(
        level: usize,
        presentation_url: String,
        canvas_index: usize,
        manifest_json_download_state: Arc<Mutex<DownloadState<ManifestDownloadInfo>>>,
        image_json_download_state: Arc<Mutex<DownloadState<ImageDownloadInfo>>>,
    ) -> Self {
        Self {
            level,
            presentation_url,
            canvas_index,
            manifest_json_download_state,
            image_json_download_state,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(
            0,
            "".to_string(),
            0,
            Arc::new(Mutex::new(DownloadState::None)),
            Arc::new(Mutex::new(DownloadState::None)),
        )
    }
}
