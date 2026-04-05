// SPDX-License-Identifier: Apache-2.0
//
// Copyright 2026 Victor Palade
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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

impl ThemePalette {
    pub const ALL: [ThemePalette; 7] = [
        ThemePalette::Gruvbox,
        ThemePalette::Nordbones,
        ThemePalette::Sonokai,
        ThemePalette::Catppuccin,
        ThemePalette::Everforest,
        ThemePalette::RosePine,
        ThemePalette::Dayfox,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ThemePalette::Gruvbox => "Gruvbox",
            ThemePalette::Nordbones => "Nordbones",
            ThemePalette::Sonokai => "Sonokai",
            ThemePalette::Catppuccin => "Catppuccin",
            ThemePalette::Everforest => "Everforest",
            ThemePalette::RosePine => "Rose Pine",
            ThemePalette::Dayfox => "Dayfox",
        }
    }

    pub fn note(self) -> &'static str {
        match self {
            ThemePalette::Nordbones => "dark-native",
            ThemePalette::Sonokai => "dark-native",
            ThemePalette::Dayfox => "light-native",
            ThemePalette::Catppuccin => "latte/macchiato",
            ThemePalette::RosePine => "dawn/moon",
            ThemePalette::Gruvbox => "light/dark",
            ThemePalette::Everforest => "light/dark",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl ThemeMode {
    pub const ALL: [ThemeMode; 3] = [ThemeMode::System, ThemeMode::Dark, ThemeMode::Light];

    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::System => "System",
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }
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

#[derive(Clone, Copy)]
struct ThemeTokens {
    color_scheme: &'static str,
    bg: &'static str,
    bg_elevated: &'static str,
    bg_card: &'static str,
    line: &'static str,
    line_strong: &'static str,
    text: &'static str,
    text_soft: &'static str,
    amber: &'static str,
    amber_deep: &'static str,
    lime: &'static str,
    rose: &'static str,
    blue: &'static str,
    shadow: &'static str,
    page_glow_primary: &'static str,
    page_glow_secondary: &'static str,
    page_gradient: &'static str,
    panel_bg: &'static str,
    banner_bg: &'static str,
    pill_bg: &'static str,
    pill_border: &'static str,
    ghost_bg: &'static str,
    ghost_border: &'static str,
    secondary_bg: &'static str,
    secondary_border: &'static str,
    quit_bg: &'static str,
    quit_border: &'static str,
    accent_text: &'static str,
    primary_shadow: &'static str,
    scene_shell_bg: &'static str,
    room_shell_bg: &'static str,
    room_shell_border: &'static str,
    chip_bg: &'static str,
    chip_border: &'static str,
    slider_bg: &'static str,
    slider_fill: &'static str,
}

impl ThemePreference {
    fn resolved_tokens(&self) -> ThemeTokens {
        let system_prefers_dark = system_prefers_dark();
        let wants_dark = match self.mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => system_prefers_dark,
        };

        match (self.palette, wants_dark) {
            (ThemePalette::Gruvbox, true) => ThemeTokens {
                color_scheme: "dark",
                bg: "#282828",
                bg_elevated: "rgba(40, 40, 40, 0.92)",
                bg_card: "rgba(50, 48, 47, 0.86)",
                line: "rgba(168, 153, 132, 0.18)",
                line_strong: "rgba(250, 189, 47, 0.34)",
                text: "#ebdbb2",
                text_soft: "rgba(235, 219, 178, 0.72)",
                amber: "#fabd2f",
                amber_deep: "#fe8019",
                lime: "#b8bb26",
                rose: "#fb4934",
                blue: "#83a598",
                shadow: "0 24px 72px rgba(0, 0, 0, 0.34)",
                page_glow_primary: "rgba(250, 189, 47, 0.18)",
                page_glow_secondary: "rgba(131, 165, 152, 0.14)",
                page_gradient: "linear-gradient(160deg, #282828 0%, #1d2021 58%, #32302f 100%)",
                panel_bg: "linear-gradient(160deg, rgba(251, 241, 199, 0.02), transparent 38%), rgba(40, 40, 40, 0.84)",
                banner_bg: "rgba(50, 48, 47, 0.86)",
                pill_bg: "rgba(60, 56, 54, 0.72)",
                pill_border: "rgba(189, 174, 147, 0.18)",
                ghost_bg: "rgba(60, 56, 54, 0.7)",
                ghost_border: "rgba(168, 153, 132, 0.18)",
                secondary_bg: "rgba(80, 73, 69, 0.66)",
                secondary_border: "rgba(250, 189, 47, 0.18)",
                quit_bg: "rgba(157, 0, 6, 0.18)",
                quit_border: "rgba(251, 73, 52, 0.22)",
                accent_text: "#282828",
                primary_shadow: "0 16px 32px rgba(254, 128, 25, 0.18)",
                scene_shell_bg: "rgba(60, 56, 54, 0.42)",
                room_shell_bg: "rgba(50, 48, 47, 0.7)",
                room_shell_border: "rgba(189, 174, 147, 0.1)",
                chip_bg: "rgba(60, 56, 54, 0.68)",
                chip_border: "rgba(168, 153, 132, 0.14)",
                slider_bg: "rgba(124, 111, 100, 0.24)",
                slider_fill: "linear-gradient(90deg, #fabd2f, #fe8019)",
            },
            (ThemePalette::Gruvbox, false) => ThemeTokens {
                color_scheme: "light",
                bg: "#fbf1c7",
                bg_elevated: "rgba(253, 244, 193, 0.94)",
                bg_card: "rgba(242, 229, 188, 0.92)",
                line: "rgba(80, 73, 69, 0.14)",
                line_strong: "rgba(215, 153, 33, 0.34)",
                text: "#3c3836",
                text_soft: "rgba(60, 56, 54, 0.68)",
                amber: "#d79921",
                amber_deep: "#d65d0e",
                lime: "#98971a",
                rose: "#cc241d",
                blue: "#458588",
                shadow: "0 20px 60px rgba(87, 68, 36, 0.12)",
                page_glow_primary: "rgba(215, 153, 33, 0.18)",
                page_glow_secondary: "rgba(69, 133, 136, 0.12)",
                page_gradient: "linear-gradient(160deg, #fbf1c7 0%, #f2e5bc 54%, #ebdbb2 100%)",
                panel_bg: "linear-gradient(160deg, rgba(255, 255, 255, 0.28), transparent 34%), rgba(253, 244, 193, 0.88)",
                banner_bg: "rgba(242, 229, 188, 0.94)",
                pill_bg: "rgba(255, 255, 255, 0.56)",
                pill_border: "rgba(80, 73, 69, 0.1)",
                ghost_bg: "rgba(255, 255, 255, 0.56)",
                ghost_border: "rgba(80, 73, 69, 0.12)",
                secondary_bg: "rgba(255, 255, 255, 0.72)",
                secondary_border: "rgba(215, 153, 33, 0.16)",
                quit_bg: "rgba(204, 36, 29, 0.08)",
                quit_border: "rgba(204, 36, 29, 0.18)",
                accent_text: "#fbf1c7",
                primary_shadow: "0 14px 30px rgba(214, 93, 14, 0.12)",
                scene_shell_bg: "rgba(255, 255, 255, 0.44)",
                room_shell_bg: "rgba(255, 255, 255, 0.56)",
                room_shell_border: "rgba(80, 73, 69, 0.08)",
                chip_bg: "rgba(255, 255, 255, 0.62)",
                chip_border: "rgba(80, 73, 69, 0.1)",
                slider_bg: "rgba(80, 73, 69, 0.12)",
                slider_fill: "linear-gradient(90deg, #d79921, #d65d0e)",
            },
            (ThemePalette::Nordbones, _) => ThemeTokens {
                color_scheme: "dark",
                bg: "#2F3541",
                bg_elevated: "rgba(47, 53, 65, 0.92)",
                bg_card: "rgba(53, 60, 73, 0.88)",
                line: "rgba(115, 124, 144, 0.22)",
                line_strong: "rgba(143, 188, 186, 0.34)",
                text: "#EBEEF3",
                text_soft: "rgba(235, 238, 243, 0.7)",
                amber: "#CF866F",
                amber_deep: "#E09680",
                lime: "#A4BE8D",
                rose: "#C1616A",
                blue: "#8FBCBA",
                shadow: "0 24px 70px rgba(20, 25, 35, 0.32)",
                page_glow_primary: "rgba(207, 134, 111, 0.16)",
                page_glow_secondary: "rgba(135, 191, 206, 0.14)",
                page_gradient: "linear-gradient(160deg, #2F3541 0%, #252b35 56%, #353c49 100%)",
                panel_bg: "linear-gradient(160deg, rgba(242, 244, 247, 0.025), transparent 38%), rgba(47, 53, 65, 0.84)",
                banner_bg: "rgba(53, 60, 73, 0.86)",
                pill_bg: "rgba(57, 64, 78, 0.76)",
                pill_border: "rgba(126, 140, 168, 0.18)",
                ghost_bg: "rgba(57, 64, 78, 0.72)",
                ghost_border: "rgba(126, 140, 168, 0.16)",
                secondary_bg: "rgba(65, 73, 89, 0.74)",
                secondary_border: "rgba(143, 188, 186, 0.18)",
                quit_bg: "rgba(193, 97, 106, 0.12)",
                quit_border: "rgba(214, 120, 127, 0.2)",
                accent_text: "#2F3541",
                primary_shadow: "0 16px 32px rgba(224, 150, 128, 0.16)",
                scene_shell_bg: "rgba(65, 73, 89, 0.44)",
                room_shell_bg: "rgba(53, 60, 73, 0.72)",
                room_shell_border: "rgba(126, 140, 168, 0.12)",
                chip_bg: "rgba(57, 64, 78, 0.68)",
                chip_border: "rgba(126, 140, 168, 0.14)",
                slider_bg: "rgba(129, 142, 171, 0.18)",
                slider_fill: "linear-gradient(90deg, #CF866F, #8FBCBA)",
            },
            (ThemePalette::Sonokai, _) => ThemeTokens {
                color_scheme: "dark",
                bg: "#2c2e34",
                bg_elevated: "rgba(44, 46, 52, 0.92)",
                bg_card: "rgba(51, 53, 63, 0.88)",
                line: "rgba(127, 132, 144, 0.2)",
                line_strong: "rgba(118, 204, 224, 0.28)",
                text: "#e2e2e3",
                text_soft: "rgba(226, 226, 227, 0.72)",
                amber: "#e7c664",
                amber_deep: "#f39660",
                lime: "#9ed072",
                rose: "#fc5d7c",
                blue: "#76cce0",
                shadow: "0 24px 72px rgba(19, 22, 30, 0.34)",
                page_glow_primary: "rgba(243, 150, 96, 0.16)",
                page_glow_secondary: "rgba(118, 204, 224, 0.14)",
                page_gradient: "linear-gradient(160deg, #2c2e34 0%, #181819 56%, #33353f 100%)",
                panel_bg: "linear-gradient(160deg, rgba(226, 226, 227, 0.02), transparent 38%), rgba(44, 46, 52, 0.84)",
                banner_bg: "rgba(51, 53, 63, 0.86)",
                pill_bg: "rgba(59, 62, 72, 0.76)",
                pill_border: "rgba(127, 132, 144, 0.16)",
                ghost_bg: "rgba(59, 62, 72, 0.74)",
                ghost_border: "rgba(127, 132, 144, 0.16)",
                secondary_bg: "rgba(65, 69, 80, 0.74)",
                secondary_border: "rgba(118, 204, 224, 0.18)",
                quit_bg: "rgba(252, 93, 124, 0.12)",
                quit_border: "rgba(252, 93, 124, 0.2)",
                accent_text: "#222327",
                primary_shadow: "0 16px 32px rgba(243, 150, 96, 0.18)",
                scene_shell_bg: "rgba(59, 62, 72, 0.44)",
                room_shell_bg: "rgba(51, 53, 63, 0.72)",
                room_shell_border: "rgba(127, 132, 144, 0.1)",
                chip_bg: "rgba(59, 62, 72, 0.68)",
                chip_border: "rgba(127, 132, 144, 0.14)",
                slider_bg: "rgba(127, 132, 144, 0.2)",
                slider_fill: "linear-gradient(90deg, #e7c664, #76cce0)",
            },
            (ThemePalette::Catppuccin, true) => ThemeTokens {
                color_scheme: "dark",
                bg: "#24273a",
                bg_elevated: "rgba(36, 39, 58, 0.94)",
                bg_card: "rgba(30, 32, 48, 0.9)",
                line: "rgba(147, 153, 178, 0.2)",
                line_strong: "rgba(138, 173, 244, 0.3)",
                text: "#cad3f5",
                text_soft: "rgba(202, 211, 245, 0.72)",
                amber: "#eed49f",
                amber_deep: "#f5a97f",
                lime: "#a6da95",
                rose: "#ed8796",
                blue: "#8aadf4",
                shadow: "0 24px 72px rgba(17, 17, 27, 0.36)",
                page_glow_primary: "rgba(245, 169, 127, 0.18)",
                page_glow_secondary: "rgba(138, 173, 244, 0.16)",
                page_gradient: "linear-gradient(160deg, #24273a 0%, #181926 58%, #1e2030 100%)",
                panel_bg: "linear-gradient(160deg, rgba(202, 211, 245, 0.02), transparent 38%), rgba(30, 32, 48, 0.86)",
                banner_bg: "rgba(30, 32, 48, 0.88)",
                pill_bg: "rgba(54, 58, 79, 0.74)",
                pill_border: "rgba(147, 153, 178, 0.18)",
                ghost_bg: "rgba(54, 58, 79, 0.74)",
                ghost_border: "rgba(147, 153, 178, 0.16)",
                secondary_bg: "rgba(54, 58, 79, 0.78)",
                secondary_border: "rgba(138, 173, 244, 0.18)",
                quit_bg: "rgba(237, 135, 150, 0.12)",
                quit_border: "rgba(237, 135, 150, 0.2)",
                accent_text: "#1e2030",
                primary_shadow: "0 16px 32px rgba(245, 169, 127, 0.18)",
                scene_shell_bg: "rgba(54, 58, 79, 0.46)",
                room_shell_bg: "rgba(30, 32, 48, 0.72)",
                room_shell_border: "rgba(147, 153, 178, 0.1)",
                chip_bg: "rgba(54, 58, 79, 0.68)",
                chip_border: "rgba(147, 153, 178, 0.14)",
                slider_bg: "rgba(147, 153, 178, 0.18)",
                slider_fill: "linear-gradient(90deg, #eed49f, #8aadf4)",
            },
            (ThemePalette::Catppuccin, false) => ThemeTokens {
                color_scheme: "light",
                bg: "#eff1f5",
                bg_elevated: "rgba(239, 241, 245, 0.96)",
                bg_card: "rgba(230, 233, 239, 0.92)",
                line: "rgba(124, 127, 147, 0.16)",
                line_strong: "rgba(30, 102, 245, 0.24)",
                text: "#4c4f69",
                text_soft: "rgba(76, 79, 105, 0.68)",
                amber: "#df8e1d",
                amber_deep: "#fe640b",
                lime: "#40a02b",
                rose: "#d20f39",
                blue: "#1e66f5",
                shadow: "0 20px 60px rgba(76, 79, 105, 0.12)",
                page_glow_primary: "rgba(223, 142, 29, 0.16)",
                page_glow_secondary: "rgba(30, 102, 245, 0.12)",
                page_gradient: "linear-gradient(160deg, #eff1f5 0%, #e6e9ef 54%, #dce0e8 100%)",
                panel_bg: "linear-gradient(160deg, rgba(255, 255, 255, 0.3), transparent 34%), rgba(239, 241, 245, 0.88)",
                banner_bg: "rgba(230, 233, 239, 0.94)",
                pill_bg: "rgba(255, 255, 255, 0.58)",
                pill_border: "rgba(124, 127, 147, 0.1)",
                ghost_bg: "rgba(255, 255, 255, 0.58)",
                ghost_border: "rgba(124, 127, 147, 0.12)",
                secondary_bg: "rgba(255, 255, 255, 0.72)",
                secondary_border: "rgba(30, 102, 245, 0.14)",
                quit_bg: "rgba(210, 15, 57, 0.08)",
                quit_border: "rgba(210, 15, 57, 0.16)",
                accent_text: "#eff1f5",
                primary_shadow: "0 14px 30px rgba(223, 142, 29, 0.12)",
                scene_shell_bg: "rgba(255, 255, 255, 0.44)",
                room_shell_bg: "rgba(255, 255, 255, 0.58)",
                room_shell_border: "rgba(124, 127, 147, 0.08)",
                chip_bg: "rgba(255, 255, 255, 0.62)",
                chip_border: "rgba(124, 127, 147, 0.1)",
                slider_bg: "rgba(124, 127, 147, 0.12)",
                slider_fill: "linear-gradient(90deg, #df8e1d, #1e66f5)",
            },
            (ThemePalette::Everforest, true) => ThemeTokens {
                color_scheme: "dark",
                bg: "#2D353B",
                bg_elevated: "rgba(45, 53, 59, 0.94)",
                bg_card: "rgba(52, 63, 68, 0.9)",
                line: "rgba(133, 146, 137, 0.22)",
                line_strong: "rgba(127, 187, 179, 0.28)",
                text: "#D3C6AA",
                text_soft: "rgba(211, 198, 170, 0.72)",
                amber: "#DBBC7F",
                amber_deep: "#E69875",
                lime: "#A7C080",
                rose: "#E67E80",
                blue: "#7FBBB3",
                shadow: "0 24px 72px rgba(18, 24, 24, 0.34)",
                page_glow_primary: "rgba(219, 188, 127, 0.16)",
                page_glow_secondary: "rgba(127, 187, 179, 0.14)",
                page_gradient: "linear-gradient(160deg, #2D353B 0%, #232A2E 56%, #343F44 100%)",
                panel_bg: "linear-gradient(160deg, rgba(211, 198, 170, 0.02), transparent 38%), rgba(45, 53, 59, 0.86)",
                banner_bg: "rgba(52, 63, 68, 0.88)",
                pill_bg: "rgba(61, 72, 77, 0.74)",
                pill_border: "rgba(133, 146, 137, 0.18)",
                ghost_bg: "rgba(61, 72, 77, 0.74)",
                ghost_border: "rgba(133, 146, 137, 0.16)",
                secondary_bg: "rgba(71, 82, 88, 0.76)",
                secondary_border: "rgba(127, 187, 179, 0.16)",
                quit_bg: "rgba(230, 126, 128, 0.12)",
                quit_border: "rgba(230, 126, 128, 0.18)",
                accent_text: "#232A2E",
                primary_shadow: "0 16px 32px rgba(230, 152, 117, 0.18)",
                scene_shell_bg: "rgba(71, 82, 88, 0.44)",
                room_shell_bg: "rgba(52, 63, 68, 0.74)",
                room_shell_border: "rgba(133, 146, 137, 0.1)",
                chip_bg: "rgba(61, 72, 77, 0.68)",
                chip_border: "rgba(133, 146, 137, 0.14)",
                slider_bg: "rgba(133, 146, 137, 0.18)",
                slider_fill: "linear-gradient(90deg, #DBBC7F, #7FBBB3)",
            },
            (ThemePalette::Everforest, false) => ThemeTokens {
                color_scheme: "light",
                bg: "#FDF6E3",
                bg_elevated: "rgba(253, 246, 227, 0.96)",
                bg_card: "rgba(244, 240, 217, 0.92)",
                line: "rgba(130, 145, 129, 0.16)",
                line_strong: "rgba(58, 148, 197, 0.22)",
                text: "#5C6A72",
                text_soft: "rgba(92, 106, 114, 0.68)",
                amber: "#DFA000",
                amber_deep: "#F57D26",
                lime: "#8DA101",
                rose: "#F85552",
                blue: "#3A94C5",
                shadow: "0 20px 60px rgba(92, 106, 114, 0.12)",
                page_glow_primary: "rgba(245, 125, 38, 0.14)",
                page_glow_secondary: "rgba(58, 148, 197, 0.12)",
                page_gradient: "linear-gradient(160deg, #FDF6E3 0%, #F4F0D9 54%, #E6E2CC 100%)",
                panel_bg: "linear-gradient(160deg, rgba(255, 255, 255, 0.3), transparent 34%), rgba(253, 246, 227, 0.9)",
                banner_bg: "rgba(244, 240, 217, 0.94)",
                pill_bg: "rgba(255, 255, 255, 0.58)",
                pill_border: "rgba(130, 145, 129, 0.1)",
                ghost_bg: "rgba(255, 255, 255, 0.58)",
                ghost_border: "rgba(130, 145, 129, 0.12)",
                secondary_bg: "rgba(255, 255, 255, 0.74)",
                secondary_border: "rgba(58, 148, 197, 0.14)",
                quit_bg: "rgba(248, 85, 82, 0.08)",
                quit_border: "rgba(248, 85, 82, 0.16)",
                accent_text: "#FDF6E3",
                primary_shadow: "0 14px 30px rgba(245, 125, 38, 0.12)",
                scene_shell_bg: "rgba(255, 255, 255, 0.44)",
                room_shell_bg: "rgba(255, 255, 255, 0.58)",
                room_shell_border: "rgba(130, 145, 129, 0.08)",
                chip_bg: "rgba(255, 255, 255, 0.62)",
                chip_border: "rgba(130, 145, 129, 0.1)",
                slider_bg: "rgba(130, 145, 129, 0.12)",
                slider_fill: "linear-gradient(90deg, #DFA000, #3A94C5)",
            },
            (ThemePalette::RosePine, true) => ThemeTokens {
                color_scheme: "dark",
                bg: "#232136",
                bg_elevated: "rgba(35, 33, 54, 0.94)",
                bg_card: "rgba(42, 39, 63, 0.9)",
                line: "rgba(144, 140, 170, 0.22)",
                line_strong: "rgba(62, 143, 176, 0.28)",
                text: "#e0def4",
                text_soft: "rgba(224, 222, 244, 0.72)",
                amber: "#f6c177",
                amber_deep: "#ea9a97",
                lime: "#9ccfd8",
                rose: "#eb6f92",
                blue: "#3e8fb0",
                shadow: "0 24px 72px rgba(18, 17, 27, 0.36)",
                page_glow_primary: "rgba(246, 193, 119, 0.18)",
                page_glow_secondary: "rgba(62, 143, 176, 0.16)",
                page_gradient: "linear-gradient(160deg, #232136 0%, #1f1d2e 56%, #2a273f 100%)",
                panel_bg: "linear-gradient(160deg, rgba(224, 222, 244, 0.02), transparent 38%), rgba(31, 29, 46, 0.86)",
                banner_bg: "rgba(42, 39, 63, 0.88)",
                pill_bg: "rgba(57, 53, 82, 0.72)",
                pill_border: "rgba(144, 140, 170, 0.18)",
                ghost_bg: "rgba(57, 53, 82, 0.72)",
                ghost_border: "rgba(144, 140, 170, 0.16)",
                secondary_bg: "rgba(57, 53, 82, 0.78)",
                secondary_border: "rgba(62, 143, 176, 0.18)",
                quit_bg: "rgba(235, 111, 146, 0.12)",
                quit_border: "rgba(235, 111, 146, 0.2)",
                accent_text: "#232136",
                primary_shadow: "0 16px 32px rgba(246, 193, 119, 0.18)",
                scene_shell_bg: "rgba(57, 53, 82, 0.46)",
                room_shell_bg: "rgba(42, 39, 63, 0.74)",
                room_shell_border: "rgba(144, 140, 170, 0.1)",
                chip_bg: "rgba(57, 53, 82, 0.68)",
                chip_border: "rgba(144, 140, 170, 0.14)",
                slider_bg: "rgba(144, 140, 170, 0.18)",
                slider_fill: "linear-gradient(90deg, #f6c177, #3e8fb0)",
            },
            (ThemePalette::RosePine, false) => ThemeTokens {
                color_scheme: "light",
                bg: "#faf4ed",
                bg_elevated: "rgba(250, 244, 237, 0.96)",
                bg_card: "rgba(255, 250, 243, 0.92)",
                line: "rgba(121, 117, 147, 0.14)",
                line_strong: "rgba(40, 105, 131, 0.22)",
                text: "#464261",
                text_soft: "rgba(70, 66, 97, 0.68)",
                amber: "#ea9d34",
                amber_deep: "#d7827e",
                lime: "#56949f",
                rose: "#b4637a",
                blue: "#286983",
                shadow: "0 20px 60px rgba(70, 66, 97, 0.12)",
                page_glow_primary: "rgba(234, 157, 52, 0.14)",
                page_glow_secondary: "rgba(40, 105, 131, 0.12)",
                page_gradient: "linear-gradient(160deg, #faf4ed 0%, #fffaf3 54%, #f2e9e1 100%)",
                panel_bg: "linear-gradient(160deg, rgba(255, 255, 255, 0.3), transparent 34%), rgba(250, 244, 237, 0.9)",
                banner_bg: "rgba(255, 250, 243, 0.94)",
                pill_bg: "rgba(255, 255, 255, 0.58)",
                pill_border: "rgba(121, 117, 147, 0.1)",
                ghost_bg: "rgba(255, 255, 255, 0.58)",
                ghost_border: "rgba(121, 117, 147, 0.12)",
                secondary_bg: "rgba(255, 255, 255, 0.72)",
                secondary_border: "rgba(40, 105, 131, 0.14)",
                quit_bg: "rgba(180, 99, 122, 0.08)",
                quit_border: "rgba(180, 99, 122, 0.16)",
                accent_text: "#faf4ed",
                primary_shadow: "0 14px 30px rgba(215, 130, 126, 0.12)",
                scene_shell_bg: "rgba(255, 255, 255, 0.44)",
                room_shell_bg: "rgba(255, 255, 255, 0.58)",
                room_shell_border: "rgba(121, 117, 147, 0.08)",
                chip_bg: "rgba(255, 255, 255, 0.62)",
                chip_border: "rgba(121, 117, 147, 0.1)",
                slider_bg: "rgba(121, 117, 147, 0.12)",
                slider_fill: "linear-gradient(90deg, #ea9d34, #286983)",
            },
            (ThemePalette::Dayfox, _) => ThemeTokens {
                color_scheme: "light",
                bg: "#f6f2ee",
                bg_elevated: "rgba(246, 242, 238, 0.96)",
                bg_card: "rgba(228, 220, 212, 0.9)",
                line: "rgba(131, 122, 114, 0.14)",
                line_strong: "rgba(40, 121, 128, 0.22)",
                text: "#3d2b5a",
                text_soft: "rgba(61, 43, 90, 0.68)",
                amber: "#AC5402",
                amber_deep: "#955f61",
                lime: "#396847",
                rose: "#a5222f",
                blue: "#2848a9",
                shadow: "0 20px 60px rgba(53, 44, 36, 0.12)",
                page_glow_primary: "rgba(172, 84, 2, 0.14)",
                page_glow_secondary: "rgba(40, 72, 169, 0.1)",
                page_gradient: "linear-gradient(160deg, #f6f2ee 0%, #e4dcd4 54%, #dbd1dd 100%)",
                panel_bg: "linear-gradient(160deg, rgba(255, 255, 255, 0.34), transparent 34%), rgba(246, 242, 238, 0.9)",
                banner_bg: "rgba(228, 220, 212, 0.94)",
                pill_bg: "rgba(255, 255, 255, 0.58)",
                pill_border: "rgba(131, 122, 114, 0.1)",
                ghost_bg: "rgba(255, 255, 255, 0.58)",
                ghost_border: "rgba(131, 122, 114, 0.12)",
                secondary_bg: "rgba(255, 255, 255, 0.72)",
                secondary_border: "rgba(40, 121, 128, 0.14)",
                quit_bg: "rgba(165, 34, 47, 0.08)",
                quit_border: "rgba(165, 34, 47, 0.16)",
                accent_text: "#f6f2ee",
                primary_shadow: "0 14px 30px rgba(172, 84, 2, 0.12)",
                scene_shell_bg: "rgba(255, 255, 255, 0.44)",
                room_shell_bg: "rgba(255, 255, 255, 0.58)",
                room_shell_border: "rgba(131, 122, 114, 0.08)",
                chip_bg: "rgba(255, 255, 255, 0.62)",
                chip_border: "rgba(131, 122, 114, 0.1)",
                slider_bg: "rgba(131, 122, 114, 0.12)",
                slider_fill: "linear-gradient(90deg, #AC5402, #287980)",
            },
        }
    }
}

