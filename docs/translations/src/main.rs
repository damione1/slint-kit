// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui {
    slint::include_modules!();
}

/// The languages this application ships. A closed set decided by the application, so it is an
/// enum; the gettext code stays an explicit conversion at the boundary with Slint.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Language {
    English,
    French,
}

impl Language {
    /// Folder name under `lang/`; `en` selects the text written in the `.slint` sources.
    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }

    /// Parse the command line once, where external input enters.
    fn parse(argument: &str) -> Option<Self> {
        match argument {
            "en" => Some(Self::English),
            "fr" => Some(Self::French),
            _ => None,
        }
    }
}

impl From<ui::AppLanguage> for Language {
    fn from(language: ui::AppLanguage) -> Self {
        match language {
            ui::AppLanguage::English => Self::English,
            ui::AppLanguage::French => Self::French,
        }
    }
}

impl From<Language> for ui::AppLanguage {
    fn from(language: Language) -> Self {
        match language {
            Language::English => Self::English,
            Language::French => Self::French,
        }
    }
}

/// Select the catalogue, then copy the translated kit text into `KitStrings`.
fn translate(app: &ui::App, language: Language) {
    slint::select_bundled_translation(language.code())
        .expect("the catalogue is bundled at build time");
    app.set_language(language.into());
    app.invoke_translate();
}

fn main() -> Result<(), slint::PlatformError> {
    let initial = std::env::args()
        .nth(1)
        .as_deref()
        .and_then(Language::parse)
        .unwrap_or(Language::English);
    let app = ui::App::new()?;
    slint_kit::install(&app);
    app.set_unread(3);
    translate(&app, initial);
    app.on_language_chosen(slint_kit::event(&app, |app, language: ui::AppLanguage| {
        translate(&app, Language::from(language));
    }));
    app.run()
}
