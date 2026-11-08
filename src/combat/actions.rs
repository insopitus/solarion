//! Auto-attacks, smart skill casting, buffs and mana regeneration.

use bevy::prelude::*;

use crate::core::stats::{MANA_ON_BASIC_ATTACK, MAX_MANA, StatBlock, base_at_level};
use crate::data::heroes::{
    BuffTarget, DamageTarget, HealTarget, HeroRoster, SkillDefinition, SkillEffect,
};
use crate::progression::LevelProgress;

use super::components::{
    ActionState, ActiveBuffs, BuffRequest, CombatProfile, ControlMode, DamageRequest, Dead, Enemy,
    EnemyAi, FinalStats, HealRequest, Health, Hero, HeroLoadout, Mana, Taunt,
};

pub fn hero_actions(
    time: Res<Time>,
    roster: Res<HeroRoster>,
    mut damage_writer: MessageWriter<DamageRequest>,
    mut heal_writer: MessageWriter<HealRequest>,
    mut buff_writer: MessageWriter<BuffRequest>,
    mut commands: Commands,
    mut actors: Query<
        (
            Entity,
            &Hero,
            &ControlMode,
            &mut ActionState,
            &mut Mana,
            &mut ActiveBuffs,
            &FinalStats,
            &LevelProgress,
            &GlobalTransform,
            Option<&Taunt>,
        ),
        Without<Dead>,
    >,
    enemies: Query<(Entity, &GlobalTransform), (With<Enemy>, Without<Dead>)>,
    allies: Query<(Entity, &Health, &GlobalTransform, &FinalStats), (With<Hero>, Without<Dead>)>,
) {
    let delta = time.delta_secs();

    for (entity, hero, control, mut action, mut mana, buffs, stats, level, transform, taunt) in
        &mut actors
    {
        action.action_timer = (action.action_timer - delta).max(0.0);
        for cooldown in &mut action.cooldowns {
            *cooldown = (*cooldown - delta).max(0.0);
        }

        if action.action_timer > 0.0 {
            continue;
        }

        let definition = roster.hero(hero.index);
        let actor_position = transform.translation();

        let enemy_positions: Vec<(Entity, Vec3)> = enemies
            .iter()
            .map(|(entity, transform)| (entity, transform.translation()))
            .collect();
        let engaged = !enemy_positions.is_empty();

        let mut lowest_ally = None;
        let mut lowest_ratio = 1.0;
        for (ally_entity, health, _, ally_stats) in &allies {
            let ratio = health.fraction(ally_stats.0.max_health);
            if ratio < lowest_ratio {
                lowest_ratio = ratio;
                lowest_ally = Some(ally_entity);
            }
        }

        let mut chosen: Option<(usize, &SkillDefinition)> = None;

        // A click queued a specific skill; validate it against the same rules
        // as the AI. Invalid clicks simply fall through to the basic attack.
        if let Some(index) = action.pending_skill.take() {
            if let Some(skill) = definition.skills.get(index) {
                if action.cooldowns[index] <= 0.0
                    && mana.current >= skill.mana_cost
                    && skill_available(
                        skill,
                        &buffs,
                        taunt,
                        &enemy_positions,
                        actor_position,
                        engaged,
                        lowest_ratio,
                    )
                {
                    chosen = Some((index, skill));
                }
            }
        }

        if chosen.is_none() && *control == ControlMode::Auto {
            for (index, skill) in definition.skills.iter().enumerate() {
                if action.cooldowns[index] > 0.0 {
                    continue;
                }
                if mana.current < skill.mana_cost {
                    continue;
                }
                if !skill_available(
                    skill,
                    &buffs,
                    taunt,
                    &enemy_positions,
                    actor_position,
                    engaged,
                    lowest_ratio,
                ) {
                    continue;
                }
                if chosen.is_none_or(|(_, current)| skill.priority > current.priority) {
                    chosen = Some((index, skill));
                }
            }
        }

        if let Some((index, skill)) = chosen {
            mana.current -= skill.mana_cost;
            action.cooldowns[index] = skill.cooldown;
            action.action_timer = stats.0.action_interval();

            match &skill.effect {
                SkillEffect::Damage {
                    multiplier,
                    hits,
                    target,
                } => match target {
                    DamageTarget::NearestEnemy => {
                        if let Some(target) =
                            nearest_enemy_within(&enemy_positions, actor_position, skill.range)
                        {
                            damage_writer.write(DamageRequest {
                                source: entity,
                                target,
                                multiplier: *multiplier,
                                hits: *hits,
                                attack_type: definition.attack_type,
                                attacker_level: level.level,
                            });
                        }
                    }
                    DamageTarget::AllEnemies => {
                        for (target, position) in &enemy_positions {
                            if in_range(actor_position, *position, skill.range) {
                                damage_writer.write(DamageRequest {
                                    source: entity,
                                    target: *target,
                                    multiplier: *multiplier,
                                    hits: *hits,
                                    attack_type: definition.attack_type,
                                    attacker_level: level.level,
                                });
                            }
                        }
                    }
                },
                SkillEffect::Heal { multiplier, target } => match target {
                    HealTarget::LowestHealthAlly => {
                        if let Some(target) = lowest_ally {
                            heal_writer.write(HealRequest {
                                source: entity,
                                target,
                                multiplier: *multiplier,
                            });
                        }
                    }
                    HealTarget::AllAllies => {
                        for (target, _, _, _) in &allies {
                            heal_writer.write(HealRequest {
                                source: entity,
                                target,
                                multiplier: *multiplier,
                            });
                        }
                    }
                },
                SkillEffect::Buff {
                    modifiers,
                    duration,
                    target,
                } => {
                    let targets: Vec<Entity> = match target {
                        BuffTarget::SelfOnly => vec![entity],
                        BuffTarget::AllAllies => {
                            allies.iter().map(|(entity, _, _, _)| entity).collect()
                        }
                    };
                    for target in targets {
                        buff_writer.write(BuffRequest {
                            source: skill.name.clone(),
                            target,
                            modifiers: modifiers.clone(),
                            duration: *duration,
                        });
                    }
                }
                SkillEffect::Taunt { duration } => {
                    commands.entity(entity).insert(Taunt {
                        remaining: *duration,
                    });
                }
            }
        } else if let Some(target) =
            nearest_enemy_within(&enemy_positions, actor_position, definition.attack_range)
        {
            damage_writer.write(DamageRequest {
                source: entity,
                target,
                multiplier: 1.0,
                hits: 1,
                attack_type: definition.attack_type,
                attacker_level: level.level,
            });
            mana.current = (mana.current + MANA_ON_BASIC_ATTACK).min(MAX_MANA);
            action.action_timer = stats.0.action_interval();
        }
    }
}

