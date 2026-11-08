//! Combat attributes and the damage model.
//!
//! The formulas here mirror `GAME_SETTINGS.md`:
//! - mitigation is `defense / (K + defense)`, where `K` depends on the
//!   attacker's level,
//! - base crit damage is 200%,
//! - percentage substats are summed first, then applied multiplicatively,
//! - crit / attack speed / dodge / penetration are additive percentage points.

use serde::{Deserialize, Serialize};

use super::rng::GameRng;

pub const MAX_MANA: f32 = 500.0;
pub const MANA_PER_SEGMENT: f32 = 100.0;
pub const BASE_CRIT_MULTIPLIER: f32 = 2.0;
/// Seconds between actions before attack speed is applied.
pub const BASE_ACTION_INTERVAL: f32 = 1.5;
/// Mana restored by an auto attack.
pub const MANA_ON_BASIC_ATTACK: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttackType {
    Physical,
    Magic,
}

impl AttackType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Physical => "物理",
            Self::Magic => "魔法",
        }
    }
}

/// Level-1 values before equipment, passives and buffs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BaseAttributes {
    pub attack: f32,
    pub max_health: f32,
    pub physical_defense: f32,
    pub magic_defense: f32,
    pub crit_chance: f32,
    pub crit_damage_bonus: f32,
    pub defense_penetration: f32,
    pub attack_speed: f32,
    pub dodge_chance: f32,
    pub healing_bonus: f32,
    pub mana_regen: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatKind {
    Attack,
    AttackPercent,
    MaxHealth,
    MaxHealthPercent,
    PhysicalDefense,
    PhysicalDefensePercent,
    MagicDefense,
    MagicDefensePercent,
    CritChance,
    CritDamage,
    DefensePenetration,
    AttackSpeed,
    DodgeChance,
    HealingBonus,
    ManaRegen,
}

impl StatKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Attack => "攻击力",
            Self::AttackPercent => "攻击力%",
            Self::MaxHealth => "生命上限",
            Self::MaxHealthPercent => "生命上限%",
            Self::PhysicalDefense => "物理防御",
            Self::PhysicalDefensePercent => "物理防御%",
            Self::MagicDefense => "魔法防御",
            Self::MagicDefensePercent => "魔法防御%",
            Self::CritChance => "暴击率",
            Self::CritDamage => "暴击伤害",
            Self::DefensePenetration => "防御穿透",
            Self::AttackSpeed => "攻击速度",
            Self::DodgeChance => "闪避率",
            Self::HealingBonus => "治疗加成",
            Self::ManaRegen => "法力回复",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatModifier {
    pub kind: StatKind,
    pub value: f32,
}

impl StatModifier {
    pub const fn new(kind: StatKind, value: f32) -> Self {
        Self { kind, value }
    }
}

/// Fully resolved combat stats for one entity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StatBlock {
    pub attack: f32,
    pub max_health: f32,
    pub physical_defense: f32,
    pub magic_defense: f32,
    pub crit_chance: f32,
    pub crit_damage_bonus: f32,
    pub defense_penetration: f32,
    pub attack_speed: f32,
    pub dodge_chance: f32,
    pub healing_bonus: f32,
    pub mana_regen: f32,
}

