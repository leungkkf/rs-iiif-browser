use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::presentation::manifest::Manifest;
use crate::rendering::tiled_image::TiledImage;
use bevy::camera::Viewport;
use bevy::prelude::{
    Camera, Commands, Entity, MessageWriter, Query, Res, ResMut, Resource, Result, Single, UVec2,
    Window, With, Without, default,
};
use bevy::window::{PrimaryWindow, RequestRedraw};
use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Button, Color32, FontFamily, FontId, Sense, Widget, vec2};
use bevy_egui::{EguiContext, EguiContexts, egui};
use std::time::Duration;

#[derive(Resource)]
pub(crate) struct EguiUiState {
    pub(crate) current_sequence: usize,
    pub(crate) presentation_url: String,
    pub(crate) toasts: egui_notify::Toasts,
    pub(crate) open_left_panel: bool,
}

/// Set up egui.
pub(crate) fn setup(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Set up image loaders for the thumbnails.
    egui_extras::install_image_loaders(ctx);

    let toasts = egui_notify::Toasts::default();

    commands.insert_resource(EguiUiState {
        current_sequence: 0,
        presentation_url: "".to_string(),
        toasts,
        open_left_panel: false,
    });

    Ok(())
}

// let presentation = presentation::manifest::ManifestComponent::try_from_url(
//     "https://iiif.lib.harvard.edu/manifests/ids:11927378",
// )?;

// let presentation = presentation::manifest::Manifest::try_from_url(
//     "https://iiif.harvardartmuseums.org/manifests/object/21116",
// )?;

// let presentation = presentation::manifest::Manifest::try_from_url(
//     "https://iiif.harvardartmuseums.org/manifests/object/303419",
// )?;

// let presentation = presentation::manifest::Manifest::try_from_url(
//     "https://iiif.harvardartmuseums.org/manifests/object/279708",
// )?;

// let presentation = presentation::manifest::Manifest::try_from_url(
//     "https://iiif.harvardartmuseums.org/manifests/object/323250",
// )?;
//
// https://www.loc.gov/item/00007086/manifest.json
// https://www.vam.ac.uk/collections

