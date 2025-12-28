use crate::UserNotification;
use crate::app::app_settings::AppSettings;
use crate::app::app_state::AppState;
use crate::presentation::manifest::Manifest;
use crate::rendering::model_image::ModelImage;
use bevy::camera::Viewport;
use bevy::prelude::{
    Camera, Commands, Entity, MessageReader, MessageWriter, Query, Res, ResMut, Resource, Result,
    Single, UVec2, Window, With, Without, default,
};
use bevy::window::{PrimaryWindow, RequestRedraw};
use bevy_egui::egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Button, Color32, FontData, FontFamily, FontId, Sense, Widget, vec2};
use bevy_egui::{EguiContext, EguiContexts, egui};
use std::time::Duration;

#[derive(Resource)]
pub(crate) struct EguiUiState {
    pub(crate) current_sequence: usize,
    pub(crate) presentation_url: String,
    pub(crate) toasts: egui_notify::Toasts,
    pub(crate) open_left_panel: bool,
    pub(crate) canvas_index: String,
}

/// Set up egui.
pub(crate) fn setup(mut contexts: EguiContexts, mut commands: Commands) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Set up image loaders for the thumbnails.
    egui_extras::install_image_loaders(ctx);

    commands.insert_resource(EguiUiState {
        current_sequence: 0,
        presentation_url: "".to_string(),
        toasts: egui_notify::Toasts::default(),
        open_left_panel: false,
        canvas_index: "".to_string(),
    });

    // Add a CJK font.
    ctx.add_font(FontInsert::new(
        "NotoSansTC",
        FontData::from_static(crate::fonts::NOTOSANSTC_REGULAR),
        vec![InsertFontFamily {
            family: FontFamily::Proportional,
            priority: FontPriority::Highest,
        }],
    ));

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
//
// https://www.vam.ac.uk/collections
// https://digitalarchive.npm.gov.tw/Collection/Featured

#[allow(clippy::too_many_arguments)]
pub(crate) fn presentation_ui_system(
    mut contexts: EguiContexts,
    mut camera_query: Query<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut egui_ui_state: ResMut<EguiUiState>,
    app_settings: Res<AppSettings>,
    mut app_state: ResMut<AppState>,
    presentation_query: Query<(Entity, &Manifest)>,
    mut redraw_request_writer: MessageWriter<RequestRedraw>,
    mut messages: MessageReader<UserNotification>,
    mut commands: Commands,
    model_image_query: Query<Entity, With<ModelImage>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Display user notifications.
    for UserNotification(msg) in messages.read() {
        egui_ui_state
            .toasts
            .info(msg)
            .show_progress_bar(true)
            .duration(Duration::from_secs(5));
    }

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

                let num_canvases = presentation_query
                    .iter()
                    .next()
                    .and_then(|(_, manifest)| {
                        (*manifest)
                            .model()
                            .get_sequence(0)
                            .ok()
                            .map(|x| x.get_canvases())
                    })
                    .map(|x| x.len())
                    .unwrap_or_default();

                // Add address bar.
                add_address_bar(
                    ui,
                    &mut egui_ui_state,
                    &mut app_state,
                    ui.available_width() - if num_canvases > 1 { 85.0 } else { 0.0 },
                );

                if num_canvases > 1 {
                    // Add page controls.
                    add_page_controls(
                        &mut egui_ui_state,
                        &mut app_state,
                        presentation_query,
                        ui,
                        num_canvases,
                        &mut commands,
                        &model_image_query,
                    );
                }
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
                    &presentation.model().get_title(&app_settings.language),
                    Some(Color32::WHITE),
                    2,
                );

                // Manifest description.
                let description = presentation
                    .model()
                    .get_description(&app_settings.language)
                    .collect::<Vec<_>>();
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
                    .get_attribution(&app_settings.language)
                    .collect::<Vec<_>>()
                    .join(",");

                if !license.is_empty() || !attribution.is_empty() {
                    add_text(ui, &format!("{} {}", attribution, license), None, 3);
                }

                let required_statements = presentation
                    .model()
                    .get_required_statements(&app_settings.language)
                    .collect::<Vec<_>>()
                    .join(". ");

                if !required_statements.is_empty() {
                    add_text(ui, &required_statements, None, 3);
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
                            .get_label(&app_settings.language)
                            .collect::<Vec<_>>()
                            .join(","),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap)
                    .show_ui(ui, |ui| {
                        for (index, seq) in presentation.model().get_sequences().enumerate() {
                            ui.selectable_value(
                                &mut egui_ui_state.current_sequence,
                                index,
                                seq.get_label(&app_settings.language)
                                    .collect::<Vec<_>>()
                                    .join(","),
                            );
                        }
                    });

                ui.separator();

                // Canvas thumbnails.
                add_canvas_thumbnails(
                    ui,
                    &mut egui_ui_state,
                    app_settings,
                    &mut app_state,
                    presentation,
                    &mut commands,
                    &model_image_query,
                )?;

                // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
                Ok(())
            })
            .response
            .rect
            .width() // height is ignored, as the panel has a height of 100% of the screen
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

    for mut camera in camera_query.iter_mut() {
        camera.viewport = Some(Viewport {
            physical_position: pos,
            physical_size: size,
            ..default()
        });
    }

    egui_ui_state.toasts.show(ctx);

    Ok(())
}

