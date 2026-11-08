//! Message-driven damage, healing, buff application and death handling.

use bevy::prelude::*;

use crate::GameState;
use crate::core::rng::GameRng;
use crate::core::stats::{AttackType, heal_amount, roll_damage};

use super::components::{
    ActiveBuff, ActiveBuffs, BuffRequest, CombatEvent, CombatEventKind, CombatTextAnchor,
    DamageRequest, Dead, FinalStats, HealRequest, Health, Hero,
};
use super::effects::spawn_floating_text;

pub fn resolve_damage(
    mut commands: Commands,
    mut requests: MessageReader<DamageRequest>,
    mut rng: ResMut<GameRng>,
    attackers: Query<&FinalStats>,
    mut targets: Query<
        (
            &mut Health,
            &FinalStats,
            &GlobalTransform,
            Option<&CombatTextAnchor>,
        ),
        Without<Dead>,
    >,
    mut events: MessageWriter<CombatEvent>,
) {
    for request in requests.read() {
        let Ok(attacker) = attackers.get(request.source) else {
            continue;
        };
        let Ok((mut health, defender, transform, anchor)) = targets.get_mut(request.target) else {
            continue;
        };

        for hit in 0..request.hits.max(1) {
            if health.current <= 0.0 {
                break;
            }

            let roll = roll_damage(
                &attacker.0,
                &defender.0,
                request.attack_type,
                request.multiplier,
                request.attacker_level,
                &mut rng,
            );

            // Keep the world anchor on the target. Multi-hit numbers are
            // stacked in screen space instead of being pushed toward other
            // characters' heads. `CombatTextAnchor` is measured from the
            // entity origin, which is at the feet for heroes and at the
            // capsule centre for enemies.
            let head_height = anchor
                .map(|anchor| anchor.0)
                .unwrap_or(1.05 * transform.scale().y + 0.25);
            let position = transform.translation() + Vec3::new(0.0, head_height, 0.0);
            let screen_offset = Vec2::new(0.0, hit as f32 * 18.0);

            if roll.dodged {
                spawn_floating_text(
                    &mut commands,
                    position,
                    screen_offset,
                    "闪避",
                    Color::srgb(0.75, 0.78, 0.85),
                    20.0,
                );
                continue;
            }

            let dealt = roll.amount.min(health.current);
            health.current = (health.current - roll.amount).max(0.0);

            events.write(CombatEvent {
                source: request.source,
                target: request.target,
                amount: dealt,
                kind: CombatEventKind::DamageDealt,
            });

            // Physical hits are red, magic hits are blue. Critical hits use
            // a brighter version of the same hue and a larger font.
            let (base_color, crit_color) = match request.attack_type {
                AttackType::Physical => {
                    (Color::srgb(1.00, 0.30, 0.26), Color::srgb(1.00, 0.48, 0.34))
                }
                AttackType::Magic => (Color::srgb(0.35, 0.62, 1.00), Color::srgb(0.35, 0.86, 1.00)),
            };
            let (text, color, size) = if roll.crit {
                (
                    format!("暴击 -{}", crate::ui::theme::format_number(roll.amount)),
                    crit_color,
                    32.0,
                )
            } else {
                (
                    format!("-{}", crate::ui::theme::format_number(roll.amount)),
                    base_color,
                    24.0,
                )
            };
            spawn_floating_text(&mut commands, position, screen_offset, text, color, size);
        }
    }
}

pub fn resolve_heals(
    mut commands: Commands,
    mut requests: MessageReader<HealRequest>,
    casters: Query<&FinalStats>,
    mut targets: Query<
        (
            &mut Health,
            &FinalStats,
            &GlobalTransform,
            Option<&CombatTextAnchor>,
        ),
        Without<Dead>,
    >,
    mut events: MessageWriter<CombatEvent>,
) {
    for request in requests.read() {
        let Ok(caster) = casters.get(request.source) else {
            continue;
        };
        let Ok((mut health, stats, transform, anchor)) = targets.get_mut(request.target) else {
            continue;
        };

        let amount = heal_amount(&caster.0, request.multiplier);
        let before = health.current;
        health.current = (health.current + amount).min(stats.0.max_health);
        let effective = health.current - before;

        if effective <= 0.0 {
            continue;
        }

        events.write(CombatEvent {
            source: request.source,
            target: request.target,
            amount: effective,
            kind: CombatEventKind::HealingDone,
        });

        let head_height = anchor
            .map(|anchor| anchor.0)
            .unwrap_or(1.05 * transform.scale().y + 0.25);
        spawn_floating_text(
            &mut commands,
            transform.translation() + Vec3::new(0.0, head_height, 0.0),
            Vec2::ZERO,
            format!("+{}", crate::ui::theme::format_number(effective)),
            Color::srgb(0.35, 1.00, 0.45),
            26.0,
        );
    }
}

pub fn apply_buffs(mut requests: MessageReader<BuffRequest>, mut targets: Query<&mut ActiveBuffs>) {
    for request in requests.read() {
        let Ok(mut buffs) = targets.get_mut(request.target) else {
            continue;
        };

        buffs.buffs.retain(|buff| buff.source != request.source);
        for modifier in &request.modifiers {
            buffs.buffs.push(ActiveBuff {
                source: request.source.clone(),
                modifier: *modifier,
                remaining: request.duration,
            });
        }
    }
}

pub fn record_combat(
    mut events: MessageReader<CombatEvent>,
    mut records: Query<&mut crate::combat::components::CombatRecord>,
) {
    for event in events.read() {
        match event.kind {
            CombatEventKind::DamageDealt => {
                if let Ok(mut record) = records.get_mut(event.source) {
                    record.damage_dealt += event.amount;
                }
                if let Ok(mut record) = records.get_mut(event.target) {
                    record.damage_taken += event.amount;
                }
            }
            CombatEventKind::HealingDone => {
                if let Ok(mut record) = records.get_mut(event.source) {
                    record.healing_done += event.amount;
                }
            }
        }
    }
}

pub fn reap_dead(mut commands: Commands, query: Query<(Entity, &Health), Without<Dead>>) {
    for (entity, health) in &query {
        if health.current > 0.0 {
            continue;
        }
        commands.entity(entity).insert((Dead, Visibility::Hidden));
    }
}

pub fn check_defeat(
    heroes: Query<(&Health, Option<&Dead>), With<Hero>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut total = 0;
    let mut alive = 0;

    for (health, dead) in &heroes {
        total += 1;
        if dead.is_none() && health.current > 0.0 {
            alive += 1;
        }
    }

    if total > 0 && alive == 0 {
        info!("全队阵亡");
        next_state.set(GameState::Defeat);
    }
}
