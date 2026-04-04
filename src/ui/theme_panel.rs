use crate::theme::{ThemeMode, ThemePalette, ThemePreference};
use leptos::prelude::*;

#[component]
pub fn ThemePanel(
    theme_preference: ReadSignal<ThemePreference>,
    on_palette_change: Callback<ThemePalette>,
    on_mode_change: Callback<ThemeMode>,
) -> impl IntoView {
    view! {
        <section class="theme-panel surface-panel">
            <div class="settings-header">
                <div>
                    <p class="panel-kicker">"Theme"</p>
                    <h2>"Look and feel"</h2>
                </div>
            </div>

            <p class="panel-copy">
                "Pick a palette family, then choose whether the app follows system light/dark or forces one side."
            </p>

            <div class="theme-grid">
                <label class="theme-field">
                    <span class="field-label">"Appearance"</span>
                    <select
                        class="theme-select"
                        prop:value=move || serialize_mode(theme_preference.get().mode)
                        on:change=move |event| {
                            on_mode_change.run(parse_mode(event_target_value(&event)));
                        }
                    >
                        {ThemeMode::ALL.into_iter().map(|mode| {
                            view! {
                                <option value=serialize_mode(mode)>{mode.label()}</option>
                            }
                        }).collect_view()}
                    </select>
                </label>

                <label class="theme-field">
                    <span class="field-label">"Palette"</span>
                    <select
                        class="theme-select"
                        prop:value=move || serialize_palette(theme_preference.get().palette)
                        on:change=move |event| {
                            on_palette_change.run(parse_palette(event_target_value(&event)));
                        }
                    >
                        {ThemePalette::ALL.into_iter().map(|palette| {
                            view! {
                                <option value=serialize_palette(palette)>
                                    {format!("{} ({})", palette.label(), palette.note())}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </label>
            </div>
        </section>
    }
}

fn serialize_mode(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "system",
        ThemeMode::Dark => "dark",
        ThemeMode::Light => "light",
    }
}

fn parse_mode(value: String) -> ThemeMode {
    match value.as_str() {
        "dark" => ThemeMode::Dark,
        "light" => ThemeMode::Light,
        _ => ThemeMode::System,
    }
}

fn serialize_palette(palette: ThemePalette) -> &'static str {
    match palette {
        ThemePalette::Gruvbox => "gruvbox",
        ThemePalette::Nordbones => "nordbones",
        ThemePalette::Sonokai => "sonokai",
        ThemePalette::Catppuccin => "catppuccin",
        ThemePalette::Everforest => "everforest",
        ThemePalette::RosePine => "rose-pine",
        ThemePalette::Dayfox => "dayfox",
    }
}

fn parse_palette(value: String) -> ThemePalette {
    match value.as_str() {
        "gruvbox" => ThemePalette::Gruvbox,
        "nordbones" => ThemePalette::Nordbones,
        "sonokai" => ThemePalette::Sonokai,
        "catppuccin" => ThemePalette::Catppuccin,
        "everforest" => ThemePalette::Everforest,
        "dayfox" => ThemePalette::Dayfox,
        _ => ThemePalette::RosePine,
    }
}
