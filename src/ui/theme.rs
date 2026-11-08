//! Shared UI palette, font and node helpers.

use bevy::prelude::*;
use bevy::text::{FontSize, FontSource};

/// Prefer a CJK-capable system font, falling back to the generic sans family.
/// This requires Bevy's `system_font_discovery` feature (enabled in Cargo.toml).
pub fn ui_font() -> FontSource {
    FontSource::Families(
        "Microsoft YaHei, Microsoft YaHei UI, Noto Sans CJK SC, Noto Sans SC, Segoe UI, sans-serif"
            .into(),
    )
}

pub fn text_font(size: f32) -> TextFont {
    TextFont {
        font: ui_font(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// UI text bundle. `no_wrap` avoids the expensive CJK word-segmentation path
/// for these short, hand-authored labels.
#[derive(Bundle)]
pub struct ThemedText {
    pub text: Text,
    pub font: TextFont,
    pub color: TextColor,
    pub layout: TextLayout,
}

pub fn ui_text(content: impl Into<String>, size: f32, color: Color) -> ThemedText {
    ThemedText {
        text: Text::new(content.into()),
        font: text_font(size),
        color: TextColor(color),
        layout: TextLayout::no_wrap(),
    }
}

pub fn panel_bg() -> Color {
    Color::srgba(0.015, 0.020, 0.035, 0.82)
}

pub fn panel_soft_bg() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.045)
}

pub fn text_primary() -> Color {
    Color::srgb(0.93, 0.94, 0.97)
}

pub fn text_muted() -> Color {
    Color::srgb(0.62, 0.66, 0.75)
}

pub fn gold() -> Color {
    Color::srgb(1.0, 0.80, 0.32)
}

pub fn health_bar() -> Color {
    Color::srgb(0.35, 0.85, 0.50)
}

pub fn health_bar_low() -> Color {
    Color::srgb(0.92, 0.35, 0.32)
}

pub fn mana_filled() -> Color {
    Color::srgb(0.38, 0.62, 1.0)
}

/// The partially filled segment of the 5-segment mana bar.
pub fn mana_partial() -> Color {
    Color::srgba(0.20, 0.32, 0.66, 0.95)
}

pub fn mana_empty() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.10)
}

pub fn skill_slot() -> Color {
    Color::srgba(0.10, 0.22, 0.44, 0.72)
}

pub fn tab_active() -> Color {
    Color::srgba(0.58, 0.44, 0.13, 0.92)
}

pub fn tab_hover() -> Color {
    Color::srgba(1.0, 1.0, 1.0, 0.14)
}

pub fn boss_bar_fill() -> Color {
    Color::srgb(0.92, 0.16, 0.16)
}

pub fn stats_damage() -> Color {
    Color::srgb(0.95, 0.36, 0.30)
}

pub fn stats_healing() -> Color {
    Color::srgb(0.35, 0.85, 0.50)
}

pub fn stats_taken() -> Color {
    Color::srgb(0.95, 0.65, 0.30)
}

pub fn format_number(value: f32) -> String {
    let rounded = value.round().max(0.0) as u64;
    let digits = rounded.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);

    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}
