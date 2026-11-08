//! Enemy and wave definitions, loaded from `assets/data/enemies.ron`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::color::Rgb;
use crate::core::stats::{AttackType, BaseAttributes};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyDefinition {
    pub name: String,
    /// How many copies of this enemy are spawned in the wave.
    #[serde(default = "default_count")]
    pub count: u32,
    pub level: u32,
    pub attack_type: AttackType,
    pub base: BaseAttributes,
    pub is_boss: bool,
    pub attack_multiplier: f32,
    pub attack_interval: f32,
    pub move_speed: f32,
    pub xp_reward: u32,
    pub color: Rgb,
    pub scale: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyWave {
    pub name: String,
    pub enemies: Vec<EnemyDefinition>,
}

#[derive(Resource, Debug, Serialize, Deserialize)]
pub struct EnemyWaves {
    pub waves: Vec<EnemyWave>,
}

impl EnemyWaves {
    pub fn len(&self) -> usize {
        self.waves.len()
    }

    pub fn wave(&self, index: usize) -> &EnemyWave {
        &self.waves[index]
    }

    pub fn name(&self, index: usize) -> &str {
        &self.waves[index].name
    }
}

fn default_count() -> u32 {
    1
}
