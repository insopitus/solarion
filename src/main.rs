mod combat;
mod core;
mod data;
mod progression;
mod ui;

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::WindowResolution;

use combat::CombatPlugin;
use core::rng::GameRng;
use progression::Progression;
use ui::UiPlugin;

#[derive(States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    Victory,
    Defeat,
}

fn main() {
    let data = data::load_game_data().unwrap_or_else(|error| {
        eprintln!("启动失败：{error}");
        std::process::exit(1);
    });

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: data::assets_root().to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Solarion - 剑与魔法远征".to_string(),
                        resolution: WindowResolution::new(1280, 720),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.070)))
        .insert_resource(GameRng::default())
        .insert_resource(data.heroes)
        .insert_resource(data.enemies)
        .insert_resource(data.equipment)
        .init_resource::<Progression>()
        .init_state::<GameState>()
        .add_plugins((CombatPlugin, UiPlugin))
        .run();
}
