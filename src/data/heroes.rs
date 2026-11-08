//! The four playable characters, their passive and three active skills.
//!
//! Skills are data-driven: the combat layer only understands
//! [`SkillEffect`], so adding a character is mostly a matter of adding data.
//! The shipped data lives in `assets/data/heroes.ron`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::color::Rgb;
use crate::core::stats::{AttackType, BaseAttributes, StatModifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Tank,
    Damage,
    Support,
    Healer,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Self::Tank => "坦克",
            Self::Damage => "输出",
            Self::Support => "辅助",
            Self::Healer => "治疗",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageTarget {
    NearestEnemy,
    AllEnemies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealTarget {
    LowestHealthAlly,
    AllAllies,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffTarget {
    SelfOnly,
    AllAllies,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SkillEffect {
    Damage {
        multiplier: f32,
        hits: u32,
        target: DamageTarget,
    },
    Heal {
        multiplier: f32,
        target: HealTarget,
    },
    Buff {
        modifiers: Vec<StatModifier>,
        duration: f32,
        target: BuffTarget,
    },
    Taunt {
        duration: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    /// Asset path relative to `assets/`. Empty means no icon.
    #[serde(default)]
    pub icon: String,
    /// Maximum distance to an enemy for offensive skills. 0 means no limit
    /// (used by ally-targeted buffs and heals).
    #[serde(default)]
    pub range: f32,
    /// Always a multiple of 100 (one mana segment).
    pub mana_cost: f32,
    pub cooldown: f32,
    /// Higher priority skills are cast first when several are ready.
    pub priority: i32,
    /// Optional smart-cast gate: only cast if an ally is below this HP ratio.
    pub ally_health_below: Option<f32>,
    pub effect: SkillEffect,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeroDefinition {
    pub name: String,
    pub title: String,
    pub role: Role,
    pub attack_type: AttackType,
    /// Range used by auto attacks.
    #[serde(default = "default_attack_range")]
    pub attack_range: f32,
    /// Optional glTF scene relative to `assets/`. Empty falls back to a capsule.
    #[serde(default)]
    pub model: String,
    pub base: BaseAttributes,
    pub passive_name: String,
    pub passive_description: String,
    pub passive_modifiers: Vec<StatModifier>,
    pub skills: [SkillDefinition; 3],
    pub color: Rgb,
}

fn default_attack_range() -> f32 {
    3.0
}

#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct HeroRoster {
    pub heroes: Vec<HeroDefinition>,
}

impl HeroRoster {
    pub fn hero(&self, index: usize) -> &HeroDefinition {
        &self.heroes[index]
    }

    pub fn len(&self) -> usize {
        self.heroes.len()
    }
}
