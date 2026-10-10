//! Apple setup content inside the existing native sign-in card.
use crate::{app::App, i18n::gettext, model::Action, theme};

pub(super) fn login_contents(app: &mut App, ui: &mut egui::Ui) {
    let (ready, loading, error) = {
        let apple = app.apple.as_ref().expect("Apple sign-in");
        (apple.ready, apple.loading, apple.error.clone())
    };
    let needs_token_import = error.is_some() || app.apple.as_ref().unwrap().token_request.is_some();
    if let Some(error) = error {
        ui.add(
            egui::Label::new(
                egui::RichText::new(error)
                    .font(theme::regular(13.0))
                    .color(app.palette.danger),
            )
            .wrap(),
        );
        ui.add_space(12.0);
    }
    if !cfg!(windows) {
        theme::text(
            ui,
            "Apple playback is currently supported on Windows.",
            theme::regular(13.0),
            app.palette.secondary,
        );
        return;
    }
    if loading {
        ui.horizontal(|ui| {
            theme::spinner(ui, 18.0, app.palette.accent);
            theme::text(
                ui,
                if ready {
                    "Waiting for Apple authorization"
                } else {
                    "Connecting to Apple Music"
                },
                theme::medium(14.0),
                app.palette.text,
            );
        });
        if ready && theme::pill_button(ui, &app.palette, "Cancel", false).clicked() {
            app.actions.push(Action::CancelSignIn);
        }
    }
    if ui
        .add_enabled_ui(
            ready && !loading && app.apple.as_ref().unwrap().token_request.is_none(),
            |ui| super::login::big_button(ui, app, "Sign in with Apple"),
        )
        .inner
    {
        app.actions.push(Action::SignIn);
    }
    ui.add_space(10.0);
    theme::text(
        ui,
        "Sign in through Apple's authorization window.",
        theme::regular(12.5),
        app.palette.secondary,
    );
    if needs_token_import {
        ui.add_space(12.0);
        token_import(app, ui);
    }
}

/// Import only a local JWT path. The private signing key never enters the app.
pub(super) fn token_import(app: &mut App, ui: &mut egui::Ui) {
    let apple = app.apple.as_mut().expect("Apple sign-in");
    ui.add(
        egui::TextEdit::singleline(&mut apple.token_path)
            .hint_text(gettext(app.locale, "Path to developer-token.txt"))
            .desired_width(ui.available_width()),
    );
    if ui
        .add_enabled_ui(
            apple.token_request.is_none() && !(apple.ready && apple.loading && !apple.authorized),
            |ui| {
                theme::pill_button(
                    ui,
                    &app.palette,
                    &gettext(app.locale, "Import developer token"),
                    false,
                )
            },
        )
        .inner
        .clicked()
    {
        app.actions.push(Action::AppleImportToken(
            apple.token_path.trim().trim_matches('"').into(),
        ));
    }
}
