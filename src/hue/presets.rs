use crate::hue::LightStateUpdate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CuratedScenePreset {
    pub name: &'static str,
    pub description: &'static str,
}

const CURATED_ROOM_SCENES: [CuratedScenePreset; 4] = [
    CuratedScenePreset {
        name: "Quiet Focus",
        description: "clean working light with low color noise",
    },
    CuratedScenePreset {
        name: "Soft Read",
        description: "warm reading light with less glare",
    },
    CuratedScenePreset {
        name: "Night Wind-Down",
        description: "very warm low light for late hours",
    },
    CuratedScenePreset {
        name: "Signal Boost",
        description: "full-bright utility light for resets and cleanup",
    },
];

pub fn curated_room_scenes() -> &'static [CuratedScenePreset] {
    &CURATED_ROOM_SCENES
}

pub fn preset_light_state(
    preset: CuratedScenePreset,
    light_index: usize,
    light_count: usize,
) -> LightStateUpdate {
    let spread = light_offset(light_index, light_count);

    match preset.name {
        "Quiet Focus" => LightStateUpdate {
            on: Some(true),
            brightness: Some((208_i16 + spread / 2).clamp(140, 254) as u8),
            saturation: Some((38_i16 + spread.abs() / 3).clamp(8, 90) as u8),
            hue: Some((8_700_i32 + i32::from(spread) * 36).clamp(0, 65_535) as u16),
            transition_time: Some(4),
        },
        "Soft Read" => LightStateUpdate {
            on: Some(true),
            brightness: Some((174_i16 + spread / 2).clamp(90, 254) as u8),
            saturation: Some((118_i16 + spread.abs() / 2).clamp(40, 180) as u8),
            hue: Some((7_800_i32 + i32::from(spread) * 28).clamp(0, 65_535) as u16),
            transition_time: Some(4),
        },
        "Night Wind-Down" => LightStateUpdate {
            on: Some(true),
            brightness: Some((44_i16 + spread / 4).clamp(8, 90) as u8),
            saturation: Some((150_i16 + spread.abs() / 2).clamp(80, 220) as u8),
            hue: Some((6_900_i32 + i32::from(spread) * 24).clamp(0, 65_535) as u16),
            transition_time: Some(6),
        },
        "Signal Boost" => LightStateUpdate {
            on: Some(true),
            brightness: Some((248_i16 - spread.abs() / 3).clamp(200, 254) as u8),
            saturation: Some((18_i16 + spread.abs() / 4).clamp(0, 60) as u8),
            hue: Some((9_400_i32 + i32::from(spread) * 20).clamp(0, 65_535) as u16),
            transition_time: Some(3),
        },
        _ => LightStateUpdate {
            on: Some(true),
            brightness: Some(180),
            saturation: Some(60),
            hue: Some(8_600),
            transition_time: Some(4),
        },
    }
}

fn light_offset(light_index: usize, light_count: usize) -> i16 {
    if light_count <= 1 {
        return 0;
    }

    let center = (light_count.saturating_sub(1)) as i16;
    ((light_index as i16 * 2) - center) * 10
}

#[cfg(test)]
mod tests {
    use super::{curated_room_scenes, preset_light_state};

    #[test]
    fn curated_scene_pack_has_expected_defaults() {
        let scenes = curated_room_scenes();
        assert_eq!(scenes.len(), 4);
        assert_eq!(scenes[0].name, "Quiet Focus");
        assert_eq!(scenes[3].name, "Signal Boost");
    }

    #[test]
    fn generated_states_stay_within_hue_ranges() {
        for preset in curated_room_scenes() {
            for index in 0..4 {
                let state = preset_light_state(*preset, index, 4);
                assert!(state.on.unwrap_or(false));
                assert!((1..=254).contains(&state.brightness.unwrap_or_default()));
                assert!(state.saturation.unwrap_or_default() <= 254);
                assert!(state.hue.is_some());
            }
        }
    }
}
