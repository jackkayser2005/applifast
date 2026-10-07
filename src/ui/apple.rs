//! Native controls for the first Apple Music listening path.
use crate::{app::App, model::Action, player::Playback};
use serde_json::json;

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let Some(state) = app.apple.as_mut() else {
        return;
    };
    let header = egui::Panel::top("apple-header").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.heading("Applifast");
            ui.label("Apple Music");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::window::custom_titlebar() {
                    ui.add_space(110.0);
                }
                if state.authorized && ui.button("Sign out").clicked() {
                    app.actions.push(Action::SignOut);
                }
                if ui
                    .button(
                        if app.settings.theme == crate::settings::ThemeChoice::Light {
                            "Dark"
                        } else {
                            "Light"
                        },
                    )
                    .clicked()
                {
                    app.actions.push(Action::SetTheme(
                        if app.settings.theme == crate::settings::ThemeChoice::Light {
                            crate::settings::ThemeChoice::Dark
                        } else {
                            crate::settings::ThemeChoice::Light
                        },
                    ));
                }
            });
        });
    });
    super::titlebar_drag(ui, header.response.rect);
    egui::Panel::bottom("apple-controls").show(ui, |ui| {
        ui.add_space(8.0);
        if let Some(track) = &state.local.track {
            ui.strong(&track.title);
            ui.label(track.artist_names());
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(state.index.is_some(), egui::Button::new("Previous"))
                .clicked()
            {
                app.actions.push(Action::Previous);
            }
            if ui
                .add_enabled(
                    state.index.is_some(),
                    egui::Button::new(if state.local.playback == Playback::Playing {
                        "Pause"
                    } else {
                        "Play"
                    }),
                )
                .clicked()
            {
                app.actions.push(Action::TogglePlay);
            }
            if ui
                .add_enabled(state.index.is_some(), egui::Button::new("Next"))
                .clicked()
            {
                app.actions.push(Action::Next);
            }
            let duration = state
                .local
                .track
                .as_ref()
                .map_or(0, |track| track.duration_ms);
            let mut seconds = state.local.position_now() as f64 / 1000.0;
            let seek = ui.add_enabled(
                duration > 0,
                egui::Slider::new(&mut seconds, 0.0..=f64::from(duration) / 1000.0)
                    .fixed_decimals(0)
                    .suffix(" s"),
            );
            if seek.drag_stopped() || (seek.changed() && !seek.dragged()) {
                app.actions.push(Action::Seek((seconds * 1000.0) as u32));
            }
            let mut volume = (u32::from(state.local.volume) * 100 / 65535) as u8;
            if ui
                .add(egui::Slider::new(&mut volume, 0..=100).text("Volume"))
                .changed()
            {
                app.actions.push(Action::SetVolume(volume));
            }
        });
        ui.add_space(8.0);
    });
    egui::CentralPanel::default().show(ui, |ui| {
        if let Some(error) = &state.error {
            ui.colored_label(app.palette.accent, error);
            ui.separator();
        }
        if !cfg!(windows) {
            ui.label("Apple Music playback is currently supported on Windows only.");
            return;
        }
        if !state.authorized {
            ui.heading("Connect your Apple Music library");
            ui.label(
                "Import your locally generated developer-token file. Keep the signing key private.",
            );
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.token_path)
                        .hint_text("Path to developer-token.txt")
                        .desired_width(360.0),
                );
                if ui.button("Import token").clicked() {
                    app.actions
                        .push(Action::AppleImportToken(state.token_path.clone().into()));
                }
            });
            if ui
                .add_enabled(state.ready, egui::Button::new("Sign in with Apple"))
                .clicked()
            {
                app.actions
                    .push(Action::AppleSend(json!({"type":"authorize"})));
            }
            if state.loading {
                ui.spinner();
            }
            return;
        }
        ui.heading("Songs");
        ui.horizontal(|ui| {
            ui.label(format!("{} songs loaded", state.songs.len()));
            ui.add(egui::TextEdit::singleline(&mut state.filter).hint_text("Filter loaded songs"));
        });
        let filter = state.filter.to_lowercase();
        egui::ScrollArea::vertical()
            .id_salt("apple-songs")
            .show(ui, |ui| {
                egui::Grid::new("apple-song-table")
                    .striped(true)
                    .num_columns(4)
                    .spacing([24.0, 8.0])
                    .show(ui, |ui| {
                        ui.strong("Title");
                        ui.strong("Artist");
                        ui.strong("Album");
                        ui.strong("Availability");
                        ui.end_row();
                        for (index, song) in state.songs.iter().enumerate() {
                            if !filter.is_empty()
                                && !format!("{} {} {}", song.title, song.artist, song.album)
                                    .to_lowercase()
                                    .contains(&filter)
                            {
                                continue;
                            }
                            let row = ui.selectable_label(
                                state.local.track.as_ref().is_some_and(|_| {
                                    state.index.is_some_and(|position| {
                                        state.queue.get(position).is_some_and(|playing| {
                                            playing.item.id == song.item.id
                                                && std::mem::discriminant(&playing.item.kind)
                                                    == std::mem::discriminant(&song.item.kind)
                                        })
                                    })
                                }),
                                &song.title,
                            );
                            if row.double_clicked() {
                                app.actions.push(Action::ApplePlaySong(index));
                            }
                            ui.label(&song.artist);
                            ui.label(&song.album);
                            ui.label(if song.available() {
                                "Ready"
                            } else {
                                "Unavailable"
                            });
                            ui.end_row();
                        }
                    });
                if state.loading {
                    ui.spinner();
                }
                if let Some(next) = &state.next
                    && ui
                        .add_enabled(!state.loading, egui::Button::new("Load more songs"))
                        .clicked()
                {
                    app.actions
                        .push(Action::AppleSend(json!({"type":"library","next":next})));
                }
            });
    });
    super::window_controls(ui, &app.palette, app.locale);
    super::window_resize(ui);
}
