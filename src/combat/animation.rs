//! Loads the hero glTF animation graph and drives idle / move / combat clips.

use bevy::animation::RepeatAnimation;
use bevy::animation::prelude::*;
use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::core::stats::AttackType;
use crate::data::heroes::HeroRoster;

use super::components::{ActionState, HeroModel, StageProgress};

/// The X Bot character converted from the Sword and Shield Pack, with the
/// idle / walk / attack / casting / death clips merged into one glb.
pub const HERO_MODEL: &str = "models/heroes/xbot_animated.glb";

#[derive(Resource)]
pub struct HeroAnimationClips {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub casting: AnimationNodeIndex,
}

/// Added to a model root once its `AnimationPlayer` is wired up.
#[derive(Component)]
pub struct HeroAnimationReady;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HeroAnimMode {
    #[default]
    Idle,
    Move,
    Attack,
    Cast,
}

#[derive(Component)]
pub struct HeroAnimationState {
    pub player: Entity,
    pub mode: HeroAnimMode,
    pub action_remaining: f32,
    pub last_action_timer: f32,
}

pub fn setup_animation_graph(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    existing: Option<Res<HeroAnimationClips>>,
) {
    if existing.is_some() {
        return;
    }

    // The merged glb stores clips in this order; labels load one clip at a
    // time, which avoids waiting for the whole `Gltf` asset to appear.
    let clip =
        |index: usize| asset_server.load(GltfAssetLabel::Animation(index).from_asset(HERO_MODEL));

    let mut graph = AnimationGraph::new();
    let root = graph.root;
    let idle = graph.add_clip(clip(0), 1.0, root);
    let run = graph.add_clip(clip(2), 1.0, root);
    let attack = graph.add_clip(clip(3), 1.0, root);
    let casting = graph.add_clip(clip(5), 1.0, root);

    commands.insert_resource(HeroAnimationClips {
        graph: graphs.add(graph),
        idle,
        run,
        attack,
        casting,
    });
}

pub fn attach_hero_animations(
    mut commands: Commands,
    clips: Option<Res<HeroAnimationClips>>,
    models: Query<(Entity, &HeroModel), Without<HeroAnimationReady>>,
    children: Query<&Children>,
    player_entities: Query<Entity, With<AnimationPlayer>>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let Some(clips) = clips else {
        return;
    };

    for (model_entity, _) in &models {
        let Some(player_entity) = find_animation_player(model_entity, &children, &player_entities)
        else {
            continue;
        };
        let Ok(mut player) = players.get_mut(player_entity) else {
            continue;
        };

        player.play(clips.idle).set_repeat(RepeatAnimation::Forever);
        commands
            .entity(player_entity)
            .insert(AnimationGraphHandle(clips.graph.clone()));
        commands.entity(model_entity).insert((
            HeroAnimationReady,
            HeroAnimationState {
                player: player_entity,
                mode: HeroAnimMode::Idle,
                action_remaining: 0.0,
                last_action_timer: 0.0,
            },
        ));
    }
}

pub fn update_hero_animations(
    time: Res<Time>,
    stage: Res<StageProgress>,
    roster: Res<HeroRoster>,
    clips: Option<Res<HeroAnimationClips>>,
    mut models: Query<(&HeroModel, &ChildOf, &mut HeroAnimationState)>,
    heroes: Query<&ActionState>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let Some(clips) = clips else {
        return;
    };
    let delta = time.delta_secs();

    for (model, parent, mut state) in &mut models {
        let Ok(mut player) = players.get_mut(state.player) else {
            continue;
        };
        let Ok(action) = heroes.get(parent.parent()) else {
            continue;
        };

        if state.action_remaining > 0.0 {
            state.action_remaining = (state.action_remaining - delta).max(0.0);
        }

        // `action_timer` counts down after an action, so only a jump upwards
        // marks a genuinely new attack or cast.
        let new_action = action.action_timer > state.last_action_timer + 0.01;
        state.last_action_timer = action.action_timer;

        // A fresh action wins over idle/move. The exact skill is not exposed
        // to the animation layer, so physical heroes use the slash clip and
        // magic users use the casting clip.
        if state.action_remaining <= 0.0 && new_action {
            let magic = roster.hero(model.hero_index).attack_type == AttackType::Magic;
            let (node, mode, duration) = if magic {
                (clips.casting, HeroAnimMode::Cast, 1.4)
            } else {
                (clips.attack, HeroAnimMode::Attack, 1.1)
            };
            state.mode = mode;
            state.action_remaining = duration;
            player.stop_all();
            player.play(node).set_repeat(RepeatAnimation::Never);
            continue;
        }

        if state.action_remaining > 0.0 {
            continue;
        }

        let base = if stage.active {
            HeroAnimMode::Idle
        } else {
            HeroAnimMode::Move
        };
        if state.mode != base {
            state.mode = base;
            player.stop_all();
            let node = if base == HeroAnimMode::Move {
                clips.run
            } else {
                clips.idle
            };
            player.play(node).set_repeat(RepeatAnimation::Forever);
        }
    }
}

fn find_animation_player(
    root: Entity,
    children: &Query<&Children>,
    players: &Query<Entity, With<AnimationPlayer>>,
) -> Option<Entity> {
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if players.contains(entity) {
            return Some(entity);
        }
        if let Ok(child_entities) = children.get(entity) {
            stack.extend(child_entities.iter());
        }
    }
    None
}