/// Add controls to change pages.
#[allow(clippy::too_many_arguments)]
fn add_page_controls(
    egui_ui_state: &mut ResMut<'_, EguiUiState>,
    app_state: &mut ResMut<'_, AppState>,
    presentation_query: Query<'_, '_, (Entity, &Manifest)>,
    ui: &mut egui::Ui,
    num_canvases: usize,
    commands: &mut Commands,
    model_image_query: &Query<Entity, With<ModelImage>>,
) {
    ui.spacing_mut().item_spacing.x = 1.0;

    let mut new_canvas_index = app_state.canvas_index;

    if ui.button("<").clicked() {
        new_canvas_index = app_state.canvas_index.saturating_sub(1);
    }

    let egui_index = egui_ui_state.canvas_index.clone();
    let response = ui
        .add(egui::TextEdit::singleline(&mut egui_ui_state.canvas_index).desired_width(30.0))
        .on_hover_text(format!(
            "Page {}/{}",
            app_state.canvas_index.saturating_add(1),
            num_canvases
        ));

    if response.changed() && !egui_ui_state.canvas_index.is_empty() {
        if let Ok(index) = egui_ui_state.canvas_index.parse::<usize>()
            && index > 0
            && index <= num_canvases
        {
            egui_ui_state.canvas_index = index.to_string();
        } else {
            egui_ui_state.canvas_index = egui_index;
        }
    }

    if response.lost_focus() {
        new_canvas_index = egui_ui_state
            .canvas_index
            .parse::<usize>()
            .unwrap_or_default()
            .saturating_sub(1);
    }
    if ui.button(">").clicked() {
        new_canvas_index = (app_state.canvas_index.saturating_add(1)).min(num_canvases - 1);
    }

    if new_canvas_index != app_state.canvas_index {
        let (_, manifest) = presentation_query
            .iter()
            .next()
            .expect("should have a manifest due to previous check on the number of canvas > 1");

        if let Err(err) = crate::web::load_canvas(
            commands,
            manifest,
            app_state,
            new_canvas_index,
            model_image_query,
        ) {
            let msg = format!("Unable to load canvas.\n'{}'", err);

            egui_ui_state
                .toasts
                .warning(msg)
                .show_progress_bar(true)
                .duration(Duration::from_secs(5));
        }
    }
}

/// Add the canvas thumbnail panel.
#[allow(clippy::too_many_arguments)]
fn add_canvas_thumbnails(
    ui: &mut egui::Ui,
    egui_ui_state: &mut ResMut<'_, EguiUiState>,
    app_settings: Res<'_, AppSettings>,
    app_state: &mut ResMut<'_, AppState>,
    presentation: &Manifest,
    commands: &mut Commands,
    model_image_query: &Query<Entity, With<ModelImage>>,
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

    let canvases: Vec<_> = canvas_iter.collect();

    egui::ScrollArea::vertical().auto_shrink(false).show_rows(
        ui,
        row_height,
        canvases.len().div_ceil(items_per_row).max(1),
        |ui, row_range| {
            egui::Grid::new("my_grid")
                .min_col_width(column_width)
                .max_col_width(column_width)
                .min_row_height(row_height)
                .show(ui, |ui| -> Result {
                    let row_start = row_range.start;

                    for (row_index, _) in canvases
                        .iter()
                        .skip(row_range.start * items_per_row)
                        .take(row_range.count() * items_per_row)
                        .step_by(items_per_row)
                        .enumerate()
                    {
                        for col_index in 0..items_per_row {
                            let canvas_index = (row_start + row_index) * items_per_row + col_index;

                            if canvas_index < canvases.len() {
                                let canvas = canvases[canvas_index];

                                if ui
                                    .vertical_centered(|ui| {
                                        let canvas_thumbnail = canvas.get_thumbnail();

                                        if !canvas_thumbnail.is_empty() {
                                            ui.add_sized(
                                                vec2(thumbnail_size, thumbnail_size),
                                                bevy_egui::egui::Image::new(canvas_thumbnail)
                                                    .max_size(vec2(thumbnail_size, thumbnail_size)),
                                            );
                                        } else {
                                            ui.add_sized(
                                                vec2(thumbnail_size, thumbnail_size),
                                                egui::Label::new("🚫").selectable(false),
                                            );
                                        }
                                        let label = format!(
                                            "({}) {}",
                                            canvas_index + 1,
                                            canvas
                                                .get_label(&app_settings.language)
                                                .collect::<Vec<_>>()
                                                .join(",")
                                        );
                                        add_text(ui, &label, None, 3);
                                    })
                                    .response
                                    .interact(Sense::CLICK)
                                    .clicked()
                                    && let Err(err) = crate::web::load_canvas(
                                        commands,
                                        presentation,
                                        app_state,
                                        canvas_index,
                                        model_image_query,
                                    )
                                {
                                    let msg = format!("Unable to load canvas.\n'{}'", err);

                                    egui_ui_state
                                        .toasts
                                        .warning(msg)
                                        .show_progress_bar(true)
                                        .duration(Duration::from_secs(5));
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
#[allow(clippy::too_many_arguments)]
fn add_address_bar(
    ui: &mut egui::Ui,
    egui_ui_state: &mut ResMut<'_, EguiUiState>,
    app_state: &mut ResMut<'_, AppState>,
    width: f32,
) {
    if ui
        .add(
            egui::TextEdit::singleline(&mut egui_ui_state.presentation_url)
                .desired_width(width)
                .hint_text("IIIF Manifest URL"),
        )
        .on_hover_text(&egui_ui_state.presentation_url)
        .lost_focus()
        && egui_ui_state.presentation_url != app_state.presentation_url
    {
        let presentation_url = egui_ui_state.presentation_url.to_string();

        crate::web::load_presentation(app_state, &presentation_url);
    }
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
