//! Equipment data model, affix pool and a small procedural item generator.
//!
//! Affixes are generated as *fractions* (0.05 == +5%). Relics intentionally
//! have no primary/sub stats for now; they are the hook for future unique
//! effects described in the design document.
//!
//! Tunable values (rarity scaling, primary stats, affix ranges, starter kits)
//! live in `assets/data/equipment.ron`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::rng::GameRng;
use crate::core::stats::{StatKind, StatModifier};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Chest,
    SecondaryArmor,
    Jewelry,
    Orb,
    Relic,
}

impl EquipmentSlot {
    pub const ALL: [Self; 6] = [
        Self::Weapon,
        Self::Chest,
        Self::SecondaryArmor,
        Self::Jewelry,
        Self::Orb,
        Self::Relic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Weapon => "武器",
            Self::Chest => "胸甲",
            Self::SecondaryArmor => "副护甲",
            Self::Jewelry => "首饰",
            Self::Orb => "宝珠",
            Self::Relic => "遗物",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JewelryKind {
    Ring,
    Necklace,
    Earring,
    Bracelet,
}

impl JewelryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Ring => "戒指",
            Self::Necklace => "项链",
            Self::Earring => "耳环",
            Self::Bracelet => "手镯",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn affix_count(self) -> usize {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Common => 0,
            Self::Uncommon => 1,
            Self::Rare => 2,
            Self::Epic => 3,
            Self::Legendary => 4,
        }
    }