#[allow(clippy::too_many_arguments)]
pub(crate) fn presentation_ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut egui_ui_state: ResMut<EguiUiState>,
    app_settings: Res<AppSettings>,
    mut app_state: ResMut<AppState>,
    presentation_query: Query<(Entity, &Manifest)>,
    tiled_image_query: Query<(Entity, &TiledImage)>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let mut top = egui::Panel::top("top_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if Button::new("☰")
                    .fill(Color32::from_black_alpha(0))
                    .ui(ui)
                    .clicked()
                {
                    egui_ui_state.open_left_panel = !egui_ui_state.open_left_panel;
                    redraw_request_writer.write(RequestRedraw);
                }

                // Add address bar.
                add_address_bar(
                    ui,
                    &mut commands,
                    &mut egui_ui_state,
                    &mut app_state,
                    presentation_query,
                    tiled_image_query,
                    &mut redraw_request_writer,
                );
            });

            ui.add_space(1.0);

            // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height(); // width is ignored, as the panel has a width of 100% of the screen
    // let mut top = 0.0;

    let mut left = if egui_ui_state.open_left_panel {
        egui::Panel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| -> Result {
                // No need to build panel if no presentation.
                let Some((_, presentation)) = presentation_query.iter().next() else {
                    return Ok(());
                };

                // Manifest title.
                add_text(
                    ui,
                    presentation.model().get_title(),
                    Some(Color32::WHITE),
                    2,
                );

                // Manifest description.
                let description = presentation.model().get_description().collect::<Vec<_>>();
                if !description.is_empty() {
                    add_text(ui, &description.join("\n"), None, 3);
                }

                // Manifest attribution and licence.
                let licence = presentation.model().get_license().collect::<Vec<_>>();
                let license = if !licence.is_empty() {
                    format!("(© {})", &licence.join(","))
                } else {
                    "".into()
                };
                let attribution = presentation
                    .model()
                    .get_attribution()
                    .collect::<Vec<_>>()
                    .join(",");

                if !license.is_empty() || !attribution.is_empty() {
                    add_text(ui, &format!("{} {}", attribution, license), None, 3);
                }

                // Manifest provider logo.
                for logo in presentation.model().get_logo() {
                    ui.add_space(6.0);
                    bevy_egui::egui::Image::new(logo).max_height(64.0).ui(ui);
                }
                ui.add_space(6.0);

                // Manifest sequence.
                egui::ComboBox::from_id_salt("Sequences")
                    .selected_text(
                        presentation
                            .model()
                            .get_sequence(egui_ui_state.current_sequence)?
                            .get_label()
                            .collect::<Vec<_>>()
                            .join(","),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap)
                    .show_ui(ui, |ui| {
                        for (index, seq) in presentation.model().get_sequences().enumerate() {
                            ui.selectable_value(
                                &mut egui_ui_state.current_sequence,
                                index,
                                seq.get_label().collect::<Vec<_>>().join(","),
                            );
                        }
                    });

                ui.separator();

                // Canvas thumbnails.
                add_canvas_thumbnails(
                    ui,
                    &mut commands,
                    &mut egui_ui_state,
                    app_settings,
                    tiled_image_query,
                    presentation,
                )?;

                // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
                Ok(())
            })
            .response
            .rect
            .width() // height is ignored, as the panel has a hight of 100% of the screen
    } else {
        0.0
    };
    // let mut right = egui::SidePanel::right("right_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Right resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .width(); // height is ignored, as the panel has a height of 100% of the screen
    let mut right = 0.0;

    // let mut bottom = egui::TopBottomPanel::bottom("bottom_panel")
    //     .resizable(true)
    //     .show(ctx, |ui| {
    //         ui.label("Bottom resizeable panel");
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .height(); // width is ignored, as the panel has a width of 100% of the screen
    let mut bottom = 0.0;

    // Scale from logical units to physical units.
    left *= window.scale_factor();
    right *= window.scale_factor();
    top *= window.scale_factor();
    bottom *= window.scale_factor();

    // -------------------------------------------------
    // |  left   |            top   ^^^^^^   |  right  |
    // |  panel  |           panel  height   |  panel  |
    // |         |                  vvvvvv   |         |
    // |         |---------------------------|         |
    // |         |                           |         |
    // |<-width->|          viewport         |<-width->|
    // |         |                           |         |
    // |         |---------------------------|         |
    // |         |          bottom   ^^^^^^  |         |
    // |         |          panel    height  |         |
    // |         |                   vvvvvv  |         |
    // -------------------------------------------------
    //
    // The upper left point of the viewport is the width of the left panel and the height of the
    // top panel
    //
    // The width of the viewport the width of the top/bottom panel
    // Alternative the width can be calculated as follow:
    // size.x = window width - left panel width - right panel width
    //
    // The height of the viewport is:
    // size.y = window height - top panel height - bottom panel height
    //
    // Therefore we use the alternative for the width, as we can callculate the Viewport as
    // following:

    let pos = UVec2::new(left as u32, top as u32);
    let size = UVec2::new(window.physical_width(), window.physical_height())
        - pos
        - UVec2::new(right as u32, bottom as u32);

    camera.viewport = Some(Viewport {
        physical_position: pos,
        physical_size: size,
        ..default()
    });

    egui_ui_state.toasts.show(ctx);

    Ok(())
}

