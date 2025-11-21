use crate::presentation::manifest::Manifest;
use crate::rendering::tiled_image::TiledImage;
use bevy::camera::Viewport;
use bevy::prelude::{
    Camera, Commands, Entity, Local, Query, Result, Single, UVec2, Window, With, Without, default,
};
use bevy::window::PrimaryWindow;
use bevy_egui::egui::text::LayoutJob;
use bevy_egui::egui::{Color32, FontFamily, FontId, Sense, Widget, vec2};
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
    presentation: Single<&Manifest>,
    tiled_image_query: Query<(Entity, &TiledImage)>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let mut left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(ctx, |ui| {
            add_text(
                ui,
                "title".to_string(),
                presentation.get_title(),
                Some(Color32::WHITE),
                2,
            );
            if !presentation.get_description().is_empty() {
                add_text(
                    ui,
                    "desc".to_string(),
                    &presentation.get_description().join("\n"),
                    None,
                    3,
                );
            }

            let license = if !presentation.get_license().is_empty() {
                format!("(© {})", &presentation.get_license().join(","))
            } else {
                "".into()
            };
            let attribution = presentation.get_attribution().join(",");

            if !license.is_empty() || !attribution.is_empty() {
                add_text(
                    ui,
                    "attribution".to_string(),
                    &format!("{} {}", attribution, license),
                    None,
                    3,
                );
            }

            for logo in presentation.get_logo() {
                ui.add_space(6.0);
                bevy_egui::egui::Image::new(logo).max_height(64.0).ui(ui);
            }
            ui.add_space(6.0);

            egui::ComboBox::from_id_salt("Sequences")
                .selected_text(
                    presentation
                        .get_sequence(ui_state.current_sequence)
                        .get_label()
                        .collect::<Vec<_>>()
                        .join(","),
                )
                .wrap_mode(egui::TextWrapMode::Wrap)
                .show_ui(ui, |ui| {
                    for (index, seq) in presentation.get_sequences().enumerate() {
                        ui.selectable_value(
                            &mut ui_state.current_sequence,
                            index,
                            seq.get_label().collect::<Vec<_>>().join(","),
                        );
                    }
                });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| -> Result {
                for (index, canvas) in presentation
                    .get_sequence(ui_state.current_sequence)
                    .get_canvases()
                    .enumerate()
                {
                    add_text(
                        ui,
                        format!("canvas{}", index),
                        &canvas.get_label().collect::<Vec<_>>().join(","),
                        None,
                        2,
                    );
                    if let Some(thumbnail) = canvas.get_thumbnail().next()
                        && ui
                            .add_sized(
                                vec2(ui.available_width(), ui.available_width()),
                                bevy_egui::egui::Image::new(thumbnail).max_size(vec2(
                                    ui.available_width() - 16.0,
                                    ui.available_width() - 16.0,
                                )),
                            )
                            .interact(Sense::CLICK)
                            .clicked()
                    {
                        for (image_entity, _tiled_image) in tiled_image_query {
                            commands.entity(image_entity).despawn();
                        }

                        let image = TiledImage::try_from_url(canvas.get_image(0).get_service())?;

                        commands.spawn(image);
                    }
                }
                Ok(())
            });

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

fn add_text(
    ui: &mut egui::Ui,
    id_salt: String,
    text: &str,
    color: Option<Color32>,
    max_rows: usize,
) {
    egui::ScrollArea::vertical()
        .id_salt(id_salt)
        .show(ui, |ui| {
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
        });
}
