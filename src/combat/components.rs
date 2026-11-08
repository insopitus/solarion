//! ECS components, messages and run-scoped resources for combat.

use bevy::prelude::*;

use crate::core::stats::{AttackType, MAX_MANA, StatBlock, StatModifier};
use crate::data::equipment::Loadout;

// ---------------------------------------------------------------------------
// Markers
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct GameRoot;

#[derive(Component)]
pub struct Hero {
    pub index: usize,
}

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Boss;

#[derive(Component)]
pub struct Dead;

#[derive(Component)]
pub struct PartyAnchor;

#[derive(Component)]
pub struct MainCamera;

/// Marks the glTF model root of a hero and links it to the logical hero index.
#[derive(Component)]
pub struct HeroModel {
    pub hero_index: usize,
}

// ---------------------------------------------------------------------------
// Runtime state
// ---------------------------------------------------------------------------

/// Static identity used for attacks. Heroes also carry [`LevelProgress`].
#[derive(Component, Clone, Debug)]
pub struct CombatProfile {
    pub level: u32,
    pub attack_type: AttackType,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FinalStats(pub StatBlock);

#[derive(Component, Clone, Copy, Debug)]
pub struct Health {
    pub current: f32,
}

impl Health {
    pub fn new(max_health: f32) -> Self {
        Self {
            current: max_health.max(0.0),
        }
    }

    pub fn fraction(&self, max_health: f32) -> f32 {
        if max_health <= 0.0 {
            0.0
        } else {
            (self.current / max_health).clamp(0.0, 1.0)
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Mana {
    pub current: f32,
}

impl Default for Mana {
    fn default() -> Self {
        Self { current: MAX_MANA }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ControlMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Component, Clone, Debug)]
pub struct ActionState {
    pub action_timer: f32,
    pub cooldowns: [f32; 3],
    /// Skill index requested by a UI click, consumed by `hero_actions`.
    pub pending_skill: Option<usize>,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            action_timer: 0.0,
            cooldowns: [0.0; 3],
            pending_skill: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ActiveBuff {
    pub source: String,
    pub modifier: StatModifier,
    pub remaining: f32,
}

#[derive(Component, Clone, Debug, Default)]
pub struct ActiveBuffs {
    pub buffs: Vec<ActiveBuff>,
}

impl ActiveBuffs {
    pub fn has_source(&self, source: &str) -> bool {
        self.buffs.iter().any(|buff| buff.source == source)
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct Taunt {
    pub remaining: f32,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CombatRecord {
    pub damage_dealt: f32,
    pub healing_done: f32,
    pub damage_taken: f32,
}

#[derive(Component, Clone, Debug)]
pub struct HeroLoadout(pub Loadout);

/// Vertical offset from the entity origin to the point where floating combat
/// text should appear (roughly above the head).
#[derive(Component, Clone, Copy, Debug)]
pub struct CombatTextAnchor(pub f32);

#[derive(Component, Clone, Debug)]
pub struct EnemyAi {
    pub attack_timer: f32,
    pub attack_interval: f32,
    pub attack_multiplier: f32,
    pub move_speed: f32,
}

#[derive(Component, Clone, Debug)]
pub struct FloatingText {
    pub world_position: Vec3,
    /// Additional screen-space offset used to stack multi-hit numbers above
    /// the same target without moving them toward other characters.
    pub screen_offset: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub velocity: Vec3,
}

#[derive(Resource, Debug)]
pub struct StageProgress {
    pub wave_index: usize,
    pub active: bool,
    pub cleared_waves: usize,
    pub run_xp: u32,
    pub run_potions: u32,
}

impl Default for StageProgress {
    fn default() -> Self {
        Self {
            wave_index: 0,
            active: false,
            cleared_waves: 0,
            run_xp: 0,
            run_potions: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Message, Clone, Copy, Debug)]
pub struct DamageRequest {
    pub source: Entity,
    pub target: Entity,
    pub multiplier: f32,
    pub hits: u32,
    pub attack_type: AttackType,
    pub attacker_level: u32,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct HealRequest {
    pub source: Entity,
    pub target: Entity,
    pub multiplier: f32,
}

#[derive(Message, Clone, Debug)]
pub struct BuffRequest {
    pub source: String,
    pub target: Entity,
    pub modifiers: Vec<StatModifier>,
    pub duration: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatEventKind {
    DamageDealt,
    HealingDone,
}

#[derive(Message, Clone, Copy, Debug)]
pub struct CombatEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub kind: CombatEventKind,
}
