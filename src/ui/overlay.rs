//! Victory / defeat result overlay.

use bevy::prelude::*;

use crate::GameState;
use crate::combat::components::StageProgress;
use crate::progression::Progression;

use super::theme::{gold, panel_bg, text_muted, text_primary, ui_text};

#[derive(Component)]
struct OverlayRoot;

pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Victory), spawn_victory)
            .add_systems(OnEnter(GameState::Defeat), spawn_defeat)
            .add_systems(OnExit(GameState::Victory), despawn_overlay)
            .add_systems(OnExit(GameState::Defeat), despawn_overlay);
    }
}

fn spawn_victory(mut commands: Commands, stage: Res<StageProgress>, progression: Res<Progression>) {
    spawn_overlay(
        &mut commands,
        "胜  利",
        gold(),
        vec![
            "关卡通关！".to_string(),
            format!("全队获得 {} 点经验", stage.run_xp),
            format!(
                "获得经验药水 x{}（库存 {}）",
                stage.run_potions, progression.potions
            ),
        ],
    );
}

fn spawn_defeat(mut commands: Commands) {
    spawn_overlay(
        &mut commands,
        "全队阵亡",
        Color::srgb(0.95, 0.35, 0.32),
        vec![
            "冒险暂时失败了。".to_string(),
            "调整装备与技能配置后再来一次。".to_string(),
        ],
    );
}

fn despawn_overlay(mut commands: Commands, overlays: Query<Entity, With<OverlayRoot>>) {
    for entity in &overlays {
        commands.entity(entity).despawn();
    }
}

fn spawn_overlay(commands: &mut Commands, title: &str, title_color: Color, lines: Vec<String>) {
    commands
        .spawn((
            OverlayRoot,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.58)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(10),
                    padding: UiRect::all(px(32)),
                    border_radius: BorderRadius::all(px(14)),
                    ..default()
                },
                BackgroundColor(panel_bg()),
            ))
            .with_children(|card| {
                card.spawn(ui_text(title, 44.0, title_color));

                for line in lines {
                    card.spawn(ui_text(line, 18.0, text_primary()));
                }

                card.spawn(ui_text("按 R 重新开始", 16.0, text_muted()));
            });
        });
}