impl StatBlock {
    /// Applies flat modifiers first, then all percentage modifiers as a single
    /// summed multiplier. Additive percentage stats are simply summed.
    pub fn compute(base: &BaseAttributes, modifiers: &[StatModifier]) -> Self {
        let mut flat_attack = 0.0;
        let mut pct_attack = 0.0;
        let mut flat_health = 0.0;
        let mut pct_health = 0.0;
        let mut flat_physical = 0.0;
        let mut pct_physical = 0.0;
        let mut flat_magic = 0.0;
        let mut pct_magic = 0.0;

        let mut crit_chance = base.crit_chance;
        let mut crit_damage = base.crit_damage_bonus;
        let mut penetration = base.defense_penetration;
        let mut attack_speed = base.attack_speed;
        let mut dodge = base.dodge_chance;
        let mut healing = base.healing_bonus;
        let mut mana_regen = base.mana_regen;

        for modifier in modifiers {
            match modifier.kind {
                StatKind::Attack => flat_attack += modifier.value,
                StatKind::AttackPercent => pct_attack += modifier.value,
                StatKind::MaxHealth => flat_health += modifier.value,
                StatKind::MaxHealthPercent => pct_health += modifier.value,
                StatKind::PhysicalDefense => flat_physical += modifier.value,
                StatKind::PhysicalDefensePercent => pct_physical += modifier.value,
                StatKind::MagicDefense => flat_magic += modifier.value,
                StatKind::MagicDefensePercent => pct_magic += modifier.value,
                StatKind::CritChance => crit_chance += modifier.value,
                StatKind::CritDamage => crit_damage += modifier.value,
                StatKind::DefensePenetration => penetration += modifier.value,
                StatKind::AttackSpeed => attack_speed += modifier.value,
                StatKind::DodgeChance => dodge += modifier.value,
                StatKind::HealingBonus => healing += modifier.value,
                StatKind::ManaRegen => mana_regen += modifier.value,
            }
        }

        Self {
            attack: (base.attack + flat_attack).max(0.0) * (1.0 + pct_attack).max(0.0),
            max_health: (base.max_health + flat_health).max(1.0) * (1.0 + pct_health).max(0.0),
            physical_defense: (base.physical_defense + flat_physical).max(0.0)
                * (1.0 + pct_physical).max(0.0),
            magic_defense: (base.magic_defense + flat_magic).max(0.0) * (1.0 + pct_magic).max(0.0),
            crit_chance: crit_chance.clamp(0.0, 1.0),
            crit_damage_bonus: crit_damage.max(0.0),
            defense_penetration: penetration.max(0.0),
            attack_speed: attack_speed.max(0.0),
            dodge_chance: dodge.clamp(0.0, 0.9),
            healing_bonus: healing.max(0.0),
            mana_regen: mana_regen.max(0.0),
        }
    }

    pub fn defense_against(&self, attack_type: AttackType) -> f32 {
        match attack_type {
            AttackType::Physical => self.physical_defense,
            AttackType::Magic => self.magic_defense,
        }
    }

    pub fn crit_multiplier(&self) -> f32 {
        BASE_CRIT_MULTIPLIER + self.crit_damage_bonus
    }

    pub fn action_interval(&self) -> f32 {
        (BASE_ACTION_INTERVAL / (1.0 + self.attack_speed)).max(0.25)
    }
}

/// Grows the four "core" stats with level. Percent stats stay flat until they
/// are modified by gear, passives or buffs.
pub fn base_at_level(base: &BaseAttributes, level: u32) -> BaseAttributes {
    let steps = level.saturating_sub(1) as f32;
    BaseAttributes {
        attack: base.attack * (1.0 + 0.09 * steps),
        max_health: base.max_health * (1.0 + 0.12 * steps),
        physical_defense: base.physical_defense * (1.0 + 0.08 * steps),
        magic_defense: base.magic_defense * (1.0 + 0.08 * steps),
        ..*base
    }
}

/// `K` only depends on the attacker's level (see `GAME_SETTINGS.md`).
pub fn defense_constant(attacker_level: u32) -> f32 {
    100.0 + 20.0 * attacker_level.max(1) as f32
}