fn planar_distance_squared(left: Vec3, right: Vec3) -> f32 {
    let dx = left.x - right.x;
    let dz = left.z - right.z;
    dx * dx + dz * dz
}

fn in_range(origin: Vec3, target: Vec3, range: f32) -> bool {
    planar_distance_squared(origin, target) <= range * range
}

fn has_enemy_within(enemies: &[(Entity, Vec3)], origin: Vec3, range: f32) -> bool {
    enemies
        .iter()
        .any(|(_, position)| in_range(origin, *position, range))
}

fn nearest_enemy_within(enemies: &[(Entity, Vec3)], origin: Vec3, range: f32) -> Option<Entity> {
    enemies
        .iter()
        .filter(|(_, position)| in_range(origin, *position, range))
        .min_by(|(_, left), (_, right)| {
            planar_distance_squared(origin, *left)
                .partial_cmp(&planar_distance_squared(origin, *right))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _)| *entity)
}

fn skill_available(
    skill: &SkillDefinition,
    buffs: &ActiveBuffs,
    taunt: Option<&Taunt>,
    enemies: &[(Entity, Vec3)],
    actor_position: Vec3,
    engaged: bool,
    lowest_ally_ratio: f32,
) -> bool {
    if let Some(threshold) = skill.ally_health_below {
        if lowest_ally_ratio >= threshold {
            return false;
        }
    }

    match &skill.effect {
        SkillEffect::Damage { .. } => has_enemy_within(enemies, actor_position, skill.range),
        // Do not waste heals/buffs before a wave has actually engaged.
        // Ally-targeted effects deliberately have no range restriction.
        SkillEffect::Heal { .. } => engaged,
        SkillEffect::Buff { target, .. } => match target {
            BuffTarget::SelfOnly | BuffTarget::AllAllies => {
                engaged && !buffs.has_source(&skill.name)
            }
        },
        SkillEffect::Taunt { .. } => {
            has_enemy_within(enemies, actor_position, skill.range)
                && taunt.is_none_or(|taunt| taunt.remaining <= 0.0)
        }
    }
}