    pub fn tier(self) -> f32 {
        self.index() as f32
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Common => "普通",
            Self::Uncommon => "优秀",
            Self::Rare => "精良",
            Self::Epic => "史诗",
            Self::Legendary => "传说",
        }
    }

    /// Used by the upcoming inventory / tooltip UI.
    #[allow(dead_code)]
    pub fn color(self) -> Color {
        match self {
            Self::Common => Color::srgb(0.86, 0.86, 0.86),
            Self::Uncommon => Color::srgb(0.35, 0.85, 0.35),
            Self::Rare => Color::srgb(0.35, 0.60, 1.00),
            Self::Epic => Color::srgb(0.72, 0.40, 1.00),
            Self::Legendary => Color::srgb(1.00, 0.62, 0.20),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AffixKind {
    AttackPercent,
    MaxHealthPercent,
    PhysicalDefensePercent,
    MagicDefensePercent,
    DefensePenetration,
    CritChance,
    CritDamage,
    AttackSpeed,
    DodgeChance,
}

impl AffixKind {
    pub const POOL: [Self; 9] = [
        Self::AttackPercent,
        Self::MaxHealthPercent,
        Self::PhysicalDefensePercent,
        Self::MagicDefensePercent,
        Self::DefensePenetration,
        Self::CritChance,
        Self::CritDamage,
        Self::AttackSpeed,
        Self::DodgeChance,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::AttackPercent => "攻击力%",
            Self::MaxHealthPercent => "生命上限%",
            Self::PhysicalDefensePercent => "物理防御%",
            Self::MagicDefensePercent => "魔法防御%",
            Self::DefensePenetration => "防御穿透",
            Self::CritChance => "暴击率",
            Self::CritDamage => "暴击伤害",
            Self::AttackSpeed => "攻击速度",
            Self::DodgeChance => "闪避率",
        }
    }

    pub fn to_stat(self, value: f32) -> StatModifier {
        let kind = match self {
            Self::AttackPercent => StatKind::AttackPercent,
            Self::MaxHealthPercent => StatKind::MaxHealthPercent,
            Self::PhysicalDefensePercent => StatKind::PhysicalDefensePercent,
            Self::MagicDefensePercent => StatKind::MagicDefensePercent,
            Self::DefensePenetration => StatKind::DefensePenetration,
            Self::CritChance => StatKind::CritChance,
            Self::CritDamage => StatKind::CritDamage,
            Self::AttackSpeed => StatKind::AttackSpeed,
            Self::DodgeChance => StatKind::DodgeChance,
        };
        StatModifier::new(kind, value)
    }

    pub fn is_percent(self) -> bool {
        matches!(
            self,
            Self::AttackPercent
                | Self::MaxHealthPercent
                | Self::PhysicalDefensePercent
                | Self::MagicDefensePercent
                | Self::CritChance
                | Self::CritDamage
                | Self::AttackSpeed
                | Self::DodgeChance
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Affix {
    pub kind: AffixKind,
    pub value: f32,
}

impl Affix {
    pub fn describe(self) -> String {
        if self.kind.is_percent() {
            format!("{}+{:.1}%", self.kind.label(), self.value * 100.0)
        } else {
            format!("{}+{:.0}", self.kind.label(), self.value)
        }
    }
}

/// A possible primary stat for a generated item.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrimaryValueConfig {
    pub slot: EquipmentSlot,
    pub jewelry_kind: Option<JewelryKind>,
    pub stat: StatKind,
    pub base_value: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AffixRangeConfig {
    pub kind: AffixKind,
    pub min: f32,
    pub max: f32,
    pub per_tier: f32,
}

/// All loot tunables loaded from `assets/data/equipment.ron`.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct EquipmentConfig {
    /// Primary stat multiplier indexed by `Rarity::index()`.
    pub primary_scale: [f32; 5],
    pub primary_values: Vec<PrimaryValueConfig>,
    pub affix_ranges: Vec<AffixRangeConfig>,
    /// Starter rarity table per hero; each row has six entries in slot order.
    pub starter_rarities: Vec<Vec<Rarity>>,
}

impl EquipmentConfig {
    pub fn primary_scale(&self, rarity: Rarity) -> f32 {
        self.primary_scale[rarity.index()]
    }

    pub fn affix_range(&self, kind: AffixKind) -> Option<&AffixRangeConfig> {
        self.affix_ranges.iter().find(|range| range.kind == kind)
    }

    pub fn validate(&self) -> Result<(), String> {
        for slot in EquipmentSlot::ALL {
            if slot == EquipmentSlot::Relic {
                continue;
            }
            if !self.primary_values.iter().any(|value| value.slot == slot) {
                return Err(format!("装备配置缺少 {} 的主属性候选", slot.label()));
            }
        }

        for kind in AffixKind::POOL {
            if self.affix_range(kind).is_none() {
                return Err(format!("装备配置缺少 {} 的词条范围", kind.label()));
            }
        }

        if self.starter_rarities.is_empty() {
            return Err("装备配置缺少初始装备品级表".to_string());
        }
        for (index, row) in self.starter_rarities.iter().enumerate() {
            if row.len() != EquipmentSlot::ALL.len() {
                return Err(format!(
                    "初始装备品级表第 {} 行需要 {} 项，实际为 {}",
                    index + 1,
                    EquipmentSlot::ALL.len(),
                    row.len()
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct EquipmentItem {
    pub name: String,
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub jewelry_kind: Option<JewelryKind>,
    pub primary: Option<StatModifier>,
    pub affixes: Vec<Affix>,
}

impl EquipmentItem {
    pub fn modifiers(&self) -> impl Iterator<Item = StatModifier> + '_ {
        self.primary.into_iter().chain(
            self.affixes
                .iter()
                .map(|affix| affix.kind.to_stat(affix.value)),
        )
    }

    pub fn describe(&self) -> String {
        let slot = match self.jewelry_kind {
            Some(kind) => kind.label(),
            None => self.slot.label(),
        };
        let mut text = format!("[{}] {}（{}）", self.rarity.label(), self.name, slot);
        if let Some(primary) = self.primary {
            text.push_str(&format!(
                "\n  主属性: {} +{:.0}",
                primary.kind.label(),
                primary.value
            ));
        }
        for affix in &self.affixes {
            text.push_str(&format!("\n  副属性: {}", affix.describe()));
        }
        if self.slot == EquipmentSlot::Relic {
            text.push_str("\n  特殊效果: 待实现");
        }
        text
    }
}

/// The six equipped slots of one character.
#[derive(Clone, Debug, Default)]
pub struct Loadout {
    pub items: Vec<EquipmentItem>,
}

impl Loadout {
    pub fn equip(&mut self, item: EquipmentItem) {
        if let Some(existing) = self.items.iter_mut().find(|slot| slot.slot == item.slot) {
            *existing = item;
        } else {
            self.items.push(item);
        }
    }

    #[allow(dead_code)]
    pub fn item(&self, slot: EquipmentSlot) -> Option<&EquipmentItem> {
        self.items.iter().find(|item| item.slot == slot)
    }

    pub fn modifiers(&self) -> Vec<StatModifier> {
        self.items
            .iter()
            .flat_map(|item| item.modifiers())
            .collect()
    }
}

fn roll_affix(kind: AffixKind, rarity: Rarity, rng: &mut GameRng, config: &EquipmentConfig) -> f32 {
    let range = config
        .affix_range(kind)
        .expect("equipment config is validated at load time");
    rng.range_f32(range.min, range.max) + range.per_tier * rarity.tier()
}

pub fn generate_item(
    slot: EquipmentSlot,
    rarity: Rarity,
    rng: &mut GameRng,
    config: &EquipmentConfig,
) -> EquipmentItem {
    let candidates: Vec<&PrimaryValueConfig> = config
        .primary_values
        .iter()
        .filter(|value| value.slot == slot)
        .collect();

    let chosen = if candidates.is_empty() {
        None
    } else {
        Some(candidates[rng.range_usize(0, candidates.len())])
    };

    let jewelry_kind = chosen.and_then(|value| value.jewelry_kind);
    let primary = chosen.map(|value| {
        StatModifier::new(value.stat, value.base_value * config.primary_scale(rarity))
    });

    let mut pool = AffixKind::POOL.to_vec();
    let mut affixes = Vec::with_capacity(rarity.affix_count());
    for _ in 0..rarity.affix_count() {
        let index = rng.range_usize(0, pool.len());
        let kind = pool.swap_remove(index);
        affixes.push(Affix {
            kind,
            value: roll_affix(kind, rarity, rng, config),
        });
    }

    let base_name = match jewelry_kind {
        Some(kind) => kind.label(),
        None => slot.label(),
    };
    let name = format!("{}{}", rarity.label(), base_name);

    EquipmentItem {
        name,
        slot,
        rarity,
        jewelry_kind,
        primary,
        affixes,
    }
}

/// A starter kit so combat has meaningful numbers from the first frame.
pub fn generate_starter_loadout(
    hero_index: usize,
    rng: &mut GameRng,
    config: &EquipmentConfig,
) -> Loadout {
    let rarities = &config.starter_rarities[hero_index % config.starter_rarities.len()];
    let mut loadout = Loadout::default();

    for (slot, rarity) in EquipmentSlot::ALL.into_iter().zip(rarities.iter().copied()) {
        loadout.equip(generate_item(slot, rarity, rng, config));
    }
    loadout
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::loader::load_game_data;

    fn config() -> EquipmentConfig {
        load_game_data()
            .expect("shipped RON data should parse")
            .equipment
    }

    #[test]
    fn affix_count_matches_rarity() {
        let config = config();
        for rarity in [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ] {
            let mut rng = GameRng::new(3);
            let item = generate_item(EquipmentSlot::Weapon, rarity, &mut rng, &config);
            assert_eq!(item.affixes.len(), rarity.affix_count());
        }
    }

    #[test]
    fn affixes_are_unique_within_an_item() {
        let config = config();
        let mut rng = GameRng::new(17);
        for _ in 0..200 {
            let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, &mut rng, &config);
            let mut kinds: Vec<_> = item.affixes.iter().map(|affix| affix.kind).collect();
            let count = kinds.len();
            kinds.sort_by_key(|kind| format!("{kind:?}"));
            kinds.dedup();
            assert_eq!(kinds.len(), count);
        }
    }

    #[test]
    fn loadout_replaces_same_slot() {
        let config = config();
        let mut rng = GameRng::new(5);
        let mut loadout = Loadout::default();
        loadout.equip(generate_item(
            EquipmentSlot::Weapon,
            Rarity::Common,
            &mut rng,
            &config,
        ));
        loadout.equip(generate_item(
            EquipmentSlot::Weapon,
            Rarity::Rare,
            &mut rng,
            &config,
        ));
        assert_eq!(loadout.items.len(), 1);
        assert_eq!(
            loadout.item(EquipmentSlot::Weapon).unwrap().rarity,
            Rarity::Rare
        );
    }

    #[test]
    fn starter_loadout_has_all_slots() {
        let config = config();
        let mut rng = GameRng::new(11);
        let loadout = generate_starter_loadout(0, &mut rng, &config);
        assert_eq!(loadout.items.len(), EquipmentSlot::ALL.len());
    }
}