pub fn mitigation(defense: f32, penetration: f32, attacker_level: u32) -> f32 {
    let effective_defense = (defense - penetration).max(0.0);
    let k = defense_constant(attacker_level);
    effective_defense / (k + effective_defense)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageRoll {
    pub amount: f32,
    pub crit: bool,
    pub dodged: bool,
}

pub fn roll_damage(
    attacker: &StatBlock,
    defender: &StatBlock,
    attack_type: AttackType,
    multiplier: f32,
    attacker_level: u32,
    rng: &mut GameRng,
) -> DamageRoll {
    if rng.chance(defender.dodge_chance) {
        return DamageRoll {
            amount: 0.0,
            crit: false,
            dodged: true,
        };
    }

    let defense = defender.defense_against(attack_type);
    let reduction = mitigation(defense, attacker.defense_penetration, attacker_level);
    let crit = rng.chance(attacker.crit_chance);
    let crit_multiplier = if crit {
        attacker.crit_multiplier()
    } else {
        1.0
    };
    let amount = (attacker.attack * multiplier * (1.0 - reduction) * crit_multiplier).max(1.0);

    DamageRoll {
        amount,
        crit,
        dodged: false,
    }
}

pub fn heal_amount(caster: &StatBlock, multiplier: f32) -> f32 {
    (caster.attack * multiplier * (1.0 + caster.healing_bonus)).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attacker() -> StatBlock {
        StatBlock {
            attack: 100.0,
            crit_chance: 0.0,
            ..Default::default()
        }
    }

    #[test]
    fn percentages_are_summed_then_applied() {
        let base = BaseAttributes {
            attack: 100.0,
            max_health: 1000.0,
            physical_defense: 100.0,
            magic_defense: 50.0,
            ..Default::default()
        };
        let modifiers = [
            StatModifier::new(StatKind::AttackPercent, 0.20),
            StatModifier::new(StatKind::AttackPercent, 0.30),
            StatModifier::new(StatKind::Attack, 50.0),
            StatModifier::new(StatKind::MaxHealthPercent, 0.10),
            StatModifier::new(StatKind::PhysicalDefense, 50.0),
            StatModifier::new(StatKind::MagicDefensePercent, 0.20),
        ];
        let stats = StatBlock::compute(&base, &modifiers);

        assert!((stats.attack - 225.0).abs() < 0.001);
        assert!((stats.max_health - 1100.0).abs() < 0.001);
        assert!((stats.physical_defense - 150.0).abs() < 0.001);
        assert!((stats.magic_defense - 60.0).abs() < 0.001);
    }

    #[test]
    fn zero_defense_means_no_mitigation() {
        assert_eq!(mitigation(0.0, 0.0, 1), 0.0);
    }

    #[test]
    fn mitigation_grows_with_defense() {
        let mut previous = 0.0;
        for defense in [0.0, 50.0, 100.0, 500.0, 5000.0] {
            let current = mitigation(defense, 0.0, 10);
            assert!(current >= previous);
            assert!(current < 1.0);
            previous = current;
        }
    }

    #[test]
    fn damage_is_reduced_by_defense() {
        let mut rng = GameRng::new(9);
        let no_defense = StatBlock::default();
        let armored = StatBlock {
            physical_defense: 300.0,
            ..Default::default()
        };

        let open = roll_damage(
            &attacker(),
            &no_defense,
            AttackType::Physical,
            1.0,
            1,
            &mut rng,
        );
        let mitigated = roll_damage(
            &attacker(),
            &armored,
            AttackType::Physical,
            1.0,
            1,
            &mut rng,
        );

        assert!((open.amount - 100.0).abs() < 0.001);
        assert!(mitigated.amount < open.amount);
    }

    #[test]
    fn dodge_results_in_zero_damage() {
        let mut rng = GameRng::new(1);
        let dodger = StatBlock {
            dodge_chance: 1.0,
            ..Default::default()
        };
        let roll = roll_damage(&attacker(), &dodger, AttackType::Physical, 1.0, 1, &mut rng);
        assert!(roll.dodged);
        assert_eq!(roll.amount, 0.0);
    }

    #[test]
    fn level_scaling_grows_core_stats_only() {
        let base = BaseAttributes {
            attack: 100.0,
            max_health: 1000.0,
            physical_defense: 100.0,
            magic_defense: 100.0,
            crit_chance: 0.1,
            ..Default::default()
        };
        let level_5 = base_at_level(&base, 5);
        assert!(level_5.attack > base.attack);
        assert!(level_5.max_health > base.max_health);
        assert!(level_5.physical_defense > base.physical_defense);
        assert_eq!(level_5.crit_chance, base.crit_chance);
    }
}
