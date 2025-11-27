use crate::app::app_settings::AppSettings;
use crate::presentation::manifest::Manifest;
use crate::rendering::tiled_image::TiledImage;
use bevy::camera::Viewport;
use bevy::prelude::{
    Camera, Commands, Entity, Local, Query, Res, Result, Single, UVec2, Window, With, Without,
    default,
};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Color32, FontFamily, FontId, Sense, TextBuffer, Widget, vec2};
use bevy_egui::{EguiContext, EguiContexts, egui};

#[derive(Debug, Default)]
pub(crate) struct UIState {
    // open_panel: bool,
    current_sequence: usize,
}

/// Set up egui.
pub(crate) fn setup(mut contexts: EguiContexts) -> Result {
    let ctx = contexts.ctx_mut()?;

    // Set up image loaders for the thumbnails.
    egui_extras::install_image_loaders(ctx);
    Ok(())
}

pub(crate) fn presentation_ui_system(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut camera: Single<&mut Camera, Without<EguiContext>>,
    window: Single<&mut Window, With<PrimaryWindow>>,
    mut ui_state: Local<UIState>,
    app_settings: Res<AppSettings>,
    presentation: Single<&Manifest>,
    tiled_image_query: Query<(Entity, &TiledImage)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let presentation = *presentation;

    let mut left = egui::Panel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            add_text(
                ui,
                presentation.model().get_title(),
                Some(Color32::WHITE),
                2,
            );

            let description = presentation.model().get_description().collect::<Vec<_>>();
            if !description.is_empty() {
                add_text(ui, &description.join("\n"), None, 3);
            }

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

            for logo in presentation.model().get_logo() {
                ui.add_space(6.0);
                bevy_egui::egui::Image::new(logo).max_height(64.0).ui(ui);
            }
            ui.add_space(6.0);

            egui::ComboBox::from_id_salt("Sequences")
                .selected_text(
                    presentation
                        .model()
                        .get_sequence(ui_state.current_sequence)
                        .get_label()
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .wrap_mode(egui::TextWrapMode::Wrap)
                .show_ui(ui, |ui| {
                    for (index, seq) in presentation.model().get_sequences().enumerate() {
                        ui.selectable_value(
                            &mut ui_state.current_sequence,
                            index,
                            seq.get_label().collect::<Vec<_>>().join(","),
                        );
                    }
                });

            ui.separator();

            let canvas_iter = presentation
                .model()
                .get_sequence(ui_state.current_sequence)
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
                                    let canvas_index =
                                        (row_start + row_index) * items_per_row + col_index;

                                    if canvas_index < visible_canvases.len() {
                                        let canvas = visible_canvases[canvas_index];

                                        if ui
                                            .vertical_centered(|ui| {
                                                let canvas_thumbnail = canvas
                                                    .get_thumbnail()
                                                    .next()
                                                    .unwrap_or_default();

                                                if !canvas_thumbnail.is_empty() {
                                                    ui.add_sized(
                                                        vec2(thumbnail_size, thumbnail_size),
                                                        bevy_egui::egui::Image::new(
                                                            canvas_thumbnail,
                                                        )
                                                        .max_size(vec2(
                                                            thumbnail_size,
                                                            thumbnail_size,
                                                        )),
                                                    );
                                                }
                                                let label = format!(
                                                    "({}) {}",
                                                    canvas_index + 1,
                                                    canvas
                                                        .get_label()
                                                        .collect::<Vec<_>>()
                                                        .join(",")
                                                );
                                                add_text(ui, &label, None, 3);
                                            })
                                            .response
                                            .interact(Sense::CLICK)
                                            .clicked()
                                        {
                                            for (image_entity, _tiled_image) in tiled_image_query {
                                                commands.entity(image_entity).despawn();
                                            }

                                            let image = TiledImage::try_from_url(
                                                canvas.get_image(0).get_service().as_str(),
                                            )?;

                                            commands.spawn(image);
                                        }
                                    }
                                }
                                ui.end_row();
                            }
                            Ok(())
                        });
                },
            );

            // ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width(); // height is ignored, as the panel has a hight of 100% of the screen

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

    // let mut top = egui::TopBottomPanel::top("top_panel")
    //     .resizable(false)
    //     .show(ctx, |ui| {
    //         // if ui.button("☰").on_hover_text("Open panel").clicked() {
    //         //     ui_state.open_panel = true;
    //         // }
    //         // ui.label(&presentation.title);
    //         // for s in &presentation.attribution {
    //         //     ui.label(s);
    //         // }
    //         ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
    //     })
    //     .response
    //     .rect
    //     .height(); // width is ignored, as the panel has a width of 100% of the screen
    let mut top = 0.0;

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

    // if ui_state.open_panel {
    //     egui::SidePanel::left("left_panel")
    //         .frame(egui::Frame::new().fill(egui::Color32::from_white_alpha(64)))
    //         .show_separator_line(false)
    //         .resizable(false)
    //         .show(ctx, |ui| {
    //             let mut close_button_ui = ui.new_child(
    //                 UiBuilder::new()
    //                     .max_rect(ui.max_rect())
    //                     .layout(Layout::right_to_left(Align::Min)),
    //             );
    //             if close_button_ui
    //                 .button("❌")
    //                 .on_hover_text("Close panel")
    //                 .clicked()
    //             {
    //                 ui_state.open_panel = false;
    //             }
    //             ui.horizontal(|ui| {
    //                 ui.label("text");
    //             });
    //         });
    // }

    Ok(())
}

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
