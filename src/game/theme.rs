//! UI 与场景名牌主题。

use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct SceneLabelStyle {
    pub font_size: f32,
    pub text: Color,
    pub background: Color,
    pub shadow: Color,
    pub shadow_offset: Vec2,
    pub offset: Vec2,
}

#[derive(Resource, Debug, Clone)]
pub struct UiTheme {
    pub plant_label: SceneLabelStyle,
    pub zombie_label: SceneLabelStyle,
    pub sun_label: SceneLabelStyle,
    pub hud_text: Color,
    pub hud_panel_background: Color,
    pub hud_panel_border: Color,
    pub help_background: Color,
    pub help_border: Color,
    pub help_text: Color,
    pub card_text: Color,
    pub card_selected_text: Color,
    pub card_disabled_text: Color,
    pub card_background: Color,
    pub card_selected_background: Color,
    pub card_disabled_background: Color,
    pub card_border: Color,
    pub card_selected_border: Color,
    pub victory_text: Color,
    pub defeat_text: Color,
    pub result_background: Color,
    pub result_subtitle: Color,
    pub sun_color: Color,
    pub sun_size: f32,
    pub hud_font_size: f32,
    pub card_font_size: f32,
    pub help_font_size: f32,
    pub result_title_size: f32,
    pub result_subtitle_size: f32,
    pub panel_radius: f32,
    pub panel_gap: f32,
    pub card_size: Vec2,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            plant_label: SceneLabelStyle {
                font_size: 16.0,
                text: Color::srgb(1.0, 0.98, 0.88),
                background: Color::srgba(0.05, 0.08, 0.04, 0.72),
                shadow: Color::srgba(0.0, 0.0, 0.0, 0.9),
                shadow_offset: Vec2::new(1.5, -1.5),
                offset: Vec2::new(0.0, -3.0),
            },
            zombie_label: SceneLabelStyle {
                font_size: 17.0,
                text: Color::srgb(1.0, 0.96, 0.88),
                background: Color::srgba(0.11, 0.08, 0.05, 0.76),
                shadow: Color::BLACK,
                shadow_offset: Vec2::new(1.5, -1.5),
                offset: Vec2::new(0.0, -4.0),
            },
            sun_label: SceneLabelStyle {
                font_size: 13.0,
                text: Color::srgb(0.22, 0.12, 0.01),
                background: Color::srgba(1.0, 0.94, 0.55, 0.88),
                shadow: Color::srgba(1.0, 1.0, 1.0, 0.65),
                shadow_offset: Vec2::new(1.0, -1.0),
                offset: Vec2::new(0.0, -28.0),
            },
            hud_text: Color::srgb(0.96, 0.98, 0.88),
            hud_panel_background: Color::srgba(0.035, 0.09, 0.045, 0.91),
            hud_panel_border: Color::srgba(0.68, 0.88, 0.43, 0.45),
            help_background: Color::srgba(0.035, 0.055, 0.035, 0.88),
            help_border: Color::srgba(0.75, 0.88, 0.62, 0.28),
            help_text: Color::srgb(0.88, 0.94, 0.83),
            card_text: Color::srgb(0.88, 0.92, 0.80),
            card_selected_text: Color::srgb(1.0, 0.95, 0.64),
            card_disabled_text: Color::srgb(0.60, 0.64, 0.56),
            card_background: Color::srgba(0.08, 0.14, 0.07, 0.92),
            card_selected_background: Color::srgba(0.22, 0.30, 0.08, 0.96),
            card_disabled_background: Color::srgba(0.075, 0.085, 0.065, 0.90),
            card_border: Color::srgba(1.0, 1.0, 1.0, 0.16),
            card_selected_border: Color::srgb(1.0, 0.78, 0.24),
            victory_text: Color::srgb(0.48, 0.96, 0.36),
            defeat_text: Color::srgb(1.0, 0.38, 0.28),
            result_background: Color::srgba(0.02, 0.035, 0.02, 0.93),
            result_subtitle: Color::srgb(0.94, 0.96, 0.90),
            sun_color: Color::srgb(1.0, 0.86, 0.15),
            sun_size: 34.0,
            hud_font_size: 18.0,
            card_font_size: 14.0,
            help_font_size: 15.0,
            result_title_size: 76.0,
            result_subtitle_size: 24.0,
            panel_radius: 10.0,
            panel_gap: 6.0,
            card_size: Vec2::splat(88.0),
        }
    }
}
