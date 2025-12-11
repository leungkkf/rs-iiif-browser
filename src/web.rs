use crate::{
    UserNotification,
    app::app_state::{AppState, DownloadState, ImageDownloadInfo, ManifestDownloadInfo},
    presentation::{manifest::Manifest, ui::EguiUiState},
    rendering::{model_image::ModelImage, tiled_image::TiledImage},
};
use bevy::{
    prelude::{Commands, Entity, MessageWriter, Query, ResMut, Result, Single, With},
    window::RequestRedraw,
};
use std::sync::{Arc, Mutex};

/// Start to fetch the URL and handle state transition.
fn load<T: Send + 'static>(url: &str, download_state: Arc<Mutex<DownloadState<T>>>, info: T) {
    let request = ehttp::Request::get(url);
    let url = url.to_string();

    // In progress now.
    *download_state.lock().unwrap() = DownloadState::InProgress {
        url: url.to_string(),
    };

    ehttp::fetch(request, move |result| {
        // Lock the mutex to make sure consistent state transition.
        let mut download_state_mutex = download_state
            .lock()
            .expect("should be able to lock the presentation download state mutex");

        // Check if the response corresponds to the latest request. If not, we will skip it.
        if let DownloadState::InProgress {
            url: in_progress_url,
        } = &(*download_state_mutex)
            && in_progress_url == &url
        {
            match result {
                Ok(response) => match String::from_utf8(response.bytes) {
                    Ok(json) => {
                        *download_state_mutex = DownloadState::Done { json, info };
                    }
                    Err(e) => {
                        *download_state_mutex = DownloadState::Error {
                            url,
                            msg: e.to_string(),
                        };
                    }
                },
                Err(e) => {
                    *download_state_mutex = DownloadState::Error { url, msg: e };
                }
            }
        }
    });
}

/// Begin loading the IIIF presentation from remote URL.
pub(crate) fn load_presentation(app_state: &mut ResMut<AppState>, presentation_url: &str) {
    load(
        presentation_url,
        Arc::clone(&app_state.manifest_json_download_state),
        ManifestDownloadInfo {
            url: presentation_url.to_string(),
        },
    );
}

/// Load presentation system to handle the status of JSON fetch.
pub(crate) fn load_presentation_system(
    mut commands: Commands,
    mut app_state: ResMut<AppState>,
    mut egui_ui_state: ResMut<EguiUiState>,
    presentation_query: Query<(Entity, &Manifest)>,
    mut redraw_request_writer: MessageWriter<'_, RequestRedraw>,
    mut messages: MessageWriter<UserNotification>,
    model_image_query: Query<Entity, With<ModelImage>>,
) -> Result {
    let download_state = Arc::clone(&app_state.manifest_json_download_state);
    let mut download_state_mutex = download_state
        .lock()
        .expect("should be able to lock the presentation download state mutex");

    match &(*download_state_mutex) {
        DownloadState::Done { info, json } => {
            match Manifest::try_from_json(json) {
                Ok(presentation) => {
                    app_state.presentation_url = info.url.to_string();

                    for (presentation_entity, _) in presentation_query {
                        commands.entity(presentation_entity).despawn();
                    }

                    app_state.canvas_index = 0;
                    egui_ui_state.presentation_url = app_state.presentation_url.to_string();
                    egui_ui_state.canvas_index =
                        (app_state.canvas_index.saturating_add(1)).to_string();

                    match load_canvas(
                        &mut commands,
                        &presentation,
                        &mut app_state,
                        0,
                        &model_image_query,
                    ) {
                        Ok(_) => {
                            commands.spawn(presentation);
                            egui_ui_state.open_left_panel = true;
                        }
                        Err(e) => {
                            messages.write(UserNotification(format!(
                                "failed to load first canvas of manifest '{}'.\n{:?}",
                                app_state.presentation_url, e
                            )));
                        }
                    }
                }
                Err(e) => {
                    messages.write(UserNotification(format!(
                        "failed to processing manifest from '{}'.\n{:?}",
                        info.url, e
                    )));
                }
            };

            *download_state_mutex = DownloadState::None;
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::InProgress { .. } => {
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::Error { url, msg } => {
            messages.write(UserNotification(format!(
                "failed to load manifest from '{}'.\n{}",
                url, msg
            )));
            *download_state_mutex = DownloadState::None;
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::None => {}
    }

    Ok(())
}

/// Begin loading the IIIF image from remote URL.
pub(crate) fn load_canvas(
    commands: &mut Commands,
    presentation: &Manifest,
    app_state: &mut ResMut<AppState>,
    canvas_index: usize,
    model_image_query: &Query<Entity, With<ModelImage>>,
) -> Result {
    let canvas = presentation
        .model()
        .get_sequence(0)?
        .get_canvas(canvas_index)?;

    let image = canvas.get_image(0)?;

    if image.get_type() == "Model" {
        for image_entity in model_image_query {
            commands.entity(image_entity).despawn();
        }

        commands.spawn(ModelImage::new(&image.get_id()));
    } else {
        let iiif_endpoint = &image.get_service();
        let image_url = TiledImage::get_image_info_url(iiif_endpoint);

        load(
            &image_url,
            Arc::clone(&app_state.image_json_download_state),
            ImageDownloadInfo {
                iiif_endpoint: iiif_endpoint.to_string(),
                canvas_index,
            },
        );
    }

    Ok(())
}

/// Load image system to handle the status of JSON fetch.
#[allow(clippy::too_many_arguments)]
pub(crate) fn load_canvas_system(
    _presentation: Single<&Manifest>,
    mut commands: Commands,
    tiled_image_query: Query<Entity, With<TiledImage>>,
    mut app_state: ResMut<AppState>,
    mut egui_ui_state: ResMut<EguiUiState>,
    mut redraw_request_writer: MessageWriter<'_, RequestRedraw>,
    mut messages: MessageWriter<UserNotification>,
) -> Result {
    let download_state = Arc::clone(&app_state.image_json_download_state);
    let mut download_state_mutex = download_state.lock().expect("msg");

    match &(*download_state_mutex) {
        DownloadState::Done { json, info } => {
            match TiledImage::try_from_json(json, &info.iiif_endpoint) {
                Ok(image) => {
                    app_state.canvas_index = info.canvas_index;

                    for image_entity in tiled_image_query {
                        commands.entity(image_entity).despawn();
                    }
                    commands.spawn(image);

                    egui_ui_state.canvas_index =
                        (app_state.canvas_index.saturating_add(1)).to_string();
                }
                Err(e) => {
                    messages.write(UserNotification(format!(
                        "failed to parse image JSON from '{}'.\n{:?}",
                        info.iiif_endpoint, e
                    )));
                }
            }

            *download_state_mutex = DownloadState::None;
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::InProgress { .. } => {
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::Error { url, msg } => {
            messages.write(UserNotification(format!(
                "failed to load image from '{}'.\n{}",
                url, msg
            )));
            *download_state_mutex = DownloadState::None;
            redraw_request_writer.write(RequestRedraw);
        }
        DownloadState::None => {}
    }

    Ok(())
}
