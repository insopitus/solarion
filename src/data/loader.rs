//! Loads all tunable game data from RON files at startup.
//!
//! Files are read from `assets/data/` relative to the current working
//! directory, with fallbacks for `cargo run` and for running the executable
//! from `target/debug`.

use std::fs;
use std::path::PathBuf;

use serde::de::DeserializeOwned;

use super::enemies::EnemyWaves;
use super::equipment::EquipmentConfig;
use super::heroes::{HeroRoster, SkillEffect};

pub struct LoadedGameData {
    pub heroes: HeroRoster,
    pub enemies: EnemyWaves,
    pub equipment: EquipmentConfig,
}

pub fn load_game_data() -> Result<LoadedGameData, String> {
    let heroes: HeroRoster = load_ron("data/heroes.ron")?;
    let enemies: EnemyWaves = load_ron("data/enemies.ron")?;
    let equipment: EquipmentConfig = load_ron("data/equipment.ron")?;

    if heroes.heroes.is_empty() {
        return Err("heroes.ron 中没有角色".to_string());
    }
    for hero in &heroes.heroes {
        if hero.attack_range <= 0.0 {
            return Err(format!("角色 {} 的普攻射程必须大于 0", hero.name));
        }
        for skill in &hero.skills {
            if skill.mana_cost <= 0.0 || skill.mana_cost % 100.0 != 0.0 {
                return Err(format!(
                    "角色 {} 的技能 {} 蓝耗必须是 100 的正整数倍",
                    hero.name, skill.name
                ));
            }

            let offensive = matches!(
                &skill.effect,
                SkillEffect::Damage { .. } | SkillEffect::Taunt { .. }
            );
            if offensive && skill.range <= 0.0 {
                return Err(format!(
                    "角色 {} 的进攻技能 {} 射程必须大于 0",
                    hero.name, skill.name
                ));
            }
        }
    }

    if enemies.waves.is_empty() {
        return Err("enemies.ron 中没有波次".to_string());
    }
    for (index, wave) in enemies.waves.iter().enumerate() {
        if wave.enemies.is_empty() {
            return Err(format!("第 {} 波没有敌人", index + 1));
        }
        if let Some(enemy) = wave.enemies.iter().find(|enemy| enemy.count == 0) {
            return Err(format!(
                "第 {} 波的 {} count 不能为 0",
                index + 1,
                enemy.name
            ));
        }
    }

    equipment.validate()?;

    Ok(LoadedGameData {
        heroes,
        enemies,
        equipment,
    })
}

fn load_ron<T: DeserializeOwned>(relative: &str) -> Result<T, String> {
    let path = assets_root().join(relative);

    let text = fs::read_to_string(&path)
        .map_err(|error| format!("读取 {} 失败：{error}", path.display()))?;

    ron::from_str(&text).map_err(|error| format!("解析 {} 失败：{error}", path.display()))
}

/// Resolves the project's `assets/` directory regardless of whether the game
/// was started with `cargo run` or by launching the executable directly.
pub fn assets_root() -> PathBuf {
    let mut candidates = vec![PathBuf::from("assets")];
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));

    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        candidates.push(dir.join("assets"));
        if let Some(parent) = dir.parent() {
            candidates.push(parent.join("assets"));
        }
    }

    let found = candidates
        .into_iter()
        .find(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from("assets"));

    // AssetPlugin joins this path with the executable directory, so make it
    // absolute to keep both `cargo run` and direct exe launches working.
    std::path::absolute(&found).unwrap_or(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::heroes::Role;

    #[test]
    fn shipped_data_parses_and_validates() {
        let data = load_game_data().expect("shipped RON data should parse");
        assert!(!data.heroes.heroes.is_empty());
        assert!(!data.enemies.waves.is_empty());
        data.equipment
            .validate()
            .expect("equipment config should be valid");
    }

    #[test]
    fn shipped_data_has_expected_structure() {
        let data = load_game_data().expect("shipped RON data should parse");

        let mut roles = [false; 4];
        for hero in &data.heroes.heroes {
            assert_eq!(hero.skills.len(), 3);
            for skill in &hero.skills {
                assert!(skill.mana_cost > 0.0);
                assert_eq!(skill.mana_cost % 100.0, 0.0);
            }
            match hero.role {
                Role::Tank => roles[0] = true,
                Role::Damage => roles[1] = true,
                Role::Support => roles[2] = true,
                Role::Healer => roles[3] = true,
            }
        }
        assert!(roles.iter().all(|covered| *covered));

        for wave in &data.enemies.waves {
            assert!(!wave.enemies.is_empty());
            assert!(wave.enemies.iter().all(|enemy| enemy.count > 0));
        }
        assert!(
            data.enemies
                .waves
                .last()
                .is_some_and(|wave| wave.enemies.iter().any(|enemy| enemy.is_boss))
        );
    }
}