/// Add the canvas thumbnail panel.
fn add_canvas_thumbnails(
    ui: &mut egui::Ui,
    commands: &mut Commands<'_, '_>,
    egui_ui_state: &mut ResMut<'_, EguiUiState>,
    app_settings: Res<'_, AppSettings>,
    tiled_image_query: Query<'_, '_, (Entity, &TiledImage)>,
    presentation: &Manifest,
) -> Result {
    let canvas_iter = presentation
        .model()
        .get_sequence(egui_ui_state.current_sequence)?
        .get_canvases();

    let thumbnail_size = app_settings.thumbnail_size;
    let text_style = egui::TextStyle::Body;
    let row_height = thumbnail_size + 3.0 * ui.text_style_height(&text_style);
    let column_width = thumbnail_size;

    let available_width = ui.available_width();
    let item_spacing = ui.spacing().item_spacing;
    let items_per_row = (available_width / (column_width + item_spacing.x))
        .round()
        .max(1.0) as usize;

    let visible_canvases: Vec<_> = canvas_iter.collect();

    egui::ScrollArea::vertical().auto_shrink(false).show_rows(
        ui,
        row_height,
        visible_canvases.len().div_ceil(items_per_row).max(1),
        |ui, row_range| {
            egui::Grid::new("my_grid")
                .min_col_width(column_width)
                .max_col_width(column_width)
                .min_row_height(row_height)
                .show(ui, |ui| -> Result {
                    let row_start = row_range.start;

                    for (row_index, _) in visible_canvases
                        .iter()
                        .skip(row_range.start * items_per_row)
                        .take(row_range.count() * items_per_row)
                        .step_by(items_per_row)
                        .enumerate()
                    {
                        for col_index in 0..items_per_row {
                            let canvas_index = (row_start + row_index) * items_per_row + col_index;

                            if canvas_index < visible_canvases.len() {
                                let canvas = visible_canvases[canvas_index];

                                if ui
                                    .vertical_centered(|ui| {
                                        let canvas_thumbnail = canvas.get_thumbnail();

                                        if !canvas_thumbnail.is_empty() {
                                            ui.add_sized(
                                                vec2(thumbnail_size, thumbnail_size),
                                                bevy_egui::egui::Image::new(canvas_thumbnail)
                                                    .max_size(vec2(thumbnail_size, thumbnail_size)),
                                            );
                                        }
                                        let label = format!(
                                            "({}) {}",
                                            canvas_index + 1,
                                            canvas.get_label().collect::<Vec<_>>().join(",")
                                        );
                                        add_text(ui, &label, None, 3);
                                    })
                                    .response
                                    .interact(Sense::CLICK)
                                    .clicked()
                                {
                                    let image_url = canvas.get_image(0)?.get_service().to_string();

                                    if let Ok(image) = TiledImage::try_from_url(&image_url) {
                                        for (image_entity, _) in tiled_image_query {
                                            commands.entity(image_entity).despawn();
                                        }
                                        commands.spawn(image);
                                    } else {
                                        let msg =
                                            format!("Unable to load image URL '{}'", image_url);

                                        egui_ui_state
                                            .toasts
                                            .warning(msg)
                                            .show_progress_bar(true)
                                            .duration(Duration::from_secs(5));
                                    }
                                }
                            }
                        }
                        ui.end_row();
                    }
                    Ok(())
                });
        },
    );

    Ok(())
}

/// Add the manifest URL address bar.
fn add_address_bar(
    ui: &mut egui::Ui,
    commands: &mut Commands<'_, '_>,
    egui_ui_state: &mut ResMut<'_, EguiUiState>,
    app_state: &mut ResMut<'_, AppState>,
    presentation_query: Query<'_, '_, (Entity, &Manifest)>,
    tiled_image_query: Query<'_, '_, (Entity, &TiledImage)>,
    redraw_request_writer: &mut MessageWriter<'_, RequestRedraw>,
) {
    ui.horizontal(|ui| {
        // ui.label(RichText::new("IIIF:").color(Color32::from_rgb(240, 240, 240)));
        if ui
            .add(
                egui::TextEdit::singleline(&mut egui_ui_state.presentation_url)
                    .desired_width(ui.available_width())
                    // .text_color(Color32::from_rgb(240, 240, 240))
                    .hint_text("IIIF Manifest URL"),
            )
            .on_hover_text(&egui_ui_state.presentation_url)
            .lost_focus()
            && egui_ui_state.presentation_url != app_state.presentation_url
        {
            let presentation_url = egui_ui_state.presentation_url.to_string();

            match crate::load_presentation(
                commands,
                app_state,
                egui_ui_state,
                &presentation_url,
                &presentation_query,
                &tiled_image_query,
            ) {
                Ok(_) => {
                    let msg = format!("Loaded manifest URL '{}'", presentation_url);

                    egui_ui_state
                        .toasts
                        .info(msg)
                        .show_progress_bar(true)
                        .duration(Duration::from_secs(5));
                    app_state.presentation_url = presentation_url;

                    redraw_request_writer.write(RequestRedraw);
                }
                Err(err) => {
                    let msg = format!(
                        "Unable to load manifest URL '{}'.\n Error: {:?}",
                        presentation_url, err
                    );

                    egui_ui_state
                        .toasts
                        .warning(msg)
                        .show_progress_bar(true)
                        .duration(Duration::from_secs(5));
                    egui_ui_state.presentation_url = app_state.presentation_url.to_string();
                }
            }
        }
    });
}

/// Add a multi-line wrapped text.
fn add_text(ui: &mut egui::Ui, text: &str, color: Option<Color32>, max_rows: usize) {
    let mut job = LayoutJob::simple_format(
        text.to_owned(),
        egui::TextFormat {
            font_id: FontId::new(12.0, FontFamily::Proportional),
            color: color.unwrap_or(Color32::GRAY),
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_rows,
        break_anywhere: true,
        ..Default::default()
    };

    ui.label(job);
}
