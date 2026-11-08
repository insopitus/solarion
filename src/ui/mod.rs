pub mod cooldown;
pub mod hud;
pub mod overlay;
pub mod stats_panel;
pub mod theme;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            cooldown::SkillCooldownPlugin,
            hud::HudPlugin,
            stats_panel::StatsPanelPlugin,
            overlay::OverlayPlugin,
        ));
    }
}
