pub mod actions;
pub mod animation;
pub mod components;
pub mod damage;
pub mod effects;
pub mod flow;
pub mod setup;

use bevy::prelude::*;

use crate::GameState;

use components::{BuffRequest, CombatEvent, DamageRequest, HealRequest, StageProgress};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StageProgress>()
            .add_message::<DamageRequest>()
            .add_message::<HealRequest>()
            .add_message::<BuffRequest>()
            .add_message::<CombatEvent>()
            .add_systems(Startup, setup::setup_scene)
            .add_systems(
                OnEnter(GameState::Playing),
                (setup::cleanup_run, setup::start_run).chain(),
            )
            .add_systems(
                Update,
                (
                    animation::setup_animation_graph,
                    animation::attach_hero_animations,
                    actions::tick_buffs,
                    actions::recompute_hero_stats,
                    actions::regenerate_mana,
                    actions::hero_actions,
                    actions::enemy_actions,
                    animation::update_hero_animations,
                    damage::resolve_damage,
                    damage::resolve_heals,
                    damage::apply_buffs,
                    damage::record_combat,
                    damage::reap_dead,
                    damage::check_defeat,
                    flow::wave_flow,
                    flow::follow_camera,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                flow::restart_on_key
                    .run_if(in_state(GameState::Victory).or_else(in_state(GameState::Defeat))),
            )
            .add_systems(Update, effects::update_floating_texts);
    }
}
