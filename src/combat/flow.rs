//! Stage progression: advancing between waves, camera follow and rewards.

use bevy::prelude::*;

use crate::GameState;
use crate::data::enemies::EnemyWaves;
use crate::progression::{LevelProgress, POTIONS_PER_STAGE, Progression, add_experience};

use super::components::{Dead, Enemy, Hero, MainCamera, PartyAnchor, StageProgress};
use super::setup::{PARTY_SPEED, camera_position, spawn_wave, wave_trigger_x};

pub fn wave_flow(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stage: ResMut<StageProgress>,
    mut next_state: ResMut<NextState<GameState>>,
    mut progression: ResMut<Progression>,
    waves: Res<EnemyWaves>,
    mut hero_levels: Query<&mut LevelProgress, With<Hero>>,
    mut anchors: Query<&mut Transform, With<PartyAnchor>>,
    enemies: Query<(), (With<Enemy>, Without<Dead>)>,
) {
    let wave_count = waves.len();
    if stage.wave_index >= wave_count {
        return;
    }

    if stage.active {
        if enemies.is_empty() {
            info!("第 {} 波清除", stage.wave_index + 1);
            stage.active = false;
            stage.cleared_waves += 1;
            stage.wave_index += 1;

            if stage.wave_index >= wave_count {
                stage.run_xp = waves
                    .waves
                    .iter()
                    .flat_map(|wave| &wave.enemies)
                    .map(|enemy| enemy.xp_reward * enemy.count)
                    .sum();
                stage.run_potions = POTIONS_PER_STAGE;

                for mut progress in &mut hero_levels {
                    add_experience(&mut progress, stage.run_xp);
                }
                progression.potions = progression.potions.saturating_add(stage.run_potions);

                info!(
                    "关卡通关：全队获得 {} 经验，经验药水 x{}（库存 {}）",
                    stage.run_xp, stage.run_potions, progression.potions
                );
                next_state.set(GameState::Victory);
            }
        }
        return;
    }

    let Ok(mut anchor) = anchors.single_mut() else {
        return;
    };

    anchor.translation.x += PARTY_SPEED * time.delta_secs();

    if anchor.translation.x >= wave_trigger_x(stage.wave_index) {
        stage.active = true;
        info!(
            "第 {} 波开始：{}",
            stage.wave_index + 1,
            waves.name(stage.wave_index)
        );
        spawn_wave(
            &mut commands,
            &mut meshes,
            &mut materials,
            &waves.wave(stage.wave_index).enemies,
            anchor.translation.x,
        );
    }
}

pub fn follow_camera(
    time: Res<Time>,
    anchors: Query<&GlobalTransform, With<PartyAnchor>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(anchor) = anchors.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let anchor_x = anchor.translation().x;
    let desired = camera_position(anchor_x);
    let alpha = 1.0 - (-6.0 * time.delta_secs()).exp();
    // The orientation is fixed once in `setup_scene`; following the party in X
    // keeps the exact same 45-degree view without re-aiming every frame.
    camera.translation = camera.translation.lerp(desired, alpha);
}

pub fn restart_on_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
}