pub fn apply_theme_preference(preference: &ThemePreference) -> Result<(), String> {
    let tokens = preference.resolved_tokens();

    let global = js_sys::global();
    let window = js_sys::Reflect::get(&global, &JsValue::from_str("window")).map_err(js_error)?;
    let document =
        js_sys::Reflect::get(&window, &JsValue::from_str("document")).map_err(js_error)?;
    let root =
        js_sys::Reflect::get(&document, &JsValue::from_str("documentElement")).map_err(js_error)?;
    let style = js_sys::Reflect::get(&root, &JsValue::from_str("style")).map_err(js_error)?;
    let set_property = js_sys::Reflect::get(&style, &JsValue::from_str("setProperty"))
        .map_err(js_error)?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "documentElement.style.setProperty is unavailable".to_string())?;

    for (name, value) in [
        ("--bg", tokens.bg),
        ("--bg-elevated", tokens.bg_elevated),
        ("--bg-card", tokens.bg_card),
        ("--line", tokens.line),
        ("--line-strong", tokens.line_strong),
        ("--text", tokens.text),
        ("--text-soft", tokens.text_soft),
        ("--amber", tokens.amber),
        ("--amber-deep", tokens.amber_deep),
        ("--lime", tokens.lime),
        ("--rose", tokens.rose),
        ("--blue", tokens.blue),
        ("--shadow", tokens.shadow),
        ("--page-glow-primary", tokens.page_glow_primary),
        ("--page-glow-secondary", tokens.page_glow_secondary),
        ("--page-gradient", tokens.page_gradient),
        ("--panel-bg", tokens.panel_bg),
        ("--banner-bg", tokens.banner_bg),
        ("--pill-bg", tokens.pill_bg),
        ("--pill-border", tokens.pill_border),
        ("--ghost-bg", tokens.ghost_bg),
        ("--ghost-border", tokens.ghost_border),
        ("--secondary-bg", tokens.secondary_bg),
        ("--secondary-border", tokens.secondary_border),
        ("--quit-bg", tokens.quit_bg),
        ("--quit-border", tokens.quit_border),
        ("--accent-text", tokens.accent_text),
        ("--primary-shadow", tokens.primary_shadow),
        ("--scene-shell-bg", tokens.scene_shell_bg),
        ("--room-shell-bg", tokens.room_shell_bg),
        ("--room-shell-border", tokens.room_shell_border),
        ("--chip-bg", tokens.chip_bg),
        ("--chip-border", tokens.chip_border),
        ("--slider-bg", tokens.slider_bg),
        ("--slider-fill", tokens.slider_fill),
    ] {
        set_property
            .call2(&style, &JsValue::from_str(name), &JsValue::from_str(value))
            .map_err(js_error)?;
    }

    let dataset = js_sys::Reflect::get(&root, &JsValue::from_str("dataset")).map_err(js_error)?;
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("themePalette"),
        &JsValue::from_str(preference.palette.label()),
    )
    .map_err(js_error)?;
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("themeMode"),
        &JsValue::from_str(preference.mode.label()),
    )
    .map_err(js_error)?;
    js_sys::Reflect::set(
        &dataset,
        &JsValue::from_str("colorScheme"),
        &JsValue::from_str(tokens.color_scheme),
    )
    .map_err(js_error)?;

    set_property
        .call2(
            &style,
            &JsValue::from_str("color-scheme"),
            &JsValue::from_str(tokens.color_scheme),
        )
        .map_err(js_error)?;

    Ok(())
}

fn system_prefers_dark() -> bool {
    let global = js_sys::global();
    let Ok(window) = js_sys::Reflect::get(&global, &JsValue::from_str("window")) else {
        return true;
    };
    let Ok(match_media) = js_sys::Reflect::get(&window, &JsValue::from_str("matchMedia")) else {
        return true;
    };
    let Ok(function) = match_media.dyn_into::<js_sys::Function>() else {
        return true;
    };
    let Ok(result) = function.call1(&window, &JsValue::from_str("(prefers-color-scheme: dark)"))
    else {
        return true;
    };

    js_sys::Reflect::get(&result, &JsValue::from_str("matches"))
        .ok()
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

fn js_error(error: JsValue) -> String {
    if let Some(message) = error.as_string() {
        return message;
    }

    js_sys::JSON::stringify(&error)
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_else(|| "unknown JavaScript theme error".to_string())
}