pub fn tick_buffs(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ActiveBuffs, Option<&mut Taunt>)>,
) {
    let delta = time.delta_secs();

    for (entity, mut buffs, taunt) in &mut query {
        for buff in &mut buffs.buffs {
            buff.remaining -= delta;
        }
        buffs.buffs.retain(|buff| buff.remaining > 0.0);

        if let Some(mut taunt) = taunt {
            taunt.remaining -= delta;
            if taunt.remaining <= 0.0 {
                commands.entity(entity).remove::<Taunt>();
            }
        }
    }
}

pub fn regenerate_mana(
    time: Res<Time>,
    mut query: Query<(&mut Mana, &FinalStats, Option<&Dead>), With<Hero>>,
) {
    let delta = time.delta_secs();
    for (mut mana, stats, dead) in &mut query {
        if dead.is_some() {
            continue;
        }
        mana.current = (mana.current + stats.0.mana_regen * delta).min(MAX_MANA);
    }
}

pub fn recompute_hero_stats(
    roster: Res<HeroRoster>,
    mut query: Query<
        (
            &Hero,
            &LevelProgress,
            &HeroLoadout,
            &ActiveBuffs,
            &mut Health,
            &mut FinalStats,
        ),
        Without<Dead>,
    >,
) {
    for (hero, level, loadout, buffs, mut health, mut final_stats) in &mut query {
        let definition = roster.hero(hero.index);
        let base = base_at_level(&definition.base, level.level);

        let mut modifiers = definition.passive_modifiers.clone();
        modifiers.extend(loadout.0.modifiers());
        modifiers.extend(buffs.buffs.iter().map(|buff| buff.modifier));

        let stats = StatBlock::compute(&base, &modifiers);
        health.current = health.current.min(stats.max_health);
        *final_stats = FinalStats(stats);
    }
}

pub fn enemy_actions(
    time: Res<Time>,
    mut damage_writer: MessageWriter<DamageRequest>,
    mut enemies: Query<
        (Entity, &mut EnemyAi, &mut Transform, &CombatProfile),
        (With<Enemy>, Without<Dead>),
    >,
    heroes: Query<(Entity, &GlobalTransform, Option<&Taunt>), (With<Hero>, Without<Dead>)>,
) {
    let delta = time.delta_secs();

    let taunt_target = heroes
        .iter()
        .filter_map(|(entity, transform, taunt)| {
            taunt.map(|taunt| (entity, transform.translation(), taunt.remaining))
        })
        .max_by(|left, right| {
            left.2
                .partial_cmp(&right.2)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, position, _)| (entity, position));

    let hero_positions: Vec<(Entity, Vec3)> = heroes
        .iter()
        .map(|(entity, transform, _)| (entity, transform.translation()))
        .collect();

    for (entity, mut ai, mut transform, profile) in &mut enemies {
        let enemy_position = transform.translation;

        let target = taunt_target.or_else(|| {
            hero_positions
                .iter()
                .min_by(|left, right| {
                    left.1
                        .distance_squared(enemy_position)
                        .partial_cmp(&right.1.distance_squared(enemy_position))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .copied()
        });

        let Some((target_entity, target_position)) = target else {
            continue;
        };

        let to_target = target_position - enemy_position;
        let planar_distance = Vec2::new(to_target.x, to_target.z).length();

        if planar_distance > 2.0 {
            let direction = Vec2::new(to_target.x, to_target.z).normalize_or_zero();
            transform.translation.x += direction.x * ai.move_speed * delta;
            transform.translation.z += direction.y * ai.move_speed * delta;
            ai.attack_timer = (ai.attack_timer - delta).max(0.0);
            continue;
        }

        if ai.attack_timer > 0.0 {
            ai.attack_timer = (ai.attack_timer - delta).max(0.0);
            continue;
        }

        damage_writer.write(DamageRequest {
            source: entity,
            target: target_entity,
            multiplier: ai.attack_multiplier,
            hits: 1,
            attack_type: profile.attack_type,
            attacker_level: profile.level,
        });
        ai.attack_timer = ai.attack_interval;
    }
}
