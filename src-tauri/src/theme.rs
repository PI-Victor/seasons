use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePalette {
    Gruvbox,
    Nordbones,
    Sonokai,
    Catppuccin,
    Everforest,
    RosePine,
    Dayfox,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ThemePreference {
    pub palette: ThemePalette,
    pub mode: ThemeMode,
}

impl Default for ThemePreference {
    fn default() -> Self {
        Self {
            palette: ThemePalette::RosePine,
            mode: ThemeMode::System,
        }
    }
}
