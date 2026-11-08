use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

/// Small serializable color used by the RON data files.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub fn color(self) -> Color {
        Color::srgb(self.r, self.g, self.b)
    }
}
