//! Typed, exhaustive German and English UI translations and locale selection.
use serde::{Deserialize, Serialize};
/// User interface language, persisted with the calculator preferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    /// German user interface.
    German,
    /// English user interface.
    English,
}
impl Default for Language {
    fn default() -> Self {
        Self::system()
    }
}
impl Language {
    /// Select the first supported language in a preference-ordered locale list.
    pub fn from_locales(locales: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for locale in locales {
            let primary = locale
                .as_ref()
                .split(['-', '_', '.', '@'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            match primary.as_str() {
                "de" => return Self::German,
                "en" => return Self::English,
                _ => {}
            }
        }
        Self::English
    }
    /// Detect browser preferences or the native operating system locale.
    pub fn system() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let locales = web_sys::window()
                .map(|window| {
                    window
                        .navigator()
                        .languages()
                        .iter()
                        .filter_map(|value| value.as_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Self::from_locales(locales)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::from_locales(sys_locale::get_locale())
        }
    }
    /// Standard HTML language tag.
    pub const fn code(self) -> &'static str {
        match self {
            Self::German => "de",
            Self::English => "en",
        }
    }
    /// Native language name for the language selector.
    pub const fn name(self) -> &'static str {
        match self {
            Self::German => "Deutsch",
            Self::English => "English",
        }
    }
    pub(crate) const fn text(self, text: Text) -> &'static str {
        match (text, self) {
            (Text::InformationSettings, Self::German) => "Informationen und Einstellungen",
            (Text::InformationSettings, Self::English) => "Information and settings",
            (Text::Settings, Self::German) => "Einstellungen",
            (Text::Settings, Self::English) => "Settings",
            (Text::General, Self::German) => "Allgemein",
            (Text::General, Self::English) => "General",
            (Text::Help, Self::German) => "Bedienung",
            (Text::Help, Self::English) => "Help",
            (Text::Diagnostics, Self::German) => "Diagnose",
            (Text::Diagnostics, Self::English) => "Diagnostics",
            (Text::Done, Self::German) => "Fertig",
            (Text::Done, Self::English) => "Done",
            (Text::Appearance, Self::German) => "Darstellung",
            (Text::Appearance, Self::English) => "Appearance",
            (Text::Behavior, Self::German) => "Verhalten",
            (Text::Behavior, Self::English) => "Behavior",
            (Text::HistoryRight, Self::German) => "Verlauf rechts im Querformat",
            (Text::HistoryRight, Self::English) => "History on right in landscape",
            (Text::Clicks, Self::German) => "Tastenklicks",
            (Text::Clicks, Self::English) => "Key sounds",
            (Text::Haptics, Self::German) => "Haptik",
            (Text::Haptics, Self::English) => "Haptics",
            (Text::NoVibration, Self::German) => {
                "Vibration wird auf diesem Gerät nicht unterstützt."
            }
            (Text::NoVibration, Self::English) => "Vibration is not supported on this device.",
            (Text::Shortcuts, Self::German) => "Hilfreiche Tastenkürzel",
            (Text::Shortcuts, Self::English) => "Keyboard shortcuts",
            (Text::ToggleSettings, Self::German) => "Einstellungen öffnen / schließen",
            (Text::ToggleSettings, Self::English) => "Open / close settings",
            (Text::TimeShortcut, Self::German) => "Zum Zeitrechner wechseln",
            (Text::TimeShortcut, Self::English) => "Switch to time calculator",
            (Text::NumberShortcut, Self::German) => "Zum Zahlenrechner wechseln",
            (Text::NumberShortcut, Self::English) => "Switch to number calculator",
            (Text::Backspace, Self::German) => "Letzte Stelle löschen",
            (Text::Backspace, Self::English) => "Delete last digit",
            (Text::ClearEntry, Self::German) => "Eingabe löschen",
            (Text::ClearEntry, Self::English) => "Clear entry",
            (Text::Clear, Self::German) => "Alles zurücksetzen",
            (Text::Clear, Self::English) => "Reset everything",
            (Text::BackspaceKey, Self::German) => "Rücktaste",
            (Text::BackspaceKey, Self::English) => "Backspace",
            (Text::DeleteKey, Self::German) => "Entf",
            (Text::DeleteKey, Self::English) => "Delete",
            (Text::ShortcutHelp, Self::German) => {
                "In den Einstellungen schließt Esc das Fenster. F2 und F3 funktionieren im Rechner."
            }
            (Text::ShortcutHelp, Self::English) => {
                "Esc closes settings. F2 and F3 work in the calculator."
            }
            (Text::AppDevice, Self::German) => "App und Gerät",
            (Text::AppDevice, Self::English) => "App and device",
            (Text::Version, Self::German) => "Version",
            (Text::Version, Self::English) => "Version",
            (Text::Screen, Self::German) => "Bildschirm",
            (Text::Screen, Self::English) => "Screen",
            (Text::Scale, Self::German) => "Pixelskalierung",
            (Text::Scale, Self::English) => "Pixel scale",
            (Text::Theme, Self::German) => "Farbschema",
            (Text::Theme, Self::English) => "Color scheme",
            (Text::Copied, Self::German) => "Diagnose kopiert",
            (Text::Copied, Self::English) => "Diagnostics copied",
            (Text::Copy, Self::German) => "Diagnose kopieren",
            (Text::Copy, Self::English) => "Copy diagnostics",
            (Text::CopyHelp, Self::German) => {
                "Kopiert den vollständigen Bericht inklusive Browserinformationen für die Fehlersuche."
            }
            (Text::CopyHelp, Self::English) => {
                "Copies the full report including browser information for troubleshooting."
            }
            (Text::Original, Self::German) => "Original",
            (Text::Original, Self::English) => "Original",
            (Text::Light, Self::German) => "Hell",
            (Text::Light, Self::English) => "Light",
            (Text::Dark, Self::German) => "Dunkel",
            (Text::Dark, Self::English) => "Dark",
            (Text::Group, Self::German) => "Gruppe",
            (Text::Group, Self::English) => "Group",
            (Text::Time, Self::German) => "Zeit",
            (Text::Time, Self::English) => "Time",
            (Text::Number, Self::German) => "Rechner",
            (Text::Number, Self::English) => "Calculator",
            (Text::Undo, Self::German) => "Letzten Schritt zurück",
            (Text::Undo, Self::English) => "Undo last step",
            (Text::Total, Self::German) => "SUMME",
            (Text::Total, Self::English) => "TOTAL",
            (Text::Undefined, Self::German) => "Nicht definiert",
            (Text::Undefined, Self::English) => "Undefined",
            (Text::Limit, Self::German) => {
                "Eingabe- oder Verlaufslimit erreicht. Bitte löschen oder rückgängig machen."
            }
            (Text::Limit, Self::English) => "Entry or history limit reached. Please clear or undo.",
            (Text::Language, Self::German) => "Sprache",
            (Text::Language, Self::English) => "Language",
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Text {
    InformationSettings,
    Settings,
    General,
    Help,
    Diagnostics,
    Done,
    Appearance,
    Behavior,
    HistoryRight,
    Clicks,
    Haptics,
    NoVibration,
    Shortcuts,
    ToggleSettings,
    TimeShortcut,
    NumberShortcut,
    Backspace,
    ClearEntry,
    Clear,
    BackspaceKey,
    DeleteKey,
    ShortcutHelp,
    AppDevice,
    Version,
    Screen,
    Scale,
    Theme,
    Copied,
    Copy,
    CopyHelp,
    Original,
    Light,
    Dark,
    Group,
    Time,
    Number,
    Undo,
    Total,
    Undefined,
    Limit,
    Language,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locale_preferences_match_regions_and_skip_unsupported_languages() {
        assert_eq!(
            Language::from_locales(["fr-FR", "DE-at", "en"]),
            Language::German
        );
        assert_eq!(
            Language::from_locales(["en_US.UTF-8", "de"]),
            Language::English
        );
        assert_eq!(Language::from_locales(["es", "ja"]), Language::English);
        assert_eq!(
            Language::from_locales(Vec::<String>::new()),
            Language::English
        );
    }
}
